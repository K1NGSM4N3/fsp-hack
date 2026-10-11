//! Public offline evaluation on the caller's own "vacancy – candidate" set (`matching::pairs`), so
//! the jury can run its data through the production matching straight from Swagger. Reads and
//! stores nothing: no database, no user data.

use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use embedder::TextKind;
use matching::pairs::{PairsInput, PairsReport, PairsVectors, TextSource};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::client_ip,
    embeddings::{EmbedError, EmbedderClient},
    error::{ApiError, ApiResult, Problem},
    extract::Json,
    state::AppState,
};

pub const BODY_LIMIT: usize = 8 * 1024 * 1024;
const RATE_LIMIT: u64 = 10;
const RATE_WINDOW: Duration = Duration::from_secs(600);
/// Texts (vacancies + resumes) a request embeds; a larger set is ranked by keywords here and by
/// meaning in the CLI, so one request does not hold the embedder for minutes.
pub const MEANING_TEXTS: usize = 300;
const MEANING_DEADLINE: Duration = Duration::from_secs(60);

/// Why a set is ranked by keywords, or `None` when it may be embedded.
pub fn keyword_reason(embeddings_on: bool, texts: usize) -> Option<String> {
    if !embeddings_on {
        return Some("эмбеддинги выключены (EMBEDDINGS=off)".to_owned());
    }
    (texts > MEANING_TEXTS).then(|| {
        format!(
            "в наборе {texts} текстов, по смыслу API считает до {MEANING_TEXTS}; весь набор — через CLI `validate pairs`"
        )
    })
}

/// Why a set meant for meaning fell back to keywords; the message goes into the report.
#[derive(Debug, thiserror::Error)]
enum MeaningError {
    #[error("эмбеддер сменил модель посреди запроса")]
    ModelChanged,
    #[error("эмбеддер недоступен: {0}")]
    Embedder(#[from] EmbedError),
    #[error("эмбеддер не ответил за {} с", MEANING_DEADLINE.as_secs())]
    Timeout,
}

async fn embed_pairs(
    client: &EmbedderClient,
    input: &PairsInput,
) -> Result<PairsVectors, MeaningError> {
    let (vacancies, candidates) = matching::pairs::texts(input);
    let run = async {
        let v = client.embed(&vacancies, TextKind::Query).await?;
        let c = client.embed(&candidates, TextKind::Passage).await?;
        Ok::<_, EmbedError>((v, c))
    };
    match tokio::time::timeout(MEANING_DEADLINE, run).await {
        Ok(Ok((v, c))) if v.model == c.model || vacancies.is_empty() || candidates.is_empty() => {
            Ok(PairsVectors {
                model: if v.model.is_empty() { c.model } else { v.model },
                vacancies: v.vectors.into_iter().map(Some).collect(),
                candidates: c.vectors.into_iter().map(Some).collect(),
            })
        }
        Ok(Ok(_)) => Err(MeaningError::ModelChanged),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err(MeaningError::Timeout),
    }
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(eval_pairs))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
}

/// The body is read only after the rate limit; an oversized one points to the CLI.
fn rejected(e: ApiError) -> ApiError {
    match e {
        ApiError::Status {
            code: "payload_too_large",
            ..
        } => ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "Тело запроса больше 8 МиБ",
        )
        .with_detail(
            "Большие наборы проверяйте через CLI: cargo run -p validate --release -- pairs",
        ),
        other => other,
    }
}

fn example() -> serde_json::Value {
    serde_json::json!({
        "now": "2026-10-01",
        "k": 3,
        "vacancies": [
            {
                "id": "go-middle",
                "text": "Go-разработчик (Middle)\nПлатёжный сервис.\nТребования: Go, PostgreSQL, Kafka.\nБудет плюсом: Kubernetes.\nУдалённо, 250–330 тыс. ₽."
            },
            {
                "id": "qa-auto",
                "text": "QA-инженер по автоматизации",
                "specialization": "qa",
                "grades": ["junior", "middle"],
                "must_skills": ["python", "pytest", "postman", "sql"],
                "salary_max": 200000,
                "work_formats": ["remote"]
            }
        ],
        "candidates": [
            {
                "id": "c1",
                "resume_text": "Go-разработчик\nМосква, удалённо, 300 000 ₽\nОпыт работы — 4 года\nСервис платежей на Go: PostgreSQL, Kafka, Redis, Kubernetes."
            },
            {
                "id": "c2",
                "resume_text": "Backend-разработчик\nОпыт работы — 3 года\nGo, PostgreSQL, RabbitMQ, gRPC.",
                "specialization": "backend",
                "grade": "middle",
                "test_result": 78,
                "salary_expectation": 280000
            },
            {
                "id": "c3",
                "resume_text": "QA-инженер (тестирование API)\nОпыт работы — 2 года 6 месяцев\nPython, pytest, Postman, SQL, тест-дизайн.\nУдалённо, 180 000 ₽"
            },
            {
                "id": "c4",
                "resume_text": "Начинающий тестировщик\nБез опыта работы, курсы 2026.\nPython, pytest, Selenium, SQL, Postman, Go, Kafka."
            }
        ],
        "labels": [
            {"vacancy_id": "go-middle", "candidate_id": "c1", "relevance": 3},
            {"vacancy_id": "go-middle", "candidate_id": "c2", "relevance": 2},
            {"vacancy_id": "go-middle", "candidate_id": "c4", "relevance": 0},
            {"vacancy_id": "qa-auto", "candidate_id": "c3", "relevance": 3},
            {"vacancy_id": "qa-auto", "candidate_id": "c4", "relevance": 1}
        ]
    })
}

/// Evaluate the matching on your own vacancies, candidates and labels
#[utoipa::path(post, path = "/eval/pairs", tag = "Валидация",
    description = "Прогоняет ваш набор «вакансия – кандидат» через тот же разбор текстов и тот же скоринг, \
        что работают в подборе, и через базовую линию «ключевые слова резюме + заявленный стаж». \
        Структурные поля необязательны: специализация, грейд, навыки, вилка, формат и город распознаются из текстов. \
        Категория, распознанная из резюме, помечается «не подтверждена тестом»; без результата теста берётся нейтральное значение — \
        середина диапазона грейда, об этом сказано в объяснении. С разметкой (`labels`, релевантность 0–3, релевантен ≥ 2) \
        считаются Precision@5, Precision@k, nDCG@k, MAP и MRR по каждой вакансии и в среднем — для нашего ранжирования и для базовой линии. \
        Близость описаний — по смыслу (эмбеддинги, посчитанные локальной моделью), если набор не больше 300 текстов \
        и эмбеддер доступен; иначе по ключевым словам, и поле `text` отчёта говорит почему. \
        Лимиты: тело до 8 МиБ, до 200 вакансий, 5 000 кандидатов и 100 000 оценок, тексты до 20 000 символов, \
        10 запросов за 10 минут с одного IP. Данные не сохраняются. Большие наборы — CLI `validate pairs` (см. README.md, раздел 6.4).",
    request_body(content = PairsInput, example = example),
    responses(
        (status = 200, body = PairsReport),
        (status = 400, description = "`invalid_json` — синтаксическая ошибка JSON", body = Problem),
        (status = 413, description = "`payload_too_large` — тело больше 8 МиБ", body = Problem),
        (status = 415, description = "`unsupported_media_type`", body = Problem),
        (status = 422, description = "`invalid_json` — поля не по схеме; `validation_failed` — превышены лимиты или ошибка в данных (поле в `errors`)", body = Problem),
        (status = 429, description = "`rate_limited`", body = Problem),
    ))]
async fn eval_pairs(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<PairsInput>, ApiError>,
) -> ApiResult<Json<PairsReport>> {
    let ip = client_ip(&headers).unwrap_or_default();
    let hit = state.kv.hit(&format!("rl:eval:{ip}"), RATE_WINDOW).await?;
    if hit.over(RATE_LIMIT) {
        return Err(ApiError::rate_limited("eval_pairs", hit.retry_after));
    }
    let Json(input) = body.map_err(rejected)?;
    input.check()?;
    let now = Utc::now().timestamp();
    let texts = input.vacancies.len() + input.candidates.len();
    let vectors = match (
        &state.embedder,
        keyword_reason(state.embedder.is_some(), texts),
    ) {
        (Some(client), None) => embed_pairs(client, &input).await.map_err(|e| e.to_string()),
        (_, Some(reason)) => Err(reason),
        (None, None) => Err("эмбеддинги выключены (EMBEDDINGS=off)".to_owned()),
    };
    let report = tokio::task::spawn_blocking(move || {
        let source = match &vectors {
            Ok(v) => TextSource::Meaning(v),
            Err(reason) => TextSource::Keywords(reason.clone()),
        };
        matching::pairs::evaluate(&input, now, source)
    })
    .await??;
    Ok(Json(report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_sets_are_embedded_and_the_rest_says_why_not() {
        assert_eq!(keyword_reason(true, 65), None);
        assert_eq!(keyword_reason(true, MEANING_TEXTS), None);
        assert!(
            keyword_reason(true, MEANING_TEXTS + 1).is_some_and(|r| r.contains("validate pairs"))
        );
        assert!(keyword_reason(false, 5).is_some_and(|r| r.contains("EMBEDDINGS=off")));
    }

    #[test]
    fn an_oversized_set_is_sent_to_the_cli_and_other_errors_pass_through() {
        let big = rejected(ApiError::payload_too_large());
        assert_eq!(big.summary().0, 413);
        assert_eq!(big.summary().1, "payload_too_large");
        let ApiError::Status { detail, .. } = big else {
            panic!("a status problem")
        };
        assert!(detail.is_some_and(|d| d.contains("cargo run -p validate --release -- pairs")));
        let field = rejected(ApiError::field("vacancies", "не больше 200"));
        assert_eq!(
            field.summary(),
            (422, "validation_failed", vec!["vacancies"])
        );
    }

    #[test]
    fn pairs_limits_are_field_errors() {
        let e = ApiError::from(matching::pairs::PairsError::TooMany {
            list: "vacancies",
            got: 201,
            max: 200,
        });
        assert_eq!(e.summary(), (422, "validation_failed", vec!["vacancies"]));
        let Err(json) = crate::extract::Json::<matching::pairs::PairsInput>::from_bytes(
            br#"{"vacancies":[{"id":"v","text":"Go","grade":"middle"}],"candidates":[]}"#,
        ) else {
            panic!("an unknown field is refused")
        };
        assert_eq!(json.summary().1, "invalid_json");
        assert_eq!(json.summary().0, 422);
        let Err(syntax) = crate::extract::Json::<matching::pairs::PairsInput>::from_bytes(b"{")
        else {
            panic!("broken JSON is refused")
        };
        assert_eq!(syntax.summary().0, 400);
    }
}
