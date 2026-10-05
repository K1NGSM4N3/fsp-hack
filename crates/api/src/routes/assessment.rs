//! Adaptive grade test (architecture.md §5): the candidate claims a grade, the CAT engine picks
//! parametric items, answers are checked by regenerating the instance from its `ItemRef`, and the
//! grade policy (T-5, T-6) decides which attempts are allowed. Only `ItemRef`s are stored.

use assessment::{
    Answer, AttemptKind, Decision, GradePolicy, ItemRef, ItemView, PolicyError, SessionState,
    TestPlan, Topic,
    policy::{Attempt, PolicyConfig},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
};
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

fn parse_kind(s: &str) -> AttemptKind {
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

fn corrupt(e: impl std::fmt::Display) -> ApiError {
    ApiError::Internal(InternalError::Corrupt(e.to_string()))
}

struct History {
    attempts: Vec<Attempt>,
    /// The latest passed test measured above the claimed grade or with high confidence: only
    /// then is the immediate upgrade test offered (T-5 "справился уверенно").
    upgrade_earned: bool,
}

async fn history(pool: &PgPool, candidate: Uuid) -> ApiResult<History> {
    let rows = db::assessment::candidate_sessions(pool, candidate).await?;
    let upgrade_earned = rows
        .iter()
        .find(|r| r.status == db::types::AssessmentStatus::Passed)
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
    for r in rows.into_iter().rev() {
        let passed = r.status == db::types::AssessmentStatus::Passed;
        if !passed && r.status != db::types::AssessmentStatus::Failed {
            continue;
        }
        out.push(Attempt {
            specialization: r.specialization.parse()?,
            grade: r.claimed_grade.parse()?,
            kind: parse_kind(&r.attempt_kind),
            passed,
            assigned: r.assigned,
            at: r.finished_at.unwrap_or(r.started_at).timestamp(),
        });
    }
    Ok(History {
        attempts: out,
        upgrade_earned,
    })
}

#[derive(Serialize, ToSchema)]
pub struct Offer {
    pub grade: Grade,
    pub kind: AttemptKind,
    pub label: String,
}

fn offers(state: &AppState, spec: Specialization, history: &History, now: i64) -> Vec<Offer> {
    let p = policy(state);
    let current = p.current_grade(spec, &history.attempts);
    let mut out: Vec<Offer> = Vec::new();
    for grade in Grade::ALL.iter().copied().filter(|g| Some(*g) != current) {
        for kind in candidate_kinds(current, grade, history.upgrade_earned) {
            if let Ok(a) = p.check(spec, &history.attempts, kind, grade, now) {
                if !out.iter().any(|o| o.grade == a.grade) {
                    out.push(Offer {
                        grade: a.grade,
                        kind: a.kind,
                        label: format!("{}: {}", kind_ru(a.kind), a.grade.name_ru()),
                    });
                }
                break;
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
}

fn load_state(v: serde_json::Value) -> ApiResult<SessionState> {
    serde_json::from_value(v).map_err(corrupt)
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
        item: assessment::render(item).map_err(corrupt)?,
        deadline,
    })
}

fn deadline_for(item: &ItemRef, now: DateTime<Utc>) -> ApiResult<DateTime<Utc>> {
    let view = assessment::render(item).map_err(corrupt)?;
    Ok(now + Duration::seconds(i64::from(view.time_limit_secs)))
}

#[derive(Deserialize, ToSchema)]
pub struct StartInput {
    pub specialization: Specialization,
    pub grade: Grade,
    /// Choice made on the proctoring step. Omitted while proctoring is optional, the test runs
    /// without video and is marked so.
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
        (status = 201, body = AssessmentSession),
        (status = 409, description = "`assessment_in_progress`, `survey_required`, `grade_change_cooldown`, `retry_blocked`, `attempt_not_allowed`, `already_assigned`", body = Problem),
        (status = 422, description = "`validation_failed`: `proctoring.consent_document_id` missing or not the current version", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn start_session(
    State(state): State<AppState>,
    user: CandidateUser,
    headers: HeaderMap,
    Json(input): Json<StartInput>,
) -> ApiResult<(StatusCode, Json<AssessmentSession>)> {
    db::assessment::expire_abandoned_sessions(&state.pool, user.id).await?;
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
    let mut last_error = PolicyError::NotAllowed;
    let mut allowed = None;
    for kind in candidate_kinds(current, input.grade, history.upgrade_earned) {
        match p.check(
            input.specialization,
            &history.attempts,
            kind,
            input.grade,
            now.timestamp(),
        ) {
            Ok(a) => {
                allowed = Some(a);
                break;
            }
            Err(e @ (PolicyError::ChangeCooldown { .. } | PolicyError::RetryBlocked { .. })) => {
                last_error = e
            }
            Err(e) if matches!(last_error, PolicyError::NotAllowed) => last_error = e,
            Err(_) => {}
        }
    }
    let allowed = allowed.ok_or_else(|| policy_problem(last_error))?;

    let camera = input.proctoring.as_ref().is_some_and(|p| p.camera);
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
    .await?;
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
    Ok((StatusCode::CREATED, Json(view(&state, user.id, id).await?)))
}

/// Records one outcome and either schedules the next item or finishes the session
/// (assigning the grade and test-confirmed skills on a pass).
async fn advance(
    tx: &mut PgConnection,
    candidate: Uuid,
    row: &db::assessment::SessionForUpdate,
    session: &mut SessionState,
    item: ItemRef,
    answer: Option<&Answer>,
    now: DateTime<Utc>,
) -> ApiResult<()> {
    let deadline = row.current_item_deadline.unwrap_or(now);
    let timed_out = now > deadline + Duration::seconds(DEADLINE_GRACE_SECS);
    let correct = !timed_out && answer.is_some_and(|a| assessment::check(&item, a));
    let shown = row.current_item_shown_at.unwrap_or(now);
    let elapsed_ms = (now - shown)
        .num_milliseconds()
        .clamp(0, i64::from(u32::MAX)) as u32;
    assessment::record(session, item.clone(), correct, elapsed_ms).map_err(corrupt)?;
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
        return Ok(());
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
            let item: ItemRef = serde_json::from_value(item).map_err(corrupt)?;
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
            Some(AssessmentResult {
                test_strength: Some(matching::test_strength(claimed, theta as f32)),
                assigned_grade: r.assigned.then_some(claimed),
                needs_review: match &proctoring {
                    Some((_, counts)) => {
                        assessment::proctoring::needs_confirmation(decision.aberrant, counts)
                    }
                    None => decision.aberrant,
                },
                topics: topic_results(&s),
                offers: offers(state, spec, &history, Utc::now().timestamp()),
                decision,
            })
        }
        _ => None,
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
    db::assessment::expire_abandoned_sessions(&state.pool, user.id).await?;
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

/// Current item without the answer key; an expired item is recorded as wrong and the next one is returned
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
    let row = db::assessment::session_for_update(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let now = Utc::now();
    if row.status == db::types::AssessmentStatus::InProgress
        && let (Some(item), Some(deadline)) = (&row.current_item, row.current_item_deadline)
        && now > deadline + Duration::seconds(DEADLINE_GRACE_SECS)
    {
        let mut session = load_state(row.state.clone())?;
        let item: ItemRef = serde_json::from_value(item.clone()).map_err(corrupt)?;
        advance(&mut tx, user.id, &row, &mut session, item, None, now).await?;
    }
    tx.commit().await?;
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
        (status = 409, description = "`assessment_finished`, `stale_item`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn answer(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
    Json(input): Json<AnswerInput>,
) -> ApiResult<Json<AssessmentSession>> {
    let mut tx = state.pool.begin().await?;
    let row = db::assessment::session_for_update(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if row.status != db::types::AssessmentStatus::InProgress {
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
        .transpose()
        .map_err(corrupt)?
        .ok_or_else(|| corrupt("no current item"))?;
    advance(
        &mut tx,
        user.id,
        &row,
        &mut session,
        item,
        Some(&input.answer),
        Utc::now(),
    )
    .await?;
    db::candidates::touch_candidate_activity(&mut *tx, user.id).await?;
    tx.commit().await?;
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
    /// When a grade change becomes possible, if it is blocked by the cooldown now.
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
    let (available, next_change_at) = match spec {
        Some(spec) => {
            let available = offers(&state, spec, &history, now);
            let next = category.and_then(|c| {
                let p = policy(&state);
                [c.grade.next(), c.grade.prev()]
                    .into_iter()
                    .flatten()
                    .find_map(|g| {
                        let kind = if g > c.grade {
                            AttemptKind::Upgrade
                        } else {
                            AttemptKind::Downgrade
                        };
                        match p.check(spec, &history.attempts, kind, g, now) {
                            Err(
                                PolicyError::ChangeCooldown { available_at }
                                | PolicyError::RetryBlocked { available_at },
                            ) => Some(at(available_at)),
                            _ => None,
                        }
                    })
            });
            (available, next)
        }
        None => (Vec::new(), None),
    };
    let grades = db::grades::grade_history(&state.pool, user.id).await?;
    let mut history_out = Vec::with_capacity(grades.len());
    for g in grades {
        history_out.push(GradeHistoryEntry {
            specialization: g.specialization.parse()?,
            grade: g.grade.parse()?,
            reason: g.reason,
            theta: g.theta,
            session_id: g.session_id,
            assigned_at: g.assigned_at,
        });
    }
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
        available,
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
    responses((status = 200, body = TestPlanDemo), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn need_test_plan(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TestPlanDemo>> {
    let company = super::company::company_id(&state.pool, user.id).await?;
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
