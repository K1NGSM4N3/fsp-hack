//! ATS integration: outbox of integration events, webhook endpoints and delivery log, company
//! API keys, hiring outcomes and the snapshots event payloads are built from.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{
    ApplicationStatus, Cursor, HiringStage, IntegrationEventStatus, InvitationStatus,
};

#[derive(Debug, Clone)]
pub struct Webhook {
    pub url: String,
    pub secret_enc: Vec<u8>,
    pub secret_hint: String,
    pub prev_secret_enc: Option<Vec<u8>>,
    pub prev_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub secret_rotated_at: DateTime<Utc>,
}

pub struct NewEvent<'a> {
    pub id: Uuid,
    pub company_id: Uuid,
    pub candidate_id: Option<Uuid>,
    pub event_type: &'a str,
    pub payload_enc: &'a [u8],
    pub next_attempt_at: DateTime<Utc>,
    pub deadline_at: DateTime<Utc>,
}

/// An event leased for delivery together with the company's current endpoint, if any.
#[derive(Debug, Clone)]
pub struct DeliverableEvent {
    pub id: Uuid,
    pub company_id: Uuid,
    pub event_type: String,
    pub payload_enc: Vec<u8>,
    pub attempts: i16,
    pub deadline_at: DateTime<Utc>,
    pub url: Option<String>,
    pub secret_enc: Option<Vec<u8>>,
    pub prev_secret_enc: Option<Vec<u8>>,
    pub prev_expires_at: Option<DateTime<Utc>>,
}

pub struct NewAttempt<'a> {
    pub id: Uuid,
    pub event_id: Uuid,
    pub url: &'a str,
    pub status_code: Option<i16>,
    pub latency_ms: i32,
    pub error: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct EventRow {
    pub id: Uuid,
    pub event_type: String,
    pub status: IntegrationEventStatus,
    pub attempts: i16,
    pub created_at: DateTime<Utc>,
    pub next_attempt_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttemptRow {
    pub event_id: Uuid,
    pub url: String,
    pub status_code: Option<i16>,
    pub latency_ms: i32,
    pub error: Option<String>,
    pub attempted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct EventCounts {
    pub pending: i64,
    pub failed: i64,
    pub delivered: i64,
}

pub struct NewApiKey<'a> {
    pub id: Uuid,
    pub company_id: Uuid,
    pub name: &'a str,
    pub prefix: &'a str,
    pub key_hash: &'a [u8],
    pub scopes: &'a [String],
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyRow {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// An active key resolved from its hash, with the company owner on whose behalf it acts.
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub id: Uuid,
    pub company_id: Uuid,
    pub owner_user_id: Uuid,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InvitationSnapshot {
    pub id: Uuid,
    pub company_id: Uuid,
    pub owner_user_id: Uuid,
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
    pub status: InvitationStatus,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub webhook_on: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationSnapshot {
    pub id: Uuid,
    pub company_id: Uuid,
    pub owner_user_id: Uuid,
    pub candidate_id: Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub status: ApplicationStatus,
    pub cover_letter: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub webhook_on: bool,
}

#[derive(Debug, Clone)]
pub struct CandidateBrief {
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExpiredInvitation {
    pub id: Uuid,
    pub company_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct OutcomeSubject {
    pub candidate_id: Uuid,
    pub open: bool,
}

pub struct NewOutcome<'a> {
    pub id: Uuid,
    pub company_id: Uuid,
    pub candidate_id: Uuid,
    pub invitation_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub stage: HiringStage,
    pub reason: Option<&'a str>,
    pub occurred_at: DateTime<Utc>,
    pub api_key_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct StoredOutcome {
    pub id: Uuid,
    pub inserted: bool,
}

#[derive(Debug, Clone)]
pub struct OutcomeRow {
    pub invitation_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub stage: HiringStage,
    pub reason: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct StageCount {
    pub stage: HiringStage,
    pub count: i64,
}

pub async fn webhook(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Option<Webhook>, sqlx::Error> {
    sqlx::query_as!(
        Webhook,
        "SELECT url, secret_enc, secret_hint, prev_secret_enc, prev_expires_at, created_at,
                updated_at, secret_rotated_at
         FROM company_webhooks
         WHERE company_id = $1",
        company_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_webhook(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    url: &str,
    secret_enc: &[u8],
    secret_hint: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO company_webhooks (company_id, url, secret_enc, secret_hint)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (company_id) DO NOTHING",
        company_id,
        url,
        secret_enc,
        secret_hint,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn update_webhook_url(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    url: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE company_webhooks SET url = $2, updated_at = now() WHERE company_id = $1",
        company_id,
        url,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn rotate_webhook_secret(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    secret_enc: &[u8],
    secret_hint: &str,
    prev_expires_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE company_webhooks
         SET prev_secret_enc = secret_enc, prev_expires_at = $4, secret_enc = $2, secret_hint = $3,
             secret_rotated_at = now(), updated_at = now()
         WHERE company_id = $1",
        company_id,
        secret_enc,
        secret_hint,
        prev_expires_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn delete_webhook(db: impl PgExecutor<'_>, company_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM company_webhooks WHERE company_id = $1",
        company_id
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn webhook_enabled(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM company_webhooks WHERE company_id = $1) AS "on!""#,
        company_id,
    )
    .fetch_one(db)
    .await
}

pub async fn insert_event(db: impl PgExecutor<'_>, e: &NewEvent<'_>) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO integration_events (id, company_id, candidate_id, event_type, payload_enc,
                                         next_attempt_at, deadline_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        e.id,
        e.company_id,
        e.candidate_id,
        e.event_type,
        e.payload_enc,
        e.next_attempt_at,
        e.deadline_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Leases up to `limit` due events by pushing `next_attempt_at` forward by `lease_secs`.
/// `SKIP LOCKED` lets several API replicas poll the same table without picking the same row;
/// the lease keeps a crashed replica's events from being lost — they become due again.
pub async fn claim_due_events(
    db: impl PgExecutor<'_>,
    limit: i64,
    lease_secs: f64,
) -> Result<Vec<DeliverableEvent>, sqlx::Error> {
    sqlx::query_as!(
        DeliverableEvent,
        r#"WITH due AS (
               SELECT ie.id, w.url, w.secret_enc, w.prev_secret_enc, w.prev_expires_at
               FROM integration_events ie
               LEFT JOIN company_webhooks w ON w.company_id = ie.company_id
               WHERE ie.status = 'pending' AND ie.next_attempt_at <= now()
               ORDER BY ie.next_attempt_at
               LIMIT $1
               FOR UPDATE OF ie SKIP LOCKED
           )
           UPDATE integration_events e
           SET next_attempt_at = now() + make_interval(secs => $2)
           FROM due
           WHERE e.id = due.id
           RETURNING e.id, e.company_id, e.event_type, e.payload_enc, e.attempts, e.deadline_at,
                     due.url AS "url?", due.secret_enc AS "secret_enc?",
                     due.prev_secret_enc AS "prev_secret_enc?",
                     due.prev_expires_at AS "prev_expires_at?""#,
        limit,
        lease_secs,
    )
    .fetch_all(db)
    .await
}

pub async fn deliverable_event(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<DeliverableEvent>, sqlx::Error> {
    sqlx::query_as!(
        DeliverableEvent,
        r#"SELECT ie.id, ie.company_id, ie.event_type, ie.payload_enc, ie.attempts, ie.deadline_at,
                  w.url AS "url?", w.secret_enc AS "secret_enc?",
                  w.prev_secret_enc AS "prev_secret_enc?", w.prev_expires_at AS "prev_expires_at?"
           FROM integration_events ie
           LEFT JOIN company_webhooks w ON w.company_id = ie.company_id
           WHERE ie.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_attempt(
    db: impl PgExecutor<'_>,
    a: &NewAttempt<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO webhook_deliveries (id, event_id, url, status_code, latency_ms, error)
         VALUES ($1, $2, $3, $4, $5, $6)",
        a.id,
        a.event_id,
        a.url,
        a.status_code,
        a.latency_ms,
        a.error,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn mark_delivered(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE integration_events
         SET status = 'delivered', delivered_at = now(), last_error = NULL
         WHERE id = $1",
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn schedule_retry(
    db: impl PgExecutor<'_>,
    id: Uuid,
    next_attempt_at: DateTime<Utc>,
    error: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE integration_events
         SET attempts = attempts + 1, next_attempt_at = $2, last_error = $3
         WHERE id = $1 AND status = 'pending'",
        id,
        next_attempt_at,
        error,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn mark_failed(
    db: impl PgExecutor<'_>,
    id: Uuid,
    error: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE integration_events
         SET status = 'failed', attempts = attempts + 1, last_error = $2
         WHERE id = $1 AND status = 'pending'",
        id,
        error,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// «Отправить ещё раз»: back to the queue right now with a fresh 24-hour window.
pub async fn retry_event(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
    deadline_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE integration_events
         SET status = 'pending', attempts = 0, next_attempt_at = now(), deadline_at = $3,
             delivered_at = NULL, last_error = NULL
         WHERE company_id = $1 AND id = $2",
        company_id,
        id,
        deadline_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn cancel_pending_events(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    reason: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE integration_events SET status = 'failed', last_error = $2
         WHERE company_id = $1 AND status = 'pending'",
        company_id,
        reason,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// The delivery log, newest first, keyset-paginated on `(created_at, id)`.
pub async fn company_events(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    status: Option<IntegrationEventStatus>,
    before: Option<Cursor>,
    limit: i64,
) -> Result<Vec<EventRow>, sqlx::Error> {
    sqlx::query_as!(
        EventRow,
        r#"SELECT id, event_type, status AS "status: IntegrationEventStatus", attempts, created_at,
                  next_attempt_at, delivered_at, last_error
           FROM integration_events
           WHERE company_id = $1
             AND ($2::integration_event_status IS NULL OR status = $2)
             AND ($3::timestamptz IS NULL OR (created_at, id) < ($3, $4::uuid))
           ORDER BY created_at DESC, id DESC
           LIMIT $5"#,
        company_id,
        status as Option<IntegrationEventStatus>,
        before.map(|c| c.at),
        before.map(|c| c.id),
        limit,
    )
    .fetch_all(db)
    .await
}

pub async fn event_attempts(
    db: impl PgExecutor<'_>,
    event_ids: &[Uuid],
) -> Result<Vec<AttemptRow>, sqlx::Error> {
    sqlx::query_as!(
        AttemptRow,
        "SELECT event_id, url, status_code, latency_ms, error, attempted_at
         FROM webhook_deliveries
         WHERE event_id = ANY($1)
         ORDER BY event_id, attempted_at",
        event_ids,
    )
    .fetch_all(db)
    .await
}

pub async fn event_counts(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<EventCounts, sqlx::Error> {
    sqlx::query_as!(
        EventCounts,
        r#"SELECT count(*) FILTER (WHERE status = 'pending') AS "pending!",
                  count(*) FILTER (WHERE status = 'failed') AS "failed!",
                  count(*) FILTER (WHERE status = 'delivered') AS "delivered!"
           FROM integration_events
           WHERE company_id = $1"#,
        company_id,
    )
    .fetch_one(db)
    .await
}

/// Payloads may carry contacts: settled events are kept only as long as the delivery log is useful.
pub async fn purge_settled_events(
    db: impl PgExecutor<'_>,
    before: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM integration_events WHERE status <> 'pending' AND created_at < $1",
        before,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Outbox events still to be delivered, for the `webhook_outbox_pending` gauge; reads only the
/// partial index of pending events.
pub async fn pending_events(db: impl PgExecutor<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "pending!" FROM integration_events WHERE status = 'pending'"#
    )
    .fetch_one(db)
    .await
}

pub async fn insert_api_key(
    db: impl PgExecutor<'_>,
    k: &NewApiKey<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO company_api_keys (id, company_id, name, prefix, key_hash, scopes, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (key_hash) DO NOTHING",
        k.id,
        k.company_id,
        k.name,
        k.prefix,
        k.key_hash,
        k.scopes,
        k.created_by,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_api_keys(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<ApiKeyRow>, sqlx::Error> {
    sqlx::query_as!(
        ApiKeyRow,
        "SELECT id, name, prefix, scopes, created_at, last_used_at, revoked_at
         FROM company_api_keys
         WHERE company_id = $1
         ORDER BY revoked_at IS NOT NULL, created_at DESC",
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn active_api_keys(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM company_api_keys
           WHERE company_id = $1 AND revoked_at IS NULL"#,
        company_id,
    )
    .fetch_one(db)
    .await
}

pub async fn revoke_api_key(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE company_api_keys SET revoked_at = now()
         WHERE company_id = $1 AND id = $2 AND revoked_at IS NULL",
        company_id,
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn api_key_by_hash(
    db: impl PgExecutor<'_>,
    key_hash: &[u8],
) -> Result<Option<ApiKeyAuth>, sqlx::Error> {
    sqlx::query_as!(
        ApiKeyAuth,
        "SELECT k.id, k.company_id, c.owner_user_id, k.scopes
         FROM company_api_keys k
         JOIN companies c ON c.id = k.company_id
         WHERE k.key_hash = $1 AND k.revoked_at IS NULL",
        key_hash,
    )
    .fetch_optional(db)
    .await
}

/// At most one write a minute per key: «last used» does not need to be exact.
pub async fn touch_api_key(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE company_api_keys SET last_used_at = now()
         WHERE id = $1 AND (last_used_at IS NULL OR last_used_at < now() - interval '1 minute')",
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn invitation_snapshot(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<InvitationSnapshot>, sqlx::Error> {
    sqlx::query_as!(
        InvitationSnapshot,
        r#"SELECT i.id, i.company_id, co.owner_user_id, i.candidate_id, c.public_code,
                  c.specialization, c.grade, i.need_id, i.vacancy_id, i.title, i.message,
                  i.salary_min, i.salary_max, i.work_format, i.status AS "status: InvitationStatus",
                  i.decline_reason, i.decline_comment, i.created_at, i.viewed_at, i.responded_at,
                  i.expires_at,
                  greatest(i.created_at, i.viewed_at, i.responded_at,
                           CASE WHEN i.status = 'expired' THEN i.expires_at END) AS "updated_at!",
                  EXISTS (SELECT 1 FROM company_webhooks w WHERE w.company_id = i.company_id)
                      AS "webhook_on!"
           FROM invitations i
           JOIN candidates c ON c.user_id = i.candidate_id
           JOIN companies co ON co.id = i.company_id
           WHERE i.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn application_snapshot(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<ApplicationSnapshot>, sqlx::Error> {
    sqlx::query_as!(
        ApplicationSnapshot,
        r#"SELECT a.id, v.company_id, co.owner_user_id, a.candidate_id, c.public_code,
                  c.specialization, c.grade, a.vacancy_id, v.title AS vacancy_title,
                  a.status AS "status: ApplicationStatus", a.cover_letter, a.created_at,
                  a.updated_at,
                  EXISTS (SELECT 1 FROM company_webhooks w WHERE w.company_id = v.company_id)
                      AS "webhook_on!"
           FROM applications a
           JOIN vacancies v ON v.id = a.vacancy_id
           JOIN companies co ON co.id = v.company_id
           JOIN candidates c ON c.user_id = a.candidate_id
           WHERE a.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn application_status(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<Option<ApplicationStatus>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT a.status AS "status: ApplicationStatus"
           FROM applications a
           JOIN vacancies v ON v.id = a.vacancy_id
           WHERE a.id = $1 AND v.company_id = $2"#,
        id,
        company_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn candidate_brief(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<CandidateBrief>, sqlx::Error> {
    sqlx::query_as!(
        CandidateBrief,
        "SELECT public_code, specialization, grade FROM candidates WHERE user_id = $1",
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

/// Same rule as `invitations::expire_stale_invitations`, returning what changed so the caller
/// can write `invitation.expired` events in the same transaction.
pub async fn expire_stale_invitations(
    db: impl PgExecutor<'_>,
) -> Result<Vec<ExpiredInvitation>, sqlx::Error> {
    sqlx::query_as!(
        ExpiredInvitation,
        "UPDATE invitations SET status = 'expired'
         WHERE status IN ('sent', 'viewed') AND expires_at < now()
         RETURNING id, company_id",
    )
    .fetch_all(db)
    .await
}

/// Polling alternative to webhooks: the company's invitations whose status changed after the
/// cursor, oldest first.
///
/// Status changes are stamped with `now()` of their transaction, that is its start, and become
/// visible only at commit. A transaction that started earlier but commits after a poll would land
/// behind a cursor that already moved past it, so the page stops at a horizon: `lag_secs` before
/// now, and never later than the start of the oldest transaction of this database that is still
/// writing. Rows past the horizon come on the next poll.
pub async fn polled_invitations(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    status: Option<InvitationStatus>,
    after: Option<Cursor>,
    lag_secs: f64,
    limit: i64,
) -> Result<Vec<InvitationSnapshot>, sqlx::Error> {
    sqlx::query_as!(
        InvitationSnapshot,
        r#"SELECT id, company_id, owner_user_id, candidate_id, public_code, specialization, grade,
                  need_id, vacancy_id, title, message, salary_min, salary_max, work_format,
                  status AS "status: InvitationStatus", decline_reason, decline_comment,
                  created_at, viewed_at, responded_at, expires_at, updated_at AS "updated_at!",
                  true AS "webhook_on!"
           FROM (
               SELECT i.*, co.owner_user_id, c.public_code, c.specialization, c.grade,
                      greatest(i.created_at, i.viewed_at, i.responded_at,
                               CASE WHEN i.status = 'expired' THEN i.expires_at END) AS updated_at
               FROM invitations i
               JOIN candidates c ON c.user_id = i.candidate_id
               JOIN companies co ON co.id = i.company_id
               WHERE i.company_id = $1
           ) t
           WHERE ($2::invitation_status IS NULL OR status = $2)
             AND ($3::timestamptz IS NULL OR (updated_at, id) > ($3, $4::uuid))
             AND updated_at < least(
                 now() - make_interval(secs => $5),
                 (SELECT min(xact_start) FROM pg_stat_activity
                  WHERE datname = current_database() AND backend_xid IS NOT NULL
                    AND pid <> pg_backend_pid()))
           ORDER BY updated_at, id
           LIMIT $6"#,
        company_id,
        status as Option<InvitationStatus>,
        after.map(|c| c.at),
        after.map(|c| c.id),
        lag_secs,
        limit,
    )
    .fetch_all(db)
    .await
}

/// The invitation is the company's; `open` — accepted, so contacts are granted.
pub async fn invitation_outcome_subject(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<Option<OutcomeSubject>, sqlx::Error> {
    sqlx::query_as!(
        OutcomeSubject,
        r#"SELECT candidate_id, status = 'accepted' AS "open!"
           FROM invitations
           WHERE company_id = $1 AND id = $2"#,
        company_id,
        id,
    )
    .fetch_optional(db)
    .await
}

/// The application is to one of the company's vacancies; `open` — not withdrawn.
pub async fn application_outcome_subject(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<Option<OutcomeSubject>, sqlx::Error> {
    sqlx::query_as!(
        OutcomeSubject,
        r#"SELECT a.candidate_id, a.status <> 'withdrawn' AS "open!"
           FROM applications a
           JOIN vacancies v ON v.id = a.vacancy_id
           WHERE v.company_id = $1 AND a.id = $2"#,
        company_id,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn upsert_outcome(
    db: impl PgExecutor<'_>,
    o: &NewOutcome<'_>,
) -> Result<StoredOutcome, sqlx::Error> {
    sqlx::query_as!(
        StoredOutcome,
        r#"INSERT INTO hiring_outcomes (id, company_id, candidate_id, invitation_id, application_id,
                                        stage, reason, occurred_at, api_key_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           ON CONFLICT (subject_id, stage) DO UPDATE
           SET reason = EXCLUDED.reason, occurred_at = EXCLUDED.occurred_at, recorded_at = now(),
               api_key_id = EXCLUDED.api_key_id
           RETURNING id, (xmax = 0) AS "inserted!""#,
        o.id,
        o.company_id,
        o.candidate_id,
        o.invitation_id,
        o.application_id,
        o.stage as HiringStage,
        o.reason,
        o.occurred_at,
        o.api_key_id,
    )
    .fetch_one(db)
    .await
}

pub async fn candidate_outcomes(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<OutcomeRow>, sqlx::Error> {
    sqlx::query_as!(
        OutcomeRow,
        r#"SELECT invitation_id, application_id, stage AS "stage: HiringStage", reason, occurred_at
           FROM hiring_outcomes
           WHERE candidate_id = $1
           ORDER BY occurred_at"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn subject_outcomes(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    subject_ids: &[Uuid],
) -> Result<Vec<OutcomeRow>, sqlx::Error> {
    sqlx::query_as!(
        OutcomeRow,
        r#"SELECT invitation_id, application_id, stage AS "stage: HiringStage", reason, occurred_at
           FROM hiring_outcomes
           WHERE company_id = $1 AND subject_id = ANY($2)
           ORDER BY occurred_at"#,
        company_id,
        subject_ids,
    )
    .fetch_all(db)
    .await
}

/// Invitations and applications that reached each stage, for the employer's funnel.
pub async fn company_stage_counts(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<StageCount>, sqlx::Error> {
    sqlx::query_as!(
        StageCount,
        r#"SELECT stage AS "stage: HiringStage", count(*) AS "count!"
           FROM hiring_outcomes
           WHERE company_id = $1
           GROUP BY stage
           ORDER BY stage"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

/// Demo provisioning: the company of a known owner account, looked up by the email blind index.
pub async fn company_by_owner_bidx(
    db: impl PgExecutor<'_>,
    email_bidx: &[u8],
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT c.id FROM companies c JOIN users u ON u.id = c.owner_user_id WHERE u.email_bidx = $1",
        email_bidx,
    )
    .fetch_optional(db)
    .await
}
