//! Notifications: the in-app feed, email letters that batch it, per-kind email settings, the
//! expiry reminders and the trust level a company was last told about.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgExecutor};
use uuid::Uuid;

use crate::types::{NotificationEmailStatus, UserRole};

pub struct NewNotification<'a> {
    pub id: Uuid,
    /// `None`: the owner of `company_id`.
    pub user_id: Option<Uuid>,
    pub kind: &'a str,
    /// Bit of the kind in `notification_settings.email_off`.
    pub kind_bit: i32,
    /// JSON text.
    pub payload: &'a str,
    pub subject_id: Option<Uuid>,
    pub candidate_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub dedup_key: Option<&'a str>,
    /// The kind goes out by email at all (some changes are in-app only).
    pub email: bool,
    /// How long the item waits for others to join its letter.
    pub hold_secs: f64,
}

#[derive(Debug, Clone)]
pub struct FeedRow {
    pub id: Uuid,
    pub kind: String,
    pub payload: String,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct EmailHistory {
    pub sent: i64,
    pub last_at: Option<DateTime<Utc>>,
    pub oldest_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct DueEmail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub attempts: i16,
}

#[derive(Debug, Clone)]
pub struct Recipient {
    pub role: UserRole,
    pub email_enc: Vec<u8>,
    pub deliverable: bool,
    pub email_off: i32,
}

#[derive(Debug, Clone)]
pub struct EmailRecord {
    pub id: Uuid,
    pub status: NotificationEmailStatus,
    pub items: i64,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct InvitationBrief {
    pub id: Uuid,
    pub company_id: Uuid,
    pub candidate_id: Uuid,
    pub public_code: String,
    pub company_name: String,
    pub title: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub decline_reason: Option<String>,
    pub decline_comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApplicationBrief {
    pub id: Uuid,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub company_id: Uuid,
    pub company_name: String,
    pub candidate_id: Uuid,
    pub public_code: String,
}

#[derive(Debug, Clone)]
pub struct TrustSeen {
    pub level: String,
    pub quota: i16,
}

/// The kind's email is on when the recipient has not turned it off, the address is confirmed,
/// and the change is one that goes out by email. Returns `false` for a missing or deleted
/// recipient and for a repeated `dedup_key`.
pub async fn insert(db: impl PgExecutor<'_>, n: &NewNotification<'_>) -> Result<bool, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO notifications (id, user_id, kind, payload, subject_id, candidate_id,
                                    company_id, dedup_key, email_due_at)
         SELECT $1, u.id, $3, $4::text::json, $5, $6, $7, $8,
                CASE WHEN $9 AND u.email_verified_at IS NOT NULL
                          AND coalesce(s.email_off, 0) & $10 = 0
                     THEN now() + make_interval(secs => $11) END
         FROM users u
         LEFT JOIN notification_settings s ON s.user_id = u.id
         WHERE u.id = coalesce($2::uuid, (SELECT owner_user_id FROM companies WHERE id = $7))
           AND u.deleted_at IS NULL
         ON CONFLICT (user_id, dedup_key) DO NOTHING",
        n.id,
        n.user_id,
        n.kind,
        n.payload,
        n.subject_id,
        n.candidate_id,
        n.company_id,
        n.dedup_key,
        n.email,
        n.kind_bit,
        n.hold_secs,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected() > 0)
}

/// Demo data: an item with fixed times and no email; `user_id: None` — the company's owner.
pub async fn insert_seeded(
    db: impl PgExecutor<'_>,
    n: &NewNotification<'_>,
    created_at: DateTime<Utc>,
    read_at: Option<DateTime<Utc>>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO notifications (id, user_id, kind, payload, subject_id, candidate_id,
                                    company_id, created_at, read_at)
         SELECT $1, coalesce($2::uuid, (SELECT owner_user_id FROM companies WHERE id = $7)),
                $3, $4::text::json, $5, $6, $7, $8, $9",
        n.id,
        n.user_id,
        n.kind,
        n.payload,
        n.subject_id,
        n.candidate_id,
        n.company_id,
        created_at,
        read_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Newest first, keyset-paged by id (UUIDv7).
pub async fn feed(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    unread_only: bool,
    before: Option<Uuid>,
    limit: i64,
) -> Result<Vec<FeedRow>, sqlx::Error> {
    sqlx::query_as!(
        FeedRow,
        r#"SELECT id, kind, payload::text AS "payload!", created_at, read_at
           FROM notifications
           WHERE user_id = $1
             AND (NOT $2 OR read_at IS NULL)
             AND ($3::uuid IS NULL OR id < $3)
           ORDER BY id DESC
           LIMIT $4"#,
        user_id,
        unread_only,
        before,
        limit,
    )
    .fetch_all(db)
    .await
}

/// Counts up to `cap`: the badge shows «99+», and the query never walks a long backlog.
pub async fn unread_count(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    cap: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!"
           FROM (SELECT 1 FROM notifications WHERE user_id = $1 AND read_at IS NULL LIMIT $2) t"#,
        user_id,
        cap,
    )
    .fetch_one(db)
    .await
}

pub async fn mark_read(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notifications SET read_at = now()
         WHERE user_id = $1 AND id = ANY($2) AND read_at IS NULL",
        user_id,
        ids,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn mark_all_read(db: impl PgExecutor<'_>, user_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notifications SET read_at = now() WHERE user_id = $1 AND read_at IS NULL",
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Opening the invitation or application itself reads what was said about it.
pub async fn mark_subject_read(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    subject_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notifications SET read_at = now()
         WHERE user_id = $1 AND subject_id = $2 AND read_at IS NULL",
        user_id,
        subject_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn email_off(db: impl PgExecutor<'_>, user_id: Uuid) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT coalesce((SELECT email_off FROM notification_settings WHERE user_id = $1), 0)
                  AS "email_off!""#,
        user_id,
    )
    .fetch_one(db)
    .await
}

/// Sets the `off` bits and clears the `on` ones in one statement, so two tabs saving different
/// switches do not overwrite each other. Returns the new mask.
pub async fn update_email_off(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    off: i32,
    on: i32,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar!(
        "INSERT INTO notification_settings (user_id, email_off) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE
         SET email_off = (notification_settings.email_off | $2) & ~$3::int4, updated_at = now()
         RETURNING email_off",
        user_id,
        off,
        on,
    )
    .fetch_one(db)
    .await
}

/// One-click unsubscribe: sets bits without reading them first.
pub async fn turn_email_off(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    bits: i32,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO notification_settings (user_id, email_off)
         SELECT id, $2 FROM users WHERE id = $1 AND deleted_at IS NULL
         ON CONFLICT (user_id) DO UPDATE
         SET email_off = notification_settings.email_off | EXCLUDED.email_off, updated_at = now()",
        user_id,
        bits,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Account deletion: the feed, the letters and the settings.
pub async fn delete_user_data(conn: &mut PgConnection, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM notifications WHERE user_id = $1", user_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query!(
        "DELETE FROM notification_emails WHERE user_id = $1",
        user_id
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "DELETE FROM notification_settings WHERE user_id = $1",
        user_id
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}

pub async fn users_with_due_email(
    db: impl PgExecutor<'_>,
    limit: i64,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT DISTINCT user_id FROM notifications WHERE email_due_at <= now() LIMIT $1",
        limit,
    )
    .fetch_all(db)
    .await
}

/// Serializes letter assembly per recipient across replicas until the transaction ends.
pub async fn try_lock_recipient(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT pg_try_advisory_xact_lock(hashtextextended($1::text, 1414)) AS "locked!""#,
        user_id.to_string(),
    )
    .fetch_one(db)
    .await
}

pub async fn earliest_queued(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT min(email_due_at) FROM notifications WHERE user_id = $1 AND email_due_at IS NOT NULL",
        user_id,
    )
    .fetch_one(db)
    .await
}

/// Letters to the user created after `since` that count against the cap.
pub async fn email_history(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    since: DateTime<Utc>,
) -> Result<EmailHistory, sqlx::Error> {
    sqlx::query_as!(
        EmailHistory,
        r#"SELECT count(*) AS "sent!", max(created_at) AS last_at, min(created_at) AS oldest_at
           FROM notification_emails
           WHERE user_id = $1 AND created_at > $2 AND status <> 'skipped'"#,
        user_id,
        since,
    )
    .fetch_one(db)
    .await
}

pub async fn defer_queued(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    until: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notifications SET email_due_at = $2
         WHERE user_id = $1 AND email_due_at IS NOT NULL",
        user_id,
        until,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_email(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO notification_emails (id, user_id) VALUES ($1, $2)",
        id,
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Everything queued for the user joins the letter, including items still in their hold.
pub async fn attach_queued(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    email_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notifications SET email_id = $2, email_due_at = NULL
         WHERE user_id = $1 AND email_due_at IS NOT NULL",
        user_id,
        email_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Leases due letters like the ATS outbox: `SKIP LOCKED` keeps replicas apart, the lease brings
/// a crashed replica's letters back.
pub async fn claim_due_emails(
    db: impl PgExecutor<'_>,
    limit: i64,
    lease_secs: f64,
) -> Result<Vec<DueEmail>, sqlx::Error> {
    sqlx::query_as!(
        DueEmail,
        "WITH due AS (
             SELECT id FROM notification_emails
             WHERE status = 'pending' AND next_attempt_at <= now()
             ORDER BY next_attempt_at
             LIMIT $1
             FOR UPDATE SKIP LOCKED
         )
         UPDATE notification_emails e
         SET next_attempt_at = now() + make_interval(secs => $2)
         FROM due
         WHERE e.id = due.id
         RETURNING e.id, e.user_id, e.attempts",
        limit,
        lease_secs,
    )
    .fetch_all(db)
    .await
}

pub async fn recipient(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<Recipient>, sqlx::Error> {
    sqlx::query_as!(
        Recipient,
        r#"SELECT u.role AS "role: UserRole", u.email_enc,
                  (u.deleted_at IS NULL AND u.email_verified_at IS NOT NULL) AS "deliverable!",
                  coalesce(s.email_off, 0) AS "email_off!"
           FROM users u
           LEFT JOIN notification_settings s ON s.user_id = u.id
           WHERE u.id = $1"#,
        user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn email_items(
    db: impl PgExecutor<'_>,
    email_id: Uuid,
) -> Result<Vec<FeedRow>, sqlx::Error> {
    sqlx::query_as!(
        FeedRow,
        r#"SELECT id, kind, payload::text AS "payload!", created_at, read_at
           FROM notifications
           WHERE email_id = $1
           ORDER BY id"#,
        email_id,
    )
    .fetch_all(db)
    .await
}

pub async fn mark_email_sent(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notification_emails
         SET status = 'sent', sent_at = now(), attempts = attempts + 1, last_error = NULL
         WHERE id = $1 AND status = 'pending'",
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn settle_email(
    db: impl PgExecutor<'_>,
    id: Uuid,
    status: NotificationEmailStatus,
    error: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notification_emails SET status = $2, last_error = $3
         WHERE id = $1 AND status = 'pending'",
        id,
        status as NotificationEmailStatus,
        error,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn schedule_email_retry(
    db: impl PgExecutor<'_>,
    id: Uuid,
    at: DateTime<Utc>,
    error: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE notification_emails
         SET attempts = attempts + 1, next_attempt_at = $2, last_error = $3
         WHERE id = $1 AND status = 'pending'",
        id,
        at,
        error,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// The data export: every letter with how many items it carried.
pub async fn user_emails(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<EmailRecord>, sqlx::Error> {
    sqlx::query_as!(
        EmailRecord,
        r#"SELECT e.id, e.status AS "status: NotificationEmailStatus",
                  (SELECT count(*) FROM notifications n WHERE n.email_id = e.id) AS "items!",
                  e.created_at, e.sent_at
           FROM notification_emails e
           WHERE e.user_id = $1
           ORDER BY e.created_at DESC"#,
        user_id,
    )
    .fetch_all(db)
    .await
}

pub async fn invitation_brief(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<InvitationBrief>, sqlx::Error> {
    sqlx::query_as!(
        InvitationBrief,
        "SELECT i.id, i.company_id, i.candidate_id, c.public_code, co.name AS company_name,
                i.title, i.salary_min, i.salary_max, i.work_format, i.expires_at,
                i.decline_reason, i.decline_comment
         FROM invitations i
         JOIN candidates c ON c.user_id = i.candidate_id
         JOIN companies co ON co.id = i.company_id
         WHERE i.id = $1",
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn application_brief(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<ApplicationBrief>, sqlx::Error> {
    sqlx::query_as!(
        ApplicationBrief,
        "SELECT a.id, a.vacancy_id, v.title AS vacancy_title, v.company_id,
                co.name AS company_name, a.candidate_id, c.public_code
         FROM applications a
         JOIN vacancies v ON v.id = a.vacancy_id
         JOIN companies co ON co.id = v.company_id
         JOIN candidates c ON c.user_id = a.candidate_id
         WHERE a.id = $1",
        id,
    )
    .fetch_optional(db)
    .await
}

/// Open invitations whose answer window closes within `within_secs` and that have no reminder.
/// The unique `(user_id, dedup_key)` is what makes the reminder single; this filter only saves
/// work.
pub async fn expiring_invitations(
    db: impl PgExecutor<'_>,
    within_secs: f64,
    limit: i64,
) -> Result<Vec<InvitationBrief>, sqlx::Error> {
    sqlx::query_as!(
        InvitationBrief,
        "SELECT i.id, i.company_id, i.candidate_id, c.public_code, co.name AS company_name,
                i.title, i.salary_min, i.salary_max, i.work_format, i.expires_at,
                i.decline_reason, i.decline_comment
         FROM invitations i
         JOIN candidates c ON c.user_id = i.candidate_id
         JOIN companies co ON co.id = i.company_id
         WHERE i.status IN ('sent', 'viewed')
           AND i.expires_at > now()
           AND i.expires_at <= now() + make_interval(secs => $1)
           AND NOT EXISTS (
               SELECT 1 FROM notifications n
               WHERE n.user_id = i.candidate_id AND n.dedup_key = 'expiring:' || i.id::text)
         ORDER BY i.expires_at
         LIMIT $2",
        within_secs,
        limit,
    )
    .fetch_all(db)
    .await
}

pub async fn trust_seen_for_update(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Option<TrustSeen>, sqlx::Error> {
    sqlx::query_as!(
        TrustSeen,
        "SELECT level, quota FROM company_trust_seen WHERE company_id = $1 FOR UPDATE",
        company_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn upsert_trust_seen(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    level: &str,
    quota: i16,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO company_trust_seen (company_id, level, quota) VALUES ($1, $2, $3)
         ON CONFLICT (company_id) DO UPDATE
         SET level = EXCLUDED.level, quota = EXCLUDED.quota, updated_at = now()",
        company_id,
        level,
        quota,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
