//! `POST /api/v1/eval/pairs` over real HTTP. The endpoint never touches the database, so the
//! pool is lazy and points nowhere.

use std::sync::Arc;

use api::{
    auth::jwt::JwtKeys,
    config::{Config, FspConfig, MailConfig, ProctoringConfig},
    crypto::Crypto,
    kv::Kv,
    mail::Mailer,
    proctoring::{ProctoringMode, RecordingStore},
    state::AppState,
};
use reqwest::{StatusCode, header};
use serde_json::{Value, json};

async fn start() -> String {
    let config = Config {
        bind: ([127, 0, 0, 1], 0).into(),
        database_url: String::new(),
        valkey_url: None,
        public_url: "http://localhost:3000".into(),
        cookie_secure: false,
        jwt_seed: Some([7; 32]),
        field_key: [1; 32],
        bidx_key: [2; 32],
        mail: MailConfig::Log,
        fsp: FspConfig {
            oidc: None,
            registry_url: None,
            registry_token: "test".into(),
            dev_link: false,
        },
        grade_change_cooldown_days: 90,
        proctoring: ProctoringConfig {
            mode: ProctoringMode::Off,
            dir: std::env::temp_dir().join("eval-pairs-test"),
            retention_days: 90,
        },
        seed_demo: false,
        log_json: false,
    };
    let state = AppState {
        crypto: Arc::new(Crypto::new(&config.field_key, &config.bidx_key)),
        jwt: Arc::new(JwtKeys::new(config.jwt_seed).unwrap()),
        mailer: Arc::new(Mailer::new(&config.mail).unwrap()),
        kv: Kv::connect(None).await.unwrap(),
        http: reqwest::Client::new(),
        metrics: None,
        recordings: Arc::new(RecordingStore::new(config.proctoring.dir.clone())),
        pool: api::db::pool("postgres://nobody@127.0.0.1:1/none").unwrap(),
        config: Arc::new(config),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!(
        "http://{}/api/v1/eval/pairs",
        listener.local_addr().unwrap()
    );
    let app = api::app(state);
    tokio::spawn(async move { axum::serve(listener, app).await });
    base
}

async fn post(url: &str, ip: &str, body: String) -> (StatusCode, header::HeaderMap, Value) {
    let res = reqwest::Client::new()
        .post(url)
        .header("x-forwarded-for", ip)
        .header(header::CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let body = serde_json::from_slice(&res.bytes().await.unwrap()).unwrap_or(Value::Null);
    (status, headers, body)
}

fn small() -> Value {
    json!({
        "now": "2026-10-01",
        "k": 3,
        "vacancies": [{
            "id": "go",
            "text": "Go-разработчик (Middle)\nТребования: Go, PostgreSQL, Kafka.\nУдалённо, 250–330 тыс. ₽."
        }],
        "candidates": [
            {"id": "c1", "resume_text": "Go-разработчик\nОпыт работы — 4 года\nGo, PostgreSQL, Kafka, Redis."},
            {"id": "c2", "resume_text": "PHP-разработчик\nОпыт работы — 4 года\nLaravel, MySQL, Go.",
             "specialization": "backend", "grade": "middle", "test_result": 40},
            {"id": "c3", "resume_text": "Бухгалтер, 1С"}
        ],
        "labels": [
            {"vacancy_id": "go", "candidate_id": "c1", "relevance": 3},
            {"vacancy_id": "go", "candidate_id": "c2", "relevance": 0}
        ]
    })
}

#[tokio::test]
async fn ranks_explains_and_scores_a_small_set() {
    let url = start().await;
    let (status, _, r) = post(&url, "10.0.0.1", small().to_string()).await;
    assert_eq!(status, StatusCode::OK, "{r}");
    let v = &r["vacancies"][0];
    assert_eq!(v["top"][0]["candidate_id"], "c1");
    let notes = v["top"][0]["notes"].as_array().unwrap();
    assert!(
        notes
            .iter()
            .any(|n| n.as_str().unwrap().contains("не подтверждена тестом")),
        "{notes:?}"
    );
    assert_eq!(r["metrics"]["vacancies"], 1);
    assert_eq!(r["metrics"]["ours"]["reciprocal_rank"]["mean"], 1.0);
    assert!(r["metrics"]["baseline"]["ndcg_at_k"]["mean"].is_number());
    assert_eq!(r["coverage"]["candidates_with_test"], 1);
}

#[tokio::test]
async fn body_over_the_cap_is_413_problem() {
    let url = start().await;
    let text = "а".repeat(api::routes::eval::BODY_LIMIT / 2 + 16);
    let body = json!({"vacancies": [], "candidates": [{"id": "x", "resume_text": text}]});
    let (status, headers, r) = post(&url, "10.0.0.2", body.to_string()).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(headers[header::CONTENT_TYPE], "application/problem+json");
    assert_eq!(r["code"], "payload_too_large");
}

#[tokio::test]
async fn item_limits_and_bad_input_are_422() {
    let url = start().await;
    let mut many = small();
    many["vacancies"] = Value::Array(
        (0..201)
            .map(|i| json!({"id": format!("v{i}"), "text": "Go"}))
            .collect(),
    );
    let (status, _, r) = post(&url, "10.0.0.3", many.to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(r["code"], "validation_failed");
    assert!(
        r["errors"]["vacancies"][0]
            .as_str()
            .unwrap()
            .contains("200")
    );

    let mut long = small();
    long["candidates"][0]["resume_text"] = Value::String("я".repeat(20_001));
    let (status, _, r) = post(&url, "10.0.0.3", long.to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(r["errors"]["candidates"].is_array(), "{r}");

    let mut unknown = small();
    unknown["labels"][0]["candidate_id"] = "nobody".into();
    let (status, _, r) = post(&url, "10.0.0.3", unknown.to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(r["errors"]["labels"].is_array(), "{r}");

    let mut typo = small();
    typo["vacancies"][0]["grade"] = "middle".into();
    let (status, _, r) = post(&url, "10.0.0.3", typo.to_string()).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(r["code"], "invalid_json");
    assert!(r["detail"].as_str().unwrap().contains("grade"), "{r}");

    let (status, _, r) = post(&url, "10.0.0.3", "{".into()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(r["code"], "invalid_json");
}

#[tokio::test]
async fn rate_limited_per_ip() {
    let url = start().await;
    for _ in 0..10 {
        let (status, _, _) = post(&url, "10.0.0.4", "{}".into()).await;
        assert_ne!(status, StatusCode::TOO_MANY_REQUESTS);
    }
    let (status, headers, r) = post(&url, "10.0.0.4", "{}".into()).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(r["code"], "rate_limited");
    assert!(headers.contains_key(header::RETRY_AFTER));
    let (status, _, _) = post(&url, "10.0.0.5", small().to_string()).await;
    assert_eq!(status, StatusCode::OK);
}
