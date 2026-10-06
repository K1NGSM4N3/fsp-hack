//! 152-ФЗ flows over real HTTP against a throwaway database: publication consent gating, the
//! candidate's contact access log, the data export, the age gate for 14–17-year-olds.

use std::sync::Arc;

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, ProctoringConfig},
    crypto::{Crypto, sha256},
    integrations::webhook::WebhookSender,
    kv::Kv,
    mail::Mailer,
    proctoring::{ProctoringMode, RecordingStore},
    state::{AppState, Services},
};
use chrono::{Duration, Months, NaiveDate, Utc};
use reqwest::{Method, StatusCode, header};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const TERMS_V1: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);
const TERMS_V2: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_0000000000f3);
const GUARDIAN: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000a01);
const PROCESSING_V2: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000a02);

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
    async fn start(pool: PgPool) -> Self {
        Self::start_with(pool, None).await
    }

    async fn start_with(pool: PgPool, valkey: Option<&str>) -> Self {
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
                dev_link: false,
            },
            grade_change_cooldown_days: 90,
            proctoring: ProctoringConfig {
                mode: ProctoringMode::Optional,
                dir: std::env::temp_dir().join(format!("privacy-test-{}", Uuid::now_v7())),
                retention_days: 90,
            },
            integrations: IntegrationsConfig::default(),
            notifications: Default::default(),
            seed_demo: false,
            seed_per_spec: 40,
            log_json: false,
        };
        let kv = Kv::connect(valkey).await.unwrap();
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
        let res = req.send().await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.bytes().await.unwrap();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        Reply {
            status,
            headers,
            body,
        }
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

    /// A backend:middle candidate who registered with every required consent.
    async fn candidate(&self, email: &str) -> Uuid {
        let id = self.account(Role::Candidate, email).await;
        db::candidates::insert_candidate(self.pool(), id, &format!("K-{}", &email[..6]))
            .await
            .unwrap();
        db::grades::set_candidate_category(self.pool(), "backend", "middle", 0.4, Utc::now(), id)
            .await
            .unwrap();
        for doc in [PROCESSING, PUBLICATION, TERMS_V1] {
            db::consents::insert_user_consent(
                self.pool(),
                Uuid::now_v7(),
                id,
                doc,
                Some("10.1.1.1"),
                None,
            )
            .await
            .unwrap();
        }
        // Written past the API, so nothing bumped the search cache the way a handler would.
        self.state.candidates.invalidate().await;
        id
    }

    async fn employer(&self, email: &str) -> (Uuid, Uuid) {
        let owner = self.account(Role::Employer, email).await;
        let company = Uuid::now_v7();
        db::companies::insert_company(self.pool(), company, owner, "Рога и копыта")
            .await
            .unwrap();
        (owner, company)
    }

    /// A published backend:middle candidate born on `birth` with a representative on file.
    async fn minor(&self, email: &str, birth: NaiveDate) -> (Uuid, Uuid) {
        let id = self.candidate(email).await;
        db::guardians::set_birth_date(self.pool(), id, birth)
            .await
            .unwrap();
        let request = Uuid::now_v7();
        let parent = format!("parent.{email}");
        db::guardians::insert_request(
            self.pool(),
            request,
            id,
            &self.state.crypto.encrypt_str("Петрова Мария").unwrap(),
            &self.state.crypto.encrypt_str(&parent).unwrap(),
            &self.state.crypto.blind_index(&parent),
        )
        .await
        .unwrap();
        self.state.candidates.invalidate().await;
        (id, request)
    }

    /// Links of a letter with known tokens: (answer link, revocation link).
    async fn letter(&self, request: Uuid, expires_in: Duration) -> (String, String) {
        let (confirm, revoke) = (format!("confirm-{request}"), format!("revoke-{request}"));
        db::guardians::set_letter(
            self.pool(),
            request,
            &sha256(confirm.as_bytes()),
            &sha256(revoke.as_bytes()),
            Utc::now() + expires_in,
        )
        .await
        .unwrap();
        (confirm, revoke)
    }

    /// The representative answers from their own browser.
    async fn guardian(&self, token: &str, action: &str) -> Reply {
        let res = self
            .http
            .post(format!("{}/api/v1/guardian/consent", self.base))
            .header("x-forwarded-for", "203.0.113.9")
            .header(header::USER_AGENT, "Firefox/140")
            .json(&json!({ "token": token, "action": action }))
            .send()
            .await
            .unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        Reply {
            status,
            headers,
            body: res.json().await.unwrap(),
        }
    }

    async fn in_pool(&self, id: Uuid) -> bool {
        db::pool::searchable_pool(self.pool(), "backend")
            .await
            .unwrap()
            .iter()
            .any(|c| c.user_id == id)
    }
}

fn privacy(visible: bool) -> Value {
    json!({
        "visible": visible,
        "show_name": false,
        "show_fsp": true,
        "show_experience": true,
        "search_status": "open",
    })
}

fn found(search: &Reply, id: Uuid) -> bool {
    search.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["candidate"]["id"] == id.to_string())
}

#[sqlx::test(migrations = "../../db/migrations", fixtures("fixtures/terms_v2.sql"))]
async fn publication_consent_gates_visibility(pool: PgPool) {
    let api = Api::start(pool).await;
    let cand = api.candidate("anna@example.ru").await;
    let (hr, _) = api.employer("hr@example.ru").await;
    let ct = api.token(cand, Role::Candidate);
    let et = api.token(hr, Role::Employer);
    let search = "/employer/candidates?specialization=backend";

    assert!(api.in_pool(cand).await);
    assert!(found(
        &api.call(Method::GET, search, Some(&et), None).await,
        cand
    ));

    let r = api
        .call(
            Method::POST,
            &format!("/me/consents/{PUBLICATION}/revoke"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = api
        .call(Method::GET, "/candidate/privacy", Some(&ct), None)
        .await;
    assert_eq!(
        r.body["visible"], false,
        "revoking publication hides the profile"
    );
    assert!(!api.in_pool(cand).await);
    let r = api.call(Method::GET, search, Some(&et), None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(!found(&r, cand));
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::NOT_FOUND,
        "no unrelated employer reaches the card"
    );

    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.headers[header::CONTENT_TYPE], "application/problem+json");
    assert_eq!(r.body["code"], "consent_required");
    assert_eq!(r.body["document_id"], PUBLICATION.to_string());
    assert_eq!(r.body["document_kind"], "pd_publication");
    let r = api
        .call(Method::GET, "/candidate/privacy", Some(&ct), None)
        .await;
    assert_eq!(r.body["visible"], false);
    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(false)),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "staying hidden needs no consent");

    for (doc, why) in [
        (TERMS_V1, "an outdated version"),
        (Uuid::now_v7(), "an unknown document"),
    ] {
        let r = api
            .call(
                Method::POST,
                "/me/consents",
                Some(&ct),
                Some(json!({ "document_id": doc })),
            )
            .await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{why}");
        assert!(r.body["errors"]["document_id"].is_array());
    }
    let r = api
        .call(
            Method::POST,
            "/me/consents",
            Some(&et),
            Some(json!({ "document_id": PUBLICATION })),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "publication is a candidate's document"
    );
    let r = api
        .call(
            Method::POST,
            "/me/consents",
            Some(&et),
            Some(json!({ "document_id": TERMS_V2 })),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::CREATED,
        "the current terms apply to employers"
    );

    let granted = api
        .call(
            Method::POST,
            "/me/consents",
            Some(&ct),
            Some(json!({ "document_id": PUBLICATION })),
        )
        .await;
    assert_eq!(granted.status, StatusCode::CREATED);
    assert_eq!(granted.body["kind"], "pd_publication");
    assert!(granted.body["revoked_at"].is_null());
    let again = api
        .call(
            Method::POST,
            "/me/consents",
            Some(&ct),
            Some(json!({ "document_id": PUBLICATION })),
        )
        .await;
    assert_eq!(again.status, StatusCode::OK, "granting twice is idempotent");
    assert_eq!(again.body["id"], granted.body["id"]);
    assert!(
        !api.in_pool(cand).await,
        "consent alone does not switch visibility on"
    );

    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(api.in_pool(cand).await);
    assert!(found(
        &api.call(Method::GET, search, Some(&et), None).await,
        cand
    ));

    let r = api
        .call(
            Method::POST,
            &format!("/me/consents/{PROCESSING}/revoke"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["document_kind"], "pd_processing");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn access_log_and_export(pool: PgPool) {
    let api = Api::start(pool).await;
    let cand = api.candidate("boris@example.ru").await;
    let (hr, company) = api.employer("hr2@example.ru").await;
    let ct = api.token(cand, Role::Candidate);
    let et = api.token(hr, Role::Employer);

    let invitation = Uuid::now_v7();
    db::invitations::insert_invitation(
        api.pool(),
        &db::invitations::NewInvitation {
            id: invitation,
            company_id: company,
            candidate_id: cand,
            need_id: None,
            vacancy_id: None,
            title: "Backend-разработчик",
            message: "Здравствуйте!",
            salary_min: 200_000,
            salary_max: 300_000,
            work_format: Some("remote"),
            contact: "@hr",
            idempotency_key: None,
            expires_at: Utc::now() + Duration::days(14),
        },
    )
    .await
    .unwrap();

    let r = api
        .call(Method::GET, "/candidate/contact-access", Some(&ct), None)
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body, json!([]));
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}/contacts"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN, "not accepted yet");
    db::invitations::respond_invitation(
        api.pool(),
        db::types::InvitationStatus::Accepted,
        None,
        None,
        invitation,
        cand,
    )
    .await
    .unwrap();
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}/contacts"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);

    let r = api
        .call(Method::GET, "/candidate/contact-access", Some(&ct), None)
        .await;
    let log = r.body.as_array().unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0]["company_name"], "Рога и копыта");
    assert_eq!(log[0]["company_id"], company.to_string());
    assert_eq!(log[0]["basis"], "invitation");
    let keys: Vec<&str> = log[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert!(
        !keys
            .iter()
            .any(|k| ["employer_user_id", "ip", "user_agent"].contains(k)),
        "the employee and their network stay in the audit log: {keys:?}"
    );
    let r = api
        .call(Method::GET, "/candidate/contact-access", Some(&et), None)
        .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);

    let session = Uuid::now_v7();
    let started = Utc::now() - Duration::days(3);
    db::seed::seed_session(
        api.pool(),
        &db::seed::SeedSession {
            id: session,
            candidate_id: cand,
            specialization: "backend",
            claimed_grade: "middle",
            attempt_kind: "placement",
            status: db::types::AssessmentStatus::Passed,
            state: &json!({ "secret": "internal state" }),
            theta: 0.4,
            se: 0.3,
            measured_grade: Some("middle"),
            confident_above_next: false,
            person_fit_lz: 0.1,
            aberrant: false,
            items_count: 12,
            started_at: started,
            finished_at: started + Duration::minutes(15),
        },
    )
    .await
    .unwrap();
    db::seed::seed_proctoring(
        api.pool(),
        &db::seed::SeedProctoring {
            session_id: session,
            candidate_id: cand,
            camera: true,
            consent_id: None,
            started_at: started,
            purged_at: None,
        },
    )
    .await
    .unwrap();
    db::seed::seed_chunks(api.pool(), session, 90, 120_000, started)
        .await
        .unwrap();
    db::proctoring::insert_event(
        api.pool(),
        session,
        &db::proctoring::Event {
            kind: db::types::IntegrityEvent::TabHidden,
            occurred_at: started + Duration::minutes(2),
            item_seq: Some(3),
            duration_ms: Some(4_000),
        },
    )
    .await
    .unwrap();
    let r = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{session}/proctoring/recording"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::NOT_FOUND,
        "chunk rows without files are not playable"
    );
    assert_eq!(r.body["code"], "recording_unavailable");

    let r = api.call(Method::GET, "/me/export", Some(&ct), None).await;
    assert_eq!(r.status, StatusCode::OK);
    let disposition = r.headers[header::CONTENT_DISPOSITION].to_str().unwrap();
    assert!(disposition.starts_with("attachment; filename=\"fsp-talents-"));
    let e = &r.body;
    assert_eq!(e["format"], "fsp-talents-export/1");
    assert_eq!(e["account"]["email"], "boris@example.ru");
    assert_eq!(e["account"]["role"], "candidate");
    assert!(e["employer"].is_null());
    let consents = e["consents"].as_array().unwrap();
    assert_eq!(consents.len(), 3);
    assert!(
        consents
            .iter()
            .all(|c| c["ip"] == "10.1.1.1" && c["version"] == 1)
    );
    let c = &e["candidate"];
    assert_eq!(c["privacy"]["visible"], true);
    assert_eq!(c["contact_access"][0]["company_name"], "Рога и копыта");
    assert_eq!(c["invitations"][0]["status"], "accepted");
    let a = &c["assessments"][0];
    assert_eq!(a["status"], "passed");
    assert_eq!(a["proctoring"]["status"], "recorded");
    assert_eq!(a["proctoring"]["chunks"], 90);
    assert_eq!(a["proctoring"]["events"]["tab_hidden"], 1);
    let text = serde_json::to_string(e).unwrap();
    for internal in ["internal state", "template_id", "\"seed\"", "current_item"] {
        assert!(
            !text.contains(internal),
            "task internals leaked: {internal}"
        );
    }

    let r = api.call(Method::GET, "/me/export", Some(&et), None).await;
    assert_eq!(r.status, StatusCode::OK);
    let e = &r.body;
    assert_eq!(e["account"]["role"], "employer");
    assert!(e["candidate"].is_null());
    assert_eq!(e["employer"]["company"]["name"], "Рога и копыта");
    assert_eq!(
        e["employer"]["invitations"][0]["candidate_id"],
        cand.to_string()
    );

    let r = api.call(Method::GET, "/me/export", None, None).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

fn years_ago(years: u32) -> NaiveDate {
    Utc::now()
        .date_naive()
        .checked_sub_months(Months::new(years * 12))
        .unwrap()
}

fn register(email: &str, age: Option<&str>, minor: Option<Value>) -> Value {
    json!({
        "email": email,
        "password": "secret123",
        "role": "candidate",
        "consents": [PROCESSING_V2, PUBLICATION, TERMS_V1],
        "age": age,
        "minor": minor,
    })
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn registration_declares_age_and_mails_the_representative_after_verification(pool: PgPool) {
    let api = Api::start(pool).await;
    let guardian = |birth: NaiveDate, email: &str| {
        Some(json!({
            "birth_date": birth,
            "guardian": { "full_name": "Петрова Мария Ивановна", "email": email },
        }))
    };
    let cases = [
        (register("a@example.ru", None, None), "age"),
        (
            register(
                "b@example.ru",
                Some("minor"),
                guardian(years_ago(13), "mama@example.ru"),
            ),
            "minor.birth_date",
        ),
        (
            register(
                "c@example.ru",
                Some("minor"),
                guardian(years_ago(18), "mama@example.ru"),
            ),
            "minor.birth_date",
        ),
        (
            register(
                "kid@example.ru",
                Some("minor"),
                guardian(years_ago(15), "KID@example.ru"),
            ),
            "minor.guardian.email",
        ),
        (register("d@example.ru", Some("minor"), None), "minor"),
    ];
    for (body, field) in cases {
        let r = api
            .call(Method::POST, "/auth/register", None, Some(body))
            .await;
        assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{field}");
        assert!(r.body["errors"][field].is_array(), "{field}: {}", r.body);
    }

    let r = api
        .call(
            Method::POST,
            "/auth/register",
            None,
            Some(register("adult@example.ru", Some("adult"), None)),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED);
    let adult: Uuid = serde_json::from_value(r.body["user_id"].clone()).unwrap();
    let age = db::guardians::candidate_age(api.pool(), adult)
        .await
        .unwrap()
        .unwrap();
    assert!(
        age.birth_date.is_none() && !age.minor,
        "adults give no birth date"
    );
    let grants = db::consents::user_consents(api.pool(), adult)
        .await
        .unwrap();
    assert!(
        grants
            .iter()
            .all(|g| g.age_bracket == Some(db::types::AgeBracket::Adult))
    );

    let r = api
        .call(
            Method::POST,
            "/auth/register",
            None,
            Some(json!({
                "email": "hr@firm.ru", "password": "secret123", "role": "employer",
                "company_name": "Фирма", "consents": [PROCESSING_V2, TERMS_V1],
            })),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::CREATED,
        "employers declare 18+ in the form"
    );
    let hr: Uuid = serde_json::from_value(r.body["user_id"].clone()).unwrap();
    assert!(
        db::consents::user_consents(api.pool(), hr)
            .await
            .unwrap()
            .iter()
            .all(|g| g.age_bracket == Some(db::types::AgeBracket::Adult))
    );

    let birth = years_ago(15);
    let r = api
        .call(
            Method::POST,
            "/auth/register",
            None,
            Some(register(
                "teen@example.ru",
                Some("minor"),
                guardian(birth, "Mama@Example.ru"),
            )),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let teen: Uuid = serde_json::from_value(r.body["user_id"].clone()).unwrap();
    let age = db::guardians::candidate_age(api.pool(), teen)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(age.birth_date, Some(birth));
    assert!(age.minor);
    assert!(
        db::consents::user_consents(api.pool(), teen)
            .await
            .unwrap()
            .iter()
            .all(|g| g.age_bracket == Some(db::types::AgeBracket::Minor))
    );
    let request = db::guardians::latest_request(api.pool(), teen)
        .await
        .unwrap()
        .unwrap();
    assert!(
        request.sent_at.is_none(),
        "no letter before the minor's own email is confirmed"
    );
    assert_eq!(
        api.state
            .crypto
            .decrypt_str(request.email_enc.as_deref().unwrap())
            .unwrap(),
        "mama@example.ru"
    );
    assert_eq!(
        request.email_bidx.as_deref(),
        Some(api.state.crypto.blind_index("mama@example.ru").as_slice())
    );

    db::users::insert_email_verification(
        api.pool(),
        &sha256(b"verify-teen"),
        teen,
        Utc::now() + Duration::hours(1),
    )
    .await
    .unwrap();
    let r = api
        .call(
            Method::POST,
            "/auth/verify-email",
            None,
            Some(json!({ "token": "verify-teen" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let request = db::guardians::latest_request(api.pool(), teen)
        .await
        .unwrap()
        .unwrap();
    assert!(
        request.sent_at.is_some(),
        "the letter goes out after verification"
    );
    assert!(request.confirm_expires_at.unwrap() > Utc::now() + Duration::days(6));

    let r = api
        .call(
            Method::GET,
            "/me",
            Some(&api.token(teen, Role::Candidate)),
            None,
        )
        .await;
    assert_eq!(r.body["candidate"]["age"]["bracket"], "minor");
    assert_eq!(r.body["candidate"]["age"]["guardian"], "pending");
    assert_eq!(
        r.body["candidate"]["age"]["adult_from"],
        json!(birth.checked_add_months(Months::new(216)).unwrap())
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn minor_is_published_only_with_the_representatives_consent(pool: PgPool) {
    let api = Api::start(pool).await;
    let (teen, request) = api.minor("teen@example.ru", years_ago(16)).await;
    let (hr, company) = api.employer("hr@example.ru").await;
    let ct = api.token(teen, Role::Candidate);
    let et = api.token(hr, Role::Employer);
    let search = "/employer/candidates?specialization=backend";
    let adults = "/employer/candidates?specialization=backend&adults_only=true";

    assert!(
        !api.in_pool(teen).await,
        "no consent of the representative yet"
    );
    assert!(!found(
        &api.call(Method::GET, search, Some(&et), None).await,
        teen
    ));
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{teen}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["code"], "guardian_consent_required");
    assert_eq!(r.body["document_kind"], "guardian_consent");
    assert_eq!(r.body["document_id"], GUARDIAN.to_string());
    let r = api
        .call(
            Method::POST,
            "/me/consents",
            Some(&ct),
            Some(json!({ "document_id": GUARDIAN })),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "a minor cannot consent for the representative"
    );
    let vacancy = Uuid::now_v7();
    db::vacancies::insert_vacancy(
        api.pool(),
        vacancy,
        company,
        db::types::VacancyStatus::Published,
        Some(Utc::now()),
        &db::vacancies::VacancyFields {
            title: "Стажёр-бэкендер",
            description: "Летняя стажировка",
            specialization: "backend",
            grade: "junior",
            skills: &[],
            salary_min: 40_000,
            salary_max: 60_000,
            work_format: "remote",
            city: None,
        },
    )
    .await
    .unwrap();
    let r = api
        .call(
            Method::POST,
            &format!("/vacancies/{vacancy}/applications"),
            Some(&ct),
            Some(json!({ "cover_letter": "" })),
        )
        .await;
    assert_eq!(
        r.body["code"], "guardian_consent_required",
        "applying would hand contacts to a company"
    );
    let r = api
        .call(Method::GET, "/candidate/guardian", Some(&ct), None)
        .await;
    assert_eq!(r.body["required"], true);
    assert_eq!(r.body["state"], "not_sent");
    assert_eq!(r.body["guardian_email"], "parent.teen@example.ru");

    let (confirm, revoke) = api.letter(request, Duration::days(7)).await;
    let r = api
        .call(
            Method::GET,
            &format!("/guardian/consent?token={confirm}"),
            None,
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["link"], "confirm");
    assert_eq!(r.body["state"], "pending");
    assert_eq!(r.body["actions"], json!(["grant", "decline"]));
    assert_eq!(r.body["participant"], "te***@example.ru");
    assert_eq!(r.body["guardian_name"], "Петрова Мария");
    assert_eq!(r.body["document"]["kind"], "guardian_consent");
    let r = api
        .call(
            Method::GET,
            &format!("/guardian/consent?token={revoke}"),
            None,
            None,
        )
        .await;
    assert_eq!(r.body["actions"], json!(["decline"]));
    let r = api.guardian(&revoke, "grant").await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(
        r.body["code"], "action_unavailable",
        "granting needs the answer link"
    );

    let r = api.guardian(&confirm, "grant").await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["state"], "granted");
    assert_eq!(r.body["actions"], json!([]));
    let grant = db::consents::user_consents(api.pool(), teen)
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.kind == "guardian_consent")
        .unwrap();
    assert!(grant.revoked_at.is_none());
    assert!(api.in_pool(teen).await);
    let r = api.call(Method::GET, search, Some(&et), None).await;
    let item = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["candidate"]["id"] == teen.to_string())
        .expect("published after the representative's consent");
    assert_eq!(item["candidate"]["minor"], true);
    let other = api.candidate("adult@example.ru").await;
    let r = api.call(Method::GET, adults, Some(&et), None).await;
    assert!(!found(&r, teen), "«Только 18+» drops minors");
    assert!(found(&r, other), "and keeps everybody else");
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{teen}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["candidate"]["minor"], true);
    assert!(
        !r.body.to_string().contains("birth"),
        "employers never see the birth date"
    );
    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    let r = api
        .call(
            Method::POST,
            &format!("/vacancies/{vacancy}/applications"),
            Some(&ct),
            Some(json!({ "cover_letter": "" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);

    let r = api.guardian(&confirm, "grant").await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["code"], "token_used", "the answer link works once");
    let r = api.guardian(&confirm, "decline").await;
    assert_eq!(r.body["code"], "token_used");
    let r = api
        .call(
            Method::POST,
            &format!("/me/consents/{GUARDIAN}/revoke"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::NOT_FOUND,
        "only the representative revokes their consent"
    );
    let r = api
        .call(Method::POST, "/candidate/guardian/resend", Some(&ct), None)
        .await;
    assert_eq!(r.body["code"], "guardian_already_confirmed");

    let r = api.guardian(&revoke, "revoke").await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["state"], "revoked");
    assert!(!api.in_pool(teen).await);
    let r = api
        .call(Method::GET, "/candidate/privacy", Some(&ct), None)
        .await;
    assert_eq!(r.body["visible"], false, "revocation hides the profile");
    let r = api.call(Method::GET, "/me", Some(&ct), None).await;
    assert_eq!(r.body["candidate"]["age"]["guardian"], "revoked");
    let r = api.guardian(&revoke, "revoke").await;
    assert_eq!(r.status, StatusCode::CONFLICT);
    assert_eq!(r.body["code"], "token_used");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn expired_unknown_and_superseded_links_are_rejected(pool: PgPool) {
    let api = Api::start(pool).await;
    let (teen, request) = api.minor("teen@example.ru", years_ago(15)).await;
    let ct = api.token(teen, Role::Candidate);

    let r = api.guardian("no-such-token", "grant").await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body["code"], "invalid_token");

    let (confirm, revoke) = api.letter(request, Duration::seconds(-1)).await;
    let r = api
        .call(
            Method::GET,
            &format!("/guardian/consent?token={confirm}"),
            None,
            None,
        )
        .await;
    assert_eq!(r.body["state"], "expired");
    assert_eq!(r.body["actions"], json!([]));
    let r = api.guardian(&confirm, "grant").await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body["code"], "token_expired");
    assert!(!api.in_pool(teen).await);

    let r = api
        .call(Method::POST, "/candidate/guardian/resend", Some(&ct), None)
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert_eq!(r.body["state"], "pending");
    let r = api.guardian(&confirm, "grant").await;
    assert_eq!(
        r.body["code"], "invalid_token",
        "a new letter retires the links of the old one"
    );
    let r = api.guardian(&revoke, "decline").await;
    assert_eq!(r.body["code"], "invalid_token");

    let r = api
        .call(
            Method::PUT,
            "/candidate/guardian",
            Some(&ct),
            Some(json!({ "full_name": "Петров Иван", "email": "teen@example.ru" })),
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "not the minor's own address"
    );
    let r = api
        .call(
            Method::PUT,
            "/candidate/guardian",
            Some(&ct),
            Some(json!({ "full_name": "Петров Иван", "email": "papa@example.ru" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["guardian_email"], "papa@example.ru");
    assert_eq!(r.body["state"], "pending");

    let (_, revoke) = api.letter(request, Duration::days(7)).await;
    let r = api.guardian(&revoke, "decline").await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["state"], "declined");
    let r = api
        .call(Method::GET, "/candidate/guardian", Some(&ct), None)
        .await;
    assert_eq!(r.body["state"], "declined");
    let r = api
        .call(Method::GET, "/candidate/privacy", Some(&ct), None)
        .await;
    assert_eq!(r.body["visible"], false);
    let r = api
        .call(Method::POST, "/candidate/guardian/resend", Some(&ct), None)
        .await;
    assert_eq!(
        r.body["state"], "pending",
        "after a refusal the minor may ask again"
    );
    assert_eq!(
        db::guardians::requests(api.pool(), teen)
            .await
            .unwrap()
            .len(),
        2,
        "the refusal stays on record"
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn eighteenth_birthday_lifts_the_gate_and_starts_the_retention_clock(pool: PgPool) {
    let api = Api::start(pool).await;
    let today = Utc::now().date_naive();
    let (birthday, _) = api.minor("today@example.ru", years_ago(18)).await;
    let tomorrow = years_ago(18).succ_opt().unwrap();
    let (eve, _) = api.minor("eve@example.ru", tomorrow).await;
    assert!(
        api.in_pool(birthday).await,
        "18 today: no representative needed"
    );
    assert!(!api.in_pool(eve).await, "18 tomorrow: still waits");
    let r = api
        .call(
            Method::GET,
            "/me",
            Some(&api.token(birthday, Role::Candidate)),
            None,
        )
        .await;
    assert_eq!(r.body["candidate"]["age"]["bracket"], "adult");
    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&api.token(birthday, Role::Candidate)),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);

    let erased_birth = today
        .checked_sub_days(chrono::Days::new(30))
        .unwrap()
        .checked_sub_months(Months::new(216))
        .unwrap();
    let kept_birth = erased_birth.succ_opt().unwrap();
    let mut grants = Vec::new();
    for (email, birth) in [
        ("old@example.ru", erased_birth),
        ("new@example.ru", kept_birth),
    ] {
        let (id, request) = api.minor(email, birth).await;
        let (confirm, _) = api.letter(request, Duration::days(7)).await;
        db::guardians::set_birth_date(api.pool(), id, years_ago(16))
            .await
            .unwrap();
        let r = api.guardian(&confirm, "grant").await;
        assert_eq!(r.status, StatusCode::OK);
        db::guardians::set_birth_date(api.pool(), id, birth)
            .await
            .unwrap();
        grants.push(id);
    }
    let report = api::guardian::sweep(api.pool()).await.unwrap();
    assert_eq!(
        (report.requests, report.grants, report.birth_dates),
        (1, 1, 1),
        "exactly the one whose 18th birthday was 30 days ago"
    );
    let (old, new) = (grants[0], grants[1]);
    let old_request = db::guardians::latest_request(api.pool(), old)
        .await
        .unwrap()
        .unwrap();
    assert!(old_request.full_name_enc.is_none() && old_request.email_enc.is_none());
    assert!(old_request.erased_at.is_some() && old_request.consent_id.is_some());
    let old_grant = db::consents::user_consents(api.pool(), old)
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.kind == "guardian_consent")
        .unwrap();
    assert!(old_grant.ip.is_none() && old_grant.user_agent.is_none());
    assert!(
        db::guardians::candidate_age(api.pool(), old)
            .await
            .unwrap()
            .unwrap()
            .birth_date
            .is_none()
    );
    let new_request = db::guardians::latest_request(api.pool(), new)
        .await
        .unwrap()
        .unwrap();
    assert!(new_request.email_enc.is_some() && new_request.erased_at.is_none());
    assert_eq!(
        api::guardian::sweep(api.pool()).await.unwrap().requests,
        0,
        "idempotent"
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn export_and_deletion_cover_the_representative(pool: PgPool) {
    let api = Api::start(pool).await;
    let birth = years_ago(15);
    let (teen, request) = api.minor("teen@example.ru", birth).await;
    let ct = api.token(teen, Role::Candidate);
    let (confirm, _) = api.letter(request, Duration::days(7)).await;
    assert_eq!(api.guardian(&confirm, "grant").await.status, StatusCode::OK);

    let r = api.call(Method::GET, "/me/export", Some(&ct), None).await;
    let age = &r.body["candidate"]["age"];
    assert_eq!(age["bracket"], "minor");
    assert_eq!(age["birth_date"], json!(birth));
    let g = &age["guardian_requests"][0];
    assert_eq!(g["state"], "granted");
    assert_eq!(g["email"], "parent.teen@example.ru");
    assert_eq!(g["full_name"], "Петрова Мария");
    let grant = r.body["consents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "guardian_consent")
        .unwrap()
        .clone();
    assert_eq!(grant["id"], g["consent_id"]);
    assert_eq!(grant["ip"], "203.0.113.9");
    assert_eq!(grant["user_agent"], "Firefox/140");

    let r = api.call(Method::DELETE, "/me", Some(&ct), None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert!(
        db::guardians::requests(api.pool(), teen)
            .await
            .unwrap()
            .is_empty(),
        "name, email and links of the representative are gone"
    );
    let grant = db::consents::user_consents(api.pool(), teen)
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.kind == "guardian_consent")
        .unwrap();
    assert!(grant.ip.is_none() && grant.user_agent.is_none());
    let r = api.guardian(&confirm, "grant").await;
    assert_eq!(r.body["code"], "invalid_token");
}

fn listed(shortlist: &Reply, id: Uuid) -> bool {
    shortlist.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["candidate"]["id"] == id.to_string())
}

async fn backend_need(api: &Api, et: &str) -> String {
    let r = api
        .call(
            Method::POST,
            "/employer/needs",
            Some(et),
            Some(json!({
                "title": "Бэкенд-разработчик",
                "description": "Платёжный сервис на Kafka и PostgreSQL",
                "specialization": "backend",
                "grades": ["middle"],
                "salary": { "min": 200000, "max": 300000 },
            })),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    format!(
        "/employer/needs/{}/shortlist",
        r.body["id"].as_str().unwrap()
    )
}

/// The search pool is cached per specialization; a revoked consent, a hidden profile or an edited
/// profile must still show on the very next request.
#[sqlx::test(migrations = "../../db/migrations")]
async fn warm_pool_cache_follows_consents_and_profile_edits(pool: PgPool) {
    let api = Api::start(pool).await;
    let anna = api.candidate("anna@example.ru").await;
    let boris = api.candidate("boris@example.ru").await;
    let (hr, _) = api.employer("hr@example.ru").await;
    let ct = api.token(anna, Role::Candidate);
    let et = api.token(hr, Role::Employer);
    let shortlist = backend_need(&api, &et).await;
    let search = "/employer/candidates?specialization=backend";

    let r = api
        .call(Method::POST, &shortlist, Some(&et), Some(json!({})))
        .await;
    assert!(listed(&r, anna) && listed(&r, boris), "{}", r.body);
    assert!(found(
        &api.call(Method::GET, search, Some(&et), None).await,
        anna
    ));

    let r = api
        .call(
            Method::POST,
            &format!("/me/consents/{PUBLICATION}/revoke"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = api.call(Method::GET, &shortlist, Some(&et), None).await;
    assert!(!listed(&r, anna), "the stored shortlist hides her at once");
    let r = api
        .call(Method::POST, &shortlist, Some(&et), Some(json!({})))
        .await;
    assert!(
        !listed(&r, anna) && listed(&r, boris),
        "a rebuild skips her"
    );
    let r = api.call(Method::GET, search, Some(&et), None).await;
    assert!(!found(&r, anna));
    assert_eq!(r.body["total"], 1, "she is not counted either");

    let r = api
        .call(
            Method::POST,
            "/me/consents",
            Some(&ct),
            Some(json!({ "document_id": PUBLICATION })),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED);
    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(true)),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(found(
        &api.call(Method::GET, search, Some(&et), None).await,
        anna
    ));

    let r = api
        .call(
            Method::PUT,
            "/candidate/privacy",
            Some(&ct),
            Some(privacy(false)),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(!found(
        &api.call(Method::GET, search, Some(&et), None).await,
        anna
    ));
    api.call(
        Method::PUT,
        "/candidate/privacy",
        Some(&ct),
        Some(privacy(true)),
    )
    .await;

    let kafka = "/employer/candidates?specialization=backend&skills=kafka";
    assert!(!found(
        &api.call(Method::GET, kafka, Some(&et), None).await,
        anna
    ));
    let r = api
        .call(
            Method::PUT,
            "/candidate/profile",
            Some(&ct),
            Some(json!({
                "full_name": "Анна Петрова",
                "contacts": { "telegram": "@anna" },
                "salary_expectation_min": 280000,
                "skills": ["kafka", "postgresql"],
            })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let r = api.call(Method::GET, kafka, Some(&et), None).await;
    assert!(
        found(&r, anna) && !found(&r, boris),
        "new skills count at once"
    );
    let cheap = "/employer/candidates?specialization=backend&max_expectation=250000";
    let r = api.call(Method::GET, cheap, Some(&et), None).await;
    assert!(
        !found(&r, anna) && found(&r, boris),
        "so does the new salary"
    );
}

/// Another replica keeps its pool until the generation reaches it, but never shows a profile that
/// stopped being searchable: the page is checked against the live tables.
#[sqlx::test(migrations = "../../db/migrations")]
async fn other_replica_never_shows_a_hidden_profile(pool: PgPool) {
    let a = Api::start(pool.clone()).await;
    let b = Api::start(pool).await;
    let anna = a.candidate("anna@example.ru").await;
    let (hr, _) = a.employer("hr@example.ru").await;
    let ct = a.token(anna, Role::Candidate);
    let et = b.token(hr, Role::Employer);
    let search = "/employer/candidates?specialization=backend";
    assert!(found(
        &b.call(Method::GET, search, Some(&et), None).await,
        anna
    ));

    let r = a
        .call(
            Method::POST,
            &format!("/me/consents/{PUBLICATION}/revoke"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = b.call(Method::GET, search, Some(&et), None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(!found(&r, anna), "replica b has no Valkey generation yet");
}

/// Whether replica `api` holds the employer card of `id` in its current backend snapshot.
async fn card_cached(api: &Api, id: Uuid) -> bool {
    api.state.candidates.current_pools().await[catalog::Specialization::Backend as usize]
        .as_ref()
        .is_some_and(|p| p.value.card(id).is_some())
}

fn card_of(r: &Reply, id: Uuid) -> Option<Value> {
    r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["candidate"]["id"] == id.to_string())
        .map(|i| i["candidate"].clone())
}

/// Cards are cached with the pool snapshot. Replica b keeps its cards until the generation
/// reaches it — here never, there is no Valkey — yet shows nothing a candidate has just hidden on
/// replica a: the profile, the name, employer names in the experience, a profile closed to offers.
#[sqlx::test(migrations = "../../db/migrations")]
async fn other_replica_with_warm_cards_never_shows_hidden_data(pool: PgPool) {
    let a = Api::start(pool.clone()).await;
    let b = Api::start(pool).await;
    let anna = a.candidate("anna@example.ru").await;
    let boris = a.candidate("boris@example.ru").await;
    let vera = a.candidate("vera@example.ru").await;
    let (hr, _) = a.employer("hr@example.ru").await;
    let et = b.token(hr, Role::Employer);
    let shown = |name: bool, experience: bool, status: &str| {
        json!({
            "visible": true,
            "show_name": name,
            "show_fsp": true,
            "show_experience": experience,
            "search_status": status,
        })
    };
    for (id, name) in [(boris, "Борис Иванов"), (vera, "Вера Смирнова")] {
        let ct = a.token(id, Role::Candidate);
        let r = a
            .call(
                Method::PUT,
                "/candidate/profile",
                Some(&ct),
                Some(json!({
                    "full_name": name,
                    "contacts": { "telegram": "@candidate" },
                    "experience": [{ "company": "Яндекс", "position": "Разработчик", "start": "2022-01" }],
                })),
            )
            .await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        let r = a
            .call(
                Method::PUT,
                "/candidate/privacy",
                Some(&ct),
                Some(shown(true, true, "open")),
            )
            .await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    }
    let shortlist = backend_need(&b, &et).await;
    let search = "/employer/candidates?specialization=backend";
    b.call(Method::POST, &shortlist, Some(&et), Some(json!({})))
        .await;
    let r = b.call(Method::GET, &shortlist, Some(&et), None).await;
    for id in [anna, boris, vera] {
        assert!(listed(&r, id), "{}", r.body);
        assert!(card_cached(&b, id).await, "the card is cached");
    }
    assert_eq!(card_of(&r, boris).unwrap()["display_name"], "Борис Иванов");
    assert_eq!(
        card_of(&r, vera).unwrap()["experience"][0]["company"],
        "Яндекс"
    );

    let r = a
        .call(
            Method::POST,
            &format!("/me/consents/{PUBLICATION}/revoke"),
            Some(&a.token(anna, Role::Candidate)),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    for (id, privacy) in [
        (boris, shown(false, true, "open")),
        (vera, shown(true, false, "closed")),
    ] {
        let r = a
            .call(
                Method::PUT,
                "/candidate/privacy",
                Some(&a.token(id, Role::Candidate)),
                Some(privacy),
            )
            .await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    }

    let r = b.call(Method::GET, &shortlist, Some(&et), None).await;
    assert!(!listed(&r, anna), "publication consent revoked");
    assert!(!listed(&r, vera), "closed to offers");
    assert_eq!(
        card_of(&r, boris).unwrap()["display_name"],
        Value::Null,
        "the name is hidden at once"
    );
    let r = b.call(Method::GET, search, Some(&et), None).await;
    assert!(!found(&r, anna) && !found(&r, vera) && found(&r, boris));
    let r = b
        .call(
            Method::GET,
            &format!("/employer/candidates/{vera}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body["candidate"]["search_status"], "closed");
    assert_eq!(
        r.body["candidate"]["experience"][0]["company"],
        Value::Null,
        "employer names are hidden at once"
    );
    let r = b
        .call(
            Method::GET,
            &format!("/employer/candidates/{anna}"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    assert!(
        card_cached(&b, anna).await,
        "b still holds the old snapshot: the live check did the hiding"
    );
}

/// Task stats on a card change without a profile write; submitting and reviewing bump the cache
/// generation, so a warm card shows them on the next request.
#[sqlx::test(migrations = "../../db/migrations")]
async fn warm_cards_follow_task_submissions_and_reviews(pool: PgPool) {
    let api = Api::start(pool).await;
    let anna = api.candidate("anna@example.ru").await;
    let (hr, _) = api.employer("hr@example.ru").await;
    let (ct, et) = (
        api.token(anna, Role::Candidate),
        api.token(hr, Role::Employer),
    );
    let r = api
        .call(
            Method::POST,
            "/employer/tasks",
            Some(&et),
            Some(json!({
                "title": "Кеш",
                "body": "Как бы вы кешировали карточки кандидатов?",
                "kind": "approach",
                "specialization": "backend",
                "grades": ["middle"],
            })),
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let task = r.body["id"].as_str().unwrap().to_owned();
    let r = api
        .call(Method::GET, "/candidate/tasks", Some(&ct), None)
        .await;
    let assignment = r.body[0]["id"].as_str().unwrap().to_owned();
    let profile = format!("/employer/candidates/{anna}");
    let warm = || async {
        api.call(
            Method::GET,
            "/employer/candidates?specialization=backend",
            Some(&et),
            None,
        )
        .await;
        api.call(Method::GET, &profile, Some(&et), None).await;
        assert!(card_cached(&api, anna).await);
    };

    warm().await;
    let r = api.call(Method::GET, &profile, Some(&et), None).await;
    assert_eq!(r.body["candidate"]["tasks_done"], 0);
    let r = api
        .call(
            Method::POST,
            &format!("/candidate/tasks/{assignment}/submit"),
            Some(&ct),
            Some(json!({ "answer": "Снимок в памяти, поколение в Valkey" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let r = api.call(Method::GET, &profile, Some(&et), None).await;
    assert_eq!(r.body["candidate"]["tasks_done"], 1);

    warm().await;
    let r = api
        .call(
            Method::GET,
            &format!("/employer/tasks/{task}/submissions"),
            Some(&et),
            None,
        )
        .await;
    let submission = r.body[0]["id"].as_str().unwrap().to_owned();
    let r = api
        .call(
            Method::POST,
            &format!("/employer/tasks/{task}/submissions/{submission}/review"),
            Some(&et),
            Some(json!({ "rating": 5 })),
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT, "{}", r.body);
    let r = api.call(Method::GET, &profile, Some(&et), None).await;
    assert_eq!(r.body["candidate"]["task_rating"], 5.0);
}

/// The company of an employer is cached; deleting the account must end it for every session at
/// once, not only for the one that deleted it.
#[sqlx::test(migrations = "../../db/migrations")]
async fn deleted_employer_loses_its_company_in_every_session(pool: PgPool) {
    let api = Api::start(pool).await;
    let (hr, _) = api.employer("hr@example.ru").await;
    let (deleting, other) = (api.token(hr, Role::Employer), api.token(hr, Role::Employer));
    let search = "/employer/candidates?specialization=backend";
    assert_eq!(
        api.call(Method::GET, search, Some(&other), None)
            .await
            .status,
        StatusCode::OK
    );
    let r = api.call(Method::DELETE, "/me", Some(&deleting), None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let r = api.call(Method::GET, search, Some(&other), None).await;
    assert_eq!(r.status, StatusCode::CONFLICT, "{}", r.body);
    assert_eq!(r.body["code"], "company_profile_required");
}

fn cache_control(r: &Reply) -> &str {
    r.headers[header::CACHE_CONTROL].to_str().unwrap()
}

/// 152-ФЗ: whatever is personal or behind credentials is `private, no-store`; only anonymous
/// reference data and published vacancies are cached publicly.
#[sqlx::test(migrations = "../../db/migrations")]
async fn cache_control_by_route(pool: PgPool) {
    let api = Api::start(pool).await;
    let cand = api.candidate("anna@example.ru").await;
    let (hr, company) = api.employer("hr@example.ru").await;
    let ct = api.token(cand, Role::Candidate);
    let et = api.token(hr, Role::Employer);
    let vacancy = Uuid::now_v7();
    db::vacancies::insert_vacancy(
        api.pool(),
        vacancy,
        company,
        db::types::VacancyStatus::Published,
        Some(Utc::now()),
        &db::vacancies::VacancyFields {
            title: "Go-разработчик",
            description: "Платежи",
            specialization: "backend",
            grade: "middle",
            skills: &[],
            salary_min: 200_000,
            salary_max: 300_000,
            work_format: "remote",
            city: None,
        },
    )
    .await
    .unwrap();

    for path in [
        "/me",
        "/me/consents",
        "/candidate/profile",
        "/candidate/privacy",
        "/candidate/contact-access",
    ] {
        let r = api.call(Method::GET, path, Some(&ct), None).await;
        assert_eq!(r.status, StatusCode::OK, "{path}");
        assert_eq!(cache_control(&r), "private, no-store", "{path}");
    }
    for path in [
        "/employer/company",
        "/employer/candidates?specialization=backend",
        "/employer/categories",
        "/employer/invitations",
    ] {
        let r = api.call(Method::GET, path, Some(&et), None).await;
        assert_eq!(r.status, StatusCode::OK, "{path}");
        assert_eq!(cache_control(&r), "private, no-store", "{path}");
    }
    let r = api.call(Method::GET, "/me", None, None).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        cache_control(&r),
        "private, no-cache",
        "a 401 holds nothing to protect"
    );
    let key = api
        .call(
            Method::POST,
            "/employer/integrations/keys",
            Some(&et),
            Some(json!({ "name": "ATS", "scopes": ["invitations:read"] })),
        )
        .await;
    assert_eq!(key.status, StatusCode::CREATED, "{}", key.body);
    assert_eq!(cache_control(&key), "private, no-store", "a secret");
    let r = api
        .call(
            Method::GET,
            "/integrations/whoami",
            key.body["secret"].as_str(),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(cache_control(&r), "private, no-store");

    let r = api
        .call(Method::GET, "/guardian/consent?token=nope", None, None)
        .await;
    assert_eq!(cache_control(&r), "no-store");
    let r = api
        .call(
            Method::POST,
            "/eval/pairs",
            None,
            Some(json!({ "vacancies": [], "candidates": [] })),
        )
        .await;
    assert_eq!(cache_control(&r), "no-store");
    let r = api
        .call(
            Method::GET,
            "/catalog/test-plan?specialization=backend&grade=middle",
            None,
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(
        cache_control(&r),
        "no-store",
        "fresh random examples each time"
    );

    let listing = "public, max-age=30, stale-while-revalidate=120";
    for path in ["/vacancies".to_owned(), format!("/vacancies/{vacancy}")] {
        let r = api.call(Method::GET, &path, Some(&ct), None).await;
        assert_eq!(r.status, StatusCode::OK, "{path}");
        assert_eq!(cache_control(&r), listing, "{path}");
        assert!(!r.body.to_string().contains("anna"), "no personal data");
    }
    let r = api
        .call(
            Method::GET,
            &format!("/vacancies/{}", Uuid::now_v7()),
            None,
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    assert_eq!(cache_control(&r), "no-store", "errors are not cached");

    let static_policy = "public, max-age=300, stale-while-revalidate=86400";
    for path in ["/catalog", "/catalog/skills?q=post", "/consents/current"] {
        let url = format!("{}/api/v1{path}", api.base);
        let first = api.http.get(&url).send().await.unwrap();
        assert_eq!(first.status(), StatusCode::OK, "{path}");
        assert_eq!(
            first.headers()[header::CACHE_CONTROL],
            static_policy,
            "{path}"
        );
        let etag = first.headers()[header::ETAG].clone();
        let again = api
            .http
            .get(&url)
            .header(header::IF_NONE_MATCH, etag.clone())
            .send()
            .await
            .unwrap();
        assert_eq!(again.status(), StatusCode::NOT_MODIFIED, "{path}");
        assert_eq!(again.headers()[header::ETAG], etag);
        assert_eq!(again.headers()[header::CACHE_CONTROL], static_policy);
        assert!(again.bytes().await.unwrap().is_empty());
    }
}

/// Run with a Valkey: `TEST_VALKEY_URL=redis://127.0.0.1:6379 cargo test -p api --test privacy
/// -- --ignored`. Replicas share the generation, so b reloads within `REMOTE_TTL` of a's write.
#[sqlx::test(migrations = "../../db/migrations")]
#[ignore = "needs TEST_VALKEY_URL"]
async fn valkey_generation_reaches_other_replicas(pool: PgPool) {
    let url = std::env::var("TEST_VALKEY_URL").expect("TEST_VALKEY_URL");
    let a = Api::start_with(pool.clone(), Some(&url)).await;
    let b = Api::start_with(pool, Some(&url)).await;
    let anna = a.candidate("anna@example.ru").await;
    let (hr, _) = a.employer("hr@example.ru").await;
    let ct = a.token(anna, Role::Candidate);
    let et = b.token(hr, Role::Employer);
    let search = "/employer/candidates?specialization=backend";
    tokio::time::sleep(api::cache::REMOTE_TTL).await;
    let r = b.call(Method::GET, search, Some(&et), None).await;
    assert!(found(&r, anna));
    assert_eq!(r.body["total"], 1);

    let r = a
        .call(
            Method::POST,
            &format!("/me/consents/{PUBLICATION}/revoke"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    tokio::time::sleep(api::cache::REMOTE_TTL + std::time::Duration::from_millis(50)).await;
    let r = b.call(Method::GET, search, Some(&et), None).await;
    assert!(!found(&r, anna));
    assert_eq!(
        r.body["total"], 0,
        "b rebuilt its pool from the shared generation"
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn concurrent_sign_ups_with_one_email_give_one_account_and_a_conflict(pool: PgPool) {
    let api = Api::start(pool).await;
    let body = register("twice@example.ru", Some("adult"), None);
    let (a, b) = tokio::join!(
        api.call(Method::POST, "/auth/register", None, Some(body.clone())),
        api.call(Method::POST, "/auth/register", None, Some(body)),
    );
    let mut statuses = [a.status, b.status];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::CREATED, StatusCode::CONFLICT]);
    let conflict = if a.status == StatusCode::CONFLICT {
        a
    } else {
        b
    };
    assert_eq!(conflict.body["code"], "email_taken");
}
