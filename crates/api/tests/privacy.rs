//! 152-ФЗ flows over real HTTP against a throwaway database: publication consent gating, the
//! candidate's contact access log, the data export.

use std::sync::Arc;

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, ProctoringConfig},
    crypto::Crypto,
    integrations::webhook::WebhookSender,
    kv::Kv,
    mail::Mailer,
    proctoring::{ProctoringMode, RecordingStore},
    state::AppState,
};
use chrono::{Duration, Utc};
use reqwest::{Method, StatusCode, header};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

const PROCESSING: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000001);
const PUBLICATION: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000002);
const TERMS_V1: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_000000000003);
const TERMS_V2: Uuid = Uuid::from_u128(0x0191f000_0000_7000_8000_0000000000f3);

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
                mode: ProctoringMode::Optional,
                dir: std::env::temp_dir().join(format!("privacy-test-{}", Uuid::now_v7())),
                retention_days: 90,
            },
            integrations: IntegrationsConfig::default(),
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
            webhooks: WebhookSender::new(Vec::new()).unwrap(),
            pool,
            config: Arc::new(config),
        };
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
            &self.state.crypto.encrypt_str(email),
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

    async fn in_pool(&self, id: Uuid) -> bool {
        db::pool::candidate_pool(self.pool(), None, "backend", None)
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
