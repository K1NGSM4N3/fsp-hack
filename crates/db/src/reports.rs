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

pub async fn recent_reports_by(
    db: impl PgExecutor<'_>,
    reporter_user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM reports
           WHERE reporter_user_id = $1 AND created_at > now() - interval '1 day'"#,
        reporter_user_id,
    )
    .fetch_one(db)
    .await
}
