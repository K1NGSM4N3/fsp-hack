use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::UserRole;

#[derive(Debug, Clone)]
pub struct ConsentDocument {
    pub id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub body: String,
    pub required_for: Vec<UserRole>,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserConsent {
    pub id: Uuid,
    pub document_id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn current_consent_documents(
    db: impl PgExecutor<'_>,
) -> Result<Vec<ConsentDocument>, sqlx::Error> {
    sqlx::query_as!(
        ConsentDocument,
        r#"SELECT DISTINCT ON (kind) id, kind, version, title, body,
                  required_for AS "required_for: Vec<UserRole>", published_at
           FROM consent_documents
           ORDER BY kind, version DESC"#,
    )
    .fetch_all(db)
    .await
}

pub async fn insert_user_consent(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    document_id: Uuid,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO user_consents (id, user_id, document_id, ip, user_agent)
         VALUES ($1, $2, $3, $4, $5)",
        id,
        user_id,
        document_id,
        ip,
        user_agent,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn user_consents(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<UserConsent>, sqlx::Error> {
    sqlx::query_as!(
        UserConsent,
        "SELECT uc.id, uc.document_id, d.kind, d.version, d.title, uc.granted_at, uc.revoked_at
         FROM user_consents uc
         JOIN consent_documents d ON d.id = uc.document_id
         WHERE uc.user_id = $1
         ORDER BY uc.granted_at DESC",
        user_id,
    )
    .fetch_all(db)
    .await
}

pub async fn revoke_user_consent(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "UPDATE user_consents uc
         SET revoked_at = now()
         FROM consent_documents d
         WHERE uc.id = $1 AND uc.user_id = $2 AND uc.revoked_at IS NULL AND d.id = uc.document_id
         RETURNING d.kind",
        id,
        user_id,
    )
    .fetch_optional(db)
    .await
}
