use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn insert_report(
    db: impl PgExecutor<'_>,
    id: Uuid,
    reporter_user_id: Uuid,
    target_kind: &str,
    target_id: Uuid,
    reason: &str,
    comment: Option<&str>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO reports (id, reporter_user_id, target_kind, target_id, reason, comment)
         VALUES ($1, $2, $3, $4, $5, $6)",
        id,
        reporter_user_id,
        target_kind,
        target_id,
        reason,
        comment,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
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
