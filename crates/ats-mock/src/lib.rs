//! Mock of an employer's ATS for demos and end-to-end tests. It receives the platform's
//! webhooks, checks `FSP-Signature` with the shared secret, keeps the last 200 entries in a
//! lock-free ring and shows them on a page whose buttons report hiring stages back through
//! `POST /api/v1/integrations/outcomes` with the company's API key.

mod ring;
mod signature;

use std::{fmt::Write as _, sync::Arc, time::Duration};

use axum::{
    Form, Json, Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use ring::Ring;
pub use signature::{TOLERANCE_SECS, Verdict, verify};

pub const DEFAULT_BIND: &str = "0.0.0.0:8082";
pub const CAPACITY: usize = 200;
const STAGES: [(&str, &str); 4] = [
    ("interview", "Собеседование"),
    ("offer", "Оффер"),
    ("hired", "Нанят"),
    ("rejected", "Отказ"),
];

#[derive(Clone)]
pub struct Config {
    /// Shared webhook signing secret (`whsec_…`), the same the platform signs with.
    pub secret: String,
    /// Platform base URL, e.g. `http://api:8000`.
    pub api_url: String,
    /// `fsp_live_…` key with `outcomes:write`; without it the stage buttons are disabled.
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    Webhook {
        received_at: DateTime<Utc>,
        event_id: Option<String>,
        event_type: Option<String>,
        verdict: Verdict,
        /// The same `FSP-Event-Id` arrived before: at-least-once delivery, processed once.
        duplicate: bool,
        body: Value,
    },
    Outcome {
        sent_at: DateTime<Utc>,
        subject: String,
        stage: String,
        status: Option<u16>,
        message: String,
    },
}

struct Inner {
    config: Config,
    entries: Ring<Entry>,
    http: reqwest::Client,
}

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl AppState {
    pub fn new(config: Config) -> Result<Self, reqwest::Error> {
        Ok(Self(Arc::new(Inner {
            config,
            entries: Ring::new(CAPACITY),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?,
        })))
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.0
            .entries
            .latest()
            .into_iter()
            .map(|e| e.1.clone())
            .collect()
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(page))
        .route("/healthz", get(|| async { "ok" }))
        .route("/events", get(events))
        .route("/webhooks", post(receive))
        .route("/outcomes", post(send_outcome))
        .with_state(state)
}

async fn events(State(state): State<AppState>) -> Json<Vec<Entry>> {
    Json(state.entries())
}

async fn receive(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let text = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    let verdict = verify(
        &state.0.config.secret,
        text("fsp-signature").as_deref(),
        &body,
        Utc::now().timestamp(),
    );
    let event_id = text("fsp-event-id");
    let duplicate = verdict == Verdict::Valid
        && event_id.as_deref().is_some_and(|id| {
            state.0.entries.latest().iter().any(|e| {
                matches!(&e.1, Entry::Webhook { event_id: Some(seen), verdict: Verdict::Valid, .. } if seen == id)
            })
        });
    let body = serde_json::from_slice(&body)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).into_owned()));
    tracing::info!(event = ?event_id, verdict = ?verdict, duplicate, "webhook received");
    state.0.entries.push(Entry::Webhook {
        received_at: Utc::now(),
        event_id,
        event_type: text("fsp-event-type"),
        verdict,
        duplicate,
        body,
    });
    if verdict == Verdict::Valid {
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(Deserialize)]
pub struct OutcomeForm {
    /// `invitation` or `application`.
    pub subject: String,
    pub id: String,
    pub stage: String,
    #[serde(default)]
    pub reason: String,
}

async fn send_outcome(State(state): State<AppState>, Form(form): Form<OutcomeForm>) -> Response {
    let field = match form.subject.as_str() {
        "invitation" => "invitation_id",
        "application" => "application_id",
        _ => return (StatusCode::BAD_REQUEST, "unknown subject").into_response(),
    };
    if !STAGES.iter().any(|(s, _)| *s == form.stage) {
        return (StatusCode::BAD_REQUEST, "unknown stage").into_response();
    }
    let (status, message) = match &state.0.config.api_key {
        None => (
            None,
            "ATS_DEMO_API_KEY не задан — этап не отправлен".to_owned(),
        ),
        Some(key) => {
            let reason = form.reason.trim();
            let mut body = json!({ field: form.id, "stage": form.stage, "at": Utc::now() });
            if !reason.is_empty() {
                body["reason"] = Value::from(reason);
            }
            let sent = state
                .0
                .http
                .post(format!(
                    "{}/api/v1/integrations/outcomes",
                    state.0.config.api_url.trim_end_matches('/')
                ))
                .bearer_auth(key)
                .json(&body)
                .send()
                .await;
            match sent {
                Ok(res) => {
                    let code = res.status().as_u16();
                    let reply: Value = res.json().await.unwrap_or(Value::Null);
                    let message = if (200..300).contains(&code) {
                        "Платформа приняла этап".to_owned()
                    } else {
                        reply["title"]
                            .as_str()
                            .unwrap_or("Платформа отклонила запрос")
                            .to_owned()
                    };
                    (Some(code), message)
                }
                Err(e) => (None, format!("Платформа недоступна: {e}")),
            }
        }
    };
    state.0.entries.push(Entry::Outcome {
        sent_at: Utc::now(),
        subject: format!(
            "{} {}",
            if field == "invitation_id" {
                "приглашение"
            } else {
                "отклик"
            },
            form.id
        ),
        stage: form.stage,
        status,
        message,
    });
    Redirect::to("/").into_response()
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn moscow(t: DateTime<Utc>) -> String {
    FixedOffset::east_opt(3 * 3600)
        .map(|tz| t.with_timezone(&tz).format("%d.%m %H:%M:%S").to_string())
        .unwrap_or_default()
}

fn decline_label(reason: &str) -> &'static str {
    match reason {
        "salary" => "зарплата",
        "stack" => "стек",
        "format" => "формат работы",
        "company" => "компания",
        "not_looking" => "не ищет работу",
        _ => "другое",
    }
}

fn type_label(t: &str) -> &'static str {
    match t {
        "invitation.sent" => "Приглашение отправлено",
        "invitation.viewed" => "Кандидат открыл приглашение",
        "invitation.accepted" => "Кандидат принял приглашение",
        "invitation.declined" => "Кандидат отклонил приглашение",
        "invitation.expired" => "Приглашение истекло",
        "invitation.withdrawn" => "Приглашение отозвано",
        "application.created" => "Новый отклик",
        "application.status_changed" => "Статус отклика изменился",
        "contacts.opened" => "Контакты открыты в кабинете",
        "integration.test" => "Тестовое событие",
        _ => "Событие",
    }
}

fn money(v: &Value) -> String {
    v.as_i64().map_or_else(String::new, |n| {
        let digits = n.to_string();
        let mut out = String::new();
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                out.push('\u{a0}');
            }
            out.push(ch);
        }
        out
    })
}

fn candidate_html(c: &Value) -> String {
    let mut out = String::new();
    let code = c["code"].as_str().unwrap_or("—");
    let category = match (
        c["category"]["specialization"].as_str(),
        c["category"]["grade"].as_str(),
    ) {
        (Some(s), Some(g)) => format!(" · {s}:{g}"),
        _ => String::new(),
    };
    let _ = write!(out, "<b>{}</b>{}", esc(code), esc(&category));
    let d = &c["disclosed"];
    if d.is_object() {
        let _ = write!(
            out,
            "<div class=\"who\">{}</div>",
            esc(d["full_name"].as_str().unwrap_or("Без имени"))
        );
        for key in ["email", "phone", "telegram"] {
            if let Some(v) = d[key].as_str() {
                let _ = write!(out, "<div class=\"muted\">{}</div>", esc(v));
            }
        }
    } else {
        out.push_str("<div class=\"muted\">контакты закрыты до согласия кандидата</div>");
    }
    out
}

fn stage_buttons(subject: &str, id: &str, enabled: bool) -> String {
    let mut out = format!(
        "<form method=\"post\" action=\"/outcomes\" class=\"stages\">\
         <input type=\"hidden\" name=\"subject\" value=\"{}\">\
         <input type=\"hidden\" name=\"id\" value=\"{}\">\
         <input name=\"reason\" placeholder=\"Причина (видна кандидату)\" maxlength=\"500\">",
        esc(subject),
        esc(id)
    );
    for (stage, label) in STAGES {
        let _ = write!(
            out,
            "<button name=\"stage\" value=\"{stage}\"{}{}>{label}</button>",
            if stage == "rejected" {
                " class=\"no\""
            } else {
                ""
            },
            if enabled { "" } else { " disabled" }
        );
    }
    out.push_str("</form>");
    out
}

fn row_html(entry: &Entry, can_send: bool) -> String {
    match entry {
        Entry::Outcome {
            sent_at,
            subject,
            stage,
            status,
            message,
        } => {
            let ok = status.is_some_and(|s| (200..300).contains(&s));
            format!(
                "<tr class=\"out\"><td class=\"muted\">{}</td><td><span class=\"tag {}\">→ ATS отправил этап</span></td>\
                 <td colspan=\"2\"><b>{}</b> · {}<div class=\"muted\">{}{}</div></td></tr>",
                moscow(*sent_at),
                if ok { "ok" } else { "bad" },
                STAGES
                    .iter()
                    .find(|(s, _)| s == stage)
                    .map_or(stage.as_str(), |(_, label)| label),
                esc(subject),
                status.map(|s| format!("HTTP {s} · ")).unwrap_or_default(),
                esc(message)
            )
        }
        Entry::Webhook {
            received_at,
            event_type,
            verdict,
            duplicate,
            body,
            ..
        } => {
            let t = event_type.as_deref().unwrap_or("");
            let data = &body["data"];
            let mut what = String::new();
            let mut action = String::new();
            if let Some(inv) = data.get("invitation") {
                let _ = write!(
                    what,
                    "{}<div class=\"muted\">{}–{} ₽</div>",
                    esc(inv["title"].as_str().unwrap_or("")),
                    money(&inv["salary"]["min"]),
                    money(&inv["salary"]["max"])
                );
                if let Some(reason) = inv["decline_reason"].as_str() {
                    let _ = write!(
                        what,
                        "<div class=\"muted\">причина отказа: {}{}</div>",
                        decline_label(reason),
                        inv["decline_comment"]
                            .as_str()
                            .map(|c| format!(" — «{}»", esc(c)))
                            .unwrap_or_default()
                    );
                }
                if t == "invitation.accepted"
                    && let Some(id) = inv["id"].as_str()
                {
                    action = stage_buttons("invitation", id, can_send);
                }
            } else if let Some(app) = data.get("application") {
                let _ = write!(
                    what,
                    "Отклик: {}<div class=\"muted\">{}</div>",
                    esc(app["vacancy_title"].as_str().unwrap_or("")),
                    esc(app["status"].as_str().unwrap_or(""))
                );
                if t == "application.created"
                    && let Some(id) = app["id"].as_str()
                {
                    action = stage_buttons("application", id, can_send);
                }
            } else if let Some(m) = data["message"].as_str() {
                what = esc(m);
            }
            let candidate = if data["candidate"].is_object() {
                candidate_html(&data["candidate"])
            } else {
                String::new()
            };
            format!(
                "<tr><td class=\"muted\">{}</td>\
                 <td><div>{}</div><code>{}</code><div><span class=\"tag {}\">{}</span>{}</div></td>\
                 <td>{}</td><td>{}{}\
                 <details><summary>JSON</summary><pre>{}</pre></details></td></tr>",
                moscow(*received_at),
                type_label(t),
                esc(t),
                if *verdict == Verdict::Valid {
                    "ok"
                } else {
                    "bad"
                },
                verdict.label_ru(),
                if *duplicate {
                    " <span class=\"tag\">повтор</span>"
                } else {
                    ""
                },
                candidate,
                what,
                action,
                esc(&serde_json::to_string_pretty(body).unwrap_or_default())
            )
        }
    }
}

async fn page(State(state): State<AppState>) -> Response {
    let entries = state.entries();
    let can_send = state.0.config.api_key.is_some();
    let mut rows = String::new();
    for e in &entries {
        rows.push_str(&row_html(e, can_send));
    }
    if entries.is_empty() {
        rows.push_str(
            "<tr><td colspan=\"4\" class=\"empty\">Событий пока нет. Отправьте приглашение кандидату \
             или нажмите «Тестовое событие» в кабинете работодателя → «Интеграции».</td></tr>",
        );
    }
    let html = format!(
        r#"<!doctype html><html lang="ru"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Демо-ATS · ФСП Таланты</title>
<style>
:root{{--p:#402fff;--ink:#1b1c21;--muted:#5a5b66;--line:#e4e4ec;--ok:#0f8a5f;--bad:#d4380d}}
*{{box-sizing:border-box}}body{{margin:0;font:14px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif;color:var(--ink);background:#f6f6fa}}
header{{background:#0e0d1f;color:#fff;padding:20px 24px}}header h1{{margin:0;font-size:20px}}
header p{{margin:6px 0 0;color:#b9b8d6}}main{{padding:16px 24px;overflow-x:auto}}
table{{width:100%;border-collapse:collapse;background:#fff;border:1px solid var(--line);border-radius:12px;overflow:hidden}}
th,td{{text-align:left;vertical-align:top;padding:10px 12px;border-bottom:1px solid var(--line)}}th{{font-size:12px;color:var(--muted);font-weight:600}}
code{{font-size:12px;color:var(--muted)}}.muted{{color:var(--muted);font-size:12.5px}}.who{{font-weight:600}}
.tag{{display:inline-block;margin-top:4px;padding:1px 8px;border-radius:99px;font-size:12px;background:#eee}}
.tag.ok{{background:#e3f6ee;color:var(--ok)}}.tag.bad{{background:#fdebe5;color:var(--bad)}}
.stages{{display:flex;flex-wrap:wrap;gap:6px;margin:6px 0}}.stages input{{flex:1 1 100%;padding:6px 8px;border:1px solid var(--line);border-radius:8px}}
button{{border:0;border-radius:8px;padding:6px 10px;background:var(--p);color:#fff;font-weight:600;cursor:pointer}}
button.no{{background:#fff;color:var(--bad);border:1px solid #f3c3b5}}button:disabled{{opacity:.4;cursor:not-allowed}}
details pre{{max-width:520px;max-height:260px;overflow:auto;background:#f6f6fa;padding:8px;border-radius:8px;font-size:12px}}
.empty{{text-align:center;color:var(--muted);padding:32px}}.bar{{display:flex;gap:12px;align-items:center;margin-bottom:12px;flex-wrap:wrap}}
a{{color:var(--p)}}tr.out{{background:#fafaff}}
</style></head><body>
<header><h1>Демо-ATS</h1><p>Принимает вебхуки ФСП Таланты, проверяет подпись FSP-Signature и отправляет этапы найма обратно через API.</p></header>
<main><div class="bar"><a href="/">Обновить</a><span class="muted">Последние {count} из {cap} · ключ API: {key} · секрет вебхука: {secret}</span></div>
<table><thead><tr><th>Время (МСК)</th><th>Событие</th><th>Кандидат</th><th>Детали и этапы</th></tr></thead><tbody>{rows}</tbody></table></main></body></html>"#,
        count = entries.len(),
        cap = CAPACITY,
        key = if can_send {
            "задан"
        } else {
            "не задан — кнопки выключены"
        },
        secret = esc(&format!(
            "{}…",
            state.0.config.secret.chars().take(10).collect::<String>()
        )),
    );
    ([(header::CACHE_CONTROL, "no-store")], Html(html)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup() {
        assert_eq!(
            esc("<a href=\"x\">'&'</a>"),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }

    #[test]
    fn disclosed_candidate_is_rendered_only_when_present() {
        let anonymous = candidate_html(&json!({ "code": "K-1", "category": null }));
        assert!(anonymous.contains("контакты закрыты"));
        let open = candidate_html(&json!({
            "code": "K-1",
            "disclosed": { "full_name": "Анна <b>", "email": "a@x.ru" }
        }));
        assert!(open.contains("Анна &lt;b&gt;") && open.contains("a@x.ru"));
    }
}
