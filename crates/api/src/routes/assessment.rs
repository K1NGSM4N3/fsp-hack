//! Adaptive grade test (architecture.md §5): the candidate claims a grade, the CAT engine picks
//! parametric items, answers are checked by regenerating the instance from its `ItemRef`, and the
//! grade policy (T-5, T-6) decides which attempts are allowed. Only `ItemRef`s are stored.

use assessment::{
    ABANDON_AFTER_SECS, Answer, AttemptKind, Decision, GradePolicy, ItemRef, ItemView, Outcome,
    PolicyError, SessionState, TestPlan, Topic,
    policy::{AllowedAttempt, Attempt, CameraOrder, CameraTest, PolicyConfig},
};
use axum::{extract::State, http::HeaderMap};
use catalog::{Category, Grade, Specialization};
use chrono::{DateTime, Duration, TimeZone, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{CandidateUser, EmployerUser, client_ip, user_agent},
    dto::AssessmentStatus,
    error::{ApiError, ApiResult, InternalError, Problem},
    extract::{Created, Json, Path, Query},
    moderation::{self, ReviewDecision},
    notify::Notifier,
    proctoring::{self, ProctoringMode, ProctoringSummary},
    skills,
    state::AppState,
};

const DEADLINE_GRACE_SECS: i64 = 5;

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_sessions, start_session))
        .routes(routes!(get_session))
        .routes(routes!(current_item))
        .routes(routes!(answer))
        .routes(routes!(category))
}

pub fn catalog_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(test_plan))
}

pub fn employer_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(need_test_plan))
}

fn kind_slug(k: AttemptKind) -> &'static str {
    match k {
        AttemptKind::Placement => "placement",
        AttemptKind::PlacementStepDown => "placement_step_down",
        AttemptKind::PlacementUpgrade => "placement_upgrade",
        AttemptKind::Upgrade => "upgrade",
        AttemptKind::Downgrade => "downgrade",
        AttemptKind::Retry => "retry",
    }
}

pub(crate) fn parse_kind(s: &str) -> AttemptKind {
    match s {
        "placement_step_down" => AttemptKind::PlacementStepDown,
        "placement_upgrade" => AttemptKind::PlacementUpgrade,
        "upgrade" => AttemptKind::Upgrade,
        "downgrade" => AttemptKind::Downgrade,
        "retry" => AttemptKind::Retry,
        _ => AttemptKind::Placement,
    }
}

fn kind_ru(k: AttemptKind) -> &'static str {
    match k {
        AttemptKind::Placement => "Первичный тест",
        AttemptKind::PlacementStepDown => "Тест на грейд ниже",
        AttemptKind::PlacementUpgrade => "Дотест на грейд выше",
        AttemptKind::Upgrade => "Повышение грейда",
        AttemptKind::Downgrade => "Смена грейда на ниже",
        AttemptKind::Retry => "Повторный тест",
    }
}

fn policy(state: &AppState) -> GradePolicy {
    GradePolicy::new(PolicyConfig {
        change_cooldown_days: state.config.grade_change_cooldown_days,
        ..PolicyConfig::default()
    })
}

fn at(ts: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now)
}

/// Lazily closes an abandoned session before anything reads or extends it.
async fn expire_abandoned(db: impl sqlx::PgExecutor<'_>, candidate: Uuid) -> ApiResult<()> {
    db::assessment::expire_abandoned_sessions(db, candidate, ABANDON_AFTER_SECS as f64).await?;
    Ok(())
}

const fn corrupt(what: &'static str) -> ApiError {
    ApiError::Internal(InternalError::Corrupt(what))
}

struct History {
    attempts: Vec<Attempt>,
    /// The latest passed test measured above the claimed grade or with high confidence: only
    /// then is the immediate upgrade test offered (T-5 "справился уверенно").
    upgrade_earned: bool,
    /// Retests a moderator ordered under the camera, and the tests that may have settled them.
    camera_orders: Vec<CameraOrder>,
    camera_tests: Vec<CameraTest>,
}

impl History {
    fn camera_required(&self, spec: Specialization, grade: Grade) -> bool {
        assessment::policy::camera_required(&self.camera_orders, &self.camera_tests, spec, grade)
    }
}

/// A result a moderator voided: annulled, or annulled with a retest ordered.
fn voided(review: Option<db::types::ReviewDecision>) -> Option<Outcome> {
    match review {
        Some(db::types::ReviewDecision::Annul) => Some(Outcome::Annulled),
        Some(db::types::ReviewDecision::Retest) => Some(Outcome::RetestOrdered),
        Some(db::types::ReviewDecision::Confirm) | None => None,
    }
}

async fn history(pool: &PgPool, candidate: Uuid) -> ApiResult<History> {
    history_from(db::assessment::candidate_sessions(pool, candidate).await?)
}

/// `rows` come newest first, as `candidate_sessions` returns them.
fn history_from(rows: Vec<db::assessment::CandidateSession>) -> ApiResult<History> {
    let upgrade_earned = rows
        .iter()
        .find(|r| r.status == db::types::AssessmentStatus::Passed && voided(r.review).is_none())
        .is_some_and(|r| {
            let above = r
                .measured_grade
                .as_deref()
                .and_then(Grade::from_slug)
                .zip(Grade::from_slug(&r.claimed_grade))
                .is_some_and(|(measured, claimed)| measured > claimed);
            above || r.confident_above_next.unwrap_or(false)
        });
    let mut out = Vec::with_capacity(rows.len());
    let mut camera_orders = Vec::new();
    let mut camera_tests = Vec::with_capacity(rows.len());
    for r in rows.into_iter().rev() {
        let specialization: Specialization = r.specialization.parse()?;
        let grade: Grade = r.claimed_grade.parse()?;
        let completed = matches!(
            r.status,
            db::types::AssessmentStatus::Passed | db::types::AssessmentStatus::Failed
        );
        camera_tests.push(CameraTest {
            specialization,
            grade,
            started: r.started_at.timestamp(),
            camera: r.camera == Some(true),
            completed,
        });
        // There is no platform-fault ending: the test never waits on the camera or the upload
        // («запись прервалась» only marks the result), so `expired` always means the candidate
        // stopped answering.
        let finished = match r.status {
            db::types::AssessmentStatus::Passed => Outcome::Passed,
            db::types::AssessmentStatus::Failed => Outcome::Failed,
            db::types::AssessmentStatus::Expired => Outcome::Abandoned,
            db::types::AssessmentStatus::InProgress => continue,
        };
        // A voided result counts from the moderator's decision (assessment::policy::Outcome).
        let (outcome, at) = match (voided(r.review), r.reviewed_at) {
            (Some(outcome), Some(decided)) => (outcome, decided),
            _ => (finished, r.finished_at.unwrap_or(r.started_at)),
        };
        if outcome == Outcome::RetestOrdered {
            camera_orders.push(CameraOrder {
                specialization,
                grade,
                at: at.timestamp(),
            });
        }
        out.push(Attempt {
            specialization,
            grade,
            kind: parse_kind(&r.attempt_kind),
            outcome,
            assigned: r.assigned,
            at: at.timestamp(),
        });
    }
    Ok(History {
        attempts: out,
        upgrade_earned,
        camera_orders,
        camera_tests,
    })
}

#[derive(Serialize, ToSchema)]
pub struct Offer {
    pub grade: Grade,
    pub kind: AttemptKind,
    pub label: String,
    /// The retest a moderator ordered: open now, outside the usual windows.
    pub ordered_retest: bool,
    /// After a retest ordered with the camera: this test starts only with the camera on.
    pub camera_required: bool,
}

/// Why a test is not available yet; the same codes `POST /candidate/assessments` answers with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LockReason {
    /// 30 days after an unsuccessful (failed or abandoned) attempt at this grade, or after a
    /// failed upgrade.
    RetryBlocked,
    /// 90 days from the last grade assignment.
    GradeChangeCooldown,
}

/// A test that opens later, with the date it opens.
#[derive(Serialize, ToSchema)]
pub struct LockedOffer {
    pub grade: Grade,
    pub kind: AttemptKind,
    pub reason: LockReason,
    pub available_at: DateTime<Utc>,
    pub label: String,
    /// It will need the camera when it opens.
    pub camera_required: bool,
}

#[derive(Default)]
struct Offers {
    available: Vec<Offer>,
    locked: Vec<LockedOffer>,
}

/// The attempt kinds that could lead to `grade`, tried in order: the first allowed one wins,
/// otherwise the block that lifts earliest, otherwise the structural refusal.
fn pick(
    p: &GradePolicy,
    spec: Specialization,
    history: &History,
    grade: Grade,
    now: i64,
) -> Result<AllowedAttempt, (AttemptKind, PolicyError)> {
    if let Some(retest) = p.ordered_retest(spec, &history.attempts)
        && retest.grade == grade
    {
        return Ok(retest);
    }
    let current = p.current_grade(spec, &history.attempts);
    let mut refusal = (AttemptKind::Placement, PolicyError::NotAllowed);
    for kind in candidate_kinds(current, grade, history.upgrade_earned) {
        match p.check(spec, &history.attempts, kind, grade, now) {
            Ok(a) => return Ok(a),
            Err(e) => {
                let earlier = match (e.available_at(), refusal.1.available_at()) {
                    (Some(new), Some(old)) => new < old,
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                    (None, None) => refusal.1 == PolicyError::NotAllowed,
                };
                if earlier {
                    refusal = (kind, e);
                }
            }
        }
    }
    Err(refusal)
}

fn offers(p: &GradePolicy, spec: Specialization, history: &History, now: i64) -> Offers {
    let current = p.current_grade(spec, &history.attempts);
    let retest = p.ordered_retest(spec, &history.attempts);
    let mut out = Offers::default();
    for grade in Grade::ALL.iter().copied().filter(|g| Some(*g) != current) {
        let camera_required = history.camera_required(spec, grade);
        match pick(p, spec, history, grade, now) {
            Ok(a) => {
                let ordered_retest = retest == Some(a);
                out.available.push(Offer {
                    grade: a.grade,
                    kind: a.kind,
                    label: if ordered_retest {
                        format!("Пересдача с видеозаписью: {}", a.grade.name_ru())
                    } else {
                        format!("{}: {}", kind_ru(a.kind), a.grade.name_ru())
                    },
                    ordered_retest,
                    camera_required,
                });
            }
            Err((kind, e)) => {
                let reason = match e {
                    PolicyError::ChangeCooldown { .. } => LockReason::GradeChangeCooldown,
                    PolicyError::RetryBlocked { .. } => LockReason::RetryBlocked,
                    PolicyError::NotAllowed | PolicyError::NoAdjacentGrade => continue,
                };
                if let Some(available_at) = e.available_at() {
                    out.locked.push(LockedOffer {
                        grade,
                        kind,
                        reason,
                        available_at: at(available_at),
                        label: format!("{}: {}", kind_ru(kind), grade.name_ru()),
                        camera_required,
                    });
                }
            }
        }
    }
    out
}

fn candidate_kinds(
    current: Option<Grade>,
    requested: Grade,
    upgrade_earned: bool,
) -> Vec<AttemptKind> {
    match current {
        None => vec![
            AttemptKind::Placement,
            AttemptKind::PlacementStepDown,
            AttemptKind::Retry,
        ],
        Some(g) if requested > g && upgrade_earned => {
            vec![AttemptKind::PlacementUpgrade, AttemptKind::Upgrade]
        }
        Some(g) if requested > g => vec![AttemptKind::Upgrade],
        Some(g) if requested < g => vec![AttemptKind::Downgrade],
        Some(_) => Vec::new(),
    }
}

fn policy_problem(e: PolicyError) -> ApiError {
    match e {
        PolicyError::ChangeCooldown { available_at } => {
            ApiError::conflict("grade_change_cooldown", "Смена грейда пока недоступна").with_detail(
                format!(
                    "Следующая попытка — после {}",
                    at(available_at).format("%d.%m.%Y")
                ),
            )
        }
        PolicyError::RetryBlocked { available_at } => {
            ApiError::conflict("retry_blocked", "Повторный тест пока недоступен").with_detail(
                format!("Пересдача — после {}", at(available_at).format("%d.%m.%Y")),
            )
        }
        PolicyError::NotAllowed => {
            ApiError::conflict("attempt_not_allowed", "Такая попытка сейчас недоступна")
        }
        PolicyError::NoAdjacentGrade => ApiError::conflict(
            "attempt_not_allowed",
            "Нет соседнего грейда в этом направлении",
        ),
    }
}

fn timed_out(deadline: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now > deadline + Duration::seconds(DEADLINE_GRACE_SECS)
}

fn camera_guard(mode: ProctoringMode, camera: bool, required: bool) -> ApiResult<()> {
    if mode == ProctoringMode::Optional && !camera && required {
        return Err(
            ApiError::conflict("camera_required", "Этот тест нужно пройти с видеозаписью")
                .with_detail(
                    "Модератор назначил пересдачу с видеозаписью: включите камеру на шаге перед тестом",
                ),
        );
    }
    Ok(())
}

#[derive(Serialize, ToSchema)]
pub struct CurrentItem {
    /// Number of already answered items; send it back with the answer.
    pub seq: u16,
    pub item: ItemView,
    pub deadline: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct TopicResult {
    pub topic: Topic,
    pub topic_ru: String,
    pub answered: u16,
    pub correct: u16,
}

#[derive(Serialize, ToSchema)]
pub struct AssessmentResult {
    pub decision: Decision,
    /// Position of θ inside the measured grade band, 0..1.
    pub test_strength: Option<f32>,
    pub assigned_grade: Option<Grade>,
    /// «Требует подтверждения»: an aberrant answer pattern (person-fit) together with integrity
    /// events — leaving the test or pasting. Sessions without proctoring data fall back to the
    /// person-fit flag alone.
    pub needs_review: bool,
    pub topics: Vec<TopicResult>,
    /// What the candidate may do next right away (step down after a fail, upgrade after a confident pass).
    pub offers: Vec<Offer>,
    /// What opens later and when: e.g. the up-test to a grade failed or abandoned less than
    /// 30 days ago.
    pub locked: Vec<LockedOffer>,
}

/// A moderator's decision on the session, as the candidate reads it.
#[derive(Serialize, ToSchema)]
pub struct SessionModeration {
    pub decision: ReviewDecision,
    /// Written by the moderator for the candidate.
    pub comment: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct AssessmentSession {
    pub id: Uuid,
    pub specialization: Specialization,
    pub claimed_grade: Grade,
    pub attempt_kind: AttemptKind,
    pub status: AssessmentStatus,
    pub answered: u16,
    pub min_items: u16,
    pub max_items: u16,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub current: Option<CurrentItem>,
    pub result: Option<AssessmentResult>,
    /// `None` when the session ran with proctoring off.
    pub proctoring: Option<ProctoringSummary>,
    /// The moderator's decision on a disputed result: confirmed, annulled (the category it gave
    /// is removed), or annulled with a retest under the camera.
    pub review: Option<SessionModeration>,
    /// Every time a moderator played this recording. Each view is logged, and the candidate
    /// sees the same dates.
    pub recording_views: Vec<DateTime<Utc>>,
}

fn load_state(v: serde_json::Value) -> ApiResult<SessionState> {
    Ok(serde_json::from_value(v)?)
}

fn topic_results(state: &SessionState) -> Vec<TopicResult> {
    let mut out: Vec<TopicResult> = Vec::new();
    for a in &state.administered {
        match out.iter_mut().find(|t| t.topic == a.topic) {
            Some(t) => {
                t.answered += 1;
                t.correct += u16::from(a.correct);
            }
            None => out.push(TopicResult {
                topic: a.topic,
                topic_ru: a.topic.name_ru().to_owned(),
                answered: 1,
                correct: u16::from(a.correct),
            }),
        }
    }
    out
}

fn render_current(item: &ItemRef, seq: u16, deadline: DateTime<Utc>) -> ApiResult<CurrentItem> {
    Ok(CurrentItem {
        seq,
        item: assessment::render(item)?,
        deadline,
    })
}

fn deadline_for(item: &ItemRef, now: DateTime<Utc>) -> ApiResult<DateTime<Utc>> {
    let view = assessment::render(item)?;
    Ok(now + Duration::seconds(i64::from(view.time_limit_secs)))
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"specialization": "backend", "grade": "middle", "proctoring": {"camera": true, "consent_document_id": "0191f000-0000-7000-8000-000000000004"}}))]
pub struct StartInput {
    pub specialization: Specialization,
    pub grade: Grade,
    /// Choice made on the proctoring step. Omitted while proctoring is optional, the test runs
    /// without video and is marked so — unless a moderator ordered a retest with the camera
    /// (409 `camera_required`).
    #[serde(default)]
    pub proctoring: Option<ProctoringChoice>,
}

#[derive(Deserialize, ToSchema)]
pub struct ProctoringChoice {
    pub camera: bool,
    /// `consent.id` from `GET /candidate/proctoring`; required with `camera = true`.
    pub consent_document_id: Option<Uuid>,
}

/// Start a test for the claimed grade. The policy decides whether it is a placement, a step down,
/// an upgrade or a retry; cooldowns return 409 with the date.
#[utoipa::path(post, path = "/assessments", tag = "assessment",
    request_body = StartInput,
    responses(
        (status = 201, body = AssessmentSession, headers(("Location" = String, description = "`/api/v1/candidate/assessments/{id}`"))),
        (status = 409, description = "`assessment_in_progress`, `survey_required`, `grade_change_cooldown`, `retry_blocked` (30 days after a failed or abandoned attempt at that grade, or after a moderator annulled a result at it), `attempt_not_allowed`, `already_assigned`, `camera_required` (a moderator ordered a retest with the camera: a test at that grade or above starts only with `proctoring.camera = true`)", body = Problem),
        (status = 422, description = "`validation_failed`: `specialization` is not the one from the survey or the current category; `proctoring.consent_document_id` missing or not the current version", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn start_session(
    State(state): State<AppState>,
    user: CandidateUser,
    headers: HeaderMap,
    Json(input): Json<StartInput>,
) -> ApiResult<Created<AssessmentSession>> {
    expire_abandoned(&state.pool, user.id).await?;
    if let Some(active) = db::assessment::active_session(&state.pool, user.id).await? {
        return Err(
            ApiError::conflict("assessment_in_progress", "Сначала завершите начатый тест")
                .with_detail(active.to_string()),
        );
    }
    let candidate = db::candidates::candidate_invitable(&state.pool, user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let survey = db::survey::latest_survey(&state.pool, user.id).await?;
    let home_spec = match (&candidate.specialization, &survey) {
        (Some(s), _) => s.parse::<Specialization>()?,
        (None, Some(s)) => s.specialization.parse::<Specialization>()?,
        (None, None) => {
            return Err(ApiError::conflict(
                "survey_required",
                "Сначала пройдите опрос",
            ));
        }
    };
    if input.specialization != home_spec {
        return Err(ApiError::field(
            "specialization",
            "Тест доступен по специализации из опроса или текущей категории",
        ));
    }

    let now = Utc::now();
    let history = history(&state.pool, user.id).await?;
    let p = policy(&state);
    let current = p.current_grade(input.specialization, &history.attempts);
    if current == Some(input.grade) {
        return Err(ApiError::conflict(
            "already_assigned",
            "Этот грейд уже подтверждён",
        ));
    }
    let allowed = pick(
        &p,
        input.specialization,
        &history,
        input.grade,
        now.timestamp(),
    )
    .map_err(|(_, e)| policy_problem(e))?;

    let camera = input.proctoring.as_ref().is_some_and(|p| p.camera);
    camera_guard(
        state.config.proctoring.mode,
        camera,
        history.camera_required(input.specialization, input.grade),
    )?;
    let consent_doc = match state.config.proctoring.mode {
        ProctoringMode::Optional if camera => {
            let doc = db::consents::current_document(&state.pool, proctoring::CONSENT_KIND)
                .await?
                .ok_or_else(|| corrupt("proctoring consent document is missing"))?;
            let confirmed = input
                .proctoring
                .as_ref()
                .and_then(|p| p.consent_document_id);
            if confirmed != Some(doc.id) {
                return Err(ApiError::field(
                    "proctoring.consent_document_id",
                    "Подтвердите согласие на видеозапись",
                ));
            }
            Some(doc.id)
        }
        _ => None,
    };

    let focus: Vec<String> = survey
        .map(|s| s.skills)
        .unwrap_or_default()
        .into_iter()
        .filter(|s| assessment::items::SKILLS.contains(&s.as_str()))
        .collect();
    let seed = rand::rng().next_u64();
    let session_state = assessment::start(input.specialization, input.grade, seed, focus);
    let first = assessment::next_item(&session_state).ok_or_else(|| corrupt("empty item bank"))?;
    let deadline = deadline_for(&first, now)?;
    let id = Uuid::now_v7();
    let mut tx = state.pool.begin().await?;
    db::assessment::insert_session(
        &mut *tx,
        &db::assessment::NewSession {
            id,
            candidate_id: user.id,
            specialization: input.specialization.slug(),
            claimed_grade: input.grade.slug(),
            attempt_kind: kind_slug(allowed.kind),
            state: &serde_json::to_value(&session_state)?,
            current_item: &serde_json::to_value(&first)?,
            deadline,
            shown_at: now,
        },
    )
    .await
    .map_err(|e| {
        // A double click races past the check above; the one-active-session index decides.
        if db::is_unique_violation(&e) {
            ApiError::conflict("assessment_in_progress", "Сначала завершите начатый тест")
        } else {
            e.into()
        }
    })?;
    if state.config.proctoring.mode == ProctoringMode::Optional {
        let consent_id = match consent_doc {
            Some(doc) => Some(
                match db::consents::active_grant(&mut *tx, user.id, doc).await? {
                    Some(grant) => grant,
                    None => {
                        let grant = Uuid::now_v7();
                        db::consents::insert_user_consent(
                            &mut *tx,
                            grant,
                            user.id,
                            doc,
                            client_ip(&headers).as_deref(),
                            user_agent(&headers).as_deref(),
                        )
                        .await?;
                        grant
                    }
                },
            ),
            None => None,
        };
        db::proctoring::insert_proctoring(
            &mut *tx,
            id,
            user.id,
            consent_id.is_some(),
            consent_id,
            now,
        )
        .await?;
    }
    tx.commit().await?;
    Ok(Created(
        format!("/api/v1/candidate/assessments/{id}"),
        view(&state, user.id, id).await?,
    ))
}

/// Records one outcome and either schedules the next item or finishes the session
/// (assigning the grade and test-confirmed skills on a pass). `Some(passed)` when it finished.
async fn advance(
    tx: &mut PgConnection,
    candidate: Uuid,
    row: &db::assessment::SessionForUpdate,
    session: &mut SessionState,
    item: ItemRef,
    answer: Option<&Answer>,
    now: DateTime<Utc>,
) -> ApiResult<Option<bool>> {
    let timed_out = timed_out(row.current_item_deadline.unwrap_or(now), now);
    let correct = !timed_out && answer.is_some_and(|a| assessment::check(&item, a));
    let shown = row.current_item_shown_at.unwrap_or(now);
    let elapsed_ms = (now - shown)
        .num_milliseconds()
        .clamp(0, i64::from(u32::MAX)) as u32;
    assessment::record(session, item.clone(), correct, elapsed_ms)?;
    let seq = session.administered.len() as i32 - 1;
    db::assessment::insert_response(
        &mut *tx,
        &db::assessment::NewResponse {
            session_id: row.id,
            seq,
            template_id: &item.template_id,
            level: i16::from(item.level),
            seed: item.seed as i64,
            answer: answer.map(serde_json::to_value).transpose()?.as_ref(),
            correct,
            timed_out,
            elapsed_ms: elapsed_ms as i32,
            theta_after: session.summary.theta,
            se_after: session.summary.se,
            answered_at: now,
        },
    )
    .await?;

    let items_count = session.administered.len() as i32;
    if let Some(d) = assessment::decision(session) {
        let status = if d.passed {
            db::types::AssessmentStatus::Passed
        } else {
            db::types::AssessmentStatus::Failed
        };
        db::assessment::finish_session(
            &mut *tx,
            row.id,
            &db::assessment::FinishedSession {
                status,
                state: &serde_json::to_value(&*session)?,
                theta: d.theta,
                se: d.se,
                measured_grade: d.grade_measured.map(Grade::slug),
                confident_above_next: d.confident_above_next,
                person_fit_lz: d.person_fit_lz,
                aberrant: d.aberrant,
                items_count,
                finished_at: now,
            },
        )
        .await?;
        if d.passed {
            let kind = parse_kind(&row.attempt_kind);
            let reason = match kind {
                AttemptKind::Upgrade | AttemptKind::PlacementUpgrade => "upgrade",
                AttemptKind::Downgrade => "downgrade",
                _ => "placement",
            };
            db::grades::insert_grade_assignment(
                &mut *tx,
                &db::grades::NewGradeAssignment {
                    id: Uuid::now_v7(),
                    candidate_id: candidate,
                    specialization: &row.specialization,
                    grade: &row.claimed_grade,
                    theta: d.theta,
                    se: d.se,
                    session_id: Some(row.id),
                    reason,
                    assigned_at: now,
                },
            )
            .await?;
            db::grades::set_candidate_category(
                &mut *tx,
                &row.specialization,
                &row.claimed_grade,
                d.theta,
                now,
                candidate,
            )
            .await?;
            let bank = assessment::bank();
            for a in session.administered.iter().filter(|a| a.correct) {
                if let Some(t) = bank.template(&a.item.template_id) {
                    for skill in t.skills.iter().filter(|s| skills::is_known(s)) {
                        db::candidates::insert_candidate_skill(&mut *tx, candidate, skill, "test")
                            .await?;
                    }
                }
            }
        }
        return Ok(Some(d.passed));
    }
    let next =
        assessment::next_item(session).ok_or_else(|| corrupt("no next item before decision"))?;
    let next_deadline = deadline_for(&next, now)?;
    db::assessment::update_session_progress(
        &mut *tx,
        &serde_json::to_value(&*session)?,
        &serde_json::to_value(&next)?,
        next_deadline,
        now,
        items_count,
        row.id,
    )
    .await?;
    Ok(None)
}

/// «Категория присвоена» or «тест не пройден», in the transaction that finished the session.
async fn notify_finished(
    state: &AppState,
    tx: &mut PgConnection,
    candidate: Uuid,
    row: &db::assessment::SessionForUpdate,
    finished: Option<bool>,
) -> ApiResult<()> {
    if let Some(passed) = finished {
        let category = Category::new(row.specialization.parse()?, row.claimed_grade.parse()?);
        Notifier::new(state)
            .assessment(&mut *tx, candidate, row.id, category, passed)
            .await?;
    }
    Ok(())
}

async fn view(state: &AppState, candidate: Uuid, id: Uuid) -> ApiResult<AssessmentSession> {
    let r = db::assessment::session_state(&state.pool, id, candidate)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let s = load_state(r.state)?;
    let spec: Specialization = r.specialization.parse()?;
    let claimed: Grade = r.claimed_grade.parse()?;
    let status = AssessmentStatus::from(r.status);
    let current = match (status, r.current_item, r.current_item_deadline) {
        (AssessmentStatus::InProgress, Some(item), Some(deadline)) => {
            let item: ItemRef = serde_json::from_value(item)?;
            Some(render_current(
                &item,
                s.administered.len() as u16,
                deadline,
            )?)
        }
        _ => None,
    };
    let proctoring = match db::proctoring::summary(&state.pool, id, Some(candidate)).await? {
        Some(p) => {
            let facts = proctoring::facts(&p, status == AssessmentStatus::InProgress);
            Some((
                proctoring::summary_view(&p, &facts, state.config.proctoring.retention_days),
                facts.counts,
            ))
        }
        None => None,
    };
    let result = match (status, r.theta, r.se) {
        (AssessmentStatus::Passed | AssessmentStatus::Failed, Some(theta), Some(se)) => {
            let grade_measured = r.measured_grade.as_deref().and_then(Grade::from_slug);
            let decision = Decision {
                passed: status == AssessmentStatus::Passed,
                theta,
                se,
                grade_measured,
                confident_above_next: r.confident_above_next.unwrap_or(false),
                person_fit_lz: r.person_fit_lz.unwrap_or(0.0),
                aberrant: r.aberrant.unwrap_or(false),
                items_administered: s.administered.len() as u16,
            };
            let history = history(&state.pool, candidate).await?;
            let next = offers(&policy(state), spec, &history, Utc::now().timestamp());
            let flagged = moderation::flagged(
                decision.aberrant,
                proctoring.as_ref().map(|(_, counts)| counts),
            );
            Some(AssessmentResult {
                test_strength: Some(matching::test_strength(claimed, theta as f32)),
                assigned_grade: r.assigned.then_some(claimed),
                needs_review: flagged && r.review.is_none(),
                topics: topic_results(&s),
                offers: next.available,
                locked: next.locked,
                decision,
            })
        }
        _ => None,
    };
    let recording_views = match &proctoring {
        Some(_) => db::moderation::recording_views(&state.pool, id).await?,
        None => Vec::new(),
    };
    Ok(AssessmentSession {
        id: r.id,
        specialization: spec,
        claimed_grade: claimed,
        attempt_kind: parse_kind(&r.attempt_kind),
        status,
        answered: s.administered.len() as u16,
        min_items: s.config.min_items,
        max_items: s.config.max_items,
        started_at: Some(r.started_at),
        finished_at: r.finished_at,
        current,
        result,
        proctoring: proctoring.map(|(summary, _)| summary),
        review: match (r.review, r.review_comment, r.reviewed_at) {
            (Some(d), Some(comment), Some(decided_at)) => Some(SessionModeration {
                decision: d.into(),
                comment,
                decided_at,
            }),
            _ => None,
        },
        recording_views,
    })
}

/// Test history
#[utoipa::path(get, path = "/assessments", tag = "assessment",
    responses((status = 200, body = Vec<AssessmentSession>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_sessions(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Vec<AssessmentSession>>> {
    expire_abandoned(&state.pool, user.id).await?;
    let ids: Vec<Uuid> = db::assessment::candidate_sessions(&state.pool, user.id)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect();
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(view(&state, user.id, id).await?);
    }
    Ok(Json(out))
}

/// Session state or result
#[utoipa::path(get, path = "/assessments/{id}", tag = "assessment",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = AssessmentSession), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_session(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AssessmentSession>> {
    Ok(Json(view(&state, user.id, id).await?))
}

/// Current item without the answer key
///
/// An item past its deadline is recorded as wrong and the next one is returned; a session left
/// 30 minutes past the deadline is closed as `expired` instead (an unsuccessful attempt).
#[utoipa::path(get, path = "/assessments/{id}/current-item", tag = "assessment",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = AssessmentSession), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn current_item(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AssessmentSession>> {
    let mut tx = state.pool.begin().await?;
    expire_abandoned(&mut *tx, user.id).await?;
    let row = db::assessment::session_for_update(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let now = Utc::now();
    let mut graded = false;
    if row.status == db::types::AssessmentStatus::InProgress
        && let (Some(item), Some(deadline)) = (&row.current_item, row.current_item_deadline)
        && timed_out(deadline, now)
    {
        let mut session = load_state(row.state.clone())?;
        let item: ItemRef = serde_json::from_value(item.clone())?;
        let finished = advance(&mut tx, user.id, &row, &mut session, item, None, now).await?;
        notify_finished(&state, &mut tx, user.id, &row, finished).await?;
        graded = finished == Some(true);
    }
    tx.commit().await?;
    if graded {
        state.candidates.invalidate().await;
    }
    Ok(Json(view(&state, user.id, id).await?))
}

#[derive(Deserialize, ToSchema)]
pub struct AnswerInput {
    /// `current.seq` of the item being answered (protects against double submits).
    pub seq: u16,
    pub answer: Answer,
}

/// Submit an answer; returns the next item or the result. Correctness is not revealed per item.
#[utoipa::path(post, path = "/assessments/{id}/answer", tag = "assessment",
    params(("id" = Uuid, Path)),
    request_body = AnswerInput,
    responses(
        (status = 200, body = AssessmentSession),
        (status = 404, body = Problem),
        (status = 409, description = "`assessment_finished` (also when the session was left 30 minutes past the item's deadline and has just been closed as abandoned), `stale_item`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn answer(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
    Json(input): Json<AnswerInput>,
) -> ApiResult<Json<AssessmentSession>> {
    let mut tx = state.pool.begin().await?;
    expire_abandoned(&mut *tx, user.id).await?;
    let row = db::assessment::session_for_update(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if row.status != db::types::AssessmentStatus::InProgress {
        tx.commit().await?;
        return Err(ApiError::conflict(
            "assessment_finished",
            "Тест уже завершён",
        ));
    }
    let mut session = load_state(row.state.clone())?;
    if usize::from(input.seq) != session.administered.len() {
        return Err(ApiError::conflict(
            "stale_item",
            "Это задание уже засчитано, обновите страницу",
        ));
    }
    let item: ItemRef = row
        .current_item
        .clone()
        .map(serde_json::from_value)
        .transpose()?
        .ok_or_else(|| corrupt("no current item"))?;
    let finished = advance(
        &mut tx,
        user.id,
        &row,
        &mut session,
        item,
        Some(&input.answer),
        Utc::now(),
    )
    .await?;
    notify_finished(&state, &mut tx, user.id, &row, finished).await?;
    db::candidates::touch_candidate_activity(&mut *tx, user.id).await?;
    tx.commit().await?;
    if finished == Some(true) {
        state.candidates.invalidate().await;
    }
    Ok(Json(view(&state, user.id, id).await?))
}

#[derive(Serialize, ToSchema)]
pub struct GradeHistoryEntry {
    pub specialization: Specialization,
    pub grade: Grade,
    pub reason: String,
    pub theta: f64,
    pub session_id: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    /// A moderator annulled the test behind it: the entry stays in the history, the category
    /// no longer comes from it.
    pub annulled_at: Option<DateTime<Utc>>,
}

pub(crate) async fn grade_history_views(
    pool: &PgPool,
    candidate: Uuid,
) -> ApiResult<Vec<GradeHistoryEntry>> {
    let grades = db::grades::grade_history(pool, candidate).await?;
    let mut out = Vec::with_capacity(grades.len());
    for g in grades {
        out.push(GradeHistoryEntry {
            specialization: g.specialization.parse()?,
            grade: g.grade.parse()?,
            reason: g.reason,
            theta: g.theta,
            session_id: g.session_id,
            assigned_at: g.assigned_at,
            annulled_at: g.annulled_at,
        });
    }
    Ok(out)
}

#[derive(Serialize, ToSchema)]
pub struct CategoryStatus {
    pub category: Option<Category>,
    pub category_name: Option<String>,
    pub theta: Option<f64>,
    pub test_strength: Option<f32>,
    pub assigned_at: Option<DateTime<Utc>>,
    /// Specialization the next test will be in (current category or the survey).
    pub specialization: Option<Specialization>,
    pub claimed_grade_in_survey: Option<Grade>,
    pub available: Vec<Offer>,
    /// Tests that open later, one per grade, with the date.
    pub locked: Vec<LockedOffer>,
    /// When a grade change becomes possible, if no change is possible now.
    pub next_change_at: Option<DateTime<Utc>>,
    pub history: Vec<GradeHistoryEntry>,
}

/// Current category, grade history and which grade tests are available now (T-5, T-6, C-4)
#[utoipa::path(get, path = "/category", tag = "assessment",
    responses((status = 200, body = CategoryStatus)),
    security(("cookie" = []), ("bearer" = [])))]
async fn category(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<CategoryStatus>> {
    expire_abandoned(&state.pool, user.id).await?;
    let c = db::candidates::candidate_profile(&state.pool, user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let survey = db::survey::latest_survey(&state.pool, user.id).await?;
    let category = match (c.specialization.as_deref(), c.grade.as_deref()) {
        (Some(s), Some(g)) => Some(Category::new(s.parse()?, g.parse()?)),
        _ => None,
    };
    let spec = category
        .map(|c| c.specialization)
        .or_else(|| survey.as_ref().and_then(|s| s.specialization.parse().ok()));
    let now = Utc::now().timestamp();
    let history = history(&state.pool, user.id).await?;
    let p = policy(&state);
    let (offers, next_change_at) = match spec {
        Some(spec) => {
            let offers = offers(&p, spec, &history, now);
            let next = category.and_then(|c| {
                let mut earliest: Option<i64> = None;
                for g in [c.grade.next(), c.grade.prev()].into_iter().flatten() {
                    let kind = if g > c.grade {
                        AttemptKind::Upgrade
                    } else {
                        AttemptKind::Downgrade
                    };
                    match p.check(spec, &history.attempts, kind, g, now) {
                        Ok(_) => return None,
                        Err(e) => {
                            if let Some(t) = e.available_at() {
                                earliest = Some(earliest.map_or(t, |x| x.min(t)));
                            }
                        }
                    }
                }
                earliest.map(at)
            });
            (offers, next)
        }
        None => (Offers::default(), None),
    };
    let history_out = grade_history_views(&state.pool, user.id).await?;
    Ok(Json(CategoryStatus {
        category_name: category.map(Category::name_ru),
        test_strength: match (category, c.theta) {
            (Some(cat), Some(t)) => Some(matching::test_strength(cat.grade, t as f32)),
            _ => None,
        },
        category,
        theta: c.theta,
        assigned_at: c.category_assigned_at,
        specialization: spec,
        claimed_grade_in_survey: survey.and_then(|s| s.claimed_grade.parse().ok()),
        available: offers.available,
        locked: offers.locked,
        next_change_at,
        history: history_out,
    }))
}

#[derive(Deserialize, IntoParams)]
pub struct PlanQuery {
    pub specialization: Specialization,
    pub grade: Grade,
    /// Comma-separated stack from the vacancy (aliases accepted).
    pub skills: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SampleCandidate {
    pub label: String,
    pub items: Vec<ItemView>,
}

#[derive(Serialize, ToSchema)]
pub struct TestPlanDemo {
    pub plan: TestPlan,
    /// The same plan rendered for two candidates: same templates and difficulty, different
    /// numbers, code and answers — a leaked instance does not help the next candidate.
    pub samples: Vec<SampleCandidate>,
}

fn plan_demo(spec: Specialization, grade: Grade, skills: &[String]) -> TestPlanDemo {
    let focus: Vec<String> = skills
        .iter()
        .filter(|s| assessment::items::SKILLS.contains(&s.as_str()))
        .cloned()
        .collect();
    let seed = rand::rng().next_u64();
    TestPlanDemo {
        plan: assessment::plan_for(spec, grade, &focus),
        samples: ["Кандидат А", "Кандидат Б"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| SampleCandidate {
                label: label.into(),
                items: assessment::sample_items(spec, grade, &focus, 3, seed ^ (i as u64 + 1)),
            })
            .collect(),
    }
}

/// How a test is built for a vacancy (T-4): topics, prioritized templates and sample items
#[utoipa::path(get, path = "/catalog/test-plan", tag = "assessment",
    params(PlanQuery),
    responses((status = 200, body = TestPlanDemo), (status = 422, body = Problem)))]
async fn test_plan(Query(q): Query<PlanQuery>) -> ApiResult<Json<TestPlanDemo>> {
    let raw: Vec<String> = q
        .skills
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let skills = skills::canonicalize(&raw)
        .map_err(|u| ApiError::field("skills", skills::unknown_message(&u)))?;
    Ok(Json(plan_demo(q.specialization, q.grade, &skills)))
}

/// Test plan derived from a need: the stack of the vacancy steers which templates the test favours
#[utoipa::path(get, path = "/needs/{id}/test-plan", tag = "assessment",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = TestPlanDemo), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn need_test_plan(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TestPlanDemo>> {
    let company = super::company::company_id(&state, user.id).await?;
    let need = super::needs::effective_need(&state.pool, company, id).await?;
    let grade = need.grades.first().copied().unwrap_or(Grade::Middle);
    let skills: Vec<String> = need
        .must_skills
        .iter()
        .chain(&need.nice_skills)
        .cloned()
        .collect();
    Ok(Json(plan_demo(need.specialization, grade, &skills)))
}

#[cfg(test)]
mod tests {
    use catalog::Specialization::Backend;
    use db::{
        assessment::CandidateSession,
        types::{AssessmentStatus as Status, ReviewDecision as Review},
    };

    use super::*;

    const DAY: i64 = 86_400;
    const HOUR: i64 = 3_600;
    const T0: i64 = 1_790_000_000;

    fn session(grade: Grade, kind: &str, status: Status, finished: i64) -> CandidateSession {
        CandidateSession {
            id: Uuid::nil(),
            specialization: Backend.slug().into(),
            claimed_grade: grade.slug().into(),
            attempt_kind: kind.into(),
            status,
            theta: None,
            se: None,
            measured_grade: None,
            confident_above_next: None,
            person_fit_lz: None,
            aberrant: None,
            items_count: 12,
            started_at: at(finished - HOUR),
            finished_at: (status != Status::InProgress).then(|| at(finished)),
            assigned: status == Status::Passed,
            camera: None,
            review: None,
            reviewed_at: None,
        }
    }

    fn confident(s: CandidateSession) -> CandidateSession {
        CandidateSession {
            confident_above_next: Some(true),
            ..s
        }
    }

    fn reviewed(s: CandidateSession, review: Review, decided: i64) -> CandidateSession {
        CandidateSession {
            assigned: s.assigned && review == Review::Confirm,
            review: Some(review),
            reviewed_at: Some(at(decided)),
            ..s
        }
    }

    /// Takes the sessions in the order they happened; the query hands them over newest first.
    fn history_of(mut sessions: Vec<CandidateSession>) -> History {
        sessions.reverse();
        history_from(sessions).unwrap()
    }

    fn attempt(
        grade: Grade,
        kind: AttemptKind,
        outcome: Outcome,
        assigned: bool,
        at: i64,
    ) -> Attempt {
        Attempt {
            specialization: Backend,
            grade,
            kind,
            outcome,
            assigned,
            at,
        }
    }

    #[test]
    fn finished_sessions_become_policy_outcomes_and_running_ones_are_skipped() {
        let h = history_of(vec![
            session(Grade::Junior, "placement", Status::Passed, T0),
            session(Grade::Middle, "upgrade", Status::Failed, T0 + DAY),
            session(Grade::Middle, "retry", Status::Expired, T0 + 2 * DAY),
            session(Grade::Senior, "upgrade", Status::InProgress, T0 + 3 * DAY),
        ]);
        assert_eq!(
            h.attempts,
            [
                attempt(
                    Grade::Junior,
                    AttemptKind::Placement,
                    Outcome::Passed,
                    true,
                    T0
                ),
                attempt(
                    Grade::Middle,
                    AttemptKind::Upgrade,
                    Outcome::Failed,
                    false,
                    T0 + DAY
                ),
                attempt(
                    Grade::Middle,
                    AttemptKind::Retry,
                    Outcome::Abandoned,
                    false,
                    T0 + 2 * DAY
                ),
            ]
        );
        assert!(h.camera_orders.is_empty());
    }

    #[test]
    fn an_abandoned_session_without_an_end_counts_from_its_start() {
        let s = CandidateSession {
            finished_at: None,
            ..session(Grade::Middle, "placement", Status::Expired, T0)
        };
        let started = s.started_at.timestamp();
        assert_eq!(history_of(vec![s]).attempts[0].at, started);
    }

    #[test]
    fn a_voided_result_counts_from_the_moderators_decision() {
        let finished = T0;
        let decided = T0 + 40 * DAY;
        let passed = || session(Grade::Middle, "placement", Status::Passed, finished);

        let h = history_of(vec![reviewed(passed(), Review::Annul, decided)]);
        assert_eq!(
            h.attempts,
            [attempt(
                Grade::Middle,
                AttemptKind::Placement,
                Outcome::Annulled,
                false,
                decided
            )]
        );
        assert!(h.camera_orders.is_empty());

        let h = history_of(vec![reviewed(passed(), Review::Retest, decided)]);
        assert_eq!(h.attempts[0].outcome, Outcome::RetestOrdered);
        assert_eq!(h.attempts[0].at, decided);
        assert_eq!(
            h.camera_orders,
            [CameraOrder {
                specialization: Backend,
                grade: Grade::Middle,
                at: decided
            }]
        );

        let h = history_of(vec![reviewed(passed(), Review::Confirm, decided)]);
        assert_eq!(
            h.attempts,
            [attempt(
                Grade::Middle,
                AttemptKind::Placement,
                Outcome::Passed,
                true,
                finished
            )],
            "a confirmed result stays as it was"
        );
        assert!(h.camera_orders.is_empty());
    }

    #[test]
    fn the_upgrade_is_earned_by_the_latest_valid_pass_above_the_claim() {
        let pass = |grade, at| session(grade, "placement", Status::Passed, at);
        let measured_above = CandidateSession {
            measured_grade: Some("senior".into()),
            ..pass(Grade::Middle, T0)
        };
        assert!(history_of(vec![measured_above]).upgrade_earned);
        assert!(history_of(vec![confident(pass(Grade::Middle, T0))]).upgrade_earned);
        let measured_at_claim = CandidateSession {
            measured_grade: Some("middle".into()),
            confident_above_next: Some(false),
            ..pass(Grade::Middle, T0)
        };
        assert!(!history_of(vec![measured_at_claim]).upgrade_earned);
        assert!(
            history_of(vec![
                confident(pass(Grade::Junior, T0)),
                session(
                    Grade::Middle,
                    "placement_upgrade",
                    Status::Failed,
                    T0 + HOUR
                ),
            ])
            .upgrade_earned,
            "a failure is not a pass: the confident pass before it still counts"
        );
        assert!(
            !history_of(vec![
                confident(pass(Grade::Junior, T0)),
                pass(Grade::Middle, T0 + 100 * DAY),
            ])
            .upgrade_earned,
            "only the latest pass counts"
        );
        assert!(
            !history_of(vec![
                pass(Grade::Junior, T0),
                reviewed(
                    confident(pass(Grade::Middle, T0 + 100 * DAY)),
                    Review::Annul,
                    T0 + 101 * DAY
                ),
            ])
            .upgrade_earned,
            "a voided pass earns nothing"
        );
    }

    #[test]
    fn every_session_is_a_camera_test_completed_only_with_a_result() {
        let with_camera = |s: CandidateSession, camera| CandidateSession { camera, ..s };
        let h = history_of(vec![
            with_camera(
                session(Grade::Junior, "placement", Status::Passed, T0),
                Some(true),
            ),
            with_camera(
                session(Grade::Middle, "upgrade", Status::Failed, T0 + DAY),
                Some(false),
            ),
            with_camera(
                session(Grade::Middle, "upgrade", Status::Expired, T0 + 2 * DAY),
                Some(true),
            ),
            session(Grade::Senior, "upgrade", Status::InProgress, T0 + 3 * DAY),
        ]);
        let test = |grade, finished: i64, camera, completed| CameraTest {
            specialization: Backend,
            grade,
            started: finished - HOUR,
            camera,
            completed,
        };
        assert_eq!(
            h.camera_tests,
            [
                test(Grade::Junior, T0, true, true),
                test(Grade::Middle, T0 + DAY, false, true),
                test(Grade::Middle, T0 + 2 * DAY, true, false),
                test(Grade::Senior, T0 + 3 * DAY, false, false),
            ]
        );
    }

    fn open(o: &Offers) -> Vec<(Grade, AttemptKind, &str, bool, bool)> {
        o.available
            .iter()
            .map(|a| {
                (
                    a.grade,
                    a.kind,
                    a.label.as_str(),
                    a.ordered_retest,
                    a.camera_required,
                )
            })
            .collect()
    }

    fn locked(o: &Offers) -> Vec<(Grade, AttemptKind, LockReason, i64, &str, bool)> {
        o.locked
            .iter()
            .map(|l| {
                (
                    l.grade,
                    l.kind,
                    l.reason,
                    l.available_at.timestamp(),
                    l.label.as_str(),
                    l.camera_required,
                )
            })
            .collect()
    }

    #[test]
    fn a_new_candidate_may_take_the_placement_at_any_grade() {
        let o = offers(&GradePolicy::default(), Backend, &history_of(vec![]), T0);
        assert_eq!(
            open(&o),
            [
                (
                    Grade::Junior,
                    AttemptKind::Placement,
                    "Первичный тест: Junior",
                    false,
                    false
                ),
                (
                    Grade::Middle,
                    AttemptKind::Placement,
                    "Первичный тест: Middle",
                    false,
                    false
                ),
                (
                    Grade::Senior,
                    AttemptKind::Placement,
                    "Первичный тест: Senior",
                    false,
                    false
                ),
            ]
        );
        assert!(o.locked.is_empty());
    }

    #[test]
    fn an_abandoned_placement_leaves_only_the_step_down_open() {
        let closed = T0;
        let h = history_of(vec![session(
            Grade::Middle,
            "placement",
            Status::Expired,
            closed,
        )]);
        let o = offers(&GradePolicy::default(), Backend, &h, closed + HOUR);
        assert_eq!(
            open(&o),
            [(
                Grade::Junior,
                AttemptKind::PlacementStepDown,
                "Тест на грейд ниже: Junior",
                false,
                false
            )]
        );
        let reopens = closed + 30 * DAY;
        assert_eq!(
            locked(&o),
            [
                (
                    Grade::Middle,
                    AttemptKind::Placement,
                    LockReason::RetryBlocked,
                    reopens,
                    "Первичный тест: Middle",
                    false
                ),
                (
                    Grade::Senior,
                    AttemptKind::Placement,
                    LockReason::RetryBlocked,
                    reopens,
                    "Первичный тест: Senior",
                    false
                ),
            ]
        );
    }

    #[test]
    fn the_up_test_after_a_step_down_waits_out_the_failure_at_that_grade() {
        let failed = T0;
        let h = history_of(vec![
            session(Grade::Middle, "placement", Status::Failed, failed),
            confident(session(
                Grade::Junior,
                "placement_step_down",
                Status::Passed,
                failed + HOUR,
            )),
        ]);
        let o = offers(&GradePolicy::default(), Backend, &h, failed + 2 * HOUR);
        assert!(o.available.is_empty());
        assert_eq!(
            locked(&o),
            [(
                Grade::Middle,
                AttemptKind::PlacementUpgrade,
                LockReason::RetryBlocked,
                failed + 30 * DAY,
                "Дотест на грейд выше: Middle",
                false
            )]
        );
    }

    #[test]
    fn the_up_test_is_immediate_without_a_recent_failure_at_that_grade() {
        let now = T0 + 40 * DAY;
        let h = history_of(vec![
            session(Grade::Middle, "placement", Status::Failed, T0),
            confident(session(
                Grade::Junior,
                "placement",
                Status::Passed,
                now - HOUR,
            )),
        ]);
        let o = offers(&GradePolicy::default(), Backend, &h, now);
        assert_eq!(
            open(&o),
            [(
                Grade::Middle,
                AttemptKind::PlacementUpgrade,
                "Дотест на грейд выше: Middle",
                false,
                false
            )]
        );
        assert!(o.locked.is_empty());
    }

    #[test]
    fn an_ordered_retest_opens_at_once_and_needs_the_camera() {
        let decided = T0 + 3 * DAY;
        let h = history_of(vec![reviewed(
            CandidateSession {
                camera: Some(false),
                ..session(Grade::Middle, "placement", Status::Passed, T0)
            },
            Review::Retest,
            decided,
        )]);
        let o = offers(&GradePolicy::default(), Backend, &h, decided + HOUR);
        assert_eq!(
            open(&o),
            [
                (
                    Grade::Junior,
                    AttemptKind::PlacementStepDown,
                    "Тест на грейд ниже: Junior",
                    false,
                    false
                ),
                (
                    Grade::Middle,
                    AttemptKind::Placement,
                    "Пересдача с видеозаписью: Middle",
                    true,
                    true
                ),
            ],
            "a lower grade is a weaker claim and stays free of the camera"
        );
        assert_eq!(
            locked(&o),
            [(
                Grade::Senior,
                AttemptKind::Placement,
                LockReason::RetryBlocked,
                decided + 30 * DAY,
                "Первичный тест: Senior",
                true
            )]
        );
    }

    #[test]
    fn an_annulled_upgrade_locks_its_grade_from_the_decision() {
        let tested = T0 + 198 * DAY;
        let decided = T0 + 200 * DAY;
        let h = history_of(vec![
            session(Grade::Junior, "placement", Status::Passed, T0),
            reviewed(
                session(Grade::Middle, "upgrade", Status::Passed, tested),
                Review::Annul,
                decided,
            ),
        ]);
        let p = GradePolicy::default();
        assert_eq!(p.current_grade(Backend, &h.attempts), Some(Grade::Junior));
        let o = offers(&p, Backend, &h, decided + HOUR);
        assert!(o.available.is_empty());
        assert_eq!(
            locked(&o),
            [(
                Grade::Middle,
                AttemptKind::Upgrade,
                LockReason::RetryBlocked,
                decided + 30 * DAY,
                "Повышение грейда: Middle",
                false
            )],
            "30 days from the decision, not from the test"
        );
    }

    #[test]
    fn a_fresh_category_locks_changes_until_the_configured_cooldown_ends() {
        let h = history_of(vec![session(
            Grade::Middle,
            "placement",
            Status::Passed,
            T0,
        )]);
        let o = offers(&GradePolicy::default(), Backend, &h, T0 + 10 * DAY);
        assert!(o.available.is_empty());
        let cooldown = T0 + 90 * DAY;
        assert_eq!(
            locked(&o),
            [
                (
                    Grade::Junior,
                    AttemptKind::Downgrade,
                    LockReason::GradeChangeCooldown,
                    cooldown,
                    "Смена грейда на ниже: Junior",
                    false
                ),
                (
                    Grade::Senior,
                    AttemptKind::Upgrade,
                    LockReason::GradeChangeCooldown,
                    cooldown,
                    "Повышение грейда: Senior",
                    false
                ),
            ]
        );

        let short = GradePolicy::new(PolicyConfig {
            change_cooldown_days: 7,
            ..PolicyConfig::default()
        });
        let o = offers(&short, Backend, &h, T0 + 10 * DAY);
        assert!(o.locked.is_empty());
        assert_eq!(
            open(&o).iter().map(|o| o.2).collect::<Vec<_>>(),
            ["Смена грейда на ниже: Junior", "Повышение грейда: Senior"]
        );
    }

    fn problem(e: PolicyError) -> (u16, &'static str, String, Option<String>) {
        let err = policy_problem(e);
        let (status, code, _) = err.summary();
        let title = err.to_string();
        let ApiError::Status { detail, .. } = err else {
            unreachable!("policy refusals are plain problems")
        };
        (status, code, title, detail)
    }

    #[test]
    fn policy_refusals_are_conflicts_with_the_date_they_lift() {
        let lifts = Utc
            .with_ymd_and_hms(2026, 11, 6, 23, 30, 0)
            .unwrap()
            .timestamp();
        assert_eq!(
            problem(PolicyError::ChangeCooldown {
                available_at: lifts
            }),
            (
                409,
                "grade_change_cooldown",
                "Смена грейда пока недоступна".to_owned(),
                Some("Следующая попытка — после 06.11.2026".to_owned())
            )
        );
        assert_eq!(
            problem(PolicyError::RetryBlocked {
                available_at: lifts
            }),
            (
                409,
                "retry_blocked",
                "Повторный тест пока недоступен".to_owned(),
                Some("Пересдача — после 06.11.2026".to_owned())
            )
        );
        assert_eq!(
            problem(PolicyError::NotAllowed),
            (
                409,
                "attempt_not_allowed",
                "Такая попытка сейчас недоступна".to_owned(),
                None
            )
        );
        assert_eq!(
            problem(PolicyError::NoAdjacentGrade),
            (
                409,
                "attempt_not_allowed",
                "Нет соседнего грейда в этом направлении".to_owned(),
                None
            )
        );
    }

    #[test]
    fn an_answer_counts_until_five_seconds_past_the_deadline() {
        let deadline = at(T0);
        assert!(!timed_out(deadline, deadline - Duration::seconds(30)));
        assert!(!timed_out(deadline, deadline));
        assert!(!timed_out(deadline, deadline + Duration::seconds(5)));
        assert!(timed_out(
            deadline,
            deadline + Duration::milliseconds(5_001)
        ));
    }

    #[test]
    fn an_ordered_camera_retest_refuses_to_start_without_the_camera() {
        let err = camera_guard(ProctoringMode::Optional, false, true).expect_err("refused");
        assert_eq!(err.summary(), (409, "camera_required", Vec::new()));
        assert_eq!(err.to_string(), "Этот тест нужно пройти с видеозаписью");
        let ApiError::Status { detail, .. } = err else {
            unreachable!()
        };
        assert_eq!(
            detail.as_deref(),
            Some(
                "Модератор назначил пересдачу с видеозаписью: включите камеру на шаге перед тестом"
            )
        );
        assert!(camera_guard(ProctoringMode::Optional, true, true).is_ok());
        assert!(camera_guard(ProctoringMode::Optional, false, false).is_ok());
        assert!(
            camera_guard(ProctoringMode::Off, false, true).is_ok(),
            "with proctoring off there is no camera to ask for"
        );
    }
}
