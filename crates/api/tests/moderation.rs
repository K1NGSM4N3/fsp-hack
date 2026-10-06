//! Moderation over real HTTP against a throwaway database: what each decision does to the grade
//! and the grade policy, the camera retest, anti-fake actions and the trust quota, the audit log
//! of actions and recording views, keyset pages, and that only moderators get in.

use std::sync::Arc;

use api::{
    auth::{Role, jwt::JwtKeys},
    config::{Config, FspConfig, IntegrationsConfig, MailConfig, ProctoringConfig},
    crypto::Crypto,
    integrations::webhook::WebhookSender,
    kv::Kv,
    mail::Mailer,
    notify::{
        Audience, Event, Notifier,
        render::{self, LetterContext},
        sweep,
    },
    proctoring::{ProctoringMode, RecordingStore},
    state::{AppState, Services},
};
use catalog::{Category, Grade, Specialization};
use chrono::{DateTime, Duration, Utc};
use reqwest::{Method, StatusCode};
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
    body: Value,
}

impl Reply {
    fn ok(&self) -> &Value {
        assert!(self.status.is_success(), "{} {}", self.status, self.body);
        &self.body
    }

    fn problem(&self, status: StatusCode, code: &str) {
        assert_eq!(self.status, status, "{}", self.body);
        assert_eq!(self.body["code"], code, "{}", self.body);
    }
}

/// A finished backend session to put in front of the moderator.
struct Session<'a> {
    grade: Grade,
    kind: &'a str,
    passed: bool,
    finished: DateTime<Utc>,
    aberrant: bool,
    /// `None`: proctoring was off for this test.
    camera: Option<bool>,
    events: &'a [db::types::IntegrityEvent],
    /// Templates answered, in order, and whether right.
    answers: &'a [(&'static str, bool)],
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
                mode: ProctoringMode::Optional,
                dir: std::env::temp_dir().join(format!("moderation-test-{}", Uuid::now_v7())),
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
        state.recordings.init().await.unwrap();
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
        let mut req = self
            .http
            .request(method, format!("{}/api/v1{path}", self.base));
        if !token.is_empty() {
            req = req.bearer_auth(token);
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
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()))
        };
        Reply { status, body }
    }

    async fn bytes(&self, path: &str, token: &str) -> (StatusCode, Vec<u8>) {
        let res = self
            .http
            .get(format!("{}/api/v1{path}", self.base))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        (res.status(), res.bytes().await.unwrap().to_vec())
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

    async fn moderator(&self) -> String {
        let id = self.account(Role::Admin, "moderator@example.ru").await;
        self.token(id, Role::Admin)
    }

    /// A published backend candidate with the survey done and every consent.
    async fn candidate(&self, email: &str) -> (Uuid, String) {
        let id = self.account(Role::Candidate, email).await;
        db::candidates::insert_candidate(
            self.pool(),
            id,
            &format!("K-{}", &id.simple().to_string()[26..]),
        )
        .await
        .unwrap();
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
        for doc in [PROCESSING, PUBLICATION, TERMS] {
            db::consents::insert_user_consent(self.pool(), Uuid::now_v7(), id, doc, None, None)
                .await
                .unwrap();
        }
        (id, self.token(id, Role::Candidate))
    }

    async fn employer(&self, email: &str) -> (String, Uuid) {
        let owner = self.account(Role::Employer, email).await;
        let company = Uuid::now_v7();
        db::companies::insert_company(self.pool(), company, owner, "Рога и копыта")
            .await
            .unwrap();
        (self.token(owner, Role::Employer), company)
    }

    /// Writes a session the way the API would have: responses, proctoring, events, and on a pass
    /// the grade assignment, the category and the skills its right answers confirm.
    async fn session(&self, candidate: Uuid, s: Session<'_>) -> Uuid {
        let state = assessment::start(Specialization::Backend, s.grade, 7, Vec::new());
        let item = assessment::next_item(&state).unwrap();
        let id = Uuid::now_v7();
        let started = s.finished - Duration::minutes(20);
        db::assessment::insert_session(
            self.pool(),
            &db::assessment::NewSession {
                id,
                candidate_id: candidate,
                specialization: "backend",
                claimed_grade: s.grade.slug(),
                attempt_kind: s.kind,
                state: &serde_json::to_value(&state).unwrap(),
                current_item: &serde_json::to_value(&item).unwrap(),
                deadline: started + Duration::minutes(3),
                shown_at: started,
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE assessment_sessions SET started_at = $2 WHERE id = $1")
            .bind(id)
            .bind(started)
            .execute(self.pool())
            .await
            .unwrap();
        let theta = s.grade.theta_cut() + if s.passed { 0.3 } else { -0.4 };
        db::assessment::finish_session(
            self.pool(),
            id,
            &db::assessment::FinishedSession {
                status: if s.passed {
                    db::types::AssessmentStatus::Passed
                } else {
                    db::types::AssessmentStatus::Failed
                },
                state: &serde_json::to_value(&state).unwrap(),
                theta,
                se: 0.3,
                measured_grade: Some(s.grade.slug()),
                confident_above_next: false,
                person_fit_lz: if s.aberrant { -2.7 } else { 0.1 },
                aberrant: s.aberrant,
                items_count: s.answers.len() as i32,
                finished_at: s.finished,
            },
        )
        .await
        .unwrap();
        for (seq, (template, correct)) in s.answers.iter().enumerate() {
            db::assessment::insert_response(
                self.pool(),
                &db::assessment::NewResponse {
                    session_id: id,
                    seq: seq as i32,
                    template_id: template,
                    level: 1,
                    seed: 1,
                    answer: None,
                    correct: *correct,
                    timed_out: false,
                    elapsed_ms: 60_000,
                    theta_after: theta,
                    se_after: 0.4,
                    answered_at: started + Duration::minutes(2 * (seq as i64 + 1)),
                },
            )
            .await
            .unwrap();
        }
        if let Some(camera) = s.camera {
            db::proctoring::insert_proctoring(self.pool(), id, candidate, camera, None, started)
                .await
                .unwrap();
            for (i, kind) in s.events.iter().enumerate() {
                db::proctoring::insert_event(
                    self.pool(),
                    id,
                    &db::proctoring::Event {
                        kind: *kind,
                        occurred_at: started + Duration::minutes(2 * i as i64 + 1),
                        item_seq: None,
                        duration_ms: Some(20_000),
                    },
                )
                .await
                .unwrap();
            }
        }
        if s.passed {
            db::grades::insert_grade_assignment(
                self.pool(),
                &db::grades::NewGradeAssignment {
                    id: Uuid::now_v7(),
                    candidate_id: candidate,
                    specialization: "backend",
                    grade: s.grade.slug(),
                    theta,
                    se: 0.3,
                    session_id: Some(id),
                    reason: if s.kind == "upgrade" {
                        "upgrade"
                    } else {
                        "placement"
                    },
                    assigned_at: s.finished,
                },
            )
            .await
            .unwrap();
            db::grades::set_candidate_category(
                self.pool(),
                "backend",
                s.grade.slug(),
                theta,
                s.finished,
                candidate,
            )
            .await
            .unwrap();
            for (template, correct) in s.answers {
                if !correct {
                    continue;
                }
                for skill in template_skills(template) {
                    db::candidates::insert_candidate_skill(self.pool(), candidate, skill, "test")
                        .await
                        .unwrap();
                }
            }
        }
        self.state.candidates.invalidate().await;
        id
    }

    async fn test_skills(&self, candidate: Uuid) -> Vec<String> {
        let mut out: Vec<String> = sqlx::query_scalar(
            "SELECT skill FROM candidate_skills WHERE candidate_id = $1 AND source = 'test'",
        )
        .bind(candidate)
        .fetch_all(self.pool())
        .await
        .unwrap();
        out.sort();
        out
    }

    async fn log(&self, token: &str, query: &str) -> Vec<Value> {
        self.call(Method::GET, &format!("/admin/log?{query}"), token, None)
            .await
            .ok()["items"]
            .as_array()
            .unwrap()
            .clone()
    }

    async fn decide(&self, token: &str, session: Uuid, decision: &str, comment: &str) -> Reply {
        self.call(
            Method::POST,
            &format!("/admin/proctoring/{session}/decision"),
            token,
            Some(json!({ "decision": decision, "comment": comment })),
        )
        .await
    }

    async fn feed(&self, token: &str) -> Vec<Value> {
        self.call(Method::GET, "/me/notifications?limit=50", token, None)
            .await
            .ok()["items"]
            .as_array()
            .unwrap()
            .clone()
    }

    /// Every letter the user could get, rendered from the stored payloads.
    async fn letters(&self, user: Uuid, audience: Audience) -> String {
        let payloads: Vec<String> = sqlx::query_scalar(
            "SELECT payload::text FROM notifications WHERE user_id = $1 ORDER BY id",
        )
        .bind(user)
        .fetch_all(self.pool())
        .await
        .unwrap();
        let events: Vec<Event> = payloads.iter().filter_map(|p| Event::parse(p)).collect();
        assert_eq!(events.len(), payloads.len(), "every payload parses");
        let letter = render::letter(
            &events,
            &LetterContext {
                public_url: "http://localhost:3000",
                audience,
                unsubscribe_url: "http://localhost:3000/notifications/unsubscribe?token=T",
                unsubscribe_label: "все письма",
            },
        );
        format!("{}\n{}\n{}", letter.subject, letter.text, letter.html)
    }

    async fn act(
        &self,
        token: &str,
        kind: &str,
        id: Uuid,
        action: &str,
        days: Option<u16>,
    ) -> Reply {
        self.call(
            Method::POST,
            "/admin/antifake/actions",
            token,
            Some(json!({
                "kind": kind,
                "id": id,
                "action": action,
                "comment": format!("{action}: проверено модератором"),
                "days": days,
            })),
        )
        .await
    }
}

fn template_skills(id: &str) -> Vec<&'static str> {
    assessment::bank()
        .template(id)
        .map(|t| {
            t.skills
                .iter()
                .copied()
                .filter(|s| matching::lookup(s).is_some())
                .collect()
        })
        .unwrap_or_default()
}

/// Backend templates whose level 1 exists and which confirm at least one dictionary skill, each
/// with a skill no earlier one has.
fn templates() -> Vec<&'static str> {
    let mut seen: Vec<&str> = Vec::new();
    let mut out = Vec::new();
    for t in assessment::bank().for_specialization(Specialization::Backend) {
        if t.level(1).is_none() {
            continue;
        }
        let skills = template_skills(t.id);
        if skills.is_empty() || skills.iter().all(|s| seen.contains(s)) {
            continue;
        }
        seen.extend(skills);
        out.push(t.id);
        if out.len() == 4 {
            break;
        }
    }
    assert_eq!(out.len(), 4, "the bank has distinct backend templates");
    out
}

fn ids(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            i.get("session_id")
                .or_else(|| i.get("id"))
                .and_then(Value::as_str)
                .unwrap()
                .to_owned()
        })
        .collect()
}

fn days_from_now(v: &Value) -> i64 {
    let at: DateTime<Utc> = v.as_str().unwrap().parse().unwrap();
    (at - Utc::now()).num_hours().div_euclid(24)
}

const BEHAVIOUR: &[db::types::IntegrityEvent] = &[
    db::types::IntegrityEvent::TabHidden,
    db::types::IntegrityEvent::Paste,
];

#[sqlx::test(migrations = "../../db/migrations")]
async fn annulled_upgrade_restores_the_previous_grade_and_its_window(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let (cand, cand_token) = api.candidate("upgrade@example.ru").await;
    let junior = api
        .session(
            cand,
            Session {
                grade: Grade::Junior,
                kind: "placement",
                passed: true,
                finished: Utc::now() - Duration::days(200),
                aberrant: false,
                camera: Some(true),
                events: &[],
                answers: &[(t[0], true), (t[3], false)],
            },
        )
        .await;
    let middle = api
        .session(
            cand,
            Session {
                grade: Grade::Middle,
                kind: "upgrade",
                passed: true,
                finished: Utc::now() - Duration::days(2),
                aberrant: true,
                camera: Some(true),
                events: BEHAVIOUR,
                answers: &[(t[0], true), (t[1], true), (t[2], true)],
            },
        )
        .await;
    let before = api.test_skills(cand).await;

    let queue = api
        .call(
            Method::GET,
            "/admin/proctoring?filter=needs_review",
            &moderator,
            None,
        )
        .await;
    assert_eq!(ids(queue.ok()), [middle.to_string()]);
    assert_eq!(queue.body["items"][0]["needs_review"], true);
    assert_eq!(queue.body["items"][0]["video"], "kept");
    assert!(
        queue.body["items"][0].get("full_name").is_none(),
        "the moderator works with the code"
    );

    let review = api
        .call(
            Method::GET,
            &format!("/admin/proctoring/{middle}"),
            &moderator,
            None,
        )
        .await;
    let r = review.ok();
    assert_eq!(r["allowed"], json!(["confirm", "annul", "retest"]));
    assert_eq!(r["assignment"]["current"], true);
    assert_eq!(r["flagged"], true);
    assert_eq!(r["answers"].as_array().unwrap().len(), 3);
    assert_eq!(
        r["events"][0]["item_seq"], 0,
        "a tab leave on the first item"
    );
    assert_eq!(
        r["events"][1]["item_seq"], 1,
        "the paste happened on the second"
    );
    assert!(
        r["person_fit"]["summary"]
            .as_str()
            .unwrap()
            .contains("ниже порога")
    );

    api.decide(&moderator, middle, "annul", "  ")
        .await
        .problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
    let decided = api
        .decide(
            &moderator,
            middle,
            "annul",
            "Ответы на трудные задания пришли сразу после уходов со вкладки.",
        )
        .await;
    assert_eq!(decided.ok()["decision"]["decision"], "annul");
    assert!(decided.body["assignment"]["annulled_at"].is_string());
    assert_eq!(decided.body["allowed"], json!([]));
    api.decide(&moderator, middle, "confirm", "передумал")
        .await
        .problem(StatusCode::CONFLICT, "already_decided");

    let category = api
        .call(Method::GET, "/candidate/category", &cand_token, None)
        .await;
    let c = category.ok();
    assert_eq!(c["category"]["grade"], "junior", "Junior is current again");
    let history = c["history"].as_array().unwrap();
    assert_eq!(
        history.len(),
        2,
        "the voided assignment stays in the history"
    );
    assert!(history[0]["annulled_at"].is_string());
    assert!(history[1]["annulled_at"].is_null());
    let locked = c["locked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["grade"] == "middle")
        .unwrap_or_else(|| panic!("middle is locked: {c}"));
    assert_eq!(locked["reason"], "retry_blocked");
    assert_eq!(
        days_from_now(&locked["available_at"]),
        29,
        "30 days from the decision, not 88 left of the voided assignment's window"
    );

    let session = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{middle}"),
            &cand_token,
            None,
        )
        .await;
    let s = session.ok();
    assert_eq!(s["review"]["decision"], "annul");
    assert!(
        s["review"]["comment"]
            .as_str()
            .unwrap()
            .starts_with("Ответы")
    );
    assert!(s["result"]["assigned_grade"].is_null());
    assert_eq!(s["result"]["needs_review"], false);

    let after = api.test_skills(cand).await;
    let junior_skills = template_skills(t[0]);
    for skill in &before {
        let kept = after.contains(skill);
        assert_eq!(
            kept,
            junior_skills.contains(&skill.as_str()),
            "{skill}: only what the Junior test confirmed keeps its test source"
        );
    }

    let row: (Option<String>, Option<String>) =
        sqlx::query_as("SELECT specialization, grade FROM candidates WHERE user_id = $1")
            .bind(cand)
            .fetch_one(api.pool())
            .await
            .unwrap();
    assert_eq!(row, (Some("backend".into()), Some("junior".into())));

    let log = api.log(&moderator, &format!("subject={middle}")).await;
    assert_eq!(log.len(), 1);
    assert_eq!(log[0]["action"], "annul_result");
    assert_eq!(log[0]["moderator"], "moderator@example.ru");
    assert!(log[0]["details"][0].as_str().unwrap().contains("Junior"));
    assert!(
        api.log(&moderator, &format!("subject={junior}"))
            .await
            .is_empty(),
        "the Junior test was not touched"
    );

    sqlx::query("UPDATE candidates SET public_code = 'K-7Q3M2X' WHERE user_id = $1")
        .bind(cand)
        .execute(api.pool())
        .await
        .unwrap();
    let by_code = api.log(&moderator, "subject=%20k-7q3m2x%20").await;
    assert_eq!(
        by_code.len(),
        1,
        "the code, typed loosely, finds the candidate's actions"
    );
    assert_eq!(by_code[0]["candidate_code"], "K-7Q3M2X");
    assert!(api.log(&moderator, "subject=K-ZZZZZZ").await.is_empty());
    let r = api
        .call(Method::GET, "/admin/log?subject=nobody", &moderator, None)
        .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body["code"], "invalid_query");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn annulled_placement_leaves_no_category_and_blocks_its_grade(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let (cand, cand_token) = api.candidate("placement@example.ru").await;
    let (hr, _) = api.employer("hr@example.ru").await;
    let session = api
        .session(
            cand,
            Session {
                grade: Grade::Middle,
                kind: "placement",
                passed: true,
                finished: Utc::now() - Duration::days(40),
                aberrant: true,
                camera: None,
                events: &[],
                answers: &[(t[0], true), (t[1], true)],
            },
        )
        .await;
    let profile = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}"),
            &hr,
            None,
        )
        .await;
    assert_eq!(profile.ok()["candidate"]["category"]["grade"], "middle");

    let queue = api
        .call(Method::GET, "/admin/proctoring", &moderator, None)
        .await;
    assert_eq!(
        ids(queue.ok()),
        [session.to_string()],
        "person-fit alone flags a test without proctoring data"
    );
    assert_eq!(queue.body["items"][0]["video"], "off");

    api.decide(
        &moderator,
        session,
        "annul",
        "Без видео, ответы не согласуются с уровнем.",
    )
    .await
    .ok();
    let category = api
        .call(Method::GET, "/candidate/category", &cand_token, None)
        .await;
    assert!(category.ok()["category"].is_null());

    let start = |grade: &str| json!({ "specialization": "backend", "grade": grade, "proctoring": { "camera": false, "consent_document_id": null } });
    let blocked = api
        .call(
            Method::POST,
            "/candidate/assessments",
            &cand_token,
            Some(start("middle")),
        )
        .await;
    blocked.problem(StatusCode::CONFLICT, "retry_blocked");
    let step_down = api
        .call(
            Method::POST,
            "/candidate/assessments",
            &cand_token,
            Some(start("junior")),
        )
        .await;
    assert_eq!(step_down.status, StatusCode::CREATED, "{}", step_down.body);
    assert_eq!(step_down.body["attempt_kind"], "placement_step_down");

    let hidden = api
        .call(
            Method::GET,
            "/employer/candidates?specialization=backend",
            &hr,
            None,
        )
        .await;
    assert_eq!(hidden.ok()["total"], 0, "no category, no search");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn ordered_retest_must_use_the_camera(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let (cand, cand_token) = api.candidate("retest@example.ru").await;
    let session = api
        .session(
            cand,
            Session {
                grade: Grade::Middle,
                kind: "placement",
                passed: true,
                finished: Utc::now() - Duration::days(5),
                aberrant: true,
                camera: Some(false),
                events: BEHAVIOUR,
                answers: &[(t[0], true), (t[1], true)],
            },
        )
        .await;
    api.decide(
        &moderator,
        session,
        "retest",
        "Без камеры спор не разрешить: пройдите тест ещё раз с видеозаписью.",
    )
    .await
    .ok();

    let category = api
        .call(Method::GET, "/candidate/category", &cand_token, None)
        .await;
    let c = category.ok();
    assert!(c["category"].is_null());
    let offer = c["available"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["grade"] == "middle")
        .unwrap_or_else(|| panic!("the retest is open now: {c}"));
    assert_eq!(offer["ordered_retest"], true);
    assert_eq!(offer["camera_required"], true);
    assert_eq!(offer["kind"], "placement");
    assert!(offer["label"].as_str().unwrap().starts_with("Пересдача"));

    for proctoring in [
        Value::Null,
        json!({ "camera": false, "consent_document_id": null }),
    ] {
        api.call(
            Method::POST,
            "/candidate/assessments",
            &cand_token,
            Some(
                json!({ "specialization": "backend", "grade": "middle", "proctoring": proctoring }),
            ),
        )
        .await
        .problem(StatusCode::CONFLICT, "camera_required");
    }
    let options = api
        .call(Method::GET, "/candidate/proctoring", &cand_token, None)
        .await;
    let doc = options.ok()["consent"]["id"].clone();
    let started = api
        .call(
            Method::POST,
            "/candidate/assessments",
            &cand_token,
            Some(json!({
                "specialization": "backend",
                "grade": "middle",
                "proctoring": { "camera": true, "consent_document_id": doc },
            })),
        )
        .await;
    assert_eq!(started.status, StatusCode::CREATED, "{}", started.body);
    assert_eq!(started.body["proctoring"]["status"], "recording");

    let log = api.log(&moderator, "action=order_retest").await;
    assert_eq!(log.len(), 1);
    assert_eq!(log[0]["target_id"], session.to_string());
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn confirmed_result_clears_the_flag_and_reaches_the_card(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let (cand, cand_token) = api.candidate("confirm@example.ru").await;
    let (hr, _) = api.employer("hr@example.ru").await;
    let session = api
        .session(
            cand,
            Session {
                grade: Grade::Middle,
                kind: "placement",
                passed: true,
                finished: Utc::now() - Duration::days(3),
                aberrant: true,
                camera: Some(true),
                events: BEHAVIOUR,
                answers: &[(t[0], true), (t[1], false)],
            },
        )
        .await;
    let before = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}"),
            &hr,
            None,
        )
        .await;
    assert_eq!(before.ok()["candidate"]["result_confirmed"], false);
    let flagged = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{session}"),
            &cand_token,
            None,
        )
        .await;
    assert_eq!(flagged.ok()["result"]["needs_review"], true);

    api.decide(
        &moderator,
        session,
        "confirm",
        "Запись чистая: вкладку открывал документацию к языку, это разрешено.",
    )
    .await
    .ok();
    let after = api
        .call(
            Method::GET,
            &format!("/employer/candidates/{cand}"),
            &hr,
            None,
        )
        .await;
    let card = &after.ok()["candidate"];
    assert_eq!(card["result_confirmed"], true);
    assert_eq!(card["category"]["grade"], "middle");
    assert!(
        !card.to_string().contains("Запись чистая"),
        "the comment stays with the candidate"
    );
    let own = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{session}"),
            &cand_token,
            None,
        )
        .await;
    assert_eq!(own.ok()["result"]["needs_review"], false);
    assert_eq!(own.body["review"]["decision"], "confirm");
    let queue = api
        .call(
            Method::GET,
            "/admin/proctoring?filter=needs_review",
            &moderator,
            None,
        )
        .await;
    assert!(ids(queue.ok()).is_empty());
    let video = api
        .call(
            Method::GET,
            "/admin/proctoring?filter=video",
            &moderator,
            None,
        )
        .await;
    assert_eq!(video.ok()["items"][0]["decision"], "confirm");

    let (other, _) = api.candidate("running@example.ru").await;
    let running = api
        .session(
            other,
            Session {
                grade: Grade::Junior,
                kind: "placement",
                passed: true,
                finished: Utc::now(),
                aberrant: false,
                camera: None,
                events: &[],
                answers: &[(t[0], true)],
            },
        )
        .await;
    sqlx::query(
        "UPDATE assessment_sessions SET status = 'in_progress', finished_at = NULL WHERE id = $1",
    )
    .bind(running)
    .execute(api.pool())
    .await
    .unwrap();
    api.decide(&moderator, running, "confirm", "рано")
        .await
        .problem(StatusCode::CONFLICT, "session_not_finished");
    api.decide(&moderator, Uuid::now_v7(), "confirm", "нет такого")
        .await
        .problem(StatusCode::NOT_FOUND, "not_found");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn queue_pages_oldest_first(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let mut expected = Vec::new();
    for (i, days) in [9, 30, 4].into_iter().enumerate() {
        let (cand, _) = api.candidate(&format!("page{i}@example.ru")).await;
        let id = api
            .session(
                cand,
                Session {
                    grade: Grade::Junior,
                    kind: "placement",
                    passed: true,
                    finished: Utc::now() - Duration::days(days),
                    aberrant: true,
                    camera: Some(true),
                    events: BEHAVIOUR,
                    answers: &[(t[0], true)],
                },
            )
            .await;
        expected.push((days, id.to_string()));
    }
    expected.sort_by_key(|(days, _)| std::cmp::Reverse(*days));
    let expected: Vec<String> = expected.into_iter().map(|(_, id)| id).collect();
    let first = api
        .call(Method::GET, "/admin/proctoring?limit=2", &moderator, None)
        .await;
    assert_eq!(ids(first.ok()), expected[..2]);
    let cursor = first.body["next_cursor"].as_str().unwrap().to_owned();
    let second = api
        .call(
            Method::GET,
            &format!("/admin/proctoring?limit=2&cursor={cursor}"),
            &moderator,
            None,
        )
        .await;
    assert_eq!(ids(second.ok()), expected[2..]);
    assert!(second.body["next_cursor"].is_null());
    api.call(
        Method::GET,
        "/admin/proctoring?cursor=nope",
        &moderator,
        None,
    )
    .await
    .problem(StatusCode::BAD_REQUEST, "invalid_query");
    let remarks = api
        .call(
            Method::GET,
            "/admin/proctoring?filter=remarks",
            &moderator,
            None,
        )
        .await;
    assert_eq!(ids(remarks.ok()).len(), 3);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn every_recording_view_is_logged_and_shown_to_the_candidate(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let (_, cand_token) = api.candidate("video@example.ru").await;
    let options = api
        .call(Method::GET, "/candidate/proctoring", &cand_token, None)
        .await;
    let doc = options.ok()["consent"]["id"].clone();
    let started = api
        .call(
            Method::POST,
            "/candidate/assessments",
            &cand_token,
            Some(json!({
                "specialization": "backend",
                "grade": "middle",
                "proctoring": { "camera": true, "consent_document_id": doc },
            })),
        )
        .await;
    let session = started.ok()["id"].as_str().unwrap().to_owned();
    for (seq, chunk) in [b"webm-head".as_slice(), b"-tail".as_slice()]
        .iter()
        .enumerate()
    {
        let res = api
            .http
            .put(format!(
                "{}/api/v1/candidate/assessments/{session}/proctoring/chunks/{seq}",
                api.base
            ))
            .bearer_auth(&cand_token)
            .header("content-type", "video/webm")
            .body(chunk.to_vec())
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
    }

    let path = format!("/admin/proctoring/{session}/recording");
    let probe = api
        .http
        .head(format!("{}/api/v1{path}", api.base))
        .bearer_auth(&moderator)
        .send()
        .await
        .unwrap();
    assert_eq!(probe.status(), StatusCode::OK);
    assert!(
        api.log(&moderator, "action=view_recording")
            .await
            .is_empty(),
        "a HEAD probe sends no video and is not a view"
    );
    for _ in 0..2 {
        let (status, body) = api.bytes(&path, &moderator).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"webm-head-tail");
    }
    let (own, _) = api
        .bytes(
            &format!("/candidate/assessments/{session}/proctoring/recording"),
            &cand_token,
        )
        .await;
    assert_eq!(own, StatusCode::OK);
    let views = api.log(&moderator, "action=view_recording").await;
    assert_eq!(
        views.len(),
        2,
        "each moderator view, not the candidate's own"
    );
    assert!(views.iter().all(|v| v["target_id"] == session.as_str()));
    assert!(views.iter().all(|v| v["comment"].is_null()));

    let mine = api
        .call(
            Method::GET,
            &format!("/candidate/assessments/{session}"),
            &cand_token,
            None,
        )
        .await;
    assert_eq!(mine.ok()["recording_views"].as_array().unwrap().len(), 2);
    let review = api
        .call(
            Method::GET,
            &format!("/admin/proctoring/{session}"),
            &moderator,
            None,
        )
        .await;
    assert_eq!(
        review.ok()["recording"]["views"].as_array().unwrap().len(),
        2
    );

    let gone = Uuid::now_v7();
    let (missing, _) = api
        .bytes(&format!("/admin/proctoring/{gone}/recording"), &moderator)
        .await;
    assert_eq!(missing, StatusCode::NOT_FOUND);
    sqlx::query("UPDATE proctoring_sessions SET purged_at = now() WHERE session_id = $1")
        .bind(session.parse::<Uuid>().unwrap())
        .execute(api.pool())
        .await
        .unwrap();
    let purged = api.call(Method::GET, &path, &moderator, None).await;
    purged.problem(StatusCode::NOT_FOUND, "recording_unavailable");
    assert_eq!(
        api.log(&moderator, "action=view_recording").await.len(),
        2,
        "a refused view is not a view"
    );
    let review = api
        .call(
            Method::GET,
            &format!("/admin/proctoring/{session}"),
            &moderator,
            None,
        )
        .await;
    assert_eq!(review.ok()["recording"]["video"], "purged");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn antifake_actions_change_the_quota_and_reach_the_company(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let (hr, company) = api.employer("hr@example.ru").await;
    sqlx::query("UPDATE companies SET website = 'https://example.ru' WHERE id = $1;")
        .bind(company)
        .execute(api.pool())
        .await
        .unwrap();
    sqlx::query(
        "UPDATE users SET created_at = now() - interval '120 days'
         WHERE id = (SELECT owner_user_id FROM companies WHERE id = $1)",
    )
    .bind(company)
    .execute(api.pool())
    .await
    .unwrap();
    let (cand, cand_token) = api.candidate("reporter@example.ru").await;
    let (other, other_token) = api.candidate("other@example.ru").await;
    for (id, grade) in [(cand, "middle"), (other, "middle")] {
        db::grades::set_candidate_category(api.pool(), "backend", grade, 0.4, Utc::now(), id)
            .await
            .unwrap();
    }
    api.state.candidates.invalidate().await;

    let vacancy = api
        .call(
            Method::POST,
            "/employer/vacancies",
            &hr,
            Some(json!({
                "title": "Go-разработчик",
                "description": "Платёжный шлюз на Go и PostgreSQL",
                "specialization": "backend",
                "grade": "middle",
                "skills": ["go"],
                "salary": { "min": 100000, "max": 500000 },
                "work_format": "remote",
                "city": null,
                "status": "published",
            })),
        )
        .await;
    let vacancy_id: Uuid = vacancy.ok()["id"].as_str().unwrap().parse().unwrap();
    for (token, kind, target) in [
        (&cand_token, "vacancy", vacancy_id),
        (&other_token, "company", company),
    ] {
        let r = api
            .call(
                Method::POST,
                "/reports",
                token,
                Some(json!({ "target_kind": kind, "target_id": target, "reason": "fake_vacancy" })),
            )
            .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    }
    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    let quota_before = trust.ok()["quota_limit"].as_i64().unwrap();

    let queue = api
        .call(Method::GET, "/admin/antifake", &moderator, None)
        .await;
    let items = queue.ok()["items"].as_array().unwrap().clone();
    let kinds: Vec<&str> = items.iter().map(|i| i["kind"].as_str().unwrap()).collect();
    for kind in ["company", "vacancy", "complaint"] {
        assert!(kinds.contains(&kind), "{kind} in {kinds:?}");
    }
    let complaint = |target: &str| {
        items
            .iter()
            .find(|i| i["kind"] == "complaint" && i["complaint"]["target_kind"] == target)
            .and_then(|i| i["id"].as_str())
            .unwrap()
            .parse::<Uuid>()
            .unwrap()
    };
    let (about_vacancy, about_company) = (complaint("vacancy"), complaint("company"));
    assert!(
        !queue.body.to_string().contains("reporter@example.ru"),
        "the reporter stays out"
    );

    api.act(&moderator, "complaint", about_company, "dismiss", None)
        .await
        .ok();
    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    assert_eq!(
        trust.ok()["quota_limit"].as_i64().unwrap(),
        quota_before + 10,
        "an unfounded complaint stops counting"
    );
    api.act(&moderator, "complaint", about_company, "dismiss", None)
        .await
        .problem(StatusCode::CONFLICT, "action_not_applicable");

    let down = api
        .act(&moderator, "complaint", about_vacancy, "unpublish", None)
        .await;
    assert_eq!(down.ok()["complaints_resolved"], 1);
    let public = api
        .call(Method::GET, &format!("/vacancies/{vacancy_id}"), "", None)
        .await;
    assert_eq!(public.status, StatusCode::NOT_FOUND);
    let own = api
        .call(
            Method::GET,
            &format!("/employer/vacancies/{vacancy_id}"),
            &hr,
            None,
        )
        .await;
    assert!(own.ok()["unpublished_by_moderator_at"].is_string());
    let mut republish = own.body.clone();
    republish["status"] = json!("published");
    api.call(
        Method::PUT,
        &format!("/employer/vacancies/{vacancy_id}"),
        &hr,
        Some(republish),
    )
    .await
    .problem(StatusCode::CONFLICT, "vacancy_unpublished");
    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    assert_eq!(
        trust.ok()["quota_limit"].as_i64().unwrap(),
        quota_before + 10,
        "the upheld complaint and the flag keep counting"
    );

    let frozen = api
        .act(&moderator, "company", company, "freeze", Some(14))
        .await;
    assert!(frozen.ok()["frozen_until"].is_string());
    api.act(&moderator, "company", company, "freeze", Some(0))
        .await
        .problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    let t = trust.ok();
    assert_eq!(
        (t["level"].as_str(), t["quota_limit"].as_i64()),
        (Some("frozen"), Some(0))
    );
    assert_eq!(days_from_now(&t["frozen_until"]), 13);
    let invite = api
        .call(
            Method::POST,
            "/employer/invitations",
            &hr,
            Some(json!({
                "candidate_id": cand,
                "title": "Go-разработчик",
                "message": "Здравствуйте!",
                "salary": { "min": 200000, "max": 260000 },
                "contact": "@hr",
            })),
        )
        .await;
    invite.problem(StatusCode::CONFLICT, "invitations_frozen");

    let sanctions = api
        .call(
            Method::GET,
            "/admin/antifake?view=sanctions",
            &moderator,
            None,
        )
        .await;
    let sanctioned: Vec<(String, String)> = sanctions.ok()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["kind"].as_str().unwrap().into(),
                i["id"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert!(sanctioned.contains(&("vacancy".into(), vacancy_id.to_string())));
    assert!(sanctioned.contains(&("company".into(), company.to_string())));

    api.act(&moderator, "company", company, "restore", None)
        .await
        .ok();
    let trust = api.call(Method::GET, "/employer/trust", &hr, None).await;
    assert!(trust.ok()["quota_limit"].as_i64().unwrap() > 0);
    assert!(trust.body["frozen_until"].is_null());
    api.act(&moderator, "vacancy", vacancy_id, "restore", None)
        .await
        .ok();
    let public = api
        .call(Method::GET, &format!("/vacancies/{vacancy_id}"), "", None)
        .await;
    assert_eq!(public.status, StatusCode::OK);

    let notices: Vec<String> = trust.body["moderation"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["action"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(
        notices,
        [
            "restore",
            "freeze_invitations",
            "unpublish_vacancy",
            "dismiss"
        ],
        "newest first; taking the company off the queue is internal"
    );
    assert!(trust.body["moderation"][0]["comment"].is_string());

    let log = api.log(&moderator, &format!("subject={company}")).await;
    let actions: Vec<&str> = log.iter().map(|e| e["action"].as_str().unwrap()).collect();
    assert_eq!(
        actions,
        [
            "restore",
            "restore",
            "freeze_invitations",
            "unpublish_vacancy",
            "dismiss"
        ]
    );
    assert!(log.iter().all(|e| e["comment"].is_string()));
    let page = api
        .call(Method::GET, "/admin/log?limit=2", &moderator, None)
        .await;
    let cursor = page.ok()["next_cursor"].as_str().unwrap().to_owned();
    let rest = api
        .call(
            Method::GET,
            &format!("/admin/log?limit=10&cursor={cursor}"),
            &moderator,
            None,
        )
        .await;
    assert_eq!(rest.ok()["items"].as_array().unwrap().len(), 3);
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn only_moderators_reach_moderation(pool: PgPool) {
    let api = Api::start(pool).await;
    let (_, candidate) = api.candidate("nosy@example.ru").await;
    let (employer, _) = api.employer("hr@example.ru").await;
    let spec = serde_json::to_value(api::openapi()).unwrap();
    let mut checked = 0;
    for (path, item) in spec["paths"].as_object().unwrap() {
        let Some(path) = path.strip_prefix("/api/v1") else {
            continue;
        };
        if !path.starts_with("/admin") {
            continue;
        }
        let path = path.replace("{session_id}", &Uuid::now_v7().to_string());
        for (method, op) in item.as_object().unwrap() {
            let method: Method = method.to_uppercase().parse().unwrap();
            assert_eq!(op["tags"][0], "Модерация", "{method} {path}");
            let body = (method == Method::POST).then(|| json!({}));
            for token in [&candidate, &employer] {
                api.call(method.clone(), &path, token, body.clone())
                    .await
                    .problem(StatusCode::FORBIDDEN, "wrong_role");
            }
            api.call(method.clone(), &path, "", body.clone())
                .await
                .problem(StatusCode::UNAUTHORIZED, "unauthorized");
            checked += 1;
        }
    }
    assert!(checked >= 9, "every moderation operation: {checked}");
    let register = api
        .call(
            Method::POST,
            "/auth/register",
            "",
            Some(json!({
                "email": "sneaky@example.ru",
                "password": "correct horse battery",
                "role": "admin",
                "consents": [PROCESSING, TERMS],
            })),
        )
        .await;
    register.problem(StatusCode::UNPROCESSABLE_ENTITY, "validation_failed");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn demo_seed_fills_the_moderation_queues(pool: PgPool) {
    let api = Api::start(pool).await;
    let crypto = Crypto::new(&[1; 32], &[2; 32]);
    api::seed::seed(api.pool(), &crypto, 90)
        .await
        .unwrap()
        .unwrap();
    let moderator: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE role = 'admin'")
        .fetch_one(api.pool())
        .await
        .unwrap();
    let token = api.token(moderator, Role::Admin);
    let overview = api.call(Method::GET, "/admin/overview", &token, None).await;
    let o = overview.ok();
    assert!(
        o["proctoring"]["needs_review"].as_i64().unwrap() >= 5,
        "{o}"
    );
    assert!(o["proctoring"]["remarks"].as_i64().unwrap() >= 5, "{o}");
    assert_eq!(o["antifake"]["complaints"], 3, "{o}");
    assert_eq!(o["antifake"]["vacancies"], 2, "{o}");
    assert!(o["antifake"]["companies"].as_i64().unwrap() >= 1, "{o}");
    let queue = api
        .call(Method::GET, "/admin/proctoring", &token, None)
        .await;
    let items = queue.ok()["items"].as_array().unwrap();
    assert!(items.iter().any(|i| i["video"] == "off"));
    assert!(
        items
            .iter()
            .any(|i| i["remarks"]["left_test"].as_i64().unwrap() > 0)
    );
    let first = items[0]["session_id"].as_str().unwrap();
    let review = api
        .call(
            Method::GET,
            &format!("/admin/proctoring/{first}"),
            &token,
            None,
        )
        .await;
    let r = review.ok();
    assert_eq!(r["person_fit"]["aberrant"], true, "{}", r["person_fit"]);
    assert!(r["person_fit"]["lz"].as_f64().unwrap() < -2.0);
}

fn kinds(feed: &[Value]) -> Vec<&str> {
    feed.iter().map(|n| n["kind"].as_str().unwrap()).collect()
}

fn text_of(item: &Value) -> String {
    format!("{} {}", item["title"], item["body"])
}

#[sqlx::test(
    migrations = "../../db/migrations",
    fixtures("fixtures/moderation_reject_notices.sql")
)]
async fn a_moderation_notice_commits_with_its_decision_or_not_at_all(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let (cand, cand_token) = api.candidate("atomic@example.ru").await;
    let session = api
        .session(
            cand,
            Session {
                grade: Grade::Middle,
                kind: "placement",
                passed: true,
                finished: Utc::now() - Duration::days(2),
                aberrant: true,
                camera: Some(true),
                events: BEHAVIOUR,
                answers: &[(t[0], true), (t[1], true)],
            },
        )
        .await;
    let skills = api.test_skills(cand).await;

    let r = api
        .decide(
            &moderator,
            session,
            "annul",
            "Ответы пришли после уходов со вкладки",
        )
        .await;
    assert_eq!(r.status, StatusCode::INTERNAL_SERVER_ERROR, "{}", r.body);
    let grade: Option<String> =
        sqlx::query_scalar("SELECT grade FROM candidates WHERE user_id = $1")
            .bind(cand)
            .fetch_one(api.pool())
            .await
            .unwrap();
    assert_eq!(
        grade.as_deref(),
        Some("middle"),
        "the annulment rolled back with its notice"
    );
    assert_eq!(
        api.test_skills(cand).await,
        skills,
        "and so did the skills it would drop"
    );
    let reviews: i64 = sqlx::query_scalar("SELECT count(*) FROM session_reviews")
        .fetch_one(api.pool())
        .await
        .unwrap();
    assert_eq!(reviews, 0);
    assert!(
        api.log(&moderator, "").await.is_empty(),
        "nor was it logged"
    );
    assert!(api.feed(&cand_token).await.is_empty());

    api.decide(&moderator, session, "confirm", "Запись чистая")
        .await
        .ok();
    assert_eq!(kinds(&api.feed(&cand_token).await), ["result_confirmed"]);

    let (hr, _) = api.employer("hr@example.ru").await;
    let vacancy = api
        .call(
            Method::POST,
            "/employer/vacancies",
            &hr,
            Some(json!({
                "title": "Go-разработчик",
                "description": "Платёжный шлюз на Go и PostgreSQL",
                "specialization": "backend",
                "grade": "middle",
                "skills": ["go"],
                "salary": { "min": 200000, "max": 260000 },
                "work_format": "remote",
                "city": null,
                "status": "published",
            })),
        )
        .await;
    let vacancy_id: Uuid = vacancy.ok()["id"].as_str().unwrap().parse().unwrap();
    let r = api
        .act(&moderator, "vacancy", vacancy_id, "unpublish", None)
        .await;
    assert_eq!(r.status, StatusCode::INTERNAL_SERVER_ERROR, "{}", r.body);
    let public = api
        .call(Method::GET, &format!("/vacancies/{vacancy_id}"), "", None)
        .await;
    assert_eq!(public.status, StatusCode::OK, "still published");
    assert_eq!(
        api.log(&moderator, "").await.len(),
        1,
        "only the confirmation"
    );
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn the_candidate_reads_each_decision_with_its_comment(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let t = templates();
    let finished = Utc::now() - Duration::days(2);
    let disputed = |grade, kind| Session {
        grade,
        kind,
        passed: true,
        finished,
        aberrant: true,
        camera: Some(true),
        events: BEHAVIOUR,
        answers: &[],
    };
    let middle = Category::new(Specialization::Backend, Grade::Middle).name_ru();
    let junior = Category::new(Specialization::Backend, Grade::Junior).name_ru();

    let (a, a_token) = api.candidate("confirmed@example.ru").await;
    let s = api.session(a, disputed(Grade::Middle, "placement")).await;
    api.decide(
        &moderator,
        s,
        "confirm",
        "Вкладку открывал на пару секунд, ответы ровные.",
    )
    .await
    .ok();
    let feed = api.feed(&a_token).await;
    assert_eq!(kinds(&feed), ["result_confirmed"]);
    let n = &feed[0];
    assert_eq!(
        n["title"],
        format!("Модератор подтвердил результат теста: {middle}")
    );
    assert_eq!(
        n["body"],
        "Отметка «требует подтверждения» снята, компании видят «Результат подтверждён». \
         Комментарий модератора: «Вкладку открывал на пару секунд, ответы ровные»."
    );
    assert_eq!(n["link"], format!("/candidate/assessment/{s}"));

    let (b, b_token) = api.candidate("annulled@example.ru").await;
    api.session(
        b,
        Session {
            finished: Utc::now() - Duration::days(200),
            aberrant: false,
            answers: &[(t[0], true)],
            ..disputed(Grade::Junior, "placement")
        },
    )
    .await;
    let s = api.session(b, disputed(Grade::Middle, "upgrade")).await;
    api.decide(
        &moderator,
        s,
        "annul",
        "Верные ответы на трудных заданиях — сразу после уходов.",
    )
    .await
    .ok();
    let n = &api.feed(&b_token).await[0];
    assert_eq!(n["kind"], "result_annulled");
    assert_eq!(n["title"], format!("Результат теста аннулирован: {middle}"));
    let body = n["body"].as_str().unwrap();
    assert!(
        body.contains("«Верные ответы на трудных заданиях — сразу после уходов»."),
        "{body}"
    );
    assert!(
        body.contains(&format!("Категория сейчас: {junior}.")),
        "{body}"
    );
    assert!(
        body.contains("Тест на Middle снова можно пройти через 30 дней."),
        "{body}"
    );

    let (c, c_token) = api.candidate("retest@example.ru").await;
    let s = api.session(c, disputed(Grade::Middle, "placement")).await;
    api.decide(&moderator, s, "retest", "Пройдите тест ещё раз с камерой.")
        .await
        .ok();
    let n = &api.feed(&c_token).await[0];
    assert_eq!(n["kind"], "retest_ordered");
    assert_eq!(
        n["title"],
        format!("Назначена пересдача с видеозаписью: {middle}")
    );
    let body = n["body"].as_str().unwrap();
    assert!(
        body.starts_with("Можно пройти сразу, без ожидания, но только с включённой камерой."),
        "{body}"
    );
    assert!(
        body.contains("«Пройдите тест ещё раз с камерой»")
            && body.ends_with("Категории сейчас нет."),
        "{body}"
    );

    for (user, token) in [(a, &a_token), (b, &b_token), (c, &c_token)] {
        let letters = api.letters(user, Audience::Candidate).await;
        let feed = serde_json::to_string(&api.feed(token).await).unwrap();
        for part in [&letters, &feed] {
            assert!(
                !part.contains("moderator@example.ru"),
                "the moderator is not named"
            );
        }
        let queued: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM notifications WHERE user_id = $1 AND email_due_at IS NOT NULL",
        )
        .bind(user)
        .fetch_one(api.pool())
        .await
        .unwrap();
        assert_eq!(queued, 1, "email is on by default");
    }

    let settings = api
        .call(Method::GET, "/me/notification-settings", &c_token, None)
        .await;
    let listed: Vec<&str> = settings.ok()["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    for kind in [
        "result_confirmed",
        "result_annulled",
        "retest_ordered",
        "fsp_achievements",
    ] {
        assert!(listed.contains(&kind), "{kind} in {listed:?}");
    }
    api.call(
        Method::PUT,
        "/me/notification-settings",
        &a_token,
        Some(json!({ "email": [{ "kind": "result_annulled", "enabled": false }] })),
    )
    .await
    .ok();
    let s = api
        .session(
            a,
            Session {
                finished: Utc::now() - Duration::days(1),
                ..disputed(Grade::Senior, "upgrade")
            },
        )
        .await;
    api.decide(&moderator, s, "annul", "Аннулировано.")
        .await
        .ok();
    let queued: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications
         WHERE user_id = $1 AND kind = 'result_annulled' AND email_due_at IS NULL",
    )
    .bind(a)
    .fetch_one(api.pool())
    .await
    .unwrap();
    assert_eq!(queued, 1, "turned off: in the cabinet only");
}

#[sqlx::test(migrations = "../../db/migrations")]
async fn the_company_hears_outcomes_but_never_who_complained(pool: PgPool) {
    let api = Api::start(pool).await;
    let moderator = api.moderator().await;
    let (hr, company) = api.employer("hr@example.ru").await;
    let owner: Uuid = sqlx::query_scalar("SELECT owner_user_id FROM companies WHERE id = $1")
        .bind(company)
        .fetch_one(api.pool())
        .await
        .unwrap();
    sqlx::query("UPDATE users SET created_at = now() - interval '120 days' WHERE id = $1")
        .bind(owner)
        .execute(api.pool())
        .await
        .unwrap();
    let (cand, cand_token) = api.candidate("anna.reporter@example.ru").await;
    db::grades::set_candidate_category(api.pool(), "backend", "middle", 0.4, Utc::now(), cand)
        .await
        .unwrap();
    api.state.candidates.invalidate().await;
    let code: String = sqlx::query_scalar("SELECT public_code FROM candidates WHERE user_id = $1")
        .bind(cand)
        .fetch_one(api.pool())
        .await
        .unwrap();

    let vacancy = api
        .call(
            Method::POST,
            "/employer/vacancies",
            &hr,
            Some(json!({
                "title": "Go-разработчик платёжного шлюза",
                "description": "Платёжный шлюз на Go и PostgreSQL",
                "specialization": "backend",
                "grade": "middle",
                "skills": ["go"],
                "salary": { "min": 200000, "max": 260000 },
                "work_format": "remote",
                "city": null,
                "status": "published",
            })),
        )
        .await;
    let vacancy_id: Uuid = vacancy.ok()["id"].as_str().unwrap().parse().unwrap();
    let invitation = api
        .call(
            Method::POST,
            "/employer/invitations",
            &hr,
            Some(json!({
                "candidate_id": cand,
                "title": "Личное предложение для Анны",
                "message": "Здравствуйте!",
                "salary": { "min": 200000, "max": 260000 },
                "contact": "@hr",
            })),
        )
        .await;
    let invitation_id: Uuid = invitation.ok()["id"].as_str().unwrap().parse().unwrap();
    let complaint_text = "Я Анна Жалобина, +7 900 111-22-33, anna.reporter@example.ru: вилка не та";
    for (kind, target) in [
        ("invitation", invitation_id),
        ("company", company),
        ("vacancy", vacancy_id),
    ] {
        api.call(
            Method::POST,
            "/reports",
            &cand_token,
            Some(json!({
                "target_kind": kind,
                "target_id": target,
                "reason": "salary_mismatch",
                "comment": complaint_text,
            })),
        )
        .await
        .ok();
    }
    let open = api
        .call(
            Method::GET,
            "/admin/antifake?kind=complaint",
            &moderator,
            None,
        )
        .await;
    let complaint = |target: &str| -> Uuid {
        open.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["complaint"]["target_kind"] == target)
            .and_then(|i| i["id"].as_str())
            .unwrap()
            .parse()
            .unwrap()
    };

    api.act(
        &moderator,
        "complaint",
        complaint("invitation"),
        "dismiss",
        None,
    )
    .await
    .ok();
    api.act(
        &moderator,
        "complaint",
        complaint("company"),
        "dismiss",
        None,
    )
    .await
    .ok();
    api.act(
        &moderator,
        "complaint",
        complaint("vacancy"),
        "unpublish",
        None,
    )
    .await
    .ok();
    let frozen = api
        .act(&moderator, "company", company, "freeze", Some(14))
        .await;
    let until: DateTime<Utc> = frozen.ok()["frozen_until"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let written = sweep::check_trust(api.pool(), Notifier::default(), Utc::now())
        .await
        .unwrap();
    assert_eq!(written, 0, "the freeze notice already told the new quota");
    api.act(&moderator, "company", company, "restore", None)
        .await
        .ok();
    api.act(&moderator, "vacancy", vacancy_id, "restore", None)
        .await
        .ok();
    assert_eq!(
        sweep::check_trust(api.pool(), Notifier::default(), Utc::now())
            .await
            .unwrap(),
        0,
        "nor does lifting it come twice"
    );

    let feed = api.feed(&hr).await;
    assert_eq!(
        kinds(&feed),
        [
            "sanction_lifted",
            "sanction_lifted",
            "invitations_frozen",
            "vacancy_unpublished",
            "complaint_resolved",
            "complaint_resolved",
        ]
    );
    assert_eq!(feed[5]["title"], "Жалоба на приглашение не подтвердилась");
    assert_eq!(feed[4]["title"], "Жалоба на компанию не подтвердилась");
    assert!(
        text_of(&feed[4]).contains("квоту она больше не снижает. Квота — "),
        "{}",
        feed[4]
    );
    assert_eq!(
        feed[3]["title"],
        "Вакансия «Go-разработчик платёжного шлюза» снята с публикации"
    );
    assert_eq!(
        feed[3]["body"],
        "Комментарий модератора: «unpublish: проверено модератором». Вернуть вакансию может \
         только модератор. Подтверждено жалоб: 1 — они снижают квоту 90 дней."
    );
    assert_eq!(feed[3]["link"], format!("/employer/vacancies/{vacancy_id}"));
    assert_eq!(
        feed[2]["title"],
        format!("Приглашения приостановлены до {}", render::day(until))
    );
    assert!(text_of(&feed[2]).contains(
        "квота приглашений — 0. Комментарий модератора: «freeze: проверено модератором»"
    ));
    assert_eq!(feed[1]["title"], "Приглашения снова доступны");
    assert!(text_of(&feed[1]).contains("за 7 дней"));
    assert_eq!(
        feed[0]["title"],
        "Вакансия «Go-разработчик платёжного шлюза» снова опубликована"
    );

    let everything = format!(
        "{}\n{}",
        serde_json::to_string(&feed).unwrap(),
        api.letters(owner, Audience::Employer).await
    );
    for leak in [
        "anna.reporter@example.ru",
        "Анна",
        "Жалобина",
        "+7 900",
        "вилка не та",
        code.as_str(),
        &cand.to_string(),
        "Личное предложение",
        "moderator@example.ru",
    ] {
        assert!(!everything.contains(leak), "{leak} reached the company");
    }
}
