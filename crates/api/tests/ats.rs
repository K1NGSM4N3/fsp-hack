//! ATS integration over real HTTP against a throwaway database: the transactional outbox,
//! API keys and scopes, hiring outcomes, vacancy import, and a full round trip through the ATS
//! mock — signed webhook out, hiring stage back in, visible to the candidate.

use std::{sync::Arc, time::Duration as StdDuration};

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, ProctoringConfig},
    contacts::Contacts,
    crypto::Crypto,
    integrations::{dispatcher::Dispatcher, events::open_payload, webhook::WebhookSender},
    kv::Kv,
    mail::Mailer,
    proctoring::{ProctoringMode, RecordingStore},
    state::AppState,
};
use chrono::Utc;
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

impl Api {
    async fn start(pool: PgPool, dispatch: bool) -> Self {
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
                dir: std::env::temp_dir().join(format!("ats-test-{}", Uuid::now_v7())),
                retention_days: 90,
            },
            integrations: IntegrationsConfig {
                webhook_dev_hosts: vec!["127.0.0.1".into()],
                dispatch_interval: Some(StdDuration::from_millis(50)),
                demo: None,
            },
            seed_demo: false,
            seed_per_spec: 40,
            log_json: false,
        };
        let webhooks = WebhookSender::new(config.integrations.webhook_dev_hosts.clone()).unwrap();
        let kv = Kv::connect(None).await.unwrap();
        let state = AppState {
            crypto: Arc::new(Crypto::new(&config.field_key, &config.bidx_key)),
            jwt: Arc::new(JwtKeys::new(config.jwt_seed).unwrap()),
            mailer: Arc::new(Mailer::new(&config.mail).unwrap()),
            kv: kv.clone(),
            candidates: Arc::new(api::cache::CandidateCache::new(kv)),
            http: reqwest::Client::new(),
            metrics: None,
            recordings: Arc::new(RecordingStore::new(config.proctoring.dir.clone())),
            webhooks: webhooks.clone(),
            pool,
            config: Arc::new(config),
        };
        if dispatch {
            Dispatcher::new(
                state.pool.clone(),
                state.crypto.clone(),
                webhooks,
                state.config.integrations.dispatch_interval,
            )
            .spawn();
        }
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

    async fn send(&self, req: reqwest::RequestBuilder) -> Reply {
        let res = req.send().await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.bytes().await.unwrap();
        let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        Reply {
            status,
            headers,
            body,
        }
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> Reply {
        let mut req = self
            .http
            .request(method, format!("{}/api/v1{path}", self.base));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
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
            &self.state.crypto.encrypt_str(email),
            &self.state.crypto.blind_index(email),
            "h",
        )
        .await
        .unwrap();
        id
    }

    /// A published backend:middle candidate with a name and contacts.
    async fn candidate(&self, email: &str, name: &str) -> Uuid {
        let id = self.account(Role::Candidate, email).await;
        db::candidates::insert_candidate(
            self.pool(),
            id,
            &format!("K-{}", email[..4].to_uppercase()),
        )
        .await
        .unwrap();
        db::grades::set_candidate_category(self.pool(), "backend", "middle", 0.4, Utc::now(), id)
            .await
            .unwrap();
        let contacts = Contacts {
            phone: Some("+7 900 123-45-67".into()),
            telegram: Some("@anna_dev".into()),
            email: Some(email.into()),
        };
        db::candidates::update_candidate_profile(
            self.pool(),
            id,
            &db::candidates::ProfileUpdate {
                full_name_enc: Some(&self.state.crypto.encrypt_str(name)),
                contacts_enc: Some(&self.state.crypto.encrypt_json(&contacts).unwrap()),
                city: Some("Москва"),
                relocation: false,
                work_formats: &["remote".to_owned()],
                salary_expectation_min: Some(250_000),
                about: "Пишу сервисы на Rust и Go",
                experience_years: Some(4.0),
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

    async fn employer(&self, email: &str) -> (String, Uuid) {
        let owner = self.account(Role::Employer, email).await;
        let company = Uuid::now_v7();
        db::companies::insert_company(self.pool(), company, owner, "Рога и копыта")
            .await
            .unwrap();
        (self.token(owner, Role::Employer), company)
    }

    async fn invite(&self, et: &str, candidate: Uuid) -> Uuid {
        let r = self
            .call(
                Method::POST,
                "/employer/invitations",
                Some(et),
                Some(json!({
                    "candidate_id": candidate,
                    "title": "Backend-разработчик в платёжную команду",
                    "message": "Здравствуйте! Посмотрите, пожалуйста.",
                    "salary": { "min": 280000, "max": 340000 },
                    "work_format": "remote",
                    "contact": "@hr_rogakopyta",
                })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        r.body["id"].as_str().unwrap().parse().unwrap()
    }

    async fn key(&self, et: &str, scopes: &[&str]) -> String {
        let r = self
            .call(
                Method::POST,
                "/employer/integrations/keys",
                Some(et),
                Some(json!({ "name": "Huntflow", "scopes": scopes })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        r.body["secret"].as_str().unwrap().to_owned()
    }

    async fn webhook(&self, et: &str, url: &str) -> String {
        let r = self
            .call(
                Method::PUT,
                "/employer/integrations/webhook",
                Some(et),
                Some(json!({ "url": url })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        r.body["secret"].as_str().unwrap().to_owned()
    }

    /// Decrypted outbox payloads of the company, oldest first.
    async fn outbox(&self, company: Uuid) -> Vec<Value> {
        let mut events = db::integrations::company_events(self.pool(), company, 100)
            .await
            .unwrap();
        events.reverse();
        let mut out = Vec::new();
        for e in events {
            let row = db::integrations::deliverable_event(self.pool(), e.id)
                .await
                .unwrap()
                .unwrap();
            let body = open_payload(&self.state.crypto, e.id, &row.payload_enc).unwrap();
            out.push(serde_json::from_slice(&body).unwrap());
        }
        out
    }
}

fn types(events: &[Value]) -> Vec<&str> {
    events.iter().map(|e| e["type"].as_str().unwrap()).collect()
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn outbox_follows_the_invitation_and_discloses_only_after_acceptance(pool: PgPool) {
    let api = Api::start(pool, false).await;
    let cand = api.candidate("anna@example.ru", "Анна Смирнова").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, company) = api.employer("hr@example.ru").await;

    let before = api.invite(&et, cand).await;
    let r = api
        .call(
            Method::POST,
            &format!("/employer/invitations/{before}/withdraw"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert!(
        api.outbox(company).await.is_empty(),
        "no webhook — no events and no disclosure"
    );

    let secret = api.webhook(&et, "http://127.0.0.1:9/hook").await;
    assert!(secret.starts_with("whsec_"));
    let r = api
        .call(Method::GET, "/employer/integrations", Some(&et), None)
        .await;
    assert_eq!(r.body["webhook"]["url"], "http://127.0.0.1:9/hook");
    assert!(
        !r.body.to_string().contains(&secret),
        "the secret is shown once"
    );

    let inv = api.invite(&et, cand).await;
    let r = api
        .call(
            Method::GET,
            &format!("/candidate/invitations/{inv}"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.body["status"], "viewed");
    let events = api.outbox(company).await;
    assert_eq!(types(&events), ["invitation.sent", "invitation.viewed"]);
    for e in &events {
        let c = &e["data"]["candidate"];
        assert_eq!(c["code"], "K-ANNA");
        assert_eq!(c["category"]["grade"], "middle");
        assert!(c.get("id").is_none() && c.get("disclosed").is_none());
        let text = e.to_string();
        assert!(!text.contains("Анна") && !text.contains("anna@") && !text.contains("+7 900"));
    }
    assert_eq!(events[0]["data"]["invitation"]["salary"]["min"], 280000);
    assert_eq!(events[0]["company_id"], company.to_string());

    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/accept"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    let events = api.outbox(company).await;
    assert_eq!(
        types(&events),
        [
            "invitation.sent",
            "invitation.viewed",
            "invitation.accepted"
        ]
    );
    let accepted = &events[2];
    assert_eq!(accepted["data"]["invitation"]["status"], "accepted");
    let c = &accepted["data"]["candidate"];
    assert_eq!(c["id"], cand.to_string());
    assert_eq!(c["disclosed"]["full_name"], "Анна Смирнова");
    assert_eq!(c["disclosed"]["email"], "anna@example.ru");
    assert_eq!(c["disclosed"]["telegram"], "@anna_dev");
    assert_eq!(c["disclosed"]["basis"], "invitation");
    assert_eq!(
        c["disclosed"]["profile_pdf_url"],
        format!("http://localhost:3000/api/v1/integrations/candidates/{cand}/profile.pdf")
    );

    let r = api
        .call(Method::GET, "/candidate/contact-access", Some(&ct), None)
        .await;
    let log = r.body.as_array().unwrap();
    assert_eq!(
        log.len(),
        1,
        "the webhook disclosure is in the candidate's access log"
    );
    assert_eq!(log[0]["channel"], "webhook");

    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}/contacts"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    let events = api.outbox(company).await;
    let opened = events.last().unwrap();
    assert_eq!(opened["type"], "contacts.opened");
    assert_eq!(opened["data"]["candidate"]["id"], cand.to_string());
    assert!(opened["data"]["candidate"].get("disclosed").is_none());
}

#[sqlx::test(
    migrations = "../../db/migrations",
    fixtures("fixtures/outbox_rejects_acceptance.sql")
)]
async fn acceptance_rolls_back_when_the_outbox_fails(pool: PgPool) {
    let api = Api::start(pool, false).await;
    let cand = api.candidate("boris@example.ru", "Борис Петров").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, company) = api.employer("hr2@example.ru").await;
    api.webhook(&et, "http://127.0.0.1:9/hook").await;
    let inv = api.invite(&et, cand).await;

    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/accept"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::INTERNAL_SERVER_ERROR);

    let r = api
        .call(
            Method::GET,
            &format!("/employer/invitations/{inv}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.body["status"], "sent", "the status change rolled back");
    assert_eq!(r.body["events"].as_array().unwrap().len(), 1);
    let r = api
        .call(Method::GET, "/candidate/contact-access", Some(&ct), None)
        .await;
    assert_eq!(r.body, json!([]), "the access log entry rolled back too");
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}/contacts"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(types(&api.outbox(company).await), ["invitation.sent"]);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn api_keys_authenticate_only_the_integration_tree(pool: PgPool) {
    let api = Api::start(pool, false).await;
    let (et, _) = api.employer("hr3@example.ru").await;

    let r = api
        .call(
            Method::POST,
            "/employer/integrations/keys",
            Some(&et),
            Some(json!({ "name": "  ", "scopes": [] })),
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(r.body["errors"]["name"].is_array() && r.body["errors"]["scopes"].is_array());

    let key = api.key(&et, &["invitations:read"]).await;
    assert!(key.starts_with("fsp_live_") && key.len() == "fsp_live_".len() + 43);
    let r = api
        .call(Method::GET, "/employer/integrations/keys", Some(&et), None)
        .await;
    let listed = &r.body[0];
    assert!(key.starts_with(listed["prefix"].as_str().unwrap()));
    assert_eq!(listed["scopes"], json!(["invitations:read"]));
    assert!(listed["last_used_at"].is_null());
    assert!(!r.body.to_string().contains(&key), "the key is shown once");

    let r = api
        .call(Method::GET, "/integrations/invitations", Some(&key), None)
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["items"], json!([]));
    let r = api
        .call(Method::GET, "/integrations/whoami", Some(&key), None)
        .await;
    assert_eq!(r.body["company_name"], "Рога и копыта");

    let r = api
        .call(
            Method::POST,
            "/integrations/outcomes",
            Some(&key),
            Some(
                json!({ "invitation_id": Uuid::now_v7(), "stage": "interview", "at": Utc::now() }),
            ),
        )
        .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    assert_eq!(r.body["code"], "insufficient_scope");

    let r = api
        .call(Method::GET, "/integrations/invitations", Some(&et), None)
        .await;
    assert_eq!(
        r.status,
        StatusCode::UNAUTHORIZED,
        "a JWT is not an API key"
    );
    assert_eq!(r.body["code"], "api_key_required");
    let r = api
        .send(
            api.http
                .get(format!("{}/api/v1/integrations/invitations", api.base))
                .header(header::COOKIE, format!("fsp_at={et}")),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::UNAUTHORIZED,
        "cookies do not reach the tree"
    );
    let r = api
        .call(Method::GET, "/employer/integrations", Some(&key), None)
        .await;
    assert_eq!(
        r.status,
        StatusCode::UNAUTHORIZED,
        "a key does not open the cabinet"
    );
    let r = api
        .call(
            Method::GET,
            "/integrations/invitations",
            Some("fsp_live_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
            None,
        )
        .await;
    assert_eq!(r.body["code"], "api_key_invalid");

    let r = api
        .call(Method::GET, "/employer/integrations/keys", Some(&et), None)
        .await;
    assert!(!r.body[0]["last_used_at"].is_null());
    let id = r.body[0]["id"].as_str().unwrap().to_owned();
    let r = api
        .call(
            Method::DELETE,
            &format!("/employer/integrations/keys/{id}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = api
        .call(Method::GET, "/integrations/invitations", Some(&key), None)
        .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(r.body["code"], "api_key_invalid");
    assert_eq!(r.headers[header::CONTENT_TYPE], "application/problem+json");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn outcomes_reach_the_candidate_and_the_funnel(pool: PgPool) {
    let api = Api::start(pool, false).await;
    let cand = api.candidate("vera@example.ru", "Вера Иванова").await;
    let other = api.candidate("gleb@example.ru", "Глеб Орлов").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _) = api.employer("hr4@example.ru").await;
    let key = api
        .key(
            &et,
            &["invitations:read", "candidates:read", "outcomes:write"],
        )
        .await;
    let inv = api.invite(&et, cand).await;
    let outcome = |stage: &str| json!({ "invitation_id": inv, "stage": stage, "at": Utc::now() });

    let r = api
        .call(
            Method::POST,
            "/integrations/outcomes",
            Some(&key),
            Some(outcome("interview")),
        )
        .await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["code"], "outcome_not_allowed");
    let r = api
        .call(
            Method::GET,
            &format!("/integrations/candidates/{cand}"),
            Some(&key),
            None,
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::NOT_FOUND,
        "no contacts before acceptance"
    );

    api.call(
        Method::POST,
        &format!("/candidate/invitations/{inv}/accept"),
        Some(&ct),
        None,
    )
    .await;

    let r = api
        .call(
            Method::GET,
            "/integrations/invitations?status=accepted",
            Some(&key),
            None,
        )
        .await;
    let item = &r.body["items"][0];
    assert_eq!(item["candidate"]["id"], cand.to_string());
    assert_eq!(item["contacts_available"], true);
    assert!(item["candidate"].get("disclosed").is_none());
    let since = r.body["next_since"].as_str().unwrap().to_owned();
    assert!(
        since.ends_with('Z'),
        "UTC cursor, safe in a query string: {since}"
    );
    let r = api
        .call(
            Method::GET,
            &format!("/integrations/invitations?since={since}"),
            Some(&key),
            None,
        )
        .await;
    assert_eq!(
        r.body["items"],
        json!([]),
        "nothing changed after the cursor"
    );

    let r = api
        .call(
            Method::GET,
            &format!("/integrations/candidates/{cand}"),
            Some(&key),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["full_name"], "Вера Иванова");
    assert_eq!(r.body["phone"], "+7 900 123-45-67");
    assert_eq!(r.body["basis"], "invitation");
    assert_eq!(r.body["category"]["specialization"], "backend");
    let r = api
        .call(
            Method::GET,
            &format!("/integrations/candidates/{other}"),
            Some(&key),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let res = api
        .http
        .get(format!(
            "{}/api/v1/integrations/candidates/{cand}/profile.pdf",
            api.base
        ))
        .bearer_auth(&key)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "application/pdf");
    assert!(res.bytes().await.unwrap().starts_with(b"%PDF"));
    let r = api
        .call(Method::GET, "/candidate/contact-access", Some(&ct), None)
        .await;
    assert!(
        r.body
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["channel"] == "api"),
        "{}",
        r.body
    );

    let r = api
        .call(
            Method::POST,
            "/integrations/outcomes",
            Some(&key),
            Some(outcome("interview")),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let first = r.body["id"].clone();
    let mut again = outcome("interview");
    again["reason"] = json!("Перенесли на четверг");
    let r = api
        .call(
            Method::POST,
            "/integrations/outcomes",
            Some(&key),
            Some(again),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::OK,
        "a repeated stage updates, not duplicates"
    );
    assert_eq!(r.body["id"], first);
    let r = api
        .call(
            Method::POST,
            "/integrations/outcomes",
            Some(&key),
            Some(json!({ "invitation_id": inv, "application_id": inv, "stage": "offer", "at": Utc::now() })),
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = api
        .call(
            Method::POST,
            "/integrations/outcomes",
            Some(&key),
            Some(json!({ "invitation_id": Uuid::now_v7(), "stage": "offer", "at": Utc::now() })),
        )
        .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);

    let r = api
        .call(
            Method::GET,
            &format!("/candidate/invitations/{inv}"),
            Some(&ct),
            None,
        )
        .await;
    let outcomes = r.body["outcomes"].as_array().unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0]["stage"], "interview");
    assert_eq!(outcomes[0]["reason"], "Перенесли на четверг");

    let r = api
        .call(Method::GET, "/employer/invitations/stats", Some(&et), None)
        .await;
    assert_eq!(
        r.body["hiring"],
        json!([{ "stage": "interview", "count": 1 }])
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn vacancy_import_creates_a_need(pool: PgPool) {
    let api = Api::start(pool, false).await;
    let (et, _) = api.employer("hr5@example.ru").await;
    let key = api.key(&et, &["needs:write"]).await;

    let r = api
        .call(
            Method::POST,
            "/integrations/needs",
            Some(&key),
            Some(json!({
                "title": "Senior Backend-разработчик (Go)",
                "text": "Платёжная команда, высокие нагрузки.\nТребования: Go, PostgreSQL, Kubernetes, Kafka.\nБудет плюсом: ClickHouse.\nЗарплата от 350 000 до 450 000 ₽, удалённо.",
            })),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert_eq!(r.body["specialization"], "backend");
    assert!(
        r.body["grades"]
            .as_array()
            .unwrap()
            .contains(&json!("senior"))
    );
    assert!(
        r.body["must_skills"]
            .as_array()
            .unwrap()
            .contains(&json!("go"))
    );
    assert_eq!(r.body["salary"], json!({ "min": 350000, "max": 450000 }));
    let id = r.body["need_id"].as_str().unwrap().to_owned();
    assert_eq!(
        r.body["shortlist_url"],
        format!("http://localhost:3000/employer/needs/{id}")
    );
    let r = api
        .call(
            Method::GET,
            &format!("/employer/needs/{id}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["title"], "Senior Backend-разработчик (Go)");

    let r = api
        .call(
            Method::POST,
            "/integrations/needs",
            Some(&key),
            Some(json!({ "text": "Backend-разработчик на Go, PostgreSQL" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(r.body["errors"]["salary"].is_array(), "{}", r.body);
    let r = api
        .call(
            Method::POST,
            "/integrations/needs",
            Some(&key),
            Some(json!({ "text": "Backend-разработчик на Go", "salary": { "min": 300000, "max": 400000 } })),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    assert!(
        !r.body["notes"].as_array().unwrap().is_empty(),
        "grade assumed"
    );
    let r = api
        .call(
            Method::POST,
            "/integrations/needs",
            Some(&key),
            Some(json!({ "text": "" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
}

async fn wait_for(mock: &ats_mock::AppState, event_type: &str) -> ats_mock::Entry {
    for _ in 0..150 {
        let found = mock.entries().into_iter().find(|e| {
            matches!(e, ats_mock::Entry::Webhook { event_type: Some(t), .. } if t == event_type)
        });
        if let Some(e) = found {
            return e;
        }
        tokio::time::sleep(StdDuration::from_millis(100)).await;
    }
    panic!("{event_type} did not reach the ATS mock");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn round_trip_through_the_ats_mock(pool: PgPool) {
    let api = Api::start(pool, true).await;
    let cand = api.candidate("dina@example.ru", "Дина Ким").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _) = api.employer("hr6@example.ru").await;
    let key = api
        .key(
            &et,
            &[
                "invitations:read",
                "candidates:read",
                "outcomes:write",
                "needs:write",
            ],
        )
        .await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_url = format!("http://{}", listener.local_addr().unwrap());
    let secret = api.webhook(&et, &format!("{mock_url}/webhooks")).await;
    let mock = ats_mock::AppState::new(ats_mock::Config {
        secret,
        api_url: api.base.clone(),
        api_key: Some(key),
    })
    .unwrap();
    let router = ats_mock::router(mock.clone());
    tokio::spawn(async move { axum::serve(listener, router).await });

    let r = api
        .call(
            Method::POST,
            "/employer/integrations/webhook/test",
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.body["delivered"], true, "{}", r.body);
    assert_eq!(r.body["status_code"], 200);

    let inv = api.invite(&et, cand).await;
    let sent = wait_for(&mock, "invitation.sent").await;
    let ats_mock::Entry::Webhook { verdict, body, .. } = sent else {
        unreachable!()
    };
    assert_eq!(verdict, ats_mock::Verdict::Valid);
    assert!(body["data"]["candidate"].get("disclosed").is_none());

    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/accept"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    let accepted = wait_for(&mock, "invitation.accepted").await;
    let ats_mock::Entry::Webhook {
        verdict,
        body,
        event_id,
        duplicate,
        ..
    } = accepted
    else {
        unreachable!()
    };
    assert_eq!(
        verdict,
        ats_mock::Verdict::Valid,
        "signature verified by the mock"
    );
    assert!(!duplicate);
    assert_eq!(event_id.as_deref(), body["id"].as_str());
    assert_eq!(
        body["data"]["candidate"]["disclosed"]["full_name"],
        "Дина Ким"
    );

    let r = api
        .call(
            Method::GET,
            "/employer/integrations/deliveries",
            Some(&et),
            None,
        )
        .await;
    let delivered = r
        .body
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["event_type"] == "invitation.accepted")
        .unwrap();
    assert_eq!(delivered["status"], "delivered");
    assert_eq!(delivered["attempts"][0]["status_code"], 200);

    let res = reqwest::Client::new()
        .post(format!("{mock_url}/outcomes"))
        .form(&[
            ("subject", "invitation"),
            ("id", &inv.to_string()),
            ("stage", "interview"),
            ("reason", "Зовём на техническое интервью"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "the mock redirects back to its page"
    );
    let sent_back = mock
        .entries()
        .into_iter()
        .find(|e| matches!(e, ats_mock::Entry::Outcome { .. }))
        .unwrap();
    let ats_mock::Entry::Outcome { status, .. } = sent_back else {
        unreachable!()
    };
    assert_eq!(status, Some(201));

    let r = api
        .call(Method::GET, "/candidate/invitations", Some(&ct), None)
        .await;
    let outcomes = &r.body[0]["outcomes"];
    assert_eq!(outcomes[0]["stage"], "interview");
    assert_eq!(outcomes[0]["reason"], "Зовём на техническое интервью");

    let r = api
        .call(
            Method::POST,
            "/employer/integrations/webhook/rotate",
            Some(&et),
            None,
        )
        .await;
    assert!(r.body["secret"].as_str().unwrap().starts_with("whsec_"));
    assert!(!r.body["webhook"]["previous_secret_valid_until"].is_null());
    let r = api
        .call(
            Method::POST,
            "/employer/integrations/webhook/test",
            Some(&et),
            None,
        )
        .await;
    assert_eq!(
        r.body["delivered"], true,
        "the old secret still signs during the overlap: {}",
        r.body
    );

    let r = api
        .call(
            Method::DELETE,
            "/employer/integrations/webhook",
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = api
        .call(Method::GET, "/employer/integrations", Some(&et), None)
        .await;
    assert!(r.body["webhook"].is_null());
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn webhook_url_is_guarded(pool: PgPool) {
    let api = Api::start(pool, false).await;
    let (et, _) = api.employer("hr7@example.ru").await;
    for (url, why) in [
        ("http://ats.example.ru/hook", "plain http"),
        ("https://10.0.0.5/hook", "private address"),
        ("https://169.254.169.254/latest", "cloud metadata"),
        ("https://user:pass@93.184.216.34/", "credentials"),
        ("ftp://ats.example.ru/", "scheme"),
    ] {
        let r = api
            .call(
                Method::PUT,
                "/employer/integrations/webhook",
                Some(&et),
                Some(json!({ "url": url })),
            )
            .await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{why}");
        assert!(r.body["errors"]["url"].is_array(), "{why}");
    }
    let r = api
        .call(
            Method::POST,
            "/employer/integrations/webhook/test",
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
}
