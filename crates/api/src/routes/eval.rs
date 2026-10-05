//! Public offline evaluation on the caller's own "vacancy – candidate" set (`matching::pairs`), so
//! the jury can run its data through the production matching straight from Swagger. Reads and
//! stores nothing: no database, no user data.

use std::time::Duration;

use axum::{
    Json,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
};
use chrono::Utc;
use matching::pairs::{PairsInput, PairsReport};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::client_ip,
    error::{ApiError, ApiResult, Problem},
    state::AppState,
};

pub const BODY_LIMIT: usize = 8 * 1024 * 1024;
const RATE_LIMIT: u64 = 10;
const RATE_WINDOW: Duration = Duration::from_secs(600);

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(eval_pairs))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
}

fn rejected(r: JsonRejection) -> ApiError {
    let status = r.status();
    match status {
        StatusCode::PAYLOAD_TOO_LARGE => {
            ApiError::new(status, "payload_too_large", "Тело запроса больше 8 МиБ").with_detail(
                "Большие наборы проверяйте через CLI: cargo run -p validate --release -- pairs",
            )
        }
        StatusCode::UNSUPPORTED_MEDIA_TYPE => ApiError::new(
            status,
            "unsupported_media_type",
            "Нужен Content-Type: application/json",
        ),
        _ => ApiError::new(status, "invalid_json", "Некорректный JSON").with_detail(r.body_text()),
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
        Лимиты: тело до 8 МиБ, до 200 вакансий, 5 000 кандидатов и 100 000 оценок, тексты до 20 000 символов, \
        10 запросов за 10 минут с одного IP. Данные не сохраняются. Большие наборы — CLI `validate pairs` (см. docs/solution.md §6.4).",
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
    body: Result<Json<PairsInput>, JsonRejection>,
) -> ApiResult<Json<PairsReport>> {
    let ip = client_ip(&headers).unwrap_or_default();
    if state.kv.hit(&format!("rl:eval:{ip}"), RATE_WINDOW).await? > RATE_LIMIT {
        return Err(ApiError::too_many_requests());
    }
    let Json(input) = body.map_err(rejected)?;
    input.check()?;
    let now = Utc::now().timestamp();
    let report =
        tokio::task::spawn_blocking(move || matching::pairs::evaluate(&input, now)).await??;
    Ok(Json(report))
}
