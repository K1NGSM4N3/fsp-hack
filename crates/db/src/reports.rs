use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

/// `false`: the person already has an open complaint about this target, nothing was written.
pub async fn insert_report(
    db: impl PgExecutor<'_>,
    id: Uuid,
    reporter_user_id: Uuid,
    target_kind: &str,
    target_id: Uuid,
    reason: &str,
    comment: Option<&str>,
) -> Result<bool, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO reports (id, reporter_user_id, target_kind, target_id, reason, comment)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (reporter_user_id, target_kind, target_id) WHERE resolved_at IS NULL
         DO NOTHING",
        id,
        reporter_user_id,
        target_kind,
        target_id,
        reason,
        comment,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected() > 0)
}

/// The person's complaint about this target that a moderator has not resolved yet.
pub async fn open_report_by(
    db: impl PgExecutor<'_>,
    reporter_user_id: Uuid,
    target_kind: &str,
    target_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM reports
         WHERE reporter_user_id = $1 AND target_kind = $2 AND target_id = $3
           AND resolved_at IS NULL",
        reporter_user_id,
        target_kind,
        target_id,
    )
    .fetch_optional(db)
    .await
}

#[derive(Debug, Clone, Copy)]
pub struct RecentReports {
    pub count: i64,
    /// When the oldest of them leaves the 24-hour window.
    pub first_slot_frees_at: Option<DateTime<Utc>>,
}

pub async fn recent_reports_by(
    db: impl PgExecutor<'_>,
    reporter_user_id: Uuid,
) -> Result<RecentReports, sqlx::Error> {
    sqlx::query_as!(
        RecentReports,
        r#"SELECT count(*) AS "count!", min(created_at) + interval '1 day' AS first_slot_frees_at
           FROM reports
           WHERE reporter_user_id = $1 AND created_at > now() - interval '1 day'"#,
        reporter_user_id,
    )
    .fetch_one(db)
    .await
}
