use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{AssessmentStatus, ReviewDecision};

#[derive(Debug, Clone)]
pub struct SessionForUpdate {
    pub id: Uuid,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: AssessmentStatus,
    pub state: Value,
    pub current_item: Option<Value>,
    pub current_item_deadline: Option<DateTime<Utc>>,
    pub current_item_shown_at: Option<DateTime<Utc>>,
    pub items_count: i32,
}

#[derive(Debug, Clone)]
pub struct CandidateSession {
    pub id: Uuid,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: AssessmentStatus,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<String>,
    pub confident_above_next: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub items_count: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    /// Assigned the grade, and no moderator voided that assignment.
    pub assigned: bool,
    /// `None` when the session ran with proctoring off.
    pub camera: Option<bool>,
    pub review: Option<ReviewDecision>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub id: Uuid,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: AssessmentStatus,
    pub state: Value,
    pub current_item: Option<Value>,
    pub current_item_deadline: Option<DateTime<Utc>>,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<String>,
    pub confident_above_next: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub items_count: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub assigned: bool,
    /// The grade assignment of this session was voided by a moderator.
    pub annulled: bool,
    pub review: Option<ReviewDecision>,
    pub review_comment: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

pub struct NewSession<'a> {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub attempt_kind: &'a str,
    pub state: &'a Value,
    pub current_item: &'a Value,
    pub deadline: DateTime<Utc>,
    pub shown_at: DateTime<Utc>,
}

pub struct FinishedSession<'a> {
    pub status: AssessmentStatus,
    pub state: &'a Value,
    pub theta: f64,
    pub se: f64,
    pub measured_grade: Option<&'a str>,
    pub confident_above_next: bool,
    pub person_fit_lz: f64,
    pub aberrant: bool,
    pub items_count: i32,
    pub finished_at: DateTime<Utc>,
}

pub struct NewResponse<'a> {
    pub session_id: Uuid,
    pub seq: i32,
    pub template_id: &'a str,
    pub level: i16,
    pub seed: i64,
    pub answer: Option<&'a Value>,
    pub correct: bool,
    pub timed_out: bool,
    pub elapsed_ms: i32,
    pub theta_after: f64,
    pub se_after: f64,
    pub answered_at: DateTime<Utc>,
}

/// Closes the candidate's session left without an answer `abandon_after_secs` past the current
/// item's deadline. `finished_at` is the moment the rule closed it rather than the moment this
/// ran: the retry window after an abandoned test must not depend on when the candidate returns.
pub async fn expire_abandoned_sessions(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    abandon_after_secs: f64,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE assessment_sessions
         SET status = 'expired',
             finished_at = coalesce(current_item_deadline, started_at) + make_interval(secs => $2)
         WHERE candidate_id = $1 AND status = 'in_progress'
           AND coalesce(current_item_deadline, started_at) + make_interval(secs => $2) < now()",
        candidate_id,
        abandon_after_secs,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn active_session(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM assessment_sessions WHERE candidate_id = $1 AND status = 'in_progress'",
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_session(
    db: impl PgExecutor<'_>,
    s: &NewSession<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO assessment_sessions (id, candidate_id, specialization, claimed_grade, attempt_kind,
                                          state, current_item, current_item_deadline, current_item_shown_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        s.id,
        s.candidate_id,
        s.specialization,
        s.claimed_grade,
        s.attempt_kind,
        s.state,
        s.current_item,
        s.deadline,
        s.shown_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn session_for_update(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<Option<SessionForUpdate>, sqlx::Error> {
    sqlx::query_as!(
        SessionForUpdate,
        r#"SELECT id, specialization, claimed_grade, attempt_kind, status AS "status: AssessmentStatus",
                  state, current_item, current_item_deadline, current_item_shown_at, items_count
           FROM assessment_sessions
           WHERE id = $1 AND candidate_id = $2
           FOR UPDATE"#,
        id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn update_session_progress(
    db: impl PgExecutor<'_>,
    state: &Value,
    current_item: &Value,
    deadline: DateTime<Utc>,
    shown_at: DateTime<Utc>,
    items_count: i32,
    id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE assessment_sessions
         SET state = $1, current_item = $2, current_item_deadline = $3,
             current_item_shown_at = $4, items_count = $5
         WHERE id = $6",
        state,
        current_item,
        deadline,
        shown_at,
        items_count,
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn finish_session(
    db: impl PgExecutor<'_>,
    id: Uuid,
    s: &FinishedSession<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE assessment_sessions
         SET status = $1, state = $2, current_item = NULL, current_item_deadline = NULL,
             theta = $3, se = $4, measured_grade = $5, confident_above_next = $6,
             person_fit_lz = $7, aberrant = $8, items_count = $9, finished_at = $10
         WHERE id = $11",
        s.status as AssessmentStatus,
        s.state,
        s.theta,
        s.se,
        s.measured_grade,
        s.confident_above_next,
        s.person_fit_lz,
        s.aberrant,
        s.items_count,
        s.finished_at,
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_response(
    db: impl PgExecutor<'_>,
    resp: &NewResponse<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO assessment_responses (session_id, seq, template_id, level, seed, answer, correct,
                                           timed_out, elapsed_ms, theta_after, se_after, answered_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        resp.session_id,
        resp.seq,
        resp.template_id,
        resp.level,
        resp.seed,
        resp.answer,
        resp.correct,
        resp.timed_out,
        resp.elapsed_ms,
        resp.theta_after,
        resp.se_after,
        resp.answered_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn candidate_sessions(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CandidateSession>, sqlx::Error> {
    sqlx::query_as!(
        CandidateSession,
        r#"SELECT s.id, s.specialization, s.claimed_grade, s.attempt_kind,
                  s.status AS "status: AssessmentStatus", s.theta, s.se, s.measured_grade,
                  s.confident_above_next, s.person_fit_lz, s.aberrant, s.items_count, s.started_at,
                  s.finished_at,
                  EXISTS (SELECT 1 FROM grade_assignments g
                           WHERE g.session_id = s.id AND g.annulled_at IS NULL) AS "assigned!",
                  p.camera AS "camera?", r.decision AS "review?: ReviewDecision",
                  r.decided_at AS "reviewed_at?"
           FROM assessment_sessions s
           LEFT JOIN proctoring_sessions p ON p.session_id = s.id
           LEFT JOIN session_reviews r ON r.session_id = s.id
           WHERE s.candidate_id = $1
           ORDER BY s.started_at DESC"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn session_state(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<Option<SessionState>, sqlx::Error> {
    sqlx::query_as!(
        SessionState,
        r#"SELECT s.id, s.specialization, s.claimed_grade, s.attempt_kind,
                  s.status AS "status: AssessmentStatus", s.state, s.current_item,
                  s.current_item_deadline, s.theta, s.se, s.measured_grade, s.confident_above_next,
                  s.person_fit_lz, s.aberrant, s.items_count, s.started_at, s.finished_at,
                  EXISTS (SELECT 1 FROM grade_assignments g
                           WHERE g.session_id = s.id AND g.annulled_at IS NULL) AS "assigned!",
                  EXISTS (SELECT 1 FROM grade_assignments g
                           WHERE g.session_id = s.id AND g.annulled_at IS NOT NULL) AS "annulled!",
                  r.decision AS "review?: ReviewDecision", r.comment AS "review_comment?",
                  r.decided_at AS "reviewed_at?"
           FROM assessment_sessions s
           LEFT JOIN session_reviews r ON r.session_id = s.id
           WHERE s.id = $1 AND s.candidate_id = $2"#,
        id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}
