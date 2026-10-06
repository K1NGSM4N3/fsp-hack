//! Moderation: decisions on disputed test results, anti-fake actions, complaint resolutions and
//! the audit log. Queues are keyset-paginated on `(time, id)`.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{
    AssessmentStatus, FlagReview, ModerationAction, ModerationTarget, ReviewDecision, VacancyStatus,
};

/// A keyset position: the sort time and the id that breaks ties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, Clone)]
pub struct QueueSession {
    pub session_id: Uuid,
    pub public_code: String,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: AssessmentStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub aberrant: Option<bool>,
    pub person_fit_lz: Option<f64>,
    pub camera: Option<bool>,
    pub purged: Option<bool>,
    pub left_test: i64,
    pub pastes: i64,
    pub fullscreen_exits: i64,
    pub technical: i64,
    pub assigned: bool,
    pub decision: Option<ReviewDecision>,
    pub decided_at: Option<DateTime<Utc>>,
}

/// Finished sessions oldest first. `filter`: `needs_review` — undecided and flagged by the rule
/// of `assessment::proctoring::needs_confirmation` (person-fit with behavioural events, or
/// person-fit alone when the session has no proctoring data); `remarks` — undecided with
/// behavioural events; anything else — every session recorded with the camera.
pub async fn proctoring_queue(
    db: impl PgExecutor<'_>,
    filter: &str,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<QueueSession>, sqlx::Error> {
    sqlx::query_as!(
        QueueSession,
        r#"SELECT s.id AS session_id, c.public_code, s.specialization, s.claimed_grade, s.attempt_kind,
                  s.status AS "status: AssessmentStatus", s.started_at,
                  s.finished_at AS "finished_at!", s.aberrant, s.person_fit_lz,
                  p.camera AS "camera?", (p.purged_at IS NOT NULL) AS "purged?",
                  ev.left_test AS "left_test!", ev.pastes AS "pastes!",
                  ev.fullscreen_exits AS "fullscreen_exits!", ev.technical AS "technical!",
                  EXISTS (SELECT 1 FROM grade_assignments g
                           WHERE g.session_id = s.id AND g.annulled_at IS NULL) AS "assigned!",
                  r.decision AS "decision?: ReviewDecision", r.decided_at AS "decided_at?"
           FROM assessment_sessions s
           JOIN candidates c ON c.user_id = s.candidate_id
           LEFT JOIN proctoring_sessions p ON p.session_id = s.id
           LEFT JOIN session_reviews r ON r.session_id = s.id
           CROSS JOIN LATERAL (
               SELECT count(*) FILTER (WHERE e.kind IN ('tab_hidden', 'window_blur')) AS left_test,
                      count(*) FILTER (WHERE e.kind = 'paste') AS pastes,
                      count(*) FILTER (WHERE e.kind = 'fullscreen_exit') AS fullscreen_exits,
                      count(*) FILTER (WHERE e.kind IN ('camera_lost', 'recording_failed')) AS technical
               FROM proctoring_events e
               WHERE e.session_id = s.id
           ) ev
           WHERE s.status IN ('passed', 'failed') AND s.finished_at IS NOT NULL
             AND ($2::timestamptz IS NULL OR (s.finished_at, s.id) > ($2, $3))
             AND CASE $1::text
                   WHEN 'needs_review' THEN r.session_id IS NULL AND coalesce(s.aberrant, false)
                        AND (p.session_id IS NULL OR ev.left_test + ev.pastes + ev.fullscreen_exits > 0)
                   WHEN 'remarks' THEN r.session_id IS NULL
                        AND ev.left_test + ev.pastes + ev.fullscreen_exits > 0
                   ELSE coalesce(p.camera, false)
                 END
           ORDER BY s.finished_at, s.id
           LIMIT $4"#,
        filter,
        after.map(|c| c.at),
        after.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone, Copy)]
pub struct Overview {
    pub needs_review: i64,
    pub remarks: i64,
    pub with_video: i64,
    pub decided_week: i64,
    pub open_complaints: i64,
    pub flagged_vacancies: i64,
    pub flagged_invitations: i64,
    pub unpublished_vacancies: i64,
    pub frozen_companies: i64,
    pub actions_week: i64,
    pub recording_views_week: i64,
}

/// Counters of the moderator's overview; the flagged-session rule matches [`proctoring_queue`].
pub async fn overview(db: impl PgExecutor<'_>) -> Result<Overview, sqlx::Error> {
    sqlx::query_as!(
        Overview,
        r#"WITH finished AS (
               SELECT s.aberrant, p.session_id IS NOT NULL AS proctored,
                      coalesce(p.camera, false) AS camera, r.session_id IS NOT NULL AS decided,
                      (SELECT count(*) FROM proctoring_events e
                        WHERE e.session_id = s.id
                          AND e.kind IN ('tab_hidden', 'window_blur', 'fullscreen_exit', 'paste')) AS behavioral
               FROM assessment_sessions s
               LEFT JOIN proctoring_sessions p ON p.session_id = s.id
               LEFT JOIN session_reviews r ON r.session_id = s.id
               WHERE s.status IN ('passed', 'failed') AND s.finished_at IS NOT NULL
           )
           SELECT
               (SELECT count(*) FROM finished
                 WHERE NOT decided AND coalesce(aberrant, false)
                   AND (NOT proctored OR behavioral > 0)) AS "needs_review!",
               (SELECT count(*) FROM finished WHERE NOT decided AND behavioral > 0) AS "remarks!",
               (SELECT count(*) FROM finished WHERE camera) AS "with_video!",
               (SELECT count(*) FROM session_reviews
                 WHERE decided_at > now() - interval '7 days') AS "decided_week!",
               (SELECT count(*) FROM reports WHERE resolved_at IS NULL) AS "open_complaints!",
               (SELECT count(*) FROM vacancies
                 WHERE flagged_at IS NOT NULL AND trust_reviewed_at IS NULL) AS "flagged_vacancies!",
               (SELECT count(*) FROM invitations
                 WHERE trust_flags <> '[]' AND trust_reviewed_at IS NULL) AS "flagged_invitations!",
               (SELECT count(*) FROM vacancies WHERE unpublished_at IS NOT NULL) AS "unpublished_vacancies!",
               (SELECT count(*) FROM companies
                 WHERE invitations_frozen_until > now()) AS "frozen_companies!",
               (SELECT count(*) FROM moderation_actions
                 WHERE at > now() - interval '7 days' AND action <> 'view_recording') AS "actions_week!",
               (SELECT count(*) FROM moderation_actions
                 WHERE at > now() - interval '7 days' AND action = 'view_recording') AS "recording_views_week!""#,
    )
    .fetch_one(db)
    .await
}

/// A session as the moderator reviews it, with the grade assignment it made and the decision.
#[derive(Debug, Clone)]
pub struct ReviewSession {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub public_code: String,
    pub specialization: String,
    pub claimed_grade: String,
    pub attempt_kind: String,
    pub status: AssessmentStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<String>,
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub current_specialization: Option<String>,
    pub current_grade: Option<String>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub annulled_at: Option<DateTime<Utc>>,
    /// The assignment is in force and no later one replaced it.
    pub assignment_current: bool,
    pub decision: Option<ReviewDecision>,
    pub decision_comment: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
}

pub async fn review_session(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<ReviewSession>, sqlx::Error> {
    sqlx::query_as!(
        ReviewSession,
        r#"SELECT s.id, s.candidate_id, c.public_code, s.specialization, s.claimed_grade,
                  s.attempt_kind, s.status AS "status: AssessmentStatus", s.started_at, s.finished_at,
                  s.theta, s.se, s.measured_grade, s.person_fit_lz, s.aberrant,
                  c.specialization AS current_specialization, c.grade AS current_grade,
                  g.assigned_at AS "assigned_at?", g.annulled_at AS "annulled_at?",
                  (g.id IS NOT NULL AND g.annulled_at IS NULL AND NOT EXISTS (
                       SELECT 1 FROM grade_assignments n
                        WHERE n.candidate_id = s.candidate_id AND n.annulled_at IS NULL
                          AND (n.assigned_at, n.id) > (g.assigned_at, g.id))) AS "assignment_current!",
                  r.decision AS "decision?: ReviewDecision", r.comment AS "decision_comment?",
                  r.decided_at AS "decided_at?"
           FROM assessment_sessions s
           JOIN candidates c ON c.user_id = s.candidate_id
           LEFT JOIN grade_assignments g ON g.session_id = s.id
           LEFT JOIN session_reviews r ON r.session_id = s.id
           WHERE s.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

#[derive(Debug, Clone)]
pub struct ResponseRow {
    pub seq: i32,
    pub template_id: String,
    pub level: i16,
    pub correct: bool,
    pub timed_out: bool,
    pub elapsed_ms: i32,
    pub answered_at: DateTime<Utc>,
    pub theta_after: f64,
}

pub async fn responses(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
) -> Result<Vec<ResponseRow>, sqlx::Error> {
    sqlx::query_as!(
        ResponseRow,
        "SELECT seq, template_id, level, correct, timed_out, elapsed_ms, answered_at, theta_after
         FROM assessment_responses
         WHERE session_id = $1
         ORDER BY seq",
        session_id,
    )
    .fetch_all(db)
    .await
}

/// When moderators played the session's recording, oldest first.
pub async fn recording_views(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
) -> Result<Vec<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT at FROM moderation_actions
         WHERE target_id = $1 AND target_kind = 'session' AND action = 'view_recording'
         ORDER BY at",
        session_id,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone, Copy)]
pub struct LockedSession {
    pub candidate_id: Uuid,
    pub status: AssessmentStatus,
}

/// Serializes decisions on one session.
pub async fn lock_session(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<LockedSession>, sqlx::Error> {
    sqlx::query_as!(
        LockedSession,
        r#"SELECT candidate_id, status AS "status: AssessmentStatus"
           FROM assessment_sessions WHERE id = $1 FOR UPDATE"#,
        id,
    )
    .fetch_optional(db)
    .await
}

/// 0 when the session already has a decision.
pub async fn insert_review(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    candidate_id: Uuid,
    decision: ReviewDecision,
    comment: &str,
    moderator_id: Uuid,
    decided_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO session_reviews (session_id, candidate_id, decision, comment, moderator_id, decided_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (session_id) DO NOTHING",
        session_id,
        candidate_id,
        decision as ReviewDecision,
        comment,
        moderator_id,
        decided_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Voids the session's grade assignment; 0 when it made none or it is void already.
pub async fn annul_assignment(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE grade_assignments SET annulled_at = $2
         WHERE session_id = $1 AND annulled_at IS NULL",
        session_id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

#[derive(Debug, Clone)]
pub struct LiveAssignment {
    pub specialization: String,
    pub grade: String,
    pub theta: f64,
    pub assigned_at: DateTime<Utc>,
}

pub async fn latest_live_assignment(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<LiveAssignment>, sqlx::Error> {
    sqlx::query_as!(
        LiveAssignment,
        "SELECT specialization, grade, theta, assigned_at
         FROM grade_assignments
         WHERE candidate_id = $1 AND annulled_at IS NULL
         ORDER BY assigned_at DESC, id DESC
         LIMIT 1",
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn clear_candidate_category(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates
         SET specialization = NULL, grade = NULL, theta = NULL, category_assigned_at = NULL,
             updated_at = now()
         WHERE user_id = $1",
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Templates answered correctly in the session.
pub async fn correct_templates(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT DISTINCT template_id FROM assessment_responses WHERE session_id = $1 AND correct",
        session_id,
    )
    .fetch_all(db)
    .await
}

/// Templates answered correctly in the candidate's sessions whose assignment is in force.
pub async fn live_correct_templates(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT DISTINCT r.template_id
         FROM assessment_responses r
         JOIN grade_assignments g ON g.session_id = r.session_id
         WHERE g.candidate_id = $1 AND g.annulled_at IS NULL AND r.correct",
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn delete_test_skills(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    skills: &[String],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM candidate_skills
         WHERE candidate_id = $1 AND source = 'test' AND skill = ANY ($2)",
        candidate_id,
        skills,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

#[derive(Debug, Clone)]
pub struct CameraOrder {
    pub specialization: String,
    pub grade: String,
    pub decided_at: DateTime<Utc>,
}

/// Retests the moderator ordered under the camera.
pub async fn camera_orders(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CameraOrder>, sqlx::Error> {
    sqlx::query_as!(
        CameraOrder,
        "SELECT s.specialization, s.claimed_grade AS grade, r.decided_at
         FROM session_reviews r
         JOIN assessment_sessions s ON s.id = r.session_id
         WHERE r.candidate_id = $1 AND r.decision = 'retest'",
        candidate_id,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct PendingVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub status: VacancyStatus,
    pub salary_min: i64,
    pub salary_max: i64,
    pub trust_flags: Value,
    pub at: DateTime<Utc>,
    pub complaints: i64,
}

/// Flagged vacancies nobody reviewed yet, oldest flag first.
pub async fn pending_vacancies(
    db: impl PgExecutor<'_>,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<PendingVacancy>, sqlx::Error> {
    sqlx::query_as!(
        PendingVacancy,
        r#"SELECT v.id, v.company_id, co.name AS company_name, v.title,
                  v.status AS "status: VacancyStatus", v.salary_min, v.salary_max, v.trust_flags,
                  v.flagged_at AS "at!",
                  (SELECT count(*) FROM reports r
                    WHERE r.target_kind = 'vacancy' AND r.target_id = v.id
                      AND r.resolved_at IS NULL) AS "complaints!"
           FROM vacancies v
           JOIN companies co ON co.id = v.company_id
           WHERE v.flagged_at IS NOT NULL AND v.trust_reviewed_at IS NULL
             AND ($1::timestamptz IS NULL OR (v.flagged_at, v.id) > ($1, $2))
           ORDER BY v.flagged_at, v.id
           LIMIT $3"#,
        after.map(|c| c.at),
        after.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct PendingInvitation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub trust_flags: Value,
    pub at: DateTime<Utc>,
}

pub async fn pending_invitations(
    db: impl PgExecutor<'_>,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<PendingInvitation>, sqlx::Error> {
    sqlx::query_as!(
        PendingInvitation,
        r#"SELECT i.id, i.company_id, co.name AS company_name, i.title, i.salary_min, i.salary_max,
                  i.trust_flags, i.created_at AS at
           FROM invitations i
           JOIN companies co ON co.id = i.company_id
           WHERE i.trust_flags <> '[]' AND i.trust_reviewed_at IS NULL
             AND ($1::timestamptz IS NULL OR (i.created_at, i.id) > ($1, $2))
           ORDER BY i.created_at, i.id
           LIMIT $3"#,
        after.map(|c| c.at),
        after.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct OpenComplaint {
    pub id: Uuid,
    pub target_kind: String,
    pub target_id: Uuid,
    pub reason: String,
    pub comment: Option<String>,
    pub at: DateTime<Utc>,
    /// The company behind the target; `None` when the target is gone.
    pub company_id: Option<Uuid>,
    pub company_name: Option<String>,
    pub target_title: Option<String>,
}

/// Open complaints, oldest first. The reporter stays out: the moderator judges the target.
pub async fn open_complaints(
    db: impl PgExecutor<'_>,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<OpenComplaint>, sqlx::Error> {
    sqlx::query_as!(
        OpenComplaint,
        r#"SELECT r.id, r.target_kind, r.target_id, r.reason, r.comment, r.created_at AS at,
                  co.id AS "company_id?", co.name AS "company_name?",
                  coalesce(v.title, i.title) AS target_title
           FROM reports r
           LEFT JOIN vacancies v ON r.target_kind = 'vacancy' AND v.id = r.target_id
           LEFT JOIN invitations i ON r.target_kind = 'invitation' AND i.id = r.target_id
           LEFT JOIN companies co ON co.id = CASE r.target_kind
                                               WHEN 'vacancy' THEN v.company_id
                                               WHEN 'invitation' THEN i.company_id
                                               ELSE r.target_id
                                             END
           WHERE r.resolved_at IS NULL
             AND ($1::timestamptz IS NULL OR (r.created_at, r.id) > ($1, $2))
           ORDER BY r.created_at, r.id
           LIMIT $3"#,
        after.map(|c| c.at),
        after.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct UnpublishedVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub trust_flags: Value,
    pub at: DateTime<Utc>,
}

pub async fn unpublished_vacancies(
    db: impl PgExecutor<'_>,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<UnpublishedVacancy>, sqlx::Error> {
    sqlx::query_as!(
        UnpublishedVacancy,
        r#"SELECT v.id, v.company_id, co.name AS company_name, v.title, v.salary_min, v.salary_max,
                  v.trust_flags, v.unpublished_at AS "at!"
           FROM vacancies v
           JOIN companies co ON co.id = v.company_id
           WHERE v.unpublished_at IS NOT NULL
             AND ($1::timestamptz IS NULL OR (v.unpublished_at, v.id) > ($1, $2))
           ORDER BY v.unpublished_at, v.id
           LIMIT $3"#,
        after.map(|c| c.at),
        after.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct FrozenCompany {
    pub id: Uuid,
    pub name: String,
    pub at: DateTime<Utc>,
    pub until: DateTime<Utc>,
}

pub async fn frozen_companies(
    db: impl PgExecutor<'_>,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<FrozenCompany>, sqlx::Error> {
    sqlx::query_as!(
        FrozenCompany,
        r#"SELECT id, name, invitations_frozen_at AS "at!", invitations_frozen_until AS "until!"
           FROM companies
           WHERE invitations_frozen_until > now() AND invitations_frozen_at IS NOT NULL
             AND ($1::timestamptz IS NULL OR (invitations_frozen_at, id) > ($1, $2))
           ORDER BY invitations_frozen_at, id
           LIMIT $3"#,
        after.map(|c| c.at),
        after.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct VacancyTarget {
    pub company_id: Uuid,
    pub title: String,
    pub status: VacancyStatus,
    pub flagged: bool,
    pub reviewed: bool,
    pub unpublished: bool,
}

pub async fn vacancy_target(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<VacancyTarget>, sqlx::Error> {
    sqlx::query_as!(
        VacancyTarget,
        r#"SELECT company_id, title, status AS "status: VacancyStatus",
                  flagged_at IS NOT NULL AS "flagged!", trust_reviewed_at IS NOT NULL AS "reviewed!",
                  unpublished_at IS NOT NULL AS "unpublished!"
           FROM vacancies WHERE id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

#[derive(Debug, Clone)]
pub struct InvitationTarget {
    pub company_id: Uuid,
    pub title: String,
    pub flagged: bool,
    pub reviewed: bool,
}

pub async fn invitation_target(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<InvitationTarget>, sqlx::Error> {
    sqlx::query_as!(
        InvitationTarget,
        r#"SELECT company_id, title, trust_flags <> '[]' AS "flagged!",
                  trust_reviewed_at IS NOT NULL AS "reviewed!"
           FROM invitations WHERE id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

#[derive(Debug, Clone)]
pub struct CompanyTarget {
    pub name: String,
    pub frozen_until: Option<DateTime<Utc>>,
}

pub async fn company_target(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<CompanyTarget>, sqlx::Error> {
    sqlx::query_as!(
        CompanyTarget,
        r#"SELECT name, CASE WHEN invitations_frozen_until > now() THEN invitations_frozen_until END
                        AS frozen_until
           FROM companies WHERE id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

#[derive(Debug, Clone)]
pub struct ComplaintTarget {
    pub target_kind: String,
    pub target_id: Uuid,
    pub resolved: bool,
    pub company_id: Option<Uuid>,
}

pub async fn complaint_target(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<ComplaintTarget>, sqlx::Error> {
    sqlx::query_as!(
        ComplaintTarget,
        r#"SELECT r.target_kind, r.target_id, r.resolved_at IS NOT NULL AS "resolved!",
                  CASE r.target_kind
                      WHEN 'vacancy' THEN (SELECT v.company_id FROM vacancies v WHERE v.id = r.target_id)
                      WHEN 'invitation' THEN (SELECT i.company_id FROM invitations i WHERE i.id = r.target_id)
                      ELSE (SELECT co.id FROM companies co WHERE co.id = r.target_id)
                  END AS company_id
           FROM reports r WHERE r.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

/// Marks the vacancy's flags reviewed: `dismissed` stops them counting, `upheld` keeps them.
/// 0 when there is nothing unreviewed.
pub async fn review_vacancy_flags(
    db: impl PgExecutor<'_>,
    id: Uuid,
    review: FlagReview,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE vacancies SET trust_review = $2, trust_reviewed_at = $3
         WHERE id = $1 AND flagged_at IS NOT NULL AND trust_reviewed_at IS NULL",
        id,
        review as FlagReview,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn review_invitation_flags(
    db: impl PgExecutor<'_>,
    id: Uuid,
    review: FlagReview,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE invitations SET trust_review = $2, trust_reviewed_at = $3
         WHERE id = $1 AND trust_flags <> '[]' AND trust_reviewed_at IS NULL",
        id,
        review as FlagReview,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn dismiss_company(
    db: impl PgExecutor<'_>,
    id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE companies SET trust_reviewed_at = $2 WHERE id = $1",
        id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// `resolution`: `dismissed`, `vacancy_unpublished`, `invitations_frozen`. 0 when already closed.
pub async fn resolve_complaint(
    db: impl PgExecutor<'_>,
    id: Uuid,
    resolution: &str,
    comment: &str,
    moderator_id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE reports
         SET resolved_at = $5, resolution = $2, resolution_comment = $3, resolved_by = $4
         WHERE id = $1 AND resolved_at IS NULL",
        id,
        resolution,
        comment,
        moderator_id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Upholds every open complaint about the vacancy.
pub async fn uphold_vacancy_complaints(
    db: impl PgExecutor<'_>,
    vacancy_id: Uuid,
    resolution: &str,
    comment: &str,
    moderator_id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE reports
         SET resolved_at = $5, resolution = $2, resolution_comment = $3, resolved_by = $4
         WHERE target_kind = 'vacancy' AND target_id = $1 AND resolved_at IS NULL",
        vacancy_id,
        resolution,
        comment,
        moderator_id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Upholds every open complaint about the company itself and about its invitations: what a
/// freeze answers. Complaints about vacancies wait for their own decision.
pub async fn uphold_company_complaints(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    resolution: &str,
    comment: &str,
    moderator_id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE reports
         SET resolved_at = $5, resolution = $2, resolution_comment = $3, resolved_by = $4
         WHERE resolved_at IS NULL
           AND ((target_kind = 'company' AND target_id = $1)
             OR (target_kind = 'invitation'
                 AND target_id IN (SELECT id FROM invitations WHERE company_id = $1)))",
        company_id,
        resolution,
        comment,
        moderator_id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Only a published vacancy can be taken down; 0 otherwise. Its flags, if any, are upheld.
pub async fn unpublish_vacancy(
    db: impl PgExecutor<'_>,
    id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE vacancies
         SET status = 'closed', unpublished_at = $2, updated_at = now(),
             trust_review = CASE WHEN flagged_at IS NULL THEN trust_review ELSE 'upheld' END,
             trust_reviewed_at = CASE WHEN flagged_at IS NULL THEN trust_reviewed_at ELSE $2 END
         WHERE id = $1 AND status = 'published' AND unpublished_at IS NULL",
        id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Lifts the take-down and publishes the vacancy again: the moderator found it fine after all,
/// so its flags stop counting.
pub async fn restore_vacancy(
    db: impl PgExecutor<'_>,
    id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE vacancies
         SET status = 'published', unpublished_at = NULL, updated_at = now(),
             published_at = coalesce(published_at, now()),
             trust_review = CASE WHEN flagged_at IS NULL THEN trust_review ELSE 'dismissed' END,
             trust_reviewed_at = CASE WHEN flagged_at IS NULL THEN trust_reviewed_at ELSE $2 END
         WHERE id = $1 AND unpublished_at IS NOT NULL",
        id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn freeze_company(
    db: impl PgExecutor<'_>,
    id: Uuid,
    at: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE companies SET invitations_frozen_at = $2, invitations_frozen_until = $3
         WHERE id = $1",
        id,
        at,
        until,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// 0 when the company is not frozen.
pub async fn unfreeze_company(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE companies SET invitations_frozen_at = NULL, invitations_frozen_until = NULL
         WHERE id = $1 AND invitations_frozen_until > now()",
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub struct NewAction<'a> {
    pub id: Uuid,
    pub moderator_id: Uuid,
    pub action: ModerationAction,
    pub target_kind: ModerationTarget,
    pub target_id: Uuid,
    pub candidate_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub comment: Option<&'a str>,
    pub details: &'a Value,
    pub at: DateTime<Utc>,
}

pub async fn insert_action(db: impl PgExecutor<'_>, a: &NewAction<'_>) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO moderation_actions (id, moderator_id, action, target_kind, target_id,
                                         candidate_id, company_id, comment, details, at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        a.id,
        a.moderator_id,
        a.action as ModerationAction,
        a.target_kind as ModerationTarget,
        a.target_id,
        a.candidate_id,
        a.company_id,
        a.comment,
        a.details,
        a.at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

#[derive(Debug, Clone)]
pub struct ActionRow {
    pub id: Uuid,
    pub moderator_id: Uuid,
    pub moderator_email_enc: Option<Vec<u8>>,
    pub action: ModerationAction,
    pub target_kind: ModerationTarget,
    pub target_id: Uuid,
    pub candidate_code: Option<String>,
    pub company_id: Option<Uuid>,
    pub company_name: Option<String>,
    pub comment: Option<String>,
    pub details: Value,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ActionFilter {
    pub action: Option<ModerationAction>,
    pub target_kind: Option<ModerationTarget>,
    pub moderator_id: Option<Uuid>,
    /// The target itself, or the candidate or company an action concerns.
    pub subject: Option<Uuid>,
}

/// The audit log, newest first.
pub async fn actions(
    db: impl PgExecutor<'_>,
    f: ActionFilter,
    before: Option<Cursor>,
    limit: i64,
) -> Result<Vec<ActionRow>, sqlx::Error> {
    sqlx::query_as!(
        ActionRow,
        r#"SELECT a.id, a.moderator_id, u.email_enc AS "moderator_email_enc?",
                  a.action AS "action: ModerationAction", a.target_kind AS "target_kind: ModerationTarget",
                  a.target_id, c.public_code AS "candidate_code?", a.company_id,
                  co.name AS "company_name?", a.comment, a.details, a.at
           FROM moderation_actions a
           LEFT JOIN users u ON u.id = a.moderator_id
           LEFT JOIN candidates c ON c.user_id = a.candidate_id
           LEFT JOIN companies co ON co.id = a.company_id
           WHERE ($1::moderation_action IS NULL OR a.action = $1)
             AND ($2::moderation_target IS NULL OR a.target_kind = $2)
             AND ($3::uuid IS NULL OR a.moderator_id = $3)
             AND ($4::uuid IS NULL OR a.target_id = $4 OR a.candidate_id = $4 OR a.company_id = $4)
             AND ($5::timestamptz IS NULL OR (a.at, a.id) < ($5, $6))
           ORDER BY a.at DESC, a.id DESC
           LIMIT $7"#,
        f.action as Option<ModerationAction>,
        f.target_kind as Option<ModerationTarget>,
        f.moderator_id,
        f.subject,
        before.map(|c| c.at),
        before.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct CompanyOutcome {
    pub action: ModerationAction,
    pub target_kind: ModerationTarget,
    pub comment: Option<String>,
    pub details: Value,
    pub at: DateTime<Utc>,
}

/// Decisions that changed something for the company, newest first. Taking the company off the
/// queue changes nothing for it and stays internal.
pub async fn company_outcomes(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    limit: i64,
) -> Result<Vec<CompanyOutcome>, sqlx::Error> {
    sqlx::query_as!(
        CompanyOutcome,
        r#"SELECT action AS "action: ModerationAction", target_kind AS "target_kind: ModerationTarget",
                  comment, details, at
           FROM moderation_actions
           WHERE company_id = $1 AND action <> 'view_recording'
             AND NOT (action = 'dismiss' AND target_kind = 'company')
           ORDER BY at DESC, id DESC
           LIMIT $2"#,
        company_id,
        limit,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct CandidateReview {
    pub session_id: Uuid,
    pub decision: ReviewDecision,
    pub comment: String,
    pub decided_at: DateTime<Utc>,
}

/// The candidate's decisions and the moments a moderator played their recordings: what the
/// data export (152-ФЗ, ст. 14) lists next to the tests.
pub async fn candidate_reviews(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CandidateReview>, sqlx::Error> {
    sqlx::query_as!(
        CandidateReview,
        r#"SELECT session_id, decision AS "decision: ReviewDecision", comment, decided_at
           FROM session_reviews WHERE candidate_id = $1 ORDER BY decided_at"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone, Copy)]
pub struct RecordingView {
    pub session_id: Uuid,
    pub at: DateTime<Utc>,
}

pub async fn candidate_recording_views(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<RecordingView>, sqlx::Error> {
    sqlx::query_as!(
        RecordingView,
        r#"SELECT target_id AS session_id, at
           FROM moderation_actions
           WHERE candidate_id = $1 AND action = 'view_recording'
           ORDER BY at"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}
