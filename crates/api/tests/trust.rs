//! Anti-fake signals over real HTTP against a throwaway database: flags written on publish and
//! invite, the trust-based invitation quota, the corporate domain badge, moderation, and the demo
//! seed for the T-4 walkthrough.

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
use chrono::Utc;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const TERMS: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);

const GO_TEXT: &str = "Ищем бэкенд-разработчика уровня Middle в команду процессинга. Стек: Go, PostgreSQL, \
    Kafka, gRPC, Docker. Будет плюсом опыт с Kubernetes и ClickHouse. Задачи: идемпотентные платёжные API, \
    outbox, шардирование. Удалённо или гибрид в Москве, ДМС и обучение за счёт компании.";
const GO_EDITED: &str = "Ищем бэкенд-разработчика уровня Middle в команду процессинга. Стек: Go, PostgreSQL, \
    Kafka, gRPC, Docker. Будет плюсом опыт с Kubernetes и Redis. Задачи: идемпотентные платёжные API, \
    outbox, шардирование. Удалённо или гибрид в Москве, ДМС и обучение за счёт компании.";
const OTHER_TEXT: &str = "Нужен фронтенд-разработчик на React и TypeScript в кабинет партнёров: таблицы, \
    фильтры, графики, формы. Next.js, своя дизайн-система, тесты на компоненты, работа в паре с дизайнером.";

struct Api {
    base: String,
    http: reqwest::Client,
    state: AppState,
}

struct Reply {
    status: StatusCode,
    body: Value,
}

impl Api {
    async fn start(pool: PgPool) -> Self {
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
                registry_url: None,
                registry_token: "test".into(),
                webhook_secret: None,
                dev_link: false,
            },
            grade_change_cooldown_days: 90,
            proctoring: ProctoringConfig {
                mode: ProctoringMode::Off,
                dir: std::env::temp_dir().join(format!("trust-test-{}", Uuid::now_v7())),
                retention_days: 90,
            },
            integrations: IntegrationsConfig::default(),
            notifications: Default::default(),
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

    fn pool(&self) -> &PgPool {
        &self.state.pool
    }

    fn token(&self, id: Uuid, role: Role) -> String {
        self.state
            .jwt
            .issue(id, role, Utc::now().timestamp())
            .unwrap()
    }

    async fn call(&self, method: Method, path: &str, token: &str, body: Option<Value>) -> Reply {
        self.call_with(method, path, token, body, &[]).await
    }

    async fn call_with(
        &self,
        method: Method,
        path: &str,
        token: &str,
        body: Option<Value>,
        headers: &[(&str, &str)],
    ) -> Reply {
        let mut req = self
            .http
            .request(method, format!("{}/api/v1{path}", self.base))
            .bearer_auth(token);
        for (name, value) in headers {
            req = req.header(*name, *value);
        }
        if let Some(b) = body {
            req = req.json(&b);
        }
        let res = req.send().await.unwrap();
        let status = res.status();
        let bytes = res.bytes().await.unwrap();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        Reply { status, body }
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
        db::users::mark_email_verified(self.pool(), id)
            .await
            .unwrap();
        id
    }

    /// A published backend:middle candidate expecting `salary` (if any).
    async fn candidate(&self, email: &str, salary: Option<i64>) -> Uuid {
        let id = self.account(Role::Candidate, email).await;
        db::candidates::insert_candidate(
            self.pool(),
            id,
            &format!("K-{}", &id.simple().to_string()[26..]),
        )
        .await
        .unwrap();
        db::grades::set_candidate_category(self.pool(), "backend", "middle", 0.4, Utc::now(), id)
            .await
            .unwrap();
        db::candidates::update_candidate_profile(
            self.pool(),
            id,
            &db::candidates::ProfileUpdate {
                full_name_enc: None,
                contacts_enc: None,
                city: Some("Москва"),
                relocation: false,
                work_formats: &["remote".to_owned()],
                salary_expectation_min: salary,
                about: "",
                experience_years: Some(3.0),
                roles: &[],
                soft_skills: &[],
                industries: &[],
                experience: &json!([]),
            },
        )
        .await
        .unwrap();
        for doc in [PROCESSING, PUBLICATION, TERMS] {
            db::consents::insert_user_consent(self.pool(), Uuid::now_v7(), id, doc, None, None)
                .await
                .unwrap();
        }
        id
    }

    /// Employer token and company id; the company profile gets `website` if given.
    async fn employer(&self, email: &str, website: Option<&str>) -> (String, Uuid) {
        let owner = self.account(Role::Employer, email).await;
        let company = Uuid::now_v7();
        db::companies::insert_company(self.pool(), company, owner, "Рога и копыта")
            .await
            .unwrap();
        let token = self.token(owner, Role::Employer);
        if let Some(site) = website {
            let r = self
                .call(
                    Method::PUT,
                    "/employer/company",
                    &token,
                    Some(json!({ "name": format!("Компания {email}"), "description": "", "website": site })),
                )
                .await;
            assert_eq!(r.status, StatusCode::OK);
        }
        (token, company)
    }

    async fn publish(&self, token: &str, title: &str, text: &str, salary: (i64, i64)) -> Value {
        let r = self
            .call(
                Method::POST,
                "/employer/vacancies",
                token,
                Some(json!({
                    "title": title,
                    "description": text,
                    "specialization": "backend",
                    "grade": "middle",
                    "skills": ["go", "postgresql"],
                    "salary": { "min": salary.0, "max": salary.1 },
                    "work_format": "remote",
                    "city": null,
                    "status": "published",
                })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        r.body
    }

    async fn flags(&self, vacancy: &Value) -> Vec<String> {
        let id: Uuid = vacancy["id"].as_str().unwrap().parse().unwrap();
        kinds(&db::trust::vacancy_flags(self.pool(), id).await.unwrap())
    }

    async fn invite(&self, token: &str, candidate: Uuid, salary: (i64, i64)) -> Reply {
        self.call(
            Method::POST,
            "/employer/invitations",
            token,
            Some(json!({
                "candidate_id": candidate,
                "title": "Go-разработчик",
                "message": "Здравствуйте! Видели ваш результат теста — хотим познакомиться.",
                "salary": { "min": salary.0, "max": salary.1 },
                "contact": "@hr",
            })),
        )
        .await
    }
}

fn kinds(flags: &Value) -> Vec<String> {
    flags
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["kind"].as_str().unwrap().to_owned())
        .collect()
}

fn reason_kinds(trust: &Value) -> Vec<String> {
    trust["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["kind"].as_str().unwrap().to_owned())
        .collect()
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn publish_writes_salary_and_duplicate_flags(pool: PgPool) {
    let api = Api::start(pool).await;
    for (i, salary) in [220_000, 240_000, 250_000, 260_000, 280_000, 300_000]
        .into_iter()
        .enumerate()
    {
        api.candidate(&format!("market{i}@example.ru"), Some(salary))
            .await;
    }
    let (hr, company) = api.employer("hr@corp.ru", None).await;

    let check = api
        .call(
            Method::GET,
            "/employer/salary-check?specialization=backend&grade=middle&min=300000&max=900000",
            &hr,
            None,
        )
        .await;
    assert_eq!(check.status, StatusCode::OK);
    assert_eq!(check.body["market"]["samples"], 6);
    let warnings: Vec<&str> = check.body["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["kind"].as_str().unwrap())
        .collect();
    assert_eq!(warnings, ["salary_above_market", "salary_wide_range"]);
    assert!(
        check.body["warnings"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Публикации это не мешает")
    );

    let first = api
        .publish(&hr, "Go-разработчик в платежи", GO_TEXT, (250_000, 320_000))
        .await;
    assert!(api.flags(&first).await.is_empty());

    let bait = api
        .publish(&hr, "Фронтенд в кабинет", OTHER_TEXT, (300_000, 900_000))
        .await;
    assert_eq!(
        api.flags(&bait).await,
        ["salary_above_market", "salary_wide_range"],
        "publishing is not blocked, the vacancy is flagged"
    );

    let similar = api
        .call(
            Method::POST,
            "/employer/vacancies/similar",
            &hr,
            Some(json!({ "title": "Go-разработчик в платежи", "description": GO_EDITED })),
        )
        .await;
    assert_eq!(similar.status, StatusCode::OK);
    assert_eq!(similar.body[0]["id"], first["id"]);
    assert!(similar.body[0]["similarity"].as_f64().unwrap() >= 0.6);
    assert_eq!(similar.body.as_array().unwrap().len(), 1);

    let copy = api
        .publish(
            &hr,
            "Go-разработчик в платежи",
            GO_EDITED,
            (250_000, 320_000),
        )
        .await;
    assert_eq!(api.flags(&copy).await, ["duplicate_own"]);
    let excluded = api
        .call(
            Method::POST,
            "/employer/vacancies/similar",
            &hr,
            Some(json!({ "title": "Go-разработчик в платежи", "description": GO_TEXT, "exclude_id": first["id"] })),
        )
        .await;
    assert_eq!(excluded.body[0]["id"], copy["id"]);

    let (rival, rival_company) = api.employer("jobs@rival.ru", None).await;
    let stolen = api
        .publish(&rival, "Go-разработчик", GO_TEXT, (250_000, 320_000))
        .await;
    assert_eq!(api.flags(&stolen).await, ["copy_of_other"]);
    let none = api
        .call(
            Method::POST,
            "/employer/vacancies/similar",
            &rival,
            Some(json!({ "title": "x", "description": OTHER_TEXT })),
        )
        .await;
    assert_eq!(
        none.body,
        json!([]),
        "other companies' vacancies are never suggested"
    );

    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    assert_eq!(trust.status, StatusCode::OK);
    assert_eq!(
        reason_kinds(&trust.body),
        ["salary_flags", "own_duplicates"]
    );
    assert_eq!(trust.body["quota_limit"], 5, "10 − 5 − 5, floored at 5");
    assert_eq!(trust.body["level"], "low");
    assert_eq!(trust.body["domain"]["check"]["status"], "no_website");

    let admin_id = api.account(Role::Admin, "admin@example.ru").await;
    let admin = api.token(admin_id, Role::Admin);
    let queue = api.call(Method::GET, "/admin/trust", &admin, None).await;
    assert_eq!(queue.status, StatusCode::OK);
    let companies: Vec<&str> = queue.body["companies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(companies.len(), 2);
    assert!(companies.contains(&company.to_string().as_str()));
    assert!(companies.contains(&rival_company.to_string().as_str()));
    let flagged: Vec<&Value> = queue.body["vacancies"].as_array().unwrap().iter().collect();
    assert_eq!(flagged.len(), 3);
    let stolen_row = flagged.iter().find(|v| v["id"] == stolen["id"]).unwrap();
    assert!(
        stolen_row["reasons"][0]
            .as_str()
            .unwrap()
            .starts_with("Почти дословно совпадает с вакансией «Go-разработчик в платежи»")
    );
    let forbidden = api.call(Method::GET, "/admin/trust", &hr, None).await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn quota_follows_trust(pool: PgPool) {
    let api = Api::start(pool).await;
    let mut candidates = Vec::new();
    for i in 0..12 {
        candidates.push(
            api.candidate(&format!("cand{i}@example.ru"), Some(250_000))
                .await,
        );
    }
    let (hr, company) = api
        .employer("hr@severny-kod.ru", Some("https://other-site.ru"))
        .await;

    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    assert_eq!(trust.body["level"], "new");
    assert_eq!(trust.body["quota_limit"], 10);
    assert_eq!(trust.body["domain"]["check"]["status"], "mismatch");
    assert!(
        trust.body["domain"]["hint"]
            .as_str()
            .unwrap()
            .contains("severny-kod.ru")
    );
    assert_eq!(trust.body["advice"].as_array().unwrap().len(), 3);

    for &c in &candidates[..10] {
        let r = api.invite(&hr, c, (250_000, 320_000)).await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    }
    let over = api.invite(&hr, candidates[10], (250_000, 320_000)).await;
    assert_eq!(over.status, StatusCode::CONFLICT);
    assert_eq!(over.body["code"], "invitation_quota_exceeded");
    assert_eq!(
        over.body["title"],
        "Лимит 10 приглашений за 7 дней исчерпан"
    );

    let r = api
        .call(
            Method::PUT,
            "/employer/company",
            &hr,
            Some(json!({ "name": "Северный Код", "description": "", "website": "https://jobs.severny-kod.ru/" })),
        )
        .await;
    assert_eq!(r.body["verified_domain"], "severny-kod.ru");
    let stats = api
        .call(Method::GET, "/employer/invitations/stats", &hr, None)
        .await;
    assert_eq!(stats.body["quota_limit"], 20);
    assert_eq!(stats.body["quota_max"], 30);
    assert_eq!(stats.body["trust_level"], "new");
    let r = api.invite(&hr, candidates[10], (250_000, 320_000)).await;
    assert_eq!(
        r.status,
        StatusCode::CREATED,
        "a confirmed domain lifts the quota"
    );

    for (i, &c) in candidates[..2].iter().enumerate() {
        let ct = api.token(c, Role::Candidate);
        let r = api
            .call(
                Method::POST,
                "/reports",
                &ct,
                Some(json!({ "target_kind": "company", "target_id": company, "reason": if i == 0 { "spam" } else { "fake_vacancy" } })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED);
    }
    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    assert_eq!(
        trust.body["quota_limit"], 5,
        "10 + 10 − 2 × 10, floored at 5"
    );
    assert_eq!(trust.body["level"], "low");
    assert_eq!(trust.body["penalty"], -20);
    let r = api.invite(&hr, candidates[11], (250_000, 320_000)).await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["code"], "invitation_quota_exceeded");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn invitation_flags_and_domain_badge(pool: PgPool) {
    let api = Api::start(pool).await;
    for (i, salary) in [220_000, 240_000, 250_000, 260_000, 280_000]
        .into_iter()
        .enumerate()
    {
        api.candidate(&format!("market{i}@example.ru"), Some(salary))
            .await;
    }
    let cand = api.candidate("anna@example.ru", None).await;
    let (hr, _) = api
        .employer("hr@пример.рф", Some("https://www.xn--e1afmkfd.xn--p1ai"))
        .await;
    let (plain, _) = api
        .employer("boss@gmail.com", Some("https://gmail.com"))
        .await;

    let r = api.invite(&hr, cand, (100_000, 450_000)).await;
    assert_eq!(r.status, StatusCode::CREATED);
    let id: Uuid = r.body["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(
        kinds(&db::trust::invitation_flags(api.pool(), id).await.unwrap()),
        ["salary_above_market", "salary_wide_range"]
    );
    let r = api.invite(&plain, cand, (250_000, 320_000)).await;
    let calm: Uuid = r.body["id"].as_str().unwrap().parse().unwrap();
    assert!(kinds(&db::trust::invitation_flags(api.pool(), calm).await.unwrap()).is_empty());

    let ct = api.token(cand, Role::Candidate);
    let inbox = api
        .call(Method::GET, "/candidate/invitations", &ct, None)
        .await;
    let by_id = |id: Uuid| {
        inbox.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["id"] == id.to_string())
            .unwrap()
            .clone()
    };
    assert_eq!(by_id(id)["company_verified_domain"], "пример.рф");
    assert!(
        by_id(calm)["company_verified_domain"].is_null(),
        "a public mailbox never confirms a company"
    );
    let text = inbox.body.to_string();
    assert!(
        !text.contains("salary_above_market") && !text.contains("trust"),
        "candidates see no salary alarm"
    );

    api.publish(&hr, "Go-разработчик", GO_TEXT, (250_000, 320_000))
        .await;
    let public = reqwest::get(format!("{}/api/v1/vacancies", api.base))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(public["items"][0]["company_verified_domain"], "пример.рф");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn demo_seed_has_six_needs_with_plans_and_shortlists(pool: PgPool) {
    let api = Api::start(pool).await;
    api::seed::seed(api.pool(), &api.state.crypto, 90)
        .await
        .unwrap()
        .unwrap();
    api::trust::backfill_signatures(api.pool()).await.unwrap();

    let employer = db::users::user_by_email_bidx(
        api.pool(),
        &api.state.crypto.blind_index("employer@demo.fsp"),
    )
    .await
    .unwrap()
    .unwrap();
    let et = api.token(employer.id, Role::Employer);
    let needs = api.call(Method::GET, "/employer/needs", &et, None).await;
    let needs = needs.body.as_array().unwrap().clone();
    assert_eq!(needs.len(), 6);
    let specs: Vec<&str> = needs
        .iter()
        .map(|n| n["specialization"].as_str().unwrap())
        .collect();
    assert_eq!(
        specs,
        ["backend", "frontend", "devops", "qa", "ml", "security"]
    );
    assert_eq!(needs[0]["title"], "Go-разработчик в команду платежей");
    let mut created: Vec<&str> = needs
        .iter()
        .map(|n| n["created_at"].as_str().unwrap())
        .collect();
    created.dedup();
    assert_eq!(
        created.len(),
        6,
        "distinct timestamps keep the order stable"
    );

    for need in &needs {
        let id = need["id"].as_str().unwrap();
        let plan = api
            .call(
                Method::GET,
                &format!("/employer/needs/{id}/test-plan"),
                &et,
                None,
            )
            .await;
        assert_eq!(plan.status, StatusCode::OK);
        assert!(
            !plan.body["plan"]["focus_templates"]
                .as_array()
                .unwrap()
                .is_empty(),
            "{}: the stack steers the test",
            need["title"]
        );
        assert!(plan.body["plan"]["topics"].as_array().unwrap().len() >= 3);
        assert_eq!(plan.body["samples"].as_array().unwrap().len(), 2);

        let list = api
            .call(
                Method::POST,
                &format!("/employer/needs/{id}/shortlist"),
                &et,
                Some(json!({})),
            )
            .await;
        assert_eq!(list.status, StatusCode::OK);
        assert!(
            list.body["items"].as_array().unwrap().len() >= 3,
            "{}: shortlist",
            need["title"]
        );
    }

    let trust = api.call(Method::GET, "/employer/trust", &et, None).await;
    assert_eq!(trust.body["domain"]["check"]["status"], "mismatch");
    assert_eq!(trust.body["quota_limit"], 20);
    let vacancies = reqwest::get(format!("{}/api/v1/vacancies?limit=100", api.base))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    let verified: Vec<&Value> = vacancies["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| !v["company_verified_domain"].is_null())
        .collect();
    assert_eq!(
        verified.len(),
        10,
        "five of six demo companies sign in from their domain"
    );
    assert!(
        db::trust::unsigned_vacancies(api.pool(), 1)
            .await
            .unwrap()
            .is_empty()
    );
}

fn vacancy_body(title: &str) -> Value {
    json!({
        "title": title,
        "description": "Платёжный шлюз: Go, PostgreSQL, Kafka. Команда из пяти человек.",
        "specialization": "backend",
        "grade": "middle",
        "skills": ["go", "postgresql"],
        "salary": { "min": 250000, "max": 320000 },
        "work_format": "remote",
        "city": null,
        "status": "published",
    })
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn a_double_click_makes_one_vacancy(pool: PgPool) {
    let api = Api::start(pool).await;
    let (hr, company) = api.employer("hr@example.ru", None).await;
    let post = async |key: &str| {
        api.call_with(
            Method::POST,
            "/employer/vacancies",
            &hr,
            Some(vacancy_body("Go-разработчик в платёжную команду")),
            &[("Idempotency-Key", key)],
        )
        .await
    };
    let first = post("vac-1").await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.body);
    let again = post("vac-1").await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.body);
    assert_eq!(again.body["id"], first.body["id"]);

    let (a, b) = tokio::join!(post("vac-2"), post("vac-2"));
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::CREATED]);
    assert_eq!(
        a.body["id"], b.body["id"],
        "the same click sent twice at once"
    );

    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM vacancies WHERE company_id = $1")
        .bind(company)
        .fetch_one(api.pool())
        .await
        .unwrap();
    assert_eq!(rows, 2, "one vacancy per key");
    assert!(
        !api.flags(&first.body)
            .await
            .contains(&"duplicate_own".to_owned()),
        "a retry is not a duplicate"
    );

    let long = "k".repeat(101);
    let r = api
        .call_with(
            Method::POST,
            "/employer/vacancies",
            &hr,
            Some(vacancy_body("Ещё одна")),
            &[("Idempotency-Key", long.as_str())],
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(r.body["errors"]["idempotency_key"].is_array(), "{}", r.body);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn one_open_complaint_per_person_and_target(pool: PgPool) {
    let api = Api::start(pool).await;
    let (_, company) = api.employer("hr@example.ru", None).await;
    let anna = api.candidate("anna@example.ru", None).await;
    let boris = api.candidate("boris@example.ru", None).await;
    let (at, bt) = (
        api.token(anna, Role::Candidate),
        api.token(boris, Role::Candidate),
    );
    let report = async |token: &str, reason: &str| {
        api.call(
            Method::POST,
            "/reports",
            token,
            Some(json!({ "target_kind": "company", "target_id": company, "reason": reason })),
        )
        .await
    };
    let first = report(&at, "spam").await;
    assert_eq!(first.status, StatusCode::CREATED, "{}", first.body);
    assert_eq!(first.body["already_open"], false);
    let again = report(&at, "abuse").await;
    assert_eq!(again.status, StatusCode::OK, "{}", again.body);
    assert_eq!(again.body["id"], first.body["id"]);
    assert_eq!(again.body["already_open"], true);
    let (x, y) = tokio::join!(report(&bt, "spam"), report(&bt, "spam"));
    assert_eq!(
        x.body["id"], y.body["id"],
        "a double click from another person"
    );
    let open: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM reports WHERE target_id = $1 AND resolved_at IS NULL",
    )
    .bind(company)
    .fetch_one(api.pool())
    .await
    .unwrap();
    assert_eq!(open, 2, "one per person");

    sqlx::query(
        "UPDATE reports SET resolved_at = now(), resolution = 'dismissed' WHERE reporter_user_id = $1",
    )
    .bind(anna)
    .execute(api.pool())
    .await
    .unwrap();
    let after = report(&at, "spam").await;
    assert_eq!(
        after.status,
        StatusCode::CREATED,
        "a resolved complaint does not block a new one"
    );
    assert_ne!(after.body["id"], first.body["id"]);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn invitation_lists_page_by_cursor_and_filter_by_status(pool: PgPool) {
    let api = Api::start(pool).await;
    let (hr, _) = api.employer("hr@example.ru", None).await;
    let mut cands = Vec::new();
    for i in 0..5 {
        let c = api.candidate(&format!("c{i}@example.ru"), None).await;
        let r = api.invite(&hr, c, (250_000, 320_000)).await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        cands.push((c, r.body["id"].as_str().unwrap().to_owned()));
    }
    let (c0, inv0) = &cands[0];
    let ct = api.token(*c0, Role::Candidate);
    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv0}/accept"),
            &ct,
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);

    let mut seen = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages = 0;
    loop {
        let path = match &cursor {
            Some(c) => format!("/employer/invitations?limit=2&cursor={c}"),
            None => "/employer/invitations?limit=2".to_owned(),
        };
        let r = api.call(Method::GET, &path, &hr, None).await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        pages += 1;
        for i in r.body["items"].as_array().unwrap() {
            seen.push(i["id"].as_str().unwrap().to_owned());
        }
        assert_eq!(
            r.body["by_status"].as_array().unwrap().len(),
            2,
            "counts on every page"
        );
        match r.body["next_cursor"].as_str() {
            Some(c) => cursor = Some(c.to_owned()),
            None => break,
        }
    }
    assert_eq!(pages, 3);
    let newest_first: Vec<String> = cands.iter().rev().map(|(_, id)| id.clone()).collect();
    assert_eq!(seen, newest_first, "every invitation once, newest first");

    let r = api
        .call(
            Method::GET,
            "/employer/invitations?status=accepted",
            &hr,
            None,
        )
        .await;
    let only: Vec<&str> = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap())
        .collect();
    assert_eq!(only, [inv0.as_str()]);
    let mut counts: Vec<(String, i64)> = r.body["by_status"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["status"].as_str().unwrap().to_owned(),
                c["count"].as_i64().unwrap(),
            )
        })
        .collect();
    counts.sort();
    assert_eq!(counts, [("accepted".to_owned(), 1), ("sent".to_owned(), 4)]);
    let r = api
        .call(
            Method::GET,
            "/employer/invitations?status=sent,viewed&limit=100",
            &hr,
            None,
        )
        .await;
    assert_eq!(r.body["items"].as_array().unwrap().len(), 4);
    let r = api
        .call(Method::GET, "/employer/invitations?status=lost", &hr, None)
        .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body["code"], "invalid_query");

    let mut inbox = Vec::new();
    for i in 0..3 {
        let (other, _) = api.employer(&format!("boss{i}@example{i}.ru"), None).await;
        let r = api.invite(&other, *c0, (260_000, 330_000)).await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        inbox.push(r.body["id"].as_str().unwrap().to_owned());
    }
    let r = api
        .call(
            Method::GET,
            "/candidate/invitations?status=sent,viewed&limit=2",
            &ct,
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let page: Vec<&str> = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap())
        .collect();
    assert_eq!(page, [inbox[2].as_str(), inbox[1].as_str()]);
    let next = r.body["next_cursor"].as_str().unwrap().to_owned();
    let r = api
        .call(
            Method::GET,
            &format!("/candidate/invitations?status=sent,viewed&limit=2&cursor={next}"),
            &ct,
            None,
        )
        .await;
    assert_eq!(r.body["items"][0]["id"], inbox[0].as_str());
    assert_eq!(r.body["next_cursor"], Value::Null);
    let r = api
        .call(
            Method::GET,
            "/candidate/invitations?status=accepted",
            &ct,
            None,
        )
        .await;
    assert_eq!(r.body["items"][0]["id"], inv0.as_str());
    let mut counts: Vec<(String, i64)> = r.body["by_status"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["status"].as_str().unwrap().to_owned(),
                c["count"].as_i64().unwrap(),
            )
        })
        .collect();
    counts.sort();
    assert_eq!(counts, [("accepted".to_owned(), 1), ("sent".to_owned(), 3)]);
}
