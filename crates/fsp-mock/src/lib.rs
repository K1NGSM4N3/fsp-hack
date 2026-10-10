//! Mock of the FSP participant registry. The API calls it after an FSP ID is linked
//! (docs/architecture.md section 7.1) to fetch the participant card and the achievements snapshot. For demos
//! an admin endpoint changes a participant's achievements and pushes `achievements.changed` to the
//! platform, signed the way the platform signs its own webhooks (see [`push`]).

mod model;
pub mod push;

use std::sync::Arc;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use serde::{Deserialize, Serialize};

pub use model::{
    Achievement, AchievementError, AchievementKind, DataError, Discipline, EditError, Entry,
    EventLevel, Participant, ParticipantCard, Place, Rank, RankSummary, Registry,
};
pub use push::{Push, PushReport};

pub const DEFAULT_BIND: &str = "0.0.0.0:8090";
pub const DEFAULT_TOKEN: &str = "dev-registry-token";

struct Inner {
    /// Swapped whole on an admin edit; readers never wait.
    registry: ArcSwap<Registry>,
    token: Box<str>,
    push: Option<Push>,
    http: reqwest::Client,
}

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl AppState {
    pub fn new(
        registry: Registry,
        token: impl Into<Box<str>>,
        push: Option<Push>,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self(Arc::new(Inner {
            registry: ArcSwap::from_pointee(registry),
            token: token.into(),
            push,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?,
        })))
    }

    pub fn registry(&self) -> Arc<Registry> {
        self.0.registry.load_full()
    }

    /// Applies `edit` to the current registry; a concurrent edit makes it retry on top of that one.
    fn update(
        &self,
        fsp_id: &str,
        edit: impl Fn(&mut Vec<Achievement>) -> Result<(), EditError>,
    ) -> Result<String, EditError> {
        loop {
            let current = self.0.registry.load_full();
            let (next, canonical) = current.edited(fsp_id, &edit)?;
            let previous = self.0.registry.compare_and_swap(&current, Arc::new(next));
            if Arc::ptr_eq(&previous, &current) {
                return Ok(canonical);
            }
        }
    }
}

pub fn router(state: AppState) -> Router {
    let participants = Router::new()
        .route("/participants/{fsp_id}", get(participant_card))
        .route(
            "/participants/{fsp_id}/achievements",
            get(participant_achievements),
        )
        .route(
            "/admin/participants/{fsp_id}/achievements",
            post(add_achievement),
        )
        .route(
            "/admin/participants/{fsp_id}/achievements/{achievement_id}",
            delete(remove_achievement),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), require_token));

    Router::new()
        .route("/healthz", get(healthz))
        .merge(participants)
        .fallback(not_found)
        .layer(middleware::from_fn(log_requests))
        .with_state(state)
}

async fn healthz() -> Response {
    json_response(StatusCode::OK, r#"{"status":"ok"}"#.to_owned())
}

async fn participant_card(State(state): State<AppState>, Path(fsp_id): Path<String>) -> Response {
    match state.registry().get(&fsp_id) {
        Some(entry) => json_response(StatusCode::OK, entry.card_json().to_owned()),
        None => participant_not_found(),
    }
}

async fn participant_achievements(
    State(state): State<AppState>,
    Path(fsp_id): Path<String>,
) -> Response {
    match state.registry().get(&fsp_id) {
        Some(entry) => json_response(StatusCode::OK, entry.achievements_json().to_owned()),
        None => participant_not_found(),
    }
}

/// An achievement to add; `id` is generated when absent. An empty body adds a ready-made demo
/// result: second place at a national algorithmic championship, dated today.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewAchievement {
    id: Option<String>,
    kind: AchievementKind,
    discipline: Discipline,
    #[serde(default)]
    rank: Option<Rank>,
    #[serde(default)]
    event_name: Option<String>,
    #[serde(default)]
    event_level: Option<EventLevel>,
    #[serde(default)]
    place: Option<Place>,
    #[serde(default)]
    team: Option<String>,
    #[serde(default)]
    role: Option<String>,
    date: String,
    #[serde(default)]
    protocol_url: Option<String>,
}

impl NewAchievement {
    fn demo() -> Self {
        Self {
            id: None,
            kind: AchievementKind::Competition,
            discipline: Discipline::Algorithmic,
            rank: None,
            event_name: Some(
                "Чемпионат России по спортивному программированию (алгоритмическое программирование)"
                    .into(),
            ),
            event_level: Some(EventLevel::National),
            place: Some(Place::Second),
            team: None,
            role: None,
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            protocol_url: None,
        }
    }

    fn into_achievement(self, id: String) -> Achievement {
        Achievement {
            id,
            kind: self.kind,
            discipline: self.discipline,
            rank: self.rank,
            event_name: self.event_name,
            event_level: self.event_level,
            place: self.place,
            team: self.team,
            role: self.role,
            date: self.date,
            protocol_url: self.protocol_url,
        }
    }
}

#[derive(Serialize)]
struct Changed {
    fsp_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    added: Option<Achievement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<String>,
    /// What the platform answered to the push; `null` — no webhook configured.
    push: Option<PushReport>,
}

fn edit_problem(e: &EditError) -> Response {
    match e {
        EditError::UnknownParticipant => participant_not_found(),
        EditError::UnknownAchievement => problem(
            StatusCode::NOT_FOUND,
            "achievement_not_found",
            "У участника нет такого достижения",
        ),
        EditError::DuplicateId(_) => problem(
            StatusCode::CONFLICT,
            "achievement_exists",
            "Достижение с таким id уже есть",
        ),
        EditError::Invalid(_) | EditError::Data(_) => problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_achievement",
            "Запись о достижении не прошла проверку",
        ),
    }
}

/// Admin: add an achievement and push `achievements.changed` to the platform.
async fn add_achievement(
    State(state): State<AppState>,
    Path(fsp_id): Path<String>,
    body: Bytes,
) -> Response {
    let input = if body.iter().all(u8::is_ascii_whitespace) {
        NewAchievement::demo()
    } else {
        match serde_json::from_slice::<NewAchievement>(&body) {
            Ok(a) => a,
            Err(_) => {
                return problem(
                    StatusCode::BAD_REQUEST,
                    "invalid_json",
                    "Тело — запись о достижении в формате реестра",
                );
            }
        }
    };
    let registry = state.registry();
    let id = match input.id.clone() {
        Some(id) => id,
        None => match registry.next_achievement_id(&fsp_id) {
            Some(id) => id,
            None => return participant_not_found(),
        },
    };
    let achievement = input.into_achievement(id);
    if let Err(reason) = achievement.check() {
        tracing::info!(%reason, "admin achievement rejected");
        return edit_problem(&EditError::Invalid(reason));
    }
    let canonical = match state.update(&fsp_id, |list| {
        list.push(achievement.clone());
        Ok(())
    }) {
        Ok(c) => c,
        Err(e) => return edit_problem(&e),
    };
    let push = push::achievements_changed(&state.0.http, state.0.push.as_ref(), &canonical).await;
    json_value(
        StatusCode::CREATED,
        &Changed {
            fsp_id: canonical,
            added: Some(achievement),
            removed: None,
            push,
        },
    )
}

/// Admin: remove an achievement and push `achievements.changed` to the platform.
async fn remove_achievement(
    State(state): State<AppState>,
    Path((fsp_id, achievement_id)): Path<(String, String)>,
) -> Response {
    let canonical = match state.update(&fsp_id, |list| {
        let before = list.len();
        list.retain(|a| a.id != achievement_id);
        if list.len() == before {
            Err(EditError::UnknownAchievement)
        } else {
            Ok(())
        }
    }) {
        Ok(c) => c,
        Err(e) => return edit_problem(&e),
    };
    let push = push::achievements_changed(&state.0.http, state.0.push.as_ref(), &canonical).await;
    json_value(
        StatusCode::OK,
        &Changed {
            fsp_id: canonical,
            added: None,
            removed: Some(achievement_id),
            push,
        },
    )
}

async fn not_found() -> Response {
    problem(StatusCode::NOT_FOUND, "not_found", "Ресурс не найден")
}

fn participant_not_found() -> Response {
    problem(
        StatusCode::NOT_FOUND,
        "participant_not_found",
        "Участник с таким ФСП ID не найден",
    )
}

async fn require_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if bearer_matches(req.headers(), &state.0.token) {
        return next.run(req).await;
    }
    let mut response = problem(
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        "Нужен токен доступа к реестру ФСП",
    );
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(r#"Bearer realm="fsp-registry""#),
    );
    response
}

fn bearer_matches(headers: &HeaderMap, expected: &str) -> bool {
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, token)) = value.split_once(' ') else {
        return false;
    };
    scheme.eq_ignore_ascii_case("bearer")
        && constant_time_eq(token.trim().as_bytes(), expected.as_bytes())
}

/// No early exit on the first mismatching byte, so timing does not leak a token prefix.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[derive(Serialize)]
struct Problem {
    #[serde(rename = "type")]
    kind: &'static str,
    title: &'static str,
    status: u16,
    code: &'static str,
}

fn problem(status: StatusCode, code: &'static str, title: &'static str) -> Response {
    let body = Problem {
        kind: "about:blank",
        title,
        status: status.as_u16(),
        code,
    };
    let json = serde_json::to_string(&body).unwrap_or_default();
    (
        status,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        )],
        json,
    )
        .into_response()
}

fn json_value(status: StatusCode, value: &impl Serialize) -> Response {
    json_response(status, serde_json::to_string(value).unwrap_or_default())
}

fn json_response(status: StatusCode, body: String) -> Response {
    (
        status,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        body,
    )
        .into_response()
}

async fn log_requests(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let started = Instant::now();
    let response = next.run(req).await;
    tracing::info!(
        %method,
        %path,
        status = response.status().as_u16(),
        elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
        "request"
    );
    response
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use serde_json::Value;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

    const TOKEN: &str = "test-token";

    struct Reply {
        status: u16,
        head: String,
        body: Value,
    }

    async fn spawn() -> SocketAddr {
        spawn_with(None).await
    }

    async fn spawn_with(push: Option<Push>) -> SocketAddr {
        let registry = Registry::embedded().expect("embedded data").clone();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = AppState::new(registry, TOKEN, push).unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router(state)).await.unwrap();
        });
        addr
    }

    async fn get(addr: SocketAddr, path: &str, auth: Option<&str>) -> Reply {
        request(addr, "GET", path, auth, "").await
    }

    async fn request(
        addr: SocketAddr,
        method: &str,
        path: &str,
        auth: Option<&str>,
        body: &str,
    ) -> Reply {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut request = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: {}\r\n",
            body.len()
        );
        if let Some(auth) = auth {
            request.push_str(&format!("Authorization: {auth}\r\n"));
        }
        request.push_str("\r\n");
        request.push_str(body);
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).await.unwrap();
        let text = String::from_utf8(raw).unwrap();
        let (head, body) = text.split_once("\r\n\r\n").unwrap();
        Reply {
            status: head[9..12].parse().unwrap(),
            head: head.to_ascii_lowercase(),
            body: serde_json::from_str(body).unwrap(),
        }
    }

    #[tokio::test]
    async fn healthz_needs_no_token() {
        let addr = spawn().await;
        let r = get(addr, "/healthz", None).await;
        assert_eq!(r.status, 200);
        assert_eq!(r.body["status"], "ok");
    }

    #[tokio::test]
    async fn participants_require_the_registry_token() {
        let addr = spawn().await;
        for auth in [
            None,
            Some("Bearer wrong-token"),
            Some("Basic dGVzdC10b2tlbg=="),
        ] {
            let r = get(addr, "/participants/FSP-000001", auth).await;
            assert_eq!(r.status, 401, "auth = {auth:?}");
            assert!(r.head.contains("www-authenticate: bearer"));
            assert!(r.head.contains("content-type: application/problem+json"));
            assert_eq!(r.body["code"], "unauthorized");
        }
        let unknown = get(addr, "/participants/FSP-999999", None).await;
        assert_eq!(
            unknown.status, 401,
            "existence is not revealed without a token"
        );
    }

    #[tokio::test]
    async fn serves_card_and_achievements() {
        let addr = spawn().await;
        let auth = Some("Bearer test-token");

        let card = get(addr, "/participants/FSP-000003", auth).await;
        assert_eq!(card.status, 200);
        assert_eq!(card.body["fsp_id"], "FSP-000003");
        assert_eq!(card.body["full_name"], "Иванов Дмитрий Сергеевич");
        assert_eq!(card.body["region"], "Новосибирская область");
        assert_eq!(card.body["ranks"][0]["discipline"], "security");
        assert_eq!(card.body["ranks"][0]["rank"], "cms");

        let list = get(
            addr,
            "/participants/fsp-000003/achievements",
            Some("bearer test-token"),
        )
        .await;
        assert_eq!(list.status, 200);
        assert_eq!(list.body["fsp_id"], "FSP-000003");
        let achievements = list.body["achievements"].as_array().unwrap();
        assert_eq!(achievements.len(), 8);
        assert_eq!(achievements[0]["date"], "2025-08-30");
        assert_eq!(achievements[0]["event_level"], "international");

        let empty = get(addr, "/participants/FSP-000010/achievements", auth).await;
        assert_eq!(empty.status, 200);
        assert_eq!(empty.body["achievements"], Value::Array(vec![]));
    }

    #[tokio::test]
    async fn unknown_participant_is_404() {
        let addr = spawn().await;
        let auth = Some("Bearer test-token");
        for path in [
            "/participants/FSP-999999",
            "/participants/FSP-999999/achievements",
            "/participants/not-an-id",
        ] {
            let r = get(addr, path, auth).await;
            assert_eq!(r.status, 404, "{path}");
            assert_eq!(r.body["code"], "participant_not_found");
        }
        let other = get(addr, "/nope", None).await;
        assert_eq!(other.status, 404);
        assert_eq!(other.body["code"], "not_found");
    }

    /// The platform's side of a push, reduced to what the test checks: the signed body.
    async fn receiver() -> (
        String,
        tokio::sync::mpsc::UnboundedReceiver<(String, String)>,
    ) {
        use axum::routing::post;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let app = Router::new().route(
            "/events",
            post(move |headers: HeaderMap, body: String| {
                let tx = tx.clone();
                async move {
                    let signature = headers
                        .get("fsp-signature")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default()
                        .to_owned();
                    let _ = tx.send((signature, body));
                    StatusCode::ACCEPTED
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/events", listener.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (url, rx)
    }

    #[tokio::test]
    async fn admin_edits_change_the_snapshot_and_push_a_signed_event() {
        let (url, mut pushed) = receiver().await;
        let addr = spawn_with(Some(Push {
            url,
            secret: "whsec_registry".into(),
        }))
        .await;
        let auth = Some("Bearer test-token");
        let path = "/admin/participants/fsp-000011/achievements";
        let refused = request(addr, "POST", path, None, "").await;
        assert_eq!(refused.status, 401, "admin needs the registry token");
        let before = get(addr, "/participants/FSP-000011/achievements", auth).await;
        let count = before.body["achievements"].as_array().unwrap().len();

        let added = request(addr, "POST", path, auth, "").await;
        assert_eq!(added.status, 201, "{}", added.body);
        assert_eq!(added.body["fsp_id"], "FSP-000011");
        assert_eq!(added.body["added"]["event_level"], "national");
        assert_eq!(added.body["push"]["status"], 202);
        let id = added.body["added"]["id"].as_str().unwrap().to_owned();
        let after = get(addr, "/participants/FSP-000011/achievements", auth).await;
        let list = after.body["achievements"].as_array().unwrap();
        assert_eq!(list.len(), count + 1);
        assert!(list.iter().any(|a| a["id"] == id.as_str()));

        let (header, body) = pushed.recv().await.unwrap();
        let event: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(event["type"], "achievements.changed");
        assert_eq!(event["fsp_id"], "FSP-000011");
        assert_eq!(event["event_id"], added.body["push"]["event_id"]);
        let t: i64 = header
            .strip_prefix("t=")
            .and_then(|h| h.split(',').next())
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(
            push::signature("whsec_registry", t, body.as_bytes()).as_deref(),
            Some(header.as_str())
        );

        let custom = r#"{"kind":"rank","discipline":"security","rank":"cms","date":"2026-09-30"}"#;
        let r = request(addr, "POST", path, auth, custom).await;
        assert_eq!(r.status, 201, "{}", r.body);
        let bad = r#"{"kind":"rank","discipline":"security","date":"2026-09-30"}"#;
        let r = request(addr, "POST", path, auth, bad).await;
        assert_eq!(
            (r.status, r.body["code"].as_str()),
            (422, Some("invalid_achievement"))
        );
        let r = request(
            addr,
            "POST",
            "/admin/participants/FSP-999999/achievements",
            auth,
            "",
        )
        .await;
        assert_eq!(r.status, 404);

        let removed = request(addr, "DELETE", &format!("{path}/{id}"), auth, "").await;
        assert_eq!(removed.status, 200, "{}", removed.body);
        assert_eq!(removed.body["removed"], id.as_str());
        pushed.recv().await.unwrap();
        pushed.recv().await.unwrap();
        let again = request(addr, "DELETE", &format!("{path}/{id}"), auth, "").await;
        assert_eq!(again.body["code"], "achievement_not_found");
    }

    #[tokio::test]
    async fn without_a_webhook_edits_apply_and_report_no_push() {
        let addr = spawn().await;
        let r = request(
            addr,
            "POST",
            "/admin/participants/FSP-000012/achievements",
            Some("Bearer test-token"),
            "",
        )
        .await;
        assert_eq!(r.status, 201);
        assert!(r.body["push"].is_null());
    }

    #[test]
    fn constant_time_eq_compares_whole_input() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"a"));
    }
}
