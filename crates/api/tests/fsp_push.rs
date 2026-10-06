//! The ФСП registry's push over real HTTP: the API and the registry mock on local ports, the
//! signature with its replay window, one application per `event_id`, the sync it triggers
//! (snapshot, skills, cache, notification), and an unknown participant that leaves no trace.

use std::sync::Arc;

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, ProctoringConfig},
    crypto::Crypto,
    integrations::{signature, webhook::WebhookSender},
    kv::Kv,
    mail::Mailer,
    proctoring::{ProctoringMode, RecordingStore},
    state::{AppState, Services},
};
use catalog::Specialization;
use chrono::Utc;
use reqwest::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpListener;
use uuid::Uuid;

const SECRET: &str = "whsec_registry_test";
const REGISTRY_TOKEN: &str = "registry-test-token";
const PARTICIPANT: &str = "FSP-000011";

struct Stack {
    api: String,
    registry: String,
    http: reqwest::Client,
    state: AppState,
}

struct Reply {
    status: StatusCode,
    body: Value,
}

/// `pushes`: the registry mock delivers its own events to the API; otherwise the test does.
async fn start(pool: PgPool, secret: Option<&str>, pushes: bool) -> Stack {
    let api_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let registry_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let api = format!("http://{}", api_listener.local_addr().unwrap());
    let registry = format!("http://{}", registry_listener.local_addr().unwrap());

    let push = pushes.then(|| fsp_mock::Push {
        url: format!("{api}/api/v1/fsp/registry/events"),
        secret: SECRET.into(),
    });
    let mock = fsp_mock::AppState::new(
        fsp_mock::Registry::embedded().unwrap().clone(),
        REGISTRY_TOKEN,
        push,
    )
    .unwrap();
    tokio::spawn(async move { axum::serve(registry_listener, fsp_mock::router(mock)).await });

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
            registry_url: Some(registry.clone()),
            registry_token: REGISTRY_TOKEN.into(),
            webhook_secret: secret.map(str::to_owned),
            dev_link: false,
        },
        grade_change_cooldown_days: 90,
        proctoring: ProctoringConfig {
            mode: ProctoringMode::Off,
            dir: std::env::temp_dir().join(format!("fsp-push-test-{}", Uuid::now_v7())),
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
    let app = api::app(state.clone());
    tokio::spawn(async move { axum::serve(api_listener, app).await });
    Stack {
        api,
        registry,
        http: reqwest::Client::new(),
        state,
    }
}

fn event(id: &str, fsp_id: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "event_id": id,
        "type": "achievements.changed",
        "fsp_id": fsp_id,
        "occurred_at": Utc::now(),
    }))
    .unwrap()
}

async fn reply(res: reqwest::Response) -> Reply {
    let status = res.status();
    let bytes = res.bytes().await.unwrap();
    Reply {
        status,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

impl Stack {
    fn pool(&self) -> &PgPool {
        &self.state.pool
    }

    /// Delivers `body` signed at `t` with `secret`; `None` sends no signature at all.
    async fn deliver(&self, body: &[u8], signed: Option<(&str, i64)>, ip: &str) -> Reply {
        let mut req = self
            .http
            .post(format!("{}/api/v1/fsp/registry/events", self.api))
            .header("content-type", "application/json")
            .header("x-forwarded-for", ip)
            .body(body.to_vec());
        if let Some((secret, t)) = signed {
            req = req.header(
                signature::SIGNATURE_HEADER,
                signature::header_value(t, &[secret.as_bytes()], body),
            );
        }
        reply(req.send().await.unwrap()).await
    }

    async fn push(&self, id: &str) -> Reply {
        self.deliver(
            &event(id, PARTICIPANT),
            Some((SECRET, Utc::now().timestamp())),
            "10.0.0.1",
        )
        .await
    }

    /// Changes the registry the way its admin would.
    async fn admin(&self, method: reqwest::Method, path: &str) -> Reply {
        reply(
            self.http
                .request(
                    method,
                    format!("{}/admin/participants/{path}", self.registry),
                )
                .bearer_auth(REGISTRY_TOKEN)
                .send()
                .await
                .unwrap(),
        )
        .await
    }

    /// A candidate linked to [`PARTICIPANT`] with the registry's current snapshot.
    async fn linked_candidate(&self) -> (Uuid, String) {
        let id = Uuid::now_v7();
        let email = "fsp@example.ru";
        db::users::insert_user(
            self.pool(),
            id,
            Role::Candidate.into(),
            &self.state.crypto.encrypt_str(email).unwrap(),
            &self.state.crypto.blind_index(email),
            "h",
        )
        .await
        .unwrap();
        db::users::mark_email_verified(self.pool(), id)
            .await
            .unwrap();
        db::candidates::insert_candidate(self.pool(), id, "K-F5P011")
            .await
            .unwrap();
        db::grades::set_candidate_category(self.pool(), "backend", "middle", 0.4, Utc::now(), id)
            .await
            .unwrap();
        let snapshot = self.snapshot_in_registry().await;
        db::seed::seed_fsp_link(self.pool(), id, PARTICIPANT, &snapshot)
            .await
            .unwrap();
        (
            id,
            self.state
                .jwt
                .issue(id, Role::Candidate, Utc::now().timestamp())
                .unwrap(),
        )
    }

    async fn snapshot_in_registry(&self) -> Value {
        let r = reply(
            self.http
                .get(format!(
                    "{}/participants/{PARTICIPANT}/achievements",
                    self.registry
                ))
                .bearer_auth(REGISTRY_TOKEN)
                .send()
                .await
                .unwrap(),
        )
        .await;
        r.body["achievements"].clone()
    }

    async fn stored(&self, candidate: Uuid) -> Vec<Value> {
        db::fsp::fsp_link(self.pool(), candidate)
            .await
            .unwrap()
            .unwrap()
            .achievements
            .as_array()
            .unwrap()
            .clone()
    }

    async fn fsp_skills(&self, candidate: Uuid) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT skill FROM candidate_skills
             WHERE candidate_id = $1 AND source = 'fsp' ORDER BY skill",
        )
        .bind(candidate)
        .fetch_all(self.pool())
        .await
        .unwrap()
    }

    async fn feed(&self, token: &str) -> Vec<Value> {
        let r = reply(
            self.http
                .get(format!("{}/api/v1/me/notifications", self.api))
                .bearer_auth(token)
                .send()
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        r.body["items"].as_array().unwrap().clone()
    }

    async fn events(&self) -> Vec<(String, bool)> {
        sqlx::query_as("SELECT event_id, changed FROM fsp_registry_events ORDER BY received_at")
            .fetch_all(self.pool())
            .await
            .unwrap()
    }

    async fn backend_pool(&self) -> Arc<api::cache::Entry<api::pool::SpecPool>> {
        self.state
            .candidates
            .pool(self.pool(), Specialization::Backend)
            .await
            .unwrap()
    }
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn signature_must_match_and_be_fresh(pool: PgPool) {
    let stack = start(pool, Some(SECRET), false).await;
    let now = Utc::now().timestamp();
    let body = event("evt_sig", "FSP-999999");

    let ok = stack.deliver(&body, Some((SECRET, now)), "10.0.0.2").await;
    assert_eq!(ok.status, StatusCode::ACCEPTED, "{}", ok.body);
    for (signed, code) in [
        (None, "signature_invalid"),
        (Some(("whsec_other", now)), "signature_invalid"),
        (Some((SECRET, now - 301)), "signature_expired"),
        (Some((SECRET, now + 301)), "signature_expired"),
    ] {
        let r = stack.deliver(&body, signed, "10.0.0.2").await;
        assert_eq!(
            (r.status, r.body["code"].as_str()),
            (StatusCode::UNAUTHORIZED, Some(code)),
            "{signed:?}"
        );
    }
    let header = signature::header_value(now, &[SECRET.as_bytes()], &body);
    let tampered = event("evt_sig", "FSP-000011");
    let r = reply(
        stack
            .http
            .post(format!("{}/api/v1/fsp/registry/events", stack.api))
            .header("content-type", "application/json")
            .header(signature::SIGNATURE_HEADER, header)
            .body(tampered)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        r.body["code"], "signature_invalid",
        "the body is under the MAC"
    );

    let garbage = b"{\"event_id\": 1}";
    let r = stack
        .deliver(garbage, Some((SECRET, now)), "10.0.0.2")
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", r.body);
    let r = stack
        .deliver(&event("", PARTICIPANT), Some((SECRET, now)), "10.0.0.2")
        .await;
    assert!(r.body["errors"]["event_id"].is_array(), "{}", r.body);

    for _ in 0..300 {
        stack.deliver(&body, None, "10.0.0.3").await;
    }
    let r = stack.deliver(&body, Some((SECRET, now)), "10.0.0.3").await;
    assert_eq!(r.status, StatusCode::TOO_MANY_REQUESTS);
    let r = stack.deliver(&body, Some((SECRET, now)), "10.0.0.4").await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "the limit is per address");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn without_a_secret_the_push_is_off(pool: PgPool) {
    let stack = start(pool, None, false).await;
    let body = event("evt_off", PARTICIPANT);
    let r = stack
        .deliver(&body, Some((SECRET, Utc::now().timestamp())), "10.0.0.5")
        .await;
    assert_eq!(
        (r.status, r.body["code"].as_str()),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Some("fsp_webhook_disabled")
        )
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn an_unknown_participant_leaves_no_trace(pool: PgPool) {
    let stack = start(pool, Some(SECRET), false).await;
    let (_, token) = stack.linked_candidate().await;
    for fsp_id in ["FSP-000012", "FSP-999999"] {
        let r = stack
            .deliver(
                &event(&format!("evt_{fsp_id}"), fsp_id),
                Some((SECRET, Utc::now().timestamp())),
                "10.0.0.6",
            )
            .await;
        assert_eq!(r.status, StatusCode::ACCEPTED);
        assert_eq!(
            r.body,
            Value::Null,
            "the same empty answer as for a linked one"
        );
    }
    assert!(stack.events().await.is_empty(), "nothing stored about them");
    assert!(stack.feed(&token).await.is_empty());
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn each_event_applies_once_and_syncs_like_a_manual_sync(pool: PgPool) {
    let stack = start(pool, Some(SECRET), false).await;
    let (candidate, token) = stack.linked_candidate().await;
    let original = stack.stored(candidate).await;
    assert!(
        stack.fsp_skills(candidate).await.is_empty(),
        "product results confirm no skills"
    );
    let warm = stack.backend_pool().await;

    let added = stack
        .admin(
            reqwest::Method::POST,
            &format!("{PARTICIPANT}/achievements"),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.body);
    assert!(
        added.body["push"].is_null(),
        "this registry does not push by itself"
    );
    let added_id = added.body["added"]["id"].as_str().unwrap().to_owned();

    let r = stack.push("evt_1").await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "{}", r.body);
    let stored = stack.stored(candidate).await;
    assert_eq!(stored.len(), original.len() + 1);
    assert!(stored.iter().any(|a| a["id"] == added_id.as_str()));
    assert_eq!(
        stack.fsp_skills(candidate).await,
        ["algorithms", "data_structures"],
        "a national second place confirms the discipline's skills"
    );
    let feed = stack.feed(&token).await;
    assert_eq!(feed.len(), 1);
    assert_eq!(feed[0]["kind"], "fsp_achievements");
    assert_eq!(feed[0]["title"], "Новые достижения ФСП в профиле");
    let body = feed[0]["body"].as_str().unwrap();
    assert!(
        body.starts_with(
            "Призёр (2 место) всероссийских соревнований по алгоритмическому программированию"
        ) && body.ends_with("ФСП-скор и навыки, подтверждённые ФСП, уже пересчитаны."),
        "{body}"
    );
    assert_eq!(feed[0]["link"], "/candidate/settings#fsp");
    let fresh = stack.backend_pool().await;
    assert!(
        !Arc::ptr_eq(&warm, &fresh),
        "the change bumped the cache generation"
    );

    stack
        .admin(
            reqwest::Method::DELETE,
            &format!("{PARTICIPANT}/achievements/{added_id}"),
        )
        .await;
    let r = stack.push("evt_1").await;
    assert_eq!(r.status, StatusCode::ACCEPTED, "a redelivery is accepted");
    assert_eq!(
        stack.stored(candidate).await.len(),
        original.len() + 1,
        "but not applied again"
    );
    assert_eq!(stack.feed(&token).await.len(), 1);

    let r = stack.push("evt_2").await;
    assert_eq!(r.status, StatusCode::ACCEPTED);
    assert_eq!(stack.stored(candidate).await, original);
    assert!(stack.fsp_skills(candidate).await.is_empty());
    let feed = stack.feed(&token).await;
    assert_eq!(feed[0]["title"], "Данные ФСП в профиле обновились");
    assert!(
        feed[0]["body"]
            .as_str()
            .unwrap()
            .starts_with("Реестр ФСП убрал 1 запись."),
        "{}",
        feed[0]
    );

    let settled = stack.backend_pool().await;
    let r = stack.push("evt_3").await;
    assert_eq!(r.status, StatusCode::ACCEPTED);
    assert_eq!(
        stack.feed(&token).await.len(),
        2,
        "nothing changed, nothing to say"
    );
    assert!(
        Arc::ptr_eq(&settled, &stack.backend_pool().await),
        "and the cache stays"
    );
    assert_eq!(
        stack.events().await,
        [
            ("evt_1".to_owned(), true),
            ("evt_2".to_owned(), true),
            ("evt_3".to_owned(), false)
        ]
    );

    let r = reply(
        stack
            .http
            .delete(format!("{}/api/v1/fsp/link", stack.api))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert!(
        stack.events().await.is_empty(),
        "the events go with the link"
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn the_registry_mock_pushes_and_the_platform_follows(pool: PgPool) {
    let stack = start(pool, Some(SECRET), true).await;
    let (candidate, token) = stack.linked_candidate().await;
    let before = stack.stored(candidate).await.len();
    let added = stack
        .admin(
            reqwest::Method::POST,
            &format!("{PARTICIPANT}/achievements"),
        )
        .await;
    assert_eq!(added.status, StatusCode::CREATED, "{}", added.body);
    assert_eq!(added.body["push"]["status"], 202, "{}", added.body);
    assert_eq!(stack.stored(candidate).await.len(), before + 1);
    let feed = stack.feed(&token).await;
    assert_eq!(feed[0]["kind"], "fsp_achievements");
    let event_id = added.body["push"]["event_id"].as_str().unwrap();
    assert_eq!(stack.events().await, [(event_id.to_owned(), true)]);
}
