//! Rules over real HTTP against a throwaway database: an abandoned test counts as a failed
//! attempt, the up-test waits out a recent failure at its grade, request errors are
//! problem+json with the limiter's real `Retry-After`, and FSP results confirm skills until the
//! candidate hides or unlinks them.

use std::sync::Arc;

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, ProctoringConfig},
    crypto::Crypto,
    integrations::webhook::WebhookSender,
    kv::Kv,
    mail::Mailer,
    proctoring::{ProctoringMode, RecordingStore},
    state::{AppState, Services},
};
use catalog::{Grade, Specialization};
use chrono::{DateTime, Duration, Utc};
use reqwest::{Method, StatusCode, header};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const TERMS: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);

struct Api {
    base: String,
    http: reqwest::Client,
    state: AppState,
}

struct Reply {
    status: StatusCode,
    headers: header::HeaderMap,
    body: Value,
}

impl Reply {
    fn problem(&self, status: StatusCode, code: &str) {
        assert_eq!(self.status, status, "{}", self.body);
        assert_eq!(
            self.headers[header::CONTENT_TYPE],
            "application/problem+json"
        );
        assert_eq!(self.body["code"], code, "{}", self.body);
        assert_eq!(self.body["status"], status.as_u16());
        assert!(!self.body["title"].as_str().unwrap_or("").is_empty());
    }
}

impl Api {
    async fn start(pool: PgPool, registry_url: Option<String>) -> Self {
        let config = Config {
            bind: ([127, 0, 0, 1], 0).into(),
            database_url: String::new(),
            database_pool_size: api::db::POOL_SIZE,
            valkey_url: None,
            public_url: "http://localhost:3000".into(),
            cookie_secure: false,
            jwt_seed: Some([7; 32]),
            field_key: [1; 32],
            bidx_key: [2; 32],
            mail: MailConfig::Log,
            fsp: FspConfig {
                oidc: None,
                registry_url,
                registry_token: "test".into(),
                dev_link: true,
            },
            grade_change_cooldown_days: 90,
            proctoring: ProctoringConfig {
                mode: ProctoringMode::Optional,
                dir: std::env::temp_dir().join(format!("rules-test-{}", Uuid::now_v7())),
                retention_days: 90,
            },
            integrations: IntegrationsConfig::default(),
            seed_demo: false,
            seed_per_spec: 40,
            log_json: false,
        };
        let kv = Kv::connect(None).await.unwrap();
        let state = AppState::from(Services {
            crypto: Arc::new(Crypto::new(&config.field_key, &config.bidx_key)),
            jwt: Arc::new(JwtKeys::new(config.jwt_seed).unwrap()),
            mailer: Arc::new(Mailer::new(&config.mail).unwrap()),
            kv: kv.clone(),
            candidates: Arc::new(api::cache::CandidateCache::new(kv)),
            http: reqwest::Client::new(),
            metrics: None,
            recordings: Arc::new(RecordingStore::new(config.proctoring.dir.clone())),
            webhooks: WebhookSender::new(Vec::new()).unwrap(),
            pool,
            config: Arc::new(config),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let app = api::app(state.clone());
        tokio::spawn(async move { axum::serve(listener, app).await });
        Self {
            base,
            http: reqwest::Client::new(),
            state,
        }
    }

    /// A hand-written HTTP/1.1 request; the answer as text, up to the first 4 KiB.
    async fn raw(&self, request: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let addr = self.base.trim_start_matches("http://");
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut buf = vec![0u8; 4096];
        let mut read = 0;
        while read < buf.len() {
            match stream.read(&mut buf[read..]).await {
                Ok(0) | Err(_) => break,
                Ok(n) => read += n,
            }
            if String::from_utf8_lossy(&buf[..read]).contains("\"code\"") {
                break;
            }
        }
        String::from_utf8_lossy(&buf[..read]).into_owned()
    }

    fn pool(&self) -> &PgPool {
        &self.state.pool
    }

    fn token(&self, id: Uuid, role: Role) -> String {
        self.state
            .jwt
            .issue(id, role, Utc::now().timestamp())
            .unwrap()
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Reply {
        let res = req.send().await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.bytes().await.unwrap();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()))
        };
        Reply {
            status,
            headers,
            body,
        }
    }

    fn request(&self, method: Method, path: &str, token: Option<&str>) -> reqwest::RequestBuilder {
        let req = self
            .http
            .request(method, format!("{}/api/v1{path}", self.base));
        match token {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    async fn call(&self, method: Method, path: &str, token: &str, body: Option<Value>) -> Reply {
        let mut req = self.request(method, path, Some(token));
        if let Some(b) = body {
            req = req.json(&b);
        }
        self.send(req).await
    }

    async fn account(&self, role: Role, email: &str) -> Uuid {
        let id = Uuid::now_v7();
        db::users::insert_user(
            self.pool(),
            id,
            role.into(),
            &self.state.crypto.encrypt_str(email).unwrap(),
            &self.state.crypto.blind_index(email),
            "h",
        )
        .await
        .unwrap();
        id
    }

    /// A candidate with every consent; with a category when `grade` is given, otherwise only
    /// the backend survey.
    async fn candidate(&self, email: &str, grade: Option<&str>) -> Uuid {
        let id = self.account(Role::Candidate, email).await;
        db::candidates::insert_candidate(self.pool(), id, &format!("K-{}", &email[..6]))
            .await
            .unwrap();
        match grade {
            Some(g) => {
                db::grades::set_candidate_category(self.pool(), "backend", g, 0.4, Utc::now(), id)
                    .await
                    .unwrap();
            }
            None => {
                db::survey::insert_survey(
                    self.pool(),
                    &db::survey::NewSurvey {
                        id: Uuid::now_v7(),
                        candidate_id: id,
                        direction: "development",
                        specialization: "backend",
                        claimed_grade: "middle",
                        skills: &[],
                        industries: &[],
                        experience_years: None,
                        self_assessment: &json!({}),
                    },
                )
                .await
                .unwrap();
            }
        }
        for doc in [PROCESSING, PUBLICATION, TERMS] {
            db::consents::insert_user_consent(self.pool(), Uuid::now_v7(), id, doc, None, None)
                .await
                .unwrap();
        }
        self.state.candidates.invalidate().await;
        id
    }

    async fn employer(&self, email: &str) -> Uuid {
        let owner = self.account(Role::Employer, email).await;
        db::companies::insert_company(self.pool(), Uuid::now_v7(), owner, "Рога и копыта")
            .await
            .unwrap();
        owner
    }

    /// A backend session whose current item was due at `deadline`.
    async fn session(
        &self,
        candidate: Uuid,
        grade: Grade,
        kind: &str,
        deadline: DateTime<Utc>,
    ) -> Uuid {
        let state = assessment::start(Specialization::Backend, grade, 7, Vec::new());
        let item = assessment::next_item(&state).unwrap();
        let id = Uuid::now_v7();
        db::assessment::insert_session(
            self.pool(),
            &db::assessment::NewSession {
                id,
                candidate_id: candidate,
                specialization: "backend",
                claimed_grade: grade.slug(),
                attempt_kind: kind,
                state: &serde_json::to_value(&state).unwrap(),
                current_item: &serde_json::to_value(&item).unwrap(),
                deadline,
                shown_at: deadline - Duration::minutes(3),
            },
        )
        .await
        .unwrap();
        id
    }

    /// A session that ended at `at`; a pass assigns the grade.
    async fn finished(
        &self,
        candidate: Uuid,
        grade: Grade,
        kind: &str,
        passed: bool,
        at: DateTime<Utc>,
    ) {
        let id = self.session(candidate, grade, kind, at).await;
        let state = assessment::start(Specialization::Backend, grade, 7, Vec::new());
        db::assessment::finish_session(
            self.pool(),
            id,
            &db::assessment::FinishedSession {
                status: if passed {
                    db::types::AssessmentStatus::Passed
                } else {
                    db::types::AssessmentStatus::Failed
                },
                state: &serde_json::to_value(&state).unwrap(),
                theta: if passed { -0.1 } else { -0.4 },
                se: 0.25,
                measured_grade: Some("junior"),
                confident_above_next: passed,
                person_fit_lz: 0.0,
                aberrant: false,
                items_count: 12,
                finished_at: at,
            },
        )
        .await
        .unwrap();
        if passed {
            db::grades::insert_grade_assignment(
                self.pool(),
                &db::grades::NewGradeAssignment {
                    id: Uuid::now_v7(),
                    candidate_id: candidate,
                    specialization: "backend",
                    grade: grade.slug(),
                    theta: -0.1,
                    se: 0.25,
                    session_id: Some(id),
                    reason: "placement",
                    assigned_at: at,
                },
            )
            .await
            .unwrap();
            db::grades::set_candidate_category(
                self.pool(),
                "backend",
                grade.slug(),
                -0.1,
                at,
                candidate,
            )
            .await
            .unwrap();
        }
    }
}

/// Unix seconds: the grade policy works in whole seconds.
fn secs(v: &Value) -> i64 {
    v.as_str()
        .unwrap()
        .parse::<DateTime<Utc>>()
        .unwrap()
        .timestamp()
}

fn offers(body: &Value, key: &str) -> Vec<(String, String)> {
    body[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            (
                o["grade"].as_str().unwrap().to_owned(),
                o["kind"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn locked<'a>(body: &'a Value, grade: &str) -> &'a Value {
    body["locked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["grade"] == grade)
        .unwrap_or_else(|| panic!("no locked {grade}: {body}"))
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn abandoned_test_counts_as_a_failed_attempt(pool: PgPool) {
    let api = Api::start(pool, None).await;
    let cand = api.candidate("anna@example.ru", None).await;
    let ct = api.token(cand, Role::Candidate);
    let deadline = Utc::now() - Duration::hours(2);
    let abandoned = api
        .session(cand, Grade::Middle, "placement", deadline)
        .await;
    let closed_at = deadline + Duration::minutes(30);

    let c = api
        .call(Method::GET, "/candidate/category", &ct, None)
        .await;
    assert_eq!(c.status, StatusCode::OK, "{}", c.body);
    assert_eq!(
        offers(&c.body, "available"),
        [("junior".to_owned(), "placement_step_down".to_owned())],
        "the step down stays immediate"
    );
    let middle = locked(&c.body, "middle");
    assert_eq!(middle["reason"], "retry_blocked");
    assert_eq!(
        secs(&middle["available_at"]),
        (closed_at + Duration::days(30)).timestamp()
    );
    assert_eq!(locked(&c.body, "senior")["reason"], "retry_blocked");

    let s = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{abandoned}"),
            &ct,
            None,
        )
        .await;
    assert_eq!(s.body["status"], "expired");
    assert_eq!(
        secs(&s.body["finished_at"]),
        closed_at.timestamp(),
        "closed by the rule, not by the read"
    );

    let start = |grade: &'static str| {
        api.call(
            Method::POST,
            "/candidate/assessments",
            &ct,
            Some(json!({ "specialization": "backend", "grade": grade })),
        )
    };
    let r = start("middle").await;
    r.problem(StatusCode::CONFLICT, "retry_blocked");
    let when = (closed_at + Duration::days(30))
        .format("%d.%m.%Y")
        .to_string();
    assert!(
        r.body["detail"].as_str().unwrap().contains(&when),
        "{}",
        r.body
    );
    let r = start("junior").await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.body["attempt_kind"], "placement_step_down");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn answer_after_the_abandon_window_closes_the_test(pool: PgPool) {
    let api = Api::start(pool, None).await;
    let cand = api.candidate("boris@example.ru", None).await;
    let ct = api.token(cand, Role::Candidate);
    let deadline = Utc::now() - Duration::minutes(45);
    let id = api
        .session(cand, Grade::Middle, "placement", deadline)
        .await;
    let r = api
        .call(
            Method::POST,
            &format!("/candidate/assessments/{id}/answer"),
            &ct,
            Some(json!({ "seq": 0, "answer": { "type": "choice", "value": 0 } })),
        )
        .await;
    r.problem(StatusCode::CONFLICT, "assessment_finished");
    let s = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{id}"),
            &ct,
            None,
        )
        .await;
    assert_eq!(s.body["status"], "expired");
    assert_eq!(s.body["answered"], 0);

    // Inside the window the late item is graded wrong and the test goes on.
    let cand = api.candidate("vera00@example.ru", None).await;
    let ct = api.token(cand, Role::Candidate);
    let id = api
        .session(
            cand,
            Grade::Middle,
            "placement",
            Utc::now() - Duration::minutes(10),
        )
        .await;
    let s = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{id}/current-item"),
            &ct,
            None,
        )
        .await;
    assert_eq!(s.status, StatusCode::OK, "{}", s.body);
    assert_eq!(s.body["status"], "in_progress");
    assert_eq!(s.body["answered"], 1);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn up_test_waits_out_a_recent_failure_at_that_grade(pool: PgPool) {
    let api = Api::start(pool, None).await;
    let cand = api.candidate("galina@example.ru", None).await;
    let ct = api.token(cand, Role::Candidate);
    let failed_at = Utc::now() - Duration::hours(3);
    api.finished(cand, Grade::Middle, "placement", false, failed_at)
        .await;
    api.finished(
        cand,
        Grade::Junior,
        "placement_step_down",
        true,
        failed_at + Duration::hours(1),
    )
    .await;

    let c = api
        .call(Method::GET, "/candidate/category", &ct, None)
        .await;
    assert_eq!(c.body["category"]["grade"], "junior", "{}", c.body);
    assert!(offers(&c.body, "available").is_empty(), "{}", c.body);
    let up = locked(&c.body, "middle");
    assert_eq!(up["kind"], "placement_upgrade");
    assert_eq!(up["reason"], "retry_blocked");
    assert_eq!(
        secs(&up["available_at"]),
        (failed_at + Duration::days(30)).timestamp()
    );

    let r = api
        .call(
            Method::POST,
            "/candidate/assessments",
            &ct,
            Some(json!({ "specialization": "backend", "grade": "middle" })),
        )
        .await;
    r.problem(StatusCode::CONFLICT, "retry_blocked");

    // Without a recent failure at Middle the confident pass opens the up-test at once.
    let cand = api.candidate("dmitry@example.ru", None).await;
    let ct = api.token(cand, Role::Candidate);
    let long_ago = Utc::now() - Duration::days(40);
    api.finished(cand, Grade::Middle, "placement", false, long_ago)
        .await;
    api.finished(
        cand,
        Grade::Junior,
        "placement",
        true,
        Utc::now() - Duration::hours(1),
    )
    .await;
    let c = api
        .call(Method::GET, "/candidate/category", &ct, None)
        .await;
    assert_eq!(
        offers(&c.body, "available"),
        [("middle".to_owned(), "placement_upgrade".to_owned())],
        "{}",
        c.body
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn request_errors_are_problem_json(pool: PgPool) {
    let api = Api::start(pool, None).await;
    let cand = api.candidate("elena0@example.ru", None).await;
    let ct = api.token(cand, Role::Candidate);
    let post = |body: &'static str, content_type: &'static str| {
        api.send(
            api.request(Method::POST, "/candidate/assessments", Some(&ct))
                .header(header::CONTENT_TYPE, content_type)
                .body(body),
        )
    };
    post(r#"{"specialization": "backend","#, "application/json")
        .await
        .problem(StatusCode::BAD_REQUEST, "invalid_json");
    post(
        r#"{"specialization": "backend", "grade": "boss"}"#,
        "application/json",
    )
    .await
    .problem(StatusCode::UNPROCESSABLE_ENTITY, "invalid_json");
    post(
        r#"{"specialization": "backend", "grade": "middle"}"#,
        "text/plain",
    )
    .await
    .problem(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");

    api.call(Method::GET, "/candidate/assessments/not-a-uuid", &ct, None)
        .await
        .problem(StatusCode::BAD_REQUEST, "invalid_path");
    api.send(api.request(
        Method::GET,
        "/catalog/test-plan?specialization=wizard&grade=middle",
        None,
    ))
    .await
    .problem(StatusCode::BAD_REQUEST, "invalid_query");
    api.send(
        api.request(Method::POST, "/candidate/resume", Some(&ct))
            .header(header::CONTENT_TYPE, "application/json")
            .body("{}"),
    )
    .await
    .problem(StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_media_type");

    // Over the route's 256 KiB: the extractor answers.
    let body = format!(r#"{{"email": "{}"}}"#, "a".repeat(300 * 1024));
    api.send(
        api.request(Method::POST, "/auth/login", None)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body),
    )
    .await
    .problem(StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large");
    // Over 10 MiB by `Content-Length`: the server-wide limit answers before reading the body.
    let raw = api
        .raw("POST /api/v1/auth/login HTTP/1.1\r\nHost: t\r\nContent-Type: application/json\r\nContent-Length: 11534336\r\n\r\n")
        .await;
    assert!(raw.starts_with("HTTP/1.1 413"), "{raw}");
    assert!(raw.contains("application/problem+json"), "{raw}");
    assert!(raw.contains(r#""code":"payload_too_large""#), "{raw}");

    let eval = || {
        api.send(
            api.request(Method::POST, "/eval/pairs", None)
                .header("x-forwarded-for", "198.51.100.7")
                .json(&json!({})),
        )
    };
    for _ in 0..10 {
        assert_ne!(eval().await.status, StatusCode::TOO_MANY_REQUESTS);
    }
    let r = eval().await;
    r.problem(StatusCode::TOO_MANY_REQUESTS, "rate_limited");
    let wait: u64 = r.headers[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        (1..=600).contains(&wait),
        "the rest of the 10-minute window, not a constant: {wait}"
    );
    assert!(
        r.body["detail"]
            .as_str()
            .unwrap()
            .starts_with("Повторите через")
    );
}

/// The registry mock: one participant with an algorithmic podium and a product start.
async fn registry() -> String {
    let achievements = json!({ "achievements": [
        { "kind": "competition", "discipline": "algorithmic", "event_level": "national",
          "place": 2, "date": "2025-11-20", "event_name": "Кубок России" },
        { "kind": "competition", "discipline": "product", "event_level": "regional",
          "place": "participant", "date": "2025-05-01" },
    ]})
    .to_string();
    let app = axum::Router::new().route(
        "/participants/FSP-ALGO/achievements",
        axum::routing::get(move || {
            let body = achievements.clone();
            async move { ([(header::CONTENT_TYPE, "application/json")], body) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await });
    url
}

fn reasons(profile: &Value) -> Vec<String> {
    profile["breakdown"]["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().to_owned())
        .collect()
}

fn card_sources(profile: &Value, slug: &str) -> Vec<String> {
    profile["candidate"]["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["slug"] == slug)
        .map(|s| {
            s["sources"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

async fn fsp_rows(pool: &PgPool, id: Uuid) -> Vec<String> {
    db::candidates::candidate_skills(pool, id)
        .await
        .unwrap()
        .into_iter()
        .filter(|s| s.source == "fsp")
        .map(|s| s.skill)
        .collect()
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn fsp_results_confirm_skills_until_hidden_or_unlinked(pool: PgPool) {
    let api = Api::start(pool, Some(registry().await)).await;
    let cand = api.candidate("olga00@example.ru", Some("middle")).await;
    db::candidates::insert_candidate_skill(api.pool(), cand, "go", "declared")
        .await
        .unwrap();
    let hr = api.employer("hr@example.ru").await;
    let (ct, et) = (
        api.token(cand, Role::Candidate),
        api.token(hr, Role::Employer),
    );
    let need = api
        .call(
            Method::POST,
            "/employer/needs",
            &et,
            Some(json!({
                "title": "Бэкенд-разработчик",
                "specialization": "backend",
                "grades": ["middle"],
                "must_skills": ["go", "algorithms"],
                "salary": { "min": 200000, "max": 300000 },
            })),
        )
        .await;
    assert_eq!(need.status, StatusCode::CREATED, "{}", need.body);
    let profile_path = format!(
        "/employer/candidates/{cand}?need_id={}",
        need.body["id"].as_str().unwrap()
    );
    let profile = || api.call(Method::GET, &profile_path, &et, None);
    let confirmed = "Алгоритмы — подтверждено ФСП".to_owned();
    assert!(
        !reasons(&profile().await.body).contains(&confirmed),
        "warm the pool cache first"
    );

    let r = api
        .call(
            Method::POST,
            "/fsp/link/dev",
            &ct,
            Some(json!({ "fsp_id": "fsp-algo" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["achievements"].as_array().unwrap().len(), 2);
    assert_eq!(
        fsp_rows(api.pool(), cand).await,
        ["algorithms", "data_structures"],
        "a podium confirms skills, a product start confirms none"
    );
    let p = profile().await.body;
    assert!(reasons(&p).contains(&confirmed), "{:?}", reasons(&p));
    assert_eq!(card_sources(&p, "algorithms"), ["fsp"]);
    let own = api.call(Method::GET, "/candidate/profile", &ct, None).await;
    assert!(
        own.body["skill_details"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["slug"] == "algorithms" && s["sources"] == json!(["fsp"])),
        "{}",
        own.body
    );

    let privacy = |show_fsp: bool| {
        api.call(
            Method::PUT,
            "/candidate/privacy",
            &ct,
            Some(json!({
                "visible": true,
                "show_name": false,
                "show_fsp": show_fsp,
                "show_experience": true,
                "search_status": "open",
            })),
        )
    };
    assert_eq!(privacy(false).await.status, StatusCode::OK);
    let p = profile().await.body;
    assert!(
        !reasons(&p).iter().any(|r| r.contains("ФСП")),
        "{:?}",
        reasons(&p)
    );
    assert!(card_sources(&p, "algorithms").is_empty());
    assert!(p["candidate"]["fsp"].is_null());
    assert!(
        p["breakdown"]["gaps"]
            .as_array()
            .unwrap()
            .contains(&json!("Нет обязательного навыка: Алгоритмы")),
        "a hidden FSP result no longer counts: {}",
        p["breakdown"]
    );
    let pooled = db::pool::searchable_pool(api.pool(), "backend")
        .await
        .unwrap();
    assert!(!pooled[0].skill_sources.iter().any(|s| s == "fsp"));

    assert_eq!(privacy(true).await.status, StatusCode::OK);
    assert!(reasons(&profile().await.body).contains(&confirmed));
    let r = api.call(Method::POST, "/fsp/sync", &ct, None).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(fsp_rows(api.pool(), cand).await.len(), 2);

    let r = api.call(Method::DELETE, "/fsp/link", &ct, None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert!(fsp_rows(api.pool(), cand).await.is_empty());
    let p = profile().await.body;
    assert!(!reasons(&p).iter().any(|r| r.contains("ФСП")));
    assert_eq!(card_sources(&p, "go"), ["declared"]);
}
