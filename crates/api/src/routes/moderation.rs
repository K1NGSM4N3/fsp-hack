//! The moderator's cabinet (architecture.md §5.7, §11.1, §9.3): counters, the proctoring queue
//! with session review and decisions, the merged anti-fake queue with actions, and the audit log.
//! Admin role only; every list is keyset-paginated with an opaque `cursor`.
//!
//! The session view carries no name, contacts or email of the candidate: judging whether a test
//! was taken alone needs the answers, the events and the video, not the person's identity, so
//! the moderator works with the anonymous code like an employer does (152-ФЗ, ст. 5 ч. 5).

use assessment::{AttemptKind, IntegrityEvent, Topic, proctoring::needs_confirmation};
use axum::{extract::State, http::Method, response::Response};
use catalog::{Category, Grade};
use chrono::{DateTime, Duration, Utc};
use matching::trust::{TrustAssessment, TrustFlag, TrustLevel};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    assessment::parse_kind,
    proctoring::{playable_chunks, stream_recording},
};
use crate::{
    auth::AdminUser,
    dto::{AssessmentStatus, SalaryRange, VacancyStatus},
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Path, Query},
    moderation::{
        self, ModerationAction, ModerationTarget, ReviewDecision, TrustAction, TrustItemKind,
        encode_cursor,
    },
    proctoring::{self as proctoring_svc, ProctoringMode, ProctoringSummary},
    state::AppState,
    trust,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(overview))
        .routes(routes!(proctoring_queue))
        .routes(routes!(session_review))
        .routes(routes!(session_recording))
        .routes(routes!(decide))
        .routes(routes!(trust_queue))
        .routes(routes!(trust_act))
        .routes(routes!(audit_log))
}

#[derive(Serialize, ToSchema)]
pub struct ProctoringCounters {
    /// Undecided sessions flagged «требует подтверждения».
    pub needs_review: i64,
    /// Undecided sessions with tab leaves, pastes or fullscreen exits.
    pub remarks: i64,
    pub with_video: i64,
    pub decided_week: i64,
}

#[derive(Serialize, ToSchema)]
pub struct TrustCounters {
    /// Companies with negative signals waiting for a look.
    pub companies: i64,
    pub vacancies: i64,
    pub invitations: i64,
    pub complaints: i64,
    pub unpublished_vacancies: i64,
    pub frozen_companies: i64,
}

#[derive(Serialize, ToSchema)]
pub struct LogCounters {
    pub actions_week: i64,
    pub recording_views_week: i64,
}

#[derive(Serialize, ToSchema)]
pub struct ModerationOverview {
    pub proctoring: ProctoringCounters,
    pub antifake: TrustCounters,
    pub log: LogCounters,
}

/// The overview: what waits in each queue and what moderators did this week
#[utoipa::path(get, path = "/overview", tag = "Модерация",
    responses(
        (status = 200, body = ModerationOverview),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn overview(
    State(state): State<AppState>,
    _user: AdminUser,
) -> ApiResult<Json<ModerationOverview>> {
    let o = db::moderation::overview(&state.pool).await?;
    let companies = pending_companies(&state, None).await?.len() as i64;
    Ok(Json(ModerationOverview {
        proctoring: ProctoringCounters {
            needs_review: o.needs_review,
            remarks: o.remarks,
            with_video: o.with_video,
            decided_week: o.decided_week,
        },
        antifake: TrustCounters {
            companies,
            vacancies: o.flagged_vacancies,
            invitations: o.flagged_invitations,
            complaints: o.open_complaints,
            unpublished_vacancies: o.unpublished_vacancies,
            frozen_companies: o.frozen_companies,
        },
        log: LogCounters {
            actions_week: o.actions_week,
            recording_views_week: o.recording_views_week,
        },
    }))
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueueFilter {
    /// «Требует подтверждения», not decided yet.
    #[default]
    NeedsReview,
    /// Tab leaves, pastes or fullscreen exits, not decided yet.
    Remarks,
    /// Every session recorded with the camera, decided or not.
    Video,
}

#[derive(Deserialize, IntoParams)]
pub struct QueueQuery {
    #[param(inline)]
    pub filter: Option<QueueFilter>,
    /// `next_cursor` of the previous page.
    pub cursor: Option<String>,
    /// 1–100, 30 by default.
    pub limit: Option<i64>,
}

/// How the session's video stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoState {
    /// Proctoring was off: no camera step, no events.
    Off,
    /// The candidate continued without the camera.
    Declined,
    /// Recorded and within retention.
    Kept,
    /// Deleted: the 90-day retention passed or the consent was revoked.
    Purged,
}

fn video_state(camera: Option<bool>, purged: bool) -> VideoState {
    match camera {
        None => VideoState::Off,
        Some(false) => VideoState::Declined,
        Some(true) if purged => VideoState::Purged,
        Some(true) => VideoState::Kept,
    }
}

#[derive(Serialize, ToSchema)]
pub struct Remarks {
    pub left_test: i64,
    pub pastes: i64,
    pub fullscreen_exits: i64,
    /// Camera lost or recording failed: technical, not behaviour.
    pub technical: i64,
}

#[derive(Serialize, ToSchema)]
pub struct QueueSession {
    pub session_id: Uuid,
    /// Anonymous code «K-XXXXXX»: the moderator does not need the name.
    pub candidate_code: String,
    pub category: Category,
    pub category_name: String,
    pub attempt_kind: AttemptKind,
    pub status: AssessmentStatus,
    pub finished_at: DateTime<Utc>,
    pub duration_secs: i64,
    pub person_fit_lz: Option<f64>,
    pub aberrant: bool,
    pub needs_review: bool,
    pub video: VideoState,
    pub remarks: Remarks,
    /// The grade this test assigned is in force (not voided).
    pub result_in_force: bool,
    pub decision: Option<ReviewDecision>,
    pub decided_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct QueuePage {
    pub items: Vec<QueueSession>,
    /// Pass as `cursor` for the next page; `null` on the last one.
    pub next_cursor: Option<String>,
}

/// Proctoring queue: finished sessions oldest first
#[utoipa::path(get, path = "/proctoring", tag = "Модерация",
    params(QueueQuery),
    responses(
        (status = 200, body = QueuePage),
        (status = 400, description = "`invalid_query`: unknown filter or a cursor not from `next_cursor`", body = Problem),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn proctoring_queue(
    State(state): State<AppState>,
    _user: AdminUser,
    Query(q): Query<QueueQuery>,
) -> ApiResult<Json<QueuePage>> {
    let after = moderation::decode_cursor(q.cursor.as_deref())?;
    let limit = moderation::page_limit(q.limit);
    let filter = match q.filter.unwrap_or_default() {
        QueueFilter::NeedsReview => "needs_review",
        QueueFilter::Remarks => "remarks",
        QueueFilter::Video => "video",
    };
    let mut rows = db::moderation::proctoring_queue(&state.pool, filter, after, limit + 1).await?;
    let more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let next_cursor = more
        .then(|| {
            rows.last()
                .map(|r| encode_cursor(r.finished_at, r.session_id))
        })
        .flatten();
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let category = Category::new(r.specialization.parse()?, r.claimed_grade.parse()?);
        let aberrant = r.aberrant.unwrap_or(false);
        let behavioral = r.left_test + r.pastes + r.fullscreen_exits;
        items.push(QueueSession {
            session_id: r.session_id,
            candidate_code: r.public_code,
            category_name: category.name_ru(),
            category,
            attempt_kind: parse_kind(&r.attempt_kind),
            status: r.status.into(),
            duration_secs: (r.finished_at - r.started_at).num_seconds().max(0),
            finished_at: r.finished_at,
            person_fit_lz: r.person_fit_lz,
            aberrant,
            needs_review: r.decision.is_none()
                && aberrant
                && (r.camera.is_none() || behavioral > 0),
            video: video_state(r.camera, r.purged.unwrap_or(false)),
            remarks: Remarks {
                left_test: r.left_test,
                pastes: r.pastes,
                fullscreen_exits: r.fullscreen_exits,
                technical: r.technical,
            },
            result_in_force: r.assigned,
            decision: r.decision.map(Into::into),
            decided_at: r.decided_at,
        });
    }
    Ok(Json(QueuePage { items, next_cursor }))
}

#[derive(Serialize, ToSchema)]
pub struct ReviewAnswer {
    /// 0-based; integrity events name the item on screen by this number.
    pub seq: u16,
    pub title: String,
    pub topic: Option<Topic>,
    pub topic_ru: Option<String>,
    pub level: u8,
    /// IRT difficulty `b` on the specialization's θ scale.
    pub difficulty: Option<f32>,
    pub correct: bool,
    pub timed_out: bool,
    pub elapsed_ms: i32,
    pub shown_at: DateTime<Utc>,
    pub answered_at: DateTime<Utc>,
    pub theta_after: f64,
}

#[derive(Serialize, ToSchema)]
pub struct ReviewEvent {
    pub kind: IntegrityEvent,
    pub at: DateTime<Utc>,
    pub duration_ms: Option<i32>,
    /// The item on screen: as the browser reported it, or by time when it did not.
    pub item_seq: Option<u16>,
    /// From the start of the test.
    pub offset_ms: i64,
}

/// The person-fit statistic and the pattern behind it.
#[derive(Serialize, ToSchema)]
pub struct PersonFit {
    pub lz: Option<f64>,
    pub threshold: f64,
    pub aberrant: bool,
    /// Items harder than θ̂ answered right, of all the items harder than θ̂.
    pub harder_correct: u16,
    pub harder_total: u16,
    /// Items easier than θ̂ answered wrong, of all the items easier than θ̂.
    pub easier_wrong: u16,
    pub easier_total: u16,
    pub summary: String,
}

#[derive(Serialize, ToSchema)]
pub struct ReviewResult {
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<Grade>,
    /// θ cut of the claimed grade.
    pub cut: f64,
    pub test_strength: Option<f32>,
}

/// What the test did to the candidate's category and what a decision would change.
#[derive(Serialize, ToSchema)]
pub struct AssignmentState {
    pub assigned_at: Option<DateTime<Utc>>,
    pub annulled_at: Option<DateTime<Utc>>,
    /// The assignment is in force and no later one replaced it: annulling it changes the
    /// category employers see.
    pub current: bool,
    /// The candidate's category right now.
    pub current_category: Option<Category>,
    pub current_category_name: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RecordingView {
    pub at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct RecordingInfo {
    pub video: VideoState,
    /// `GET …/recording` should play it (files may still be missing on this server).
    pub playable: bool,
    pub delete_after: Option<DateTime<Utc>>,
    /// Every time a moderator played it; the candidate sees the same dates.
    pub views: Vec<RecordingView>,
}

#[derive(Serialize, ToSchema)]
pub struct DecisionView {
    pub decision: ReviewDecision,
    pub comment: String,
    pub decided_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct SessionReview {
    pub session_id: Uuid,
    pub candidate_code: String,
    pub category: Category,
    pub category_name: String,
    pub attempt_kind: AttemptKind,
    pub status: AssessmentStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub result: ReviewResult,
    pub assignment: AssignmentState,
    pub person_fit: PersonFit,
    /// «Требует подтверждения» as the candidate saw it before any decision.
    pub flagged: bool,
    /// `None` when the session ran with proctoring off.
    pub proctoring: Option<ProctoringSummary>,
    pub recording: RecordingInfo,
    pub answers: Vec<ReviewAnswer>,
    pub events: Vec<ReviewEvent>,
    pub decision: Option<DecisionView>,
    /// Decisions possible right now.
    pub allowed: Vec<ReviewDecision>,
}

fn narrow_seq(seq: i32) -> u16 {
    u16::try_from(seq.max(0)).unwrap_or(u16::MAX)
}

fn person_fit(
    theta: Option<f64>,
    lz: Option<f64>,
    aberrant: bool,
    answers: &[ReviewAnswer],
) -> PersonFit {
    let threshold = assessment::CatConfig::default().aberrant_lz;
    let (mut harder_correct, mut harder_total, mut easier_wrong, mut easier_total) =
        (0u16, 0, 0, 0);
    if let Some(t) = theta {
        for a in answers {
            let Some(b) = a.difficulty.map(f64::from) else {
                continue;
            };
            if b > t {
                harder_total += 1;
                harder_correct += u16::from(a.correct);
            } else {
                easier_total += 1;
                easier_wrong += u16::from(!a.correct);
            }
        }
    }
    let pattern = format!(
        "Задания сложнее уровня кандидата: верно {harder_correct} из {harder_total}. Задания легче: неверно {easier_wrong} из {easier_total}."
    );
    let summary = match lz {
        None => "Тест не завершён — статистики нет".to_owned(),
        Some(v) if aberrant => format!(
            "l_z = {} ниже порога {}: так согласуются ответы примерно у 2 % честных кандидатов. {pattern} Обычно бывает наоборот: трудные чаще не решены, лёгкие — решены.",
            ru_num(v),
            ru_num(threshold)
        ),
        Some(v) => format!(
            "l_z = {}: картина ответов обычная для этого уровня. {pattern}",
            ru_num(v)
        ),
    };
    PersonFit {
        lz,
        threshold,
        aberrant,
        harder_correct,
        harder_total,
        easier_wrong,
        easier_total,
        summary,
    }
}

fn ru_num(v: f64) -> String {
    format!("{v:.2}").replace('-', "−").replace('.', ",")
}

async fn load_review(state: &AppState, session_id: Uuid) -> ApiResult<SessionReview> {
    let s = db::moderation::review_session(&state.pool, session_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let spec = s.specialization.parse()?;
    let claimed: Grade = s.claimed_grade.parse()?;
    let category = Category::new(spec, claimed);
    let status = AssessmentStatus::from(s.status);
    let in_progress = status == AssessmentStatus::InProgress;

    let bank = assessment::bank();
    let mut answers = Vec::new();
    for r in db::moderation::responses(&state.pool, session_id).await? {
        let level = u8::try_from(r.level.max(0)).unwrap_or(0);
        let template = bank.template(&r.template_id);
        answers.push(ReviewAnswer {
            seq: narrow_seq(r.seq),
            title: template.map_or_else(|| r.template_id.clone(), |t| t.title.to_owned()),
            topic: template.map(|t| t.topic),
            topic_ru: template.map(|t| t.topic.name_ru().to_owned()),
            level,
            difficulty: template
                .and_then(|t| t.level(level))
                .map(|l| l.params.b as f32),
            correct: r.correct,
            timed_out: r.timed_out,
            elapsed_ms: r.elapsed_ms,
            shown_at: r.answered_at - Duration::milliseconds(i64::from(r.elapsed_ms.max(0))),
            answered_at: r.answered_at,
            theta_after: r.theta_after,
        });
    }

    let summary = db::proctoring::summary(&state.pool, session_id, None).await?;
    let (proctoring, counts, camera, purged) = match &summary {
        Some(p) => {
            let facts = proctoring_svc::facts(p, in_progress);
            (
                Some(proctoring_svc::summary_view(
                    p,
                    &facts,
                    state.config.proctoring.retention_days,
                )),
                Some(facts.counts),
                Some(p.camera),
                p.purged_at.is_some(),
            )
        }
        None => (None, None, None, false),
    };
    let mut events = Vec::new();
    if summary.is_some() {
        for e in db::proctoring::events(&state.pool, session_id).await? {
            let item_seq = e.item_seq.and_then(|v| u16::try_from(v).ok()).or_else(|| {
                answers
                    .iter()
                    .find(|a| a.shown_at <= e.occurred_at && e.occurred_at <= a.answered_at)
                    .map(|a| a.seq)
            });
            events.push(ReviewEvent {
                kind: proctoring_svc::from_db(e.kind),
                offset_ms: (e.occurred_at - s.started_at).num_milliseconds().max(0),
                at: e.occurred_at,
                duration_ms: e.duration_ms,
                item_seq,
            });
        }
    }
    let aberrant = s.aberrant.unwrap_or(false);
    let flagged = match &counts {
        Some(c) => needs_confirmation(aberrant, c),
        None => aberrant,
    };
    let views = db::moderation::recording_views(&state.pool, session_id)
        .await?
        .into_iter()
        .map(|at| RecordingView { at })
        .collect();
    let in_force = s.assigned_at.is_some() && s.annulled_at.is_none();
    let finished = matches!(status, AssessmentStatus::Passed | AssessmentStatus::Failed);
    let mut allowed = Vec::new();
    if s.decision.is_none() && finished {
        allowed.push(ReviewDecision::Confirm);
        if in_force {
            allowed.push(ReviewDecision::Annul);
        }
        if s.assignment_current && state.config.proctoring.mode == ProctoringMode::Optional {
            allowed.push(ReviewDecision::Retest);
        }
    }
    let current_category = match (
        s.current_specialization.as_deref(),
        s.current_grade.as_deref(),
    ) {
        (Some(sp), Some(g)) => Some(Category::new(sp.parse()?, g.parse()?)),
        _ => None,
    };
    let person_fit = person_fit(s.theta, s.person_fit_lz, aberrant, &answers);
    Ok(SessionReview {
        session_id,
        candidate_code: s.public_code,
        category_name: category.name_ru(),
        category,
        attempt_kind: parse_kind(&s.attempt_kind),
        status,
        started_at: s.started_at,
        finished_at: s.finished_at,
        result: ReviewResult {
            theta: s.theta,
            se: s.se,
            measured_grade: s.measured_grade.as_deref().and_then(Grade::from_slug),
            cut: claimed.theta_cut(),
            test_strength: s.theta.map(|t| matching::test_strength(claimed, t as f32)),
        },
        assignment: AssignmentState {
            assigned_at: s.assigned_at,
            annulled_at: s.annulled_at,
            current: s.assignment_current,
            current_category_name: current_category.map(Category::name_ru),
            current_category,
        },
        person_fit,
        flagged,
        recording: RecordingInfo {
            video: video_state(camera, purged),
            playable: proctoring.as_ref().is_some_and(|p| p.playable),
            delete_after: proctoring.as_ref().and_then(|p| p.delete_after),
            views,
        },
        proctoring,
        answers,
        events,
        decision: match (s.decision, s.decision_comment, s.decided_at) {
            (Some(d), Some(comment), Some(decided_at)) => Some(DecisionView {
                decision: d.into(),
                comment,
                decided_at,
            }),
            _ => None,
        },
        allowed,
    })
}

/// A session for review: result, person-fit, answers and integrity events on one timeline, video
#[utoipa::path(get, path = "/proctoring/{session_id}", tag = "Модерация",
    params(("session_id" = Uuid, Path)),
    responses(
        (status = 200, body = SessionReview),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
        (status = 404, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn session_review(
    State(state): State<AppState>,
    _user: AdminUser,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Json<SessionReview>> {
    Ok(Json(load_review(&state, session_id).await?))
}

/// Play any session's recording; every view is written to the audit log before the video is sent
///
/// `HEAD` answers whether the recording can be played without sending it, so it is not a view
/// and is not logged.
#[utoipa::path(get, path = "/proctoring/{session_id}/recording", tag = "Модерация",
    params(("session_id" = Uuid, Path)),
    responses(
        (status = 200, content_type = "video/webm", body = Vec<u8>),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
        (status = 404, description = "`not_found`, `recording_unavailable`: no camera, deleted after retention, or the files are missing", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn session_recording(
    State(state): State<AppState>,
    user: AdminUser,
    method: Method,
    Path(session_id): Path<Uuid>,
) -> ApiResult<Response> {
    let s = db::proctoring::summary(&state.pool, session_id, None)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let chunks = playable_chunks(&state, &s).await?;
    if method != Method::HEAD {
        moderation::log_recording_view(&state, user.id, session_id, s.candidate_id).await?;
    }
    Ok(stream_recording(&state, &s, chunks))
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"decision": "confirm", "comment": "Запись просмотрена: соискатель отвечал самостоятельно, уходы со вкладки — по 2–3 секунды на заданиях с кодом."}))]
pub struct DecisionInput {
    pub decision: ReviewDecision,
    /// Up to 2000 characters. The candidate reads it in the test history.
    pub comment: String,
}

/// Decide on a session: confirm the result, annul it, or order a retest with the camera
#[utoipa::path(post, path = "/proctoring/{session_id}/decision", tag = "Модерация",
    params(("session_id" = Uuid, Path)),
    request_body = DecisionInput,
    responses(
        (status = 200, body = SessionReview),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
        (status = 404, body = Problem),
        (status = 409, description = "`already_decided`, `session_not_finished`, `nothing_to_annul` (the test assigned no grade or it is void already), `retest_not_applicable` (a later test replaced this result), `proctoring_off`", body = Problem),
        (status = 422, description = "`validation_failed`: `comment` empty or longer than 2000 characters", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn decide(
    State(state): State<AppState>,
    user: AdminUser,
    Path(session_id): Path<Uuid>,
    Json(input): Json<DecisionInput>,
) -> ApiResult<Json<SessionReview>> {
    let comment = moderation::comment(&input.comment)?;
    moderation::decide(&state, user.id, session_id, input.decision, comment).await?;
    Ok(Json(load_review(&state, session_id).await?))
}

#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustView {
    /// Waiting for a decision.
    #[default]
    Open,
    /// Sanctions in force: vacancies taken down and frozen companies, to restore.
    Sanctions,
}

#[derive(Deserialize, IntoParams)]
pub struct TrustQuery {
    #[param(inline)]
    pub view: Option<TrustView>,
    /// Only this kind of item.
    #[param(inline)]
    pub kind: Option<TrustItemKind>,
    pub cursor: Option<String>,
    /// 1–100, 30 by default.
    pub limit: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct CompanyRef {
    pub id: Uuid,
    pub name: String,
    pub level: Option<TrustLevel>,
    /// Invitations per 7 days right now.
    pub quota: Option<i64>,
    pub frozen_until: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct ComplaintView {
    pub target_kind: TrustItemKind,
    pub target_id: Uuid,
    pub reason: String,
    pub reason_ru: String,
    /// The candidate's own words; the reporter is not shown.
    pub comment: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct TrustItem {
    pub kind: TrustItemKind,
    pub id: Uuid,
    /// When it entered the queue (or the sanction started).
    pub at: DateTime<Utc>,
    pub title: String,
    pub company: Option<CompanyRef>,
    /// Why it is here, in words.
    pub reasons: Vec<String>,
    pub vacancy_status: Option<VacancyStatus>,
    pub salary: Option<SalaryRange>,
    /// Open complaints about this vacancy.
    pub open_complaints: i64,
    pub complaint: Option<ComplaintView>,
    /// Sanctions view: the freeze ends then.
    pub until: Option<DateTime<Utc>>,
    /// What can be done with it now.
    pub actions: Vec<TrustAction>,
}

#[derive(Serialize, ToSchema)]
pub struct TrustPage {
    pub items: Vec<TrustItem>,
    pub next_cursor: Option<String>,
}

fn flag_reasons(flags: serde_json::Value) -> ApiResult<Vec<String>> {
    let flags: Vec<TrustFlag> = serde_json::from_value(flags)?;
    Ok(flags.iter().map(TrustFlag::moderator_text).collect())
}

fn report_reason_ru(reason: &str) -> &'static str {
    match reason {
        "fake_vacancy" => "Фиктивная вакансия",
        "salary_mismatch" => "Зарплата не соответствует",
        "spam" => "Спам",
        "abuse" => "Оскорбления или давление",
        _ => "Другое",
    }
}

fn item_kind(target_kind: &str) -> TrustItemKind {
    match target_kind {
        "vacancy" => TrustItemKind::Vacancy,
        "invitation" => TrustItemKind::Invitation,
        _ => TrustItemKind::Company,
    }
}

struct Assessed {
    id: Uuid,
    name: String,
    assessment: TrustAssessment,
    frozen_until: Option<DateTime<Utc>>,
    /// Entered the queue: the newest dated signal, or the account's creation for companies whose
    /// signals carry no date.
    at: DateTime<Utc>,
    pending: bool,
}

/// Every company with its trust assessment, sorted by id for lookups. A company waits in the
/// queue while it has a negative signal, is not frozen, and got a dated signal after a
/// moderator last took it off.
async fn assessed_companies(state: &AppState) -> ApiResult<Vec<Assessed>> {
    let now = Utc::now().timestamp();
    let mut out: Vec<Assessed> = db::trust::company_signals(&state.pool, None)
        .await?
        .into_iter()
        .map(|row| {
            let assessment = matching::trust::assess(&trust::signals(&row), now);
            let pending = assessment.has_negative()
                && row.frozen_until.is_none()
                && match (row.trust_reviewed_at, row.signal_at) {
                    (None, _) => true,
                    (Some(reviewed), Some(signal)) => signal > reviewed,
                    (Some(_), None) => false,
                };
            Assessed {
                id: row.company_id,
                at: row.signal_at.unwrap_or(row.owner_created_at),
                name: row.name,
                frozen_until: row.frozen_until,
                assessment,
                pending,
            }
        })
        .collect();
    out.sort_by_key(|a| a.id);
    Ok(out)
}

async fn pending_companies(
    state: &AppState,
    after: Option<db::moderation::Cursor>,
) -> ApiResult<Vec<Assessed>> {
    Ok(assessed_companies(state)
        .await?
        .into_iter()
        .filter(|a| a.pending && after.is_none_or(|c| (a.at, a.id) > (c.at, c.id)))
        .collect())
}

fn company_ref(companies: &[Assessed], id: Uuid, name: String) -> CompanyRef {
    match companies.binary_search_by_key(&id, |a| a.id) {
        Ok(i) => {
            let a = &companies[i];
            CompanyRef {
                id,
                name,
                level: Some(a.assessment.level),
                quota: Some(i64::from(a.assessment.quota)),
                frozen_until: a.frozen_until,
            }
        }
        Err(_) => CompanyRef {
            id,
            name,
            level: None,
            quota: None,
            frozen_until: None,
        },
    }
}

fn wanted(filter: Option<TrustItemKind>, kind: TrustItemKind) -> bool {
    filter.is_none_or(|k| k == kind)
}

/// Anti-fake queue: flagged companies, vacancies and invitations with open complaints, oldest first
#[utoipa::path(get, path = "/antifake", tag = "Модерация",
    params(TrustQuery),
    responses(
        (status = 200, body = TrustPage),
        (status = 400, description = "`invalid_query`", body = Problem),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn trust_queue(
    State(state): State<AppState>,
    _user: AdminUser,
    Query(q): Query<TrustQuery>,
) -> ApiResult<Json<TrustPage>> {
    let after = moderation::decode_cursor(q.cursor.as_deref())?;
    let limit = moderation::page_limit(q.limit);
    let fetch = limit + 1;
    let companies = assessed_companies(&state).await?;
    let mut items: Vec<TrustItem> = Vec::new();
    match q.view.unwrap_or_default() {
        TrustView::Open => {
            if wanted(q.kind, TrustItemKind::Company) {
                let mut pending: Vec<&Assessed> = companies
                    .iter()
                    .filter(|a| a.pending && after.is_none_or(|c| (a.at, a.id) > (c.at, c.id)))
                    .collect();
                pending.sort_by_key(|a| (a.at, a.id));
                for a in pending.into_iter().take(fetch as usize) {
                    items.push(TrustItem {
                        kind: TrustItemKind::Company,
                        id: a.id,
                        at: a.at,
                        title: a.name.clone(),
                        company: Some(company_ref(&companies, a.id, a.name.clone())),
                        reasons: a
                            .assessment
                            .reasons
                            .iter()
                            .filter(|r| r.points < 0)
                            .map(|r| format!("{} ({})", r.text, r.points))
                            .collect(),
                        vacancy_status: None,
                        salary: None,
                        open_complaints: 0,
                        complaint: None,
                        until: None,
                        actions: vec![TrustAction::Dismiss, TrustAction::Freeze],
                    });
                }
            }
            if wanted(q.kind, TrustItemKind::Vacancy) {
                for v in db::moderation::pending_vacancies(&state.pool, after, fetch).await? {
                    let published = v.status == db::types::VacancyStatus::Published;
                    let mut actions = vec![TrustAction::Dismiss];
                    if published {
                        actions.push(TrustAction::Unpublish);
                    }
                    actions.push(TrustAction::Freeze);
                    items.push(TrustItem {
                        kind: TrustItemKind::Vacancy,
                        id: v.id,
                        at: v.at,
                        title: v.title,
                        company: Some(company_ref(&companies, v.company_id, v.company_name)),
                        reasons: flag_reasons(v.trust_flags)?,
                        vacancy_status: Some(v.status.into()),
                        salary: Some(SalaryRange {
                            min: v.salary_min,
                            max: v.salary_max,
                        }),
                        open_complaints: v.complaints,
                        complaint: None,
                        until: None,
                        actions,
                    });
                }
            }
            if wanted(q.kind, TrustItemKind::Invitation) {
                for i in db::moderation::pending_invitations(&state.pool, after, fetch).await? {
                    items.push(TrustItem {
                        kind: TrustItemKind::Invitation,
                        id: i.id,
                        at: i.at,
                        title: i.title,
                        company: Some(company_ref(&companies, i.company_id, i.company_name)),
                        reasons: flag_reasons(i.trust_flags)?,
                        vacancy_status: None,
                        salary: Some(SalaryRange {
                            min: i.salary_min,
                            max: i.salary_max,
                        }),
                        open_complaints: 0,
                        complaint: None,
                        until: None,
                        actions: vec![TrustAction::Dismiss, TrustAction::Freeze],
                    });
                }
            }
            if wanted(q.kind, TrustItemKind::Complaint) {
                for r in db::moderation::open_complaints(&state.pool, after, fetch).await? {
                    let target = item_kind(&r.target_kind);
                    let mut actions = vec![TrustAction::Dismiss];
                    if target == TrustItemKind::Vacancy {
                        actions.push(TrustAction::Unpublish);
                    }
                    if r.company_id.is_some() {
                        actions.push(TrustAction::Freeze);
                    }
                    let reason_ru = report_reason_ru(&r.reason);
                    items.push(TrustItem {
                        kind: TrustItemKind::Complaint,
                        id: r.id,
                        at: r.at,
                        title: r
                            .target_title
                            .clone()
                            .or_else(|| r.company_name.clone())
                            .unwrap_or_else(|| "Объект жалобы удалён".to_owned()),
                        company: match (r.company_id, r.company_name) {
                            (Some(id), Some(name)) => Some(company_ref(&companies, id, name)),
                            _ => None,
                        },
                        reasons: vec![reason_ru.to_owned()],
                        vacancy_status: None,
                        salary: None,
                        open_complaints: 0,
                        complaint: Some(ComplaintView {
                            target_kind: target,
                            target_id: r.target_id,
                            reason: r.reason,
                            reason_ru: reason_ru.to_owned(),
                            comment: r.comment,
                        }),
                        until: None,
                        actions,
                    });
                }
            }
        }
        TrustView::Sanctions => {
            if wanted(q.kind, TrustItemKind::Vacancy) {
                for v in db::moderation::unpublished_vacancies(&state.pool, after, fetch).await? {
                    items.push(TrustItem {
                        kind: TrustItemKind::Vacancy,
                        id: v.id,
                        at: v.at,
                        title: v.title,
                        company: Some(company_ref(&companies, v.company_id, v.company_name)),
                        reasons: flag_reasons(v.trust_flags)?,
                        vacancy_status: Some(VacancyStatus::Closed),
                        salary: Some(SalaryRange {
                            min: v.salary_min,
                            max: v.salary_max,
                        }),
                        open_complaints: 0,
                        complaint: None,
                        until: None,
                        actions: vec![TrustAction::Restore],
                    });
                }
            }
            if wanted(q.kind, TrustItemKind::Company) {
                for c in db::moderation::frozen_companies(&state.pool, after, fetch).await? {
                    items.push(TrustItem {
                        kind: TrustItemKind::Company,
                        id: c.id,
                        at: c.at,
                        title: c.name.clone(),
                        company: Some(company_ref(&companies, c.id, c.name)),
                        reasons: Vec::new(),
                        vacancy_status: None,
                        salary: None,
                        open_complaints: 0,
                        complaint: None,
                        until: Some(c.until),
                        actions: vec![TrustAction::Restore],
                    });
                }
            }
        }
    }
    items.sort_by_key(|i| (i.at, i.id));
    let more = items.len() as i64 > limit;
    items.truncate(limit as usize);
    let next_cursor = more
        .then(|| items.last().map(|i| encode_cursor(i.at, i.id)))
        .flatten();
    Ok(Json(TrustPage { items, next_cursor }))
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "kind": "company",
    "id": "019a6f00-1111-7222-8333-444455556666",
    "action": "freeze",
    "comment": "Три подтверждённые жалобы: после приглашения вилку занижают вдвое. Приглашения приостановлены до проверки.",
    "days": 14
}))]
pub struct TrustActionInput {
    pub kind: TrustItemKind,
    pub id: Uuid,
    pub action: TrustAction,
    /// Up to 2000 characters. The company sees it on its «Доверие и квота» page.
    pub comment: String,
    /// `freeze` only: 1–90 days.
    pub days: Option<u16>,
}

#[derive(Serialize, ToSchema)]
pub struct TrustActionResult {
    pub company_id: Option<Uuid>,
    /// Complaints this action closed: the one acted on and those it answers.
    pub complaints_resolved: u64,
    pub frozen_until: Option<DateTime<Utc>>,
}

/// Act on an anti-fake item: dismiss, take a vacancy down, freeze the company's invitations, restore
#[utoipa::path(post, path = "/antifake/actions", tag = "Модерация",
    request_body = TrustActionInput,
    responses(
        (status = 200, body = TrustActionResult),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
        (status = 404, description = "No such item", body = Problem),
        (status = 409, description = "`action_not_applicable`: nothing left to dismiss, the vacancy is not published, the complaint is closed, nothing to restore", body = Problem),
        (status = 422, description = "`validation_failed`: `comment`, `days`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn trust_act(
    State(state): State<AppState>,
    user: AdminUser,
    Json(input): Json<TrustActionInput>,
) -> ApiResult<Json<TrustActionResult>> {
    let comment = moderation::comment(&input.comment)?;
    let out = moderation::act(
        &state,
        user.id,
        input.kind,
        input.id,
        input.action,
        comment,
        input.days,
    )
    .await?;
    Ok(Json(TrustActionResult {
        company_id: out.company_id,
        complaints_resolved: out.complaints_resolved,
        frozen_until: out.frozen_until,
    }))
}

#[derive(Deserialize, IntoParams)]
pub struct LogQuery {
    #[param(inline)]
    pub action: Option<ModerationAction>,
    #[param(inline)]
    pub target_kind: Option<ModerationTarget>,
    /// Only this moderator's actions.
    pub moderator_id: Option<Uuid>,
    /// A session, vacancy, invitation, company or complaint, or a candidate or company the
    /// actions concern.
    pub subject: Option<Uuid>,
    pub cursor: Option<String>,
    /// 1–100, 30 by default.
    pub limit: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AuditEntry {
    pub id: Uuid,
    pub at: DateTime<Utc>,
    pub moderator_id: Uuid,
    /// The moderator's login email; `null` once the account is deleted.
    pub moderator: Option<String>,
    pub action: ModerationAction,
    pub target_kind: ModerationTarget,
    pub target_id: Uuid,
    /// What the target was called when the action happened: a title, a company, a category.
    pub target_label: Option<String>,
    pub candidate_code: Option<String>,
    pub company_name: Option<String>,
    pub comment: Option<String>,
    /// Specifics in words: the freeze term, complaints closed, the category restored.
    pub details: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AuditPage {
    pub items: Vec<AuditEntry>,
    pub next_cursor: Option<String>,
}

fn audit_details(action: ModerationAction, d: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(days) = d["days"].as_u64() {
        let until = d["until"]
            .as_str()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .map(|t| format!(", до {}", t.format("%d.%m.%Y")))
            .unwrap_or_default();
        out.push(format!("Срок: {days} дн.{until}"));
    }
    if let Some(n) = d["complaints_resolved"].as_u64().filter(|n| *n > 0) {
        out.push(format!("Закрыто жалоб: {n}"));
    }
    if matches!(
        action,
        ModerationAction::AnnulResult | ModerationAction::OrderRetest
    ) {
        out.push(match d["restored_category"].as_str() {
            Some(key) => format!(
                "Категория теперь: {}",
                moderation::category_name(key).unwrap_or_else(|| key.to_owned())
            ),
            None => "Категория снята".to_owned(),
        });
    }
    out
}

/// Audit log of moderation, newest first: decisions, anti-fake actions and recording views
#[utoipa::path(get, path = "/log", tag = "Модерация",
    params(LogQuery),
    responses(
        (status = 200, body = AuditPage),
        (status = 400, description = "`invalid_query`", body = Problem),
        (status = 403, description = "`wrong_role`: moderators only", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn audit_log(
    State(state): State<AppState>,
    _user: AdminUser,
    Query(q): Query<LogQuery>,
) -> ApiResult<Json<AuditPage>> {
    let before = moderation::decode_cursor(q.cursor.as_deref())?;
    let limit = moderation::page_limit(q.limit);
    let mut rows = db::moderation::actions(
        &state.pool,
        db::moderation::ActionFilter {
            action: q.action.map(Into::into),
            target_kind: q.target_kind.map(Into::into),
            moderator_id: q.moderator_id,
            subject: q.subject,
        },
        before,
        limit + 1,
    )
    .await?;
    let more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let next_cursor = more
        .then(|| rows.last().map(|r| encode_cursor(r.at, r.id)))
        .flatten();
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        let action = ModerationAction::from(r.action);
        let target_label = r.details["title"].as_str().map(str::to_owned).or_else(|| {
            r.details["category"]
                .as_str()
                .and_then(moderation::category_name)
        });
        items.push(AuditEntry {
            id: r.id,
            at: r.at,
            moderator_id: r.moderator_id,
            moderator: r
                .moderator_email_enc
                .as_deref()
                .map(|e| state.crypto.decrypt_str(e))
                .transpose()?,
            action,
            target_kind: r.target_kind.into(),
            target_id: r.target_id,
            target_label,
            candidate_code: r.candidate_code,
            company_name: r.company_name,
            comment: r.comment,
            details: audit_details(action, &r.details),
        });
    }
    Ok(Json(AuditPage { items, next_cursor }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_states() {
        assert_eq!(video_state(None, false), VideoState::Off);
        assert_eq!(video_state(Some(false), false), VideoState::Declined);
        assert_eq!(video_state(Some(true), false), VideoState::Kept);
        assert_eq!(video_state(Some(true), true), VideoState::Purged);
    }

    #[test]
    fn freeze_details_read_well() {
        let d = serde_json::json!({ "days": 14, "until": "2026-10-20T10:00:00Z", "complaints_resolved": 2 });
        assert_eq!(
            audit_details(ModerationAction::FreezeInvitations, &d),
            ["Срок: 14 дн., до 20.10.2026", "Закрыто жалоб: 2"]
        );
        let d = serde_json::json!({ "category": "backend:middle", "restored_category": null });
        assert_eq!(
            audit_details(ModerationAction::AnnulResult, &d),
            ["Категория снята"]
        );
    }
}
