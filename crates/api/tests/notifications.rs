//! Notifications over real HTTP against a throwaway database: items written with their domain
//! change, letters assembled under the rate rule, the expiry reminder, settings, one-click
//! unsubscribe, and no candidate PII in company letters.

use std::{sync::Arc, time::Duration as StdDuration};

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, NotifyConfig, ProctoringConfig},
    contacts::Contacts,
    crypto::Crypto,
    kv::Kv,
    mail::Mailer,
    notify::{
        NotificationKind, Notifier,
        email::{Assembled, Composed, EmailDispatcher},
        sweep,
        token::{self, Scope},
    },
    proctoring::{ProctoringMode, RecordingStore},
    state::{AppState, Services},
};
use chrono::{Duration, Utc};
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

fn rule(hold_secs: u64, gap_secs: u64, daily_cap: u32) -> NotifyConfig {
    NotifyConfig {
        hold: StdDuration::from_secs(hold_secs),
        min_gap: StdDuration::from_secs(gap_secs),
        daily_cap,
        interval: StdDuration::from_millis(50),
    }
}

impl Api {
    async fn start(pool: PgPool, notifications: NotifyConfig) -> Self {
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
                dir: std::env::temp_dir().join(format!("notify-test-{}", Uuid::now_v7())),
                retention_days: 90,
            },
            integrations: IntegrationsConfig::default(),
            notifications,
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
            webhooks: api::integrations::webhook::WebhookSender::new(Vec::new()).unwrap(),
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

    async fn send(&self, req: reqwest::RequestBuilder) -> Reply {
        let res = req.send().await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.bytes().await.unwrap();
        Reply {
            status,
            headers,
            body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
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

    /// A published backend:middle candidate with a name and contacts.
    async fn candidate(&self, email: &str, name: &str) -> (Uuid, String) {
        let id = self.account(Role::Candidate, email).await;
        let code = format!("K-{}", email[..4].to_uppercase());
        db::candidates::insert_candidate(self.pool(), id, &code)
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
                full_name_enc: Some(&self.state.crypto.encrypt_str(name).unwrap()),
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
        (id, code)
    }

    async fn employer(&self, email: &str, name: &str) -> (String, Uuid, Uuid) {
        let owner = self.account(Role::Employer, email).await;
        let company = Uuid::now_v7();
        db::companies::insert_company(self.pool(), company, owner, name)
            .await
            .unwrap();
        (self.token(owner, Role::Employer), company, owner)
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
                    "contact": "@hr",
                })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
        r.body["id"].as_str().unwrap().parse().unwrap()
    }

    async fn feed(&self, token: &str, query: &str) -> Value {
        let r = self
            .call(
                Method::GET,
                &format!("/me/notifications{query}"),
                Some(token),
                None,
            )
            .await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        r.body
    }

    async fn unread(&self, token: &str) -> i64 {
        let r = self
            .call(
                Method::GET,
                "/me/notifications/unread-count",
                Some(token),
                None,
            )
            .await;
        assert_eq!(r.status, StatusCode::OK, "{}", r.body);
        r.body["count"].as_i64().unwrap()
    }

    fn dispatcher(&self) -> EmailDispatcher {
        EmailDispatcher::new(&self.state)
    }

    async fn queued(&self, user: Uuid) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM notifications WHERE user_id = $1 AND email_due_at IS NOT NULL",
        )
        .bind(user)
        .fetch_one(self.pool())
        .await
        .unwrap()
    }

    async fn letter_items(&self, email: Uuid) -> i64 {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM notifications WHERE email_id = $1")
            .bind(email)
            .fetch_one(self.pool())
            .await
            .unwrap()
    }
}

fn kinds(feed: &Value) -> Vec<&str> {
    feed["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect()
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn feed_follows_the_invitation_both_ways(pool: PgPool) {
    let api = Api::start(pool, NotifyConfig::default()).await;
    let (cand, code) = api.candidate("anna@example.ru", "Анна Смирнова").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _, _) = api.employer("hr@example.ru", "Рога и копыта").await;

    let inv = api.invite(&et, cand).await;
    assert_eq!(api.unread(&ct).await, 1, "written with the invitation");
    let feed = api.feed(&ct, "").await;
    let item = &feed["items"][0];
    assert_eq!(item["kind"], "invitation_received");
    assert_eq!(
        item["title"],
        "Рога и копыта приглашает: Backend-разработчик в платёжную команду"
    );
    assert!(
        item["body"]
            .as_str()
            .unwrap()
            .contains("280\u{a0}000 – 340\u{a0}000\u{a0}₽")
    );
    assert_eq!(item["link"], format!("/candidate/invitations/{inv}"));
    assert!(item["read_at"].is_null());

    let r = api
        .call(
            Method::GET,
            &format!("/candidate/invitations/{inv}"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(
        api.unread(&ct).await,
        0,
        "opening the invitation reads its item"
    );

    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/accept"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let feed = api.feed(&et, "").await;
    assert_eq!(kinds(&feed), ["invitation_accepted"]);
    let item = &feed["items"][0];
    assert_eq!(
        item["title"],
        format!("Кандидат {code} принял приглашение — контакты в кабинете")
    );
    assert_eq!(item["link"], format!("/employer/candidates/{cand}"));
    let text = feed.to_string();
    for leak in ["Анна", "+7 900", "anna@example.ru"] {
        assert!(!text.contains(leak), "{leak} in the company feed");
    }

    let second = api
        .invite(&et, api.candidate("boris@example.ru", "Борис").await.0)
        .await;
    let r = api
        .call(
            Method::POST,
            &format!("/employer/invitations/{second}/withdraw"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
}

#[sqlx::test(
    migrations = "../../db/migrations",
    fixtures("fixtures/notifications_reject_acceptance.sql")
)]
async fn an_item_commits_with_its_change_or_not_at_all(pool: PgPool) {
    let api = Api::start(pool, NotifyConfig::default()).await;
    let (cand, _) = api.candidate("vera@example.ru", "Вера").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _, owner) = api.employer("hr@example.ru", "Альфа").await;
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
    assert_eq!(
        r.body["status"], "sent",
        "the acceptance rolled back with its item"
    );
    let r = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}/contacts"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(
        r.status,
        StatusCode::FORBIDDEN,
        "no contacts without the acceptance"
    );
    assert_eq!(api.unread(&api.token(owner, Role::Employer)).await, 0);

    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/decline"),
            Some(&ct),
            Some(json!({ "reason": "salary", "comment": "от 400 000" })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "other kinds still go through");
    let feed = api.feed(&et, "").await;
    assert_eq!(kinds(&feed), ["invitation_declined"]);
    assert!(
        feed["items"][0]["body"]
            .as_str()
            .unwrap()
            .contains("Причина: зарплата")
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn company_letter_carries_no_candidate_pii(pool: PgPool) {
    let api = Api::start(pool, rule(0, 600, 10)).await;
    let (cand, code) = api.candidate("anna@example.ru", "Анна Смирнова").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _, owner) = api.employer("hr@example.ru", "Рога и копыта").await;
    let inv = api.invite(&et, cand).await;
    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/accept"),
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);

    let d = api.dispatcher();
    let Assembled::Letter(letter) = d.assemble(owner, Utc::now()).await.unwrap() else {
        panic!("the acceptance is due at once with no hold")
    };
    let Composed::Ready(out) = d.compose(letter, owner).await.unwrap() else {
        panic!("a letter to send")
    };
    assert_eq!(out.to, "hr@example.ru");
    assert_eq!(
        out.letter.subject,
        format!("Кандидат {code} принял приглашение — контакты в кабинете")
    );
    assert!(
        out.letter
            .text
            .contains(&format!("http://localhost:3000/employer/candidates/{cand}"))
    );
    for part in [&out.letter.subject, &out.letter.text, &out.letter.html] {
        for leak in [
            "Анна",
            "Смирнова",
            "+7 900 123-45-67",
            "anna@example.ru",
            "@anna_dev",
        ] {
            assert!(
                !part.contains(leak),
                "{leak} leaked into the company letter"
            );
        }
    }
    assert!(
        out.list_unsubscribe
            .starts_with("http://localhost:3000/api/v1/notifications/unsubscribe?token=")
    );
    assert!(out.message_id.starts_with(&format!("<{letter}@")));
    assert_eq!(out.kinds, [NotificationKind::InvitationAccepted]);

    let sent = d.send_due().await.unwrap();
    assert_eq!(sent, 1);
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM notification_emails WHERE id = $1")
            .bind(letter)
            .fetch_one(api.pool())
            .await
            .unwrap();
    assert_eq!(status, "sent");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn invitations_skip_the_gap_but_not_the_hold_or_the_cap(pool: PgPool) {
    let api = Api::start(pool, rule(120, 600, 2)).await;
    let (cand, _) = api.candidate("anna@example.ru", "Анна").await;
    let d = api.dispatcher();
    let invite_from = async |i: usize| {
        let (et, _, _) = api
            .employer(&format!("hr{i}@example{i}.ru"), &format!("Компания {i}"))
            .await;
        (et.clone(), api.invite(&et, cand).await)
    };
    let mut sent = Vec::new();
    for i in 0..3 {
        sent.push(invite_from(i).await);
    }
    let now = Utc::now();
    assert_eq!(
        d.assemble(cand, now).await.unwrap(),
        Assembled::NotDue,
        "still within the two-minute hold"
    );
    let Assembled::Letter(first) = d
        .assemble(cand, now + Duration::seconds(121))
        .await
        .unwrap()
    else {
        panic!("due after the hold")
    };
    assert_eq!(
        api.letter_items(first).await,
        3,
        "three invitations, one letter"
    );
    let Composed::Ready(out) = d.compose(first, cand).await.unwrap() else {
        panic!()
    };
    assert_eq!(
        out.letter.subject,
        "Компания 2 приглашает: Backend-разработчик в платёжную команду (и ещё 2 приглашения)"
    );
    for i in 0..3 {
        assert!(
            out.letter
                .text
                .contains(&format!("Компания {i} приглашает"))
        );
    }

    let (et, inv) = &sent[0];
    let r = api
        .call(
            Method::POST,
            &format!("/employer/invitations/{inv}/withdraw"),
            Some(et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let later = Utc::now() + Duration::seconds(130);
    match d.assemble(cand, later).await.unwrap() {
        Assembled::Deferred(at) => {
            assert!(at > later, "a withdrawal waits out the ten-minute gap");
            assert!(at <= Utc::now() + Duration::minutes(10) + Duration::seconds(5));
        }
        other => panic!("expected the gap to hold the letter, got {other:?}"),
    }

    invite_from(3).await;
    assert_eq!(
        api.queued(cand).await,
        2,
        "the new invitation joins the deferred withdrawal"
    );
    let Assembled::Letter(second) = d
        .assemble(cand, Utc::now() + Duration::seconds(121))
        .await
        .unwrap()
    else {
        panic!("an invitation goes when its hold ends, gap or not")
    };
    assert_eq!(api.letter_items(second).await, 2, "they leave together");
    let Composed::Ready(out) = d.compose(second, cand).await.unwrap() else {
        panic!()
    };
    assert_eq!(
        out.letter.subject,
        "Компания 3 приглашает: Backend-разработчик в платёжную команду (и ещё 1 уведомление)"
    );
    assert!(
        out.letter.text.find("Компания 3 приглашает").unwrap()
            < out.letter.text.find("отозвала").unwrap(),
        "{}",
        out.letter.text
    );

    invite_from(4).await;
    match d
        .assemble(cand, Utc::now() + Duration::seconds(121))
        .await
        .unwrap()
    {
        Assembled::Deferred(at) => assert!(
            at >= Utc::now() + Duration::hours(23),
            "two letters a day: even an invitation waits for the first to leave the window, {at}"
        ),
        other => panic!("expected the daily cap, got {other:?}"),
    }
    let due: chrono::DateTime<Utc> = sqlx::query_scalar(
        "SELECT email_due_at FROM notifications WHERE user_id = $1 AND email_due_at IS NOT NULL",
    )
    .bind(cand)
    .fetch_one(api.pool())
    .await
    .unwrap();
    assert!(due >= Utc::now() + Duration::hours(23), "{due}");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn expiry_reminder_fires_once_even_with_two_sweepers(pool: PgPool) {
    let api = Api::start(pool, NotifyConfig::default()).await;
    let (cand, _) = api.candidate("anna@example.ru", "Анна").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _, _) = api.employer("hr@example.ru", "Рога и копыта").await;
    let soon = api.invite(&et, cand).await;
    let (other, _) = api.candidate("boris@example.ru", "Борис").await;
    api.invite(&et, other).await;
    sqlx::query("UPDATE invitations SET expires_at = now() + interval '47 hours' WHERE id = $1")
        .bind(soon)
        .execute(api.pool())
        .await
        .unwrap();

    let notifier = Notifier::default();
    let (a, b) = tokio::join!(
        sweep::remind_expiring(api.pool(), notifier),
        sweep::remind_expiring(api.pool(), notifier)
    );
    assert_eq!(a.unwrap() + b.unwrap(), 1, "two sweepers, one reminder");
    assert_eq!(
        sweep::remind_expiring(api.pool(), notifier).await.unwrap(),
        0,
        "never repeated"
    );
    let feed = api.feed(&ct, "").await;
    assert_eq!(kinds(&feed), ["invitation_expiring", "invitation_received"]);
    assert!(
        feed["items"][0]["title"]
            .as_str()
            .unwrap()
            .contains("закроется через двое суток")
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM notifications WHERE kind = 'invitation_expiring'")
            .fetch_one(api.pool())
            .await
            .unwrap();
    assert_eq!(count, 1, "the invitation 14 days out gets nothing yet");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn trust_change_is_announced_after_the_baseline(pool: PgPool) {
    let api = Api::start(pool, NotifyConfig::default()).await;
    let (et, _, _) = api.employer("hr@example.ru", "Рога и копыта").await;
    let notifier = Notifier::default();
    assert_eq!(
        sweep::check_trust(api.pool(), notifier, Utc::now())
            .await
            .unwrap(),
        0,
        "baseline"
    );
    assert_eq!(
        sweep::check_trust(api.pool(), notifier, Utc::now())
            .await
            .unwrap(),
        0
    );

    let (cand, _) = api.candidate("anna@example.ru", "Анна").await;
    let inv = api.invite(&et, cand).await;
    let r = api
        .call(
            Method::POST,
            &format!("/candidate/invitations/{inv}/accept"),
            Some(&api.token(cand, Role::Candidate)),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(
        sweep::check_trust(api.pool(), notifier, Utc::now())
            .await
            .unwrap(),
        1
    );
    let feed = api.feed(&et, "").await;
    assert_eq!(kinds(&feed), ["trust_changed", "invitation_accepted"]);
    assert_eq!(
        feed["items"][0]["title"],
        "Квота приглашений: 10 → 15 за 7 дней"
    );
    assert_eq!(
        sweep::check_trust(api.pool(), notifier, Utc::now())
            .await
            .unwrap(),
        0
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn settings_decide_what_goes_by_email(pool: PgPool) {
    let api = Api::start(pool, rule(0, 0, 10)).await;
    let (cand, _) = api.candidate("anna@example.ru", "Анна").await;
    let ct = api.token(cand, Role::Candidate);
    let (et, _, _) = api.employer("hr@example.ru", "Рога и копыта").await;

    let r = api
        .call(Method::GET, "/me/notification-settings", Some(&ct), None)
        .await;
    let items = r.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 11, "the candidate's kinds only");
    assert!(
        items
            .iter()
            .all(|i| i["email"] == true && i["in_app"] == true)
    );

    let r = api
        .call(
            Method::PUT,
            "/me/notification-settings",
            Some(&ct),
            Some(json!({ "email": [{ "kind": "invitation_accepted", "enabled": false }] })),
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY, "a company kind");
    let r = api
        .call(
            Method::PUT,
            "/me/notification-settings",
            Some(&ct),
            Some(json!({ "email": [{ "kind": "invitation_received", "enabled": false }] })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let off: Vec<&str> = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["email"] == false)
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert_eq!(off, ["invitation_received"]);

    let inv = api.invite(&et, cand).await;
    assert_eq!(api.unread(&ct).await, 1, "in-app stays on");
    assert_eq!(api.queued(cand).await, 0, "no email for a kind turned off");

    let r = api
        .call(
            Method::POST,
            &format!("/employer/invitations/{inv}/withdraw"),
            Some(&et),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(api.queued(cand).await, 1, "other kinds still go");
    let d = api.dispatcher();
    let Assembled::Letter(letter) = d.assemble(cand, Utc::now()).await.unwrap() else {
        panic!()
    };
    let r = api
        .call(
            Method::PUT,
            "/me/notification-settings",
            Some(&ct),
            Some(json!({ "email": [{ "kind": "invitation_withdrawn", "enabled": false }] })),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(
        matches!(d.compose(letter, cand).await.unwrap(), Composed::Skip(_)),
        "turned off after it was queued: dropped at send time"
    );
    d.send_due().await.unwrap();
    let status: String =
        sqlx::query_scalar("SELECT status::text FROM notification_emails WHERE id = $1")
            .bind(letter)
            .fetch_one(api.pool())
            .await
            .unwrap();
    assert_eq!(status, "skipped");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn one_click_unsubscribe_needs_our_signature(pool: PgPool) {
    let api = Api::start(pool, NotifyConfig::default()).await;
    let (cand, _) = api.candidate("anna@example.ru", "Анна").await;
    let ct = api.token(cand, Role::Candidate);
    let key = api.state.crypto.derive_key(token::KEY_PURPOSE);
    let good = token::sign(
        &key,
        cand,
        Scope::Kind(NotificationKind::InvitationExpiring),
    );

    let no_redirects = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let r = api
        .send(no_redirects.get(format!(
            "{}/api/v1/notifications/unsubscribe?token={good}",
            api.base
        )))
        .await;
    assert_eq!(r.status, StatusCode::SEE_OTHER, "a GET never unsubscribes");
    assert_eq!(
        r.headers[header::LOCATION],
        format!("http://localhost:3000/notifications/unsubscribe?token={good}").as_str()
    );
    let r = api
        .call(Method::GET, "/me/notification-settings", Some(&ct), None)
        .await;
    assert!(
        r.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["email"] == true)
    );

    let mut tampered = good.clone().into_bytes();
    let last = tampered.len() - 3;
    tampered[last] = if tampered[last] == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(tampered).unwrap();
    for bad in [tampered.as_str(), "garbage", ""] {
        let r = api
            .call(
                Method::POST,
                &format!("/notifications/unsubscribe?token={bad}"),
                None,
                None,
            )
            .await;
        assert_eq!(r.status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(r.body["code"], "invalid_token");
    }
    let forged = token::sign(&[9; 32], cand, Scope::All);
    let r = api
        .call(
            Method::POST,
            &format!("/notifications/unsubscribe?token={forged}"),
            None,
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST, "signed with another key");

    let one_click = api
        .send(
            api.http
                .post(format!(
                    "{}/api/v1/notifications/unsubscribe?token={good}",
                    api.base
                ))
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body("List-Unsubscribe=One-Click"),
        )
        .await;
    assert_eq!(one_click.status, StatusCode::OK, "{}", one_click.body);
    assert_eq!(one_click.body["kind"], "invitation_expiring");
    assert_eq!(one_click.body["title"], "Напоминание о сроке ответа");
    assert_eq!(
        one_click.body["settings_path"],
        "/candidate/settings#notifications"
    );
    let again = api
        .call(
            Method::POST,
            &format!("/notifications/unsubscribe?token={good}"),
            None,
            None,
        )
        .await;
    assert_eq!(again.status, StatusCode::OK, "idempotent");
    let r = api
        .call(Method::GET, "/me/notification-settings", Some(&ct), None)
        .await;
    let off: Vec<&str> = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["email"] == false)
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert_eq!(off, ["invitation_expiring"]);

    let all = token::sign(&key, cand, Scope::All);
    let r = api
        .call(
            Method::POST,
            &format!("/notifications/unsubscribe?token={all}"),
            None,
            None,
        )
        .await;
    assert_eq!(r.body["kind"], Value::Null);
    let r = api
        .call(Method::GET, "/me/notification-settings", Some(&ct), None)
        .await;
    assert!(
        r.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["email"] == false)
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn feed_pages_marks_read_exports_and_goes_with_the_account(pool: PgPool) {
    let api = Api::start(pool, NotifyConfig::default()).await;
    let (cand, _) = api.candidate("anna@example.ru", "Анна").await;
    let ct = api.token(cand, Role::Candidate);
    let mut ids = Vec::new();
    for i in 0..5 {
        let (et, _, _) = api
            .employer(&format!("hr{i}@example{i}.ru"), &format!("Компания {i}"))
            .await;
        ids.push(api.invite(&et, cand).await);
    }
    let first = api.feed(&ct, "?limit=2").await;
    assert_eq!(first["items"].as_array().unwrap().len(), 2);
    assert!(
        first["items"][0]["title"]
            .as_str()
            .unwrap()
            .starts_with("Компания 4"),
        "newest first"
    );
    let cursor = first["next_cursor"].as_str().unwrap();
    let second = api.feed(&ct, &format!("?limit=2&cursor={cursor}")).await;
    assert!(
        second["items"][0]["title"]
            .as_str()
            .unwrap()
            .starts_with("Компания 2")
    );
    let third = api
        .feed(
            &ct,
            &format!(
                "?limit=2&cursor={}",
                second["next_cursor"].as_str().unwrap()
            ),
        )
        .await;
    assert_eq!(third["items"].as_array().unwrap().len(), 1);
    assert!(third["next_cursor"].is_null());

    let read_id = first["items"][0]["id"].as_str().unwrap();
    let r = api
        .call(
            Method::POST,
            "/me/notifications/read",
            Some(&ct),
            Some(json!({ "ids": [read_id] })),
        )
        .await;
    assert_eq!(r.body["count"], 4);
    let unread = api.feed(&ct, "?unread=true").await;
    assert_eq!(unread["items"].as_array().unwrap().len(), 4);
    let r = api
        .call(
            Method::POST,
            "/me/notifications/read",
            Some(&ct),
            Some(json!({})),
        )
        .await;
    assert_eq!(r.status, StatusCode::UNPROCESSABLE_ENTITY);
    let r = api
        .call(
            Method::POST,
            "/me/notifications/read",
            Some(&ct),
            Some(json!({ "all": true })),
        )
        .await;
    assert_eq!(r.body["count"], 0);
    let r = api
        .call(
            Method::GET,
            "/me/notifications?cursor=not-a-uuid",
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body["code"], "invalid_query");
    let r = api.call(Method::GET, "/me/notifications", None, None).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(r.headers[header::CACHE_CONTROL], "private, no-cache");
    let r = api
        .call(
            Method::GET,
            "/me/notifications/unread-count",
            Some(&ct),
            None,
        )
        .await;
    assert_eq!(r.headers[header::CACHE_CONTROL], "private, no-store");

    let r = api.call(Method::GET, "/me/export", Some(&ct), None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(
        r.body["notifications"]["items"].as_array().unwrap().len(),
        5
    );
    assert_eq!(
        r.body["notifications"]["settings"]["items"]
            .as_array()
            .unwrap()
            .len(),
        11
    );

    let r = api.call(Method::DELETE, "/me", Some(&ct), None).await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1")
        .bind(cand)
        .fetch_one(api.pool())
        .await
        .unwrap();
    assert_eq!(left, 0, "deleted with the account");
}
