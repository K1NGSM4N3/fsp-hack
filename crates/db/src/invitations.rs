use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::InvitationStatus;

pub struct NewInvitation<'a> {
    pub id: Uuid,
    pub company_id: Uuid,
    pub candidate_id: Uuid,
    pub need_id: Option<Uuid>,
    pub vacancy_id: Option<Uuid>,
    pub title: &'a str,
    pub message: &'a str,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<&'a str>,
    pub contact: &'a str,
    pub idempotency_key: Option<&'a str>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct InvitationEvent {
    pub status: InvitationStatus,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EmployerInvitation {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub need_id: Option<Uuid>,
    pub vacancy_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub contact: String,
    pub status: InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CandidateInvitation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_description: String,
    pub company_industry: Option<String>,
    pub company_city: Option<String>,
    pub company_website: Option<String>,
    pub vacancy_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub contact: String,
    pub status: InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusCount {
    pub status: InvitationStatus,
    pub count: i64,
}

#[derive(Debug, Clone)]
pub struct DeclineReasonCount {
    pub reason: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct ResponseRate {
    pub answered: i64,
    pub total: i64,
}

pub async fn expire_stale_invitations(db: impl PgExecutor<'_>) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE invitations SET status = 'expired'
         WHERE status IN ('sent', 'viewed') AND expires_at < now()"
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_invitations_since(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    since: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM invitations WHERE company_id = $1 AND created_at >= $2"#,
        company_id,
        since,
    )
    .fetch_one(db)
    .await
}

pub async fn invitation_by_idempotency_key(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    idempotency_key: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM invitations WHERE company_id = $1 AND idempotency_key = $2",
        company_id,
        idempotency_key,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_invitation(
    db: impl PgExecutor<'_>,
    i: &NewInvitation<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO invitations (id, company_id, candidate_id, need_id, vacancy_id, title, message,
                                  salary_min, salary_max, work_format, contact, idempotency_key, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        i.id,
        i.company_id,
        i.candidate_id,
        i.need_id,
        i.vacancy_id,
        i.title,
        i.message,
        i.salary_min,
        i.salary_max,
        i.work_format,
        i.contact,
        i.idempotency_key,
        i.expires_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_invitation_event(
    db: impl PgExecutor<'_>,
    id: Uuid,
    invitation_id: Uuid,
    status: InvitationStatus,
    actor_user_id: Option<Uuid>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO invitation_events (id, invitation_id, status, actor_user_id)
         VALUES ($1, $2, $3, $4)",
        id,
        invitation_id,
        status as InvitationStatus,
        actor_user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn invitation_events(
    db: impl PgExecutor<'_>,
    invitation_id: Uuid,
) -> Result<Vec<InvitationEvent>, sqlx::Error> {
    sqlx::query_as!(
        InvitationEvent,
        r#"SELECT status AS "status: InvitationStatus", at
           FROM invitation_events
           WHERE invitation_id = $1
           ORDER BY at"#,
        invitation_id,
    )
    .fetch_all(db)
    .await
}

pub async fn employer_invitations(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<EmployerInvitation>, sqlx::Error> {
    sqlx::query_as!(
        EmployerInvitation,
        r#"SELECT i.id, i.candidate_id, c.public_code, c.specialization, c.grade, i.need_id,
                  i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format,
                  i.contact, i.status AS "status: InvitationStatus", i.decline_reason,
                  i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at
           FROM invitations i
           JOIN candidates c ON c.user_id = i.candidate_id
           WHERE i.company_id = $1
           ORDER BY i.created_at DESC"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn employer_invitation(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<Option<EmployerInvitation>, sqlx::Error> {
    sqlx::query_as!(
        EmployerInvitation,
        r#"SELECT i.id, i.candidate_id, c.public_code, c.specialization, c.grade, i.need_id,
                  i.vacancy_id, i.title, i.message, i.salary_min, i.salary_max, i.work_format,
                  i.contact, i.status AS "status: InvitationStatus", i.decline_reason,
                  i.decline_comment, i.created_at, i.viewed_at, i.responded_at, i.expires_at
           FROM invitations i
           JOIN candidates c ON c.user_id = i.candidate_id
           WHERE i.company_id = $1 AND i.id = $2"#,
        company_id,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn candidate_invitations(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CandidateInvitation>, sqlx::Error> {
    sqlx::query_as!(
        CandidateInvitation,
        r#"SELECT i.id, i.company_id, co.name AS company_name, co.description AS company_description,
                  co.industry AS company_industry, co.city AS company_city,
                  co.website AS company_website, i.vacancy_id, i.title, i.message, i.salary_min,
                  i.salary_max, i.work_format, i.contact, i.status AS "status: InvitationStatus",
                  i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at,
                  i.expires_at
           FROM invitations i
           JOIN companies co ON co.id = i.company_id
           WHERE i.candidate_id = $1
           ORDER BY i.created_at DESC"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn candidate_invitation(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    id: Uuid,
) -> Result<Option<CandidateInvitation>, sqlx::Error> {
    sqlx::query_as!(
        CandidateInvitation,
        r#"SELECT i.id, i.company_id, co.name AS company_name, co.description AS company_description,
                  co.industry AS company_industry, co.city AS company_city,
                  co.website AS company_website, i.vacancy_id, i.title, i.message, i.salary_min,
                  i.salary_max, i.work_format, i.contact, i.status AS "status: InvitationStatus",
                  i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at,
                  i.expires_at
           FROM invitations i
           JOIN companies co ON co.id = i.company_id
           WHERE i.candidate_id = $1 AND i.id = $2"#,
        candidate_id,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn mark_invitation_viewed(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE invitations SET status = 'viewed', viewed_at = now()
         WHERE id = $1 AND candidate_id = $2 AND status = 'sent'",
        id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn respond_invitation(
    db: impl PgExecutor<'_>,
    status: InvitationStatus,
    decline_reason: Option<&str>,
    decline_comment: Option<&str>,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE invitations
         SET status = $1, responded_at = now(), decline_reason = $2, decline_comment = $3
         WHERE id = $4 AND candidate_id = $5 AND status IN ('sent', 'viewed') AND expires_at >= now()",
        status as InvitationStatus,
        decline_reason,
        decline_comment,
        id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn withdraw_invitation(
    db: impl PgExecutor<'_>,
    id: Uuid,
    company_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE invitations SET status = 'withdrawn', responded_at = now()
         WHERE id = $1 AND company_id = $2 AND status IN ('sent', 'viewed')",
        id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_invitation_status_counts(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<StatusCount>, sqlx::Error> {
    sqlx::query_as!(
        StatusCount,
        r#"SELECT status AS "status: InvitationStatus", count(*) AS "count!"
           FROM invitations
           WHERE company_id = $1
           GROUP BY status"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn company_decline_reasons(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<DeclineReasonCount>, sqlx::Error> {
    sqlx::query_as!(
        DeclineReasonCount,
        r#"SELECT decline_reason AS reason, count(*) AS "count!"
           FROM invitations
           WHERE company_id = $1 AND status = 'declined'
           GROUP BY decline_reason
           ORDER BY count(*) DESC"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn company_response_rate(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<ResponseRate, sqlx::Error> {
    sqlx::query_as!(
        ResponseRate,
        r#"SELECT count(*) FILTER (WHERE status IN ('accepted', 'declined')) AS "answered!",
                  count(*) AS "total!"
           FROM invitations
           WHERE company_id = $1 AND status <> 'withdrawn'"#,
        company_id,
    )
    .fetch_one(db)
    .await
}
