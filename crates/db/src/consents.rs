use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::UserRole;

pub const PROCESSING: &str = "pd_processing";
pub const PUBLICATION: &str = "pd_publication";

#[derive(Debug, Clone)]
pub struct ConsentDocument {
    pub id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub body: String,
    pub required_for: Vec<UserRole>,
    pub applies_to: Vec<UserRole>,
    pub published_at: DateTime<Utc>,
}

/// A document version as seen by `POST /me/consents`: only the latest version of a kind is grantable.
#[derive(Debug, Clone)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub applies_to: Vec<UserRole>,
    pub current: bool,
}

/// The current version of a document the user holds no active grant of (any version).
#[derive(Debug, Clone)]
pub struct MissingConsent {
    pub id: Uuid,
    pub kind: String,
    pub title: String,
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
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Grant {
    pub id: Uuid,
    pub granted_at: DateTime<Utc>,
    /// `false` when an active grant already existed and was returned as is.
    pub created: bool,
}

pub async fn current_consent_documents(
    db: impl PgExecutor<'_>,
) -> Result<Vec<ConsentDocument>, sqlx::Error> {
    sqlx::query_as!(
        ConsentDocument,
        r#"SELECT DISTINCT ON (kind) id, kind, version, title, body,
                  required_for AS "required_for: Vec<UserRole>",
                  applies_to AS "applies_to: Vec<UserRole>", published_at
           FROM consent_documents
           ORDER BY kind, version DESC"#,
    )
    .fetch_all(db)
    .await
}

pub async fn current_document(
    db: impl PgExecutor<'_>,
    kind: &str,
) -> Result<Option<ConsentDocument>, sqlx::Error> {
    sqlx::query_as!(
        ConsentDocument,
        r#"SELECT id, kind, version, title, body,
                  required_for AS "required_for: Vec<UserRole>",
                  applies_to AS "applies_to: Vec<UserRole>", published_at
           FROM consent_documents
           WHERE kind = $1
           ORDER BY version DESC
           LIMIT 1"#,
        kind,
    )
    .fetch_optional(db)
    .await
}

pub async fn document_version(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<DocumentVersion>, sqlx::Error> {
    sqlx::query_as!(
        DocumentVersion,
        r#"SELECT d.id, d.kind, d.version, d.title, d.applies_to AS "applies_to: Vec<UserRole>",
                  NOT EXISTS (SELECT 1 FROM consent_documents n
                               WHERE n.kind = d.kind AND n.version > d.version) AS "current!"
           FROM consent_documents d
           WHERE d.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

/// Of `kinds`, the current documents the user has no unrevoked grant for, ordered by kind.
/// A grant of an older version still counts: publishing a new wording does not unpublish anyone.
pub async fn missing_consents(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    kinds: &[String],
) -> Result<Vec<MissingConsent>, sqlx::Error> {
    sqlx::query_as!(
        MissingConsent,
        "SELECT DISTINCT ON (d.kind) d.id, d.kind, d.title
         FROM consent_documents d
         WHERE d.kind = ANY ($2)
           AND NOT EXISTS (SELECT 1 FROM user_consents uc
                            JOIN consent_documents g ON g.id = uc.document_id
                            WHERE uc.user_id = $1 AND uc.revoked_at IS NULL AND g.kind = d.kind)
         ORDER BY d.kind, d.version DESC",
        user_id,
        kinds,
    )
    .fetch_all(db)
    .await
}

/// The user's unrevoked grant of this exact document version, if any.
pub async fn active_grant(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    document_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT id FROM user_consents
         WHERE user_id = $1 AND document_id = $2 AND revoked_at IS NULL
         ORDER BY granted_at DESC
         LIMIT 1",
        user_id,
        document_id,
    )
    .fetch_optional(db)
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

/// Idempotent: with an active grant of the document in place, that grant is returned unchanged.
/// The no-op `DO UPDATE` makes the conflicting row visible to `RETURNING` in the same statement
/// (no race with a concurrent grant); `xmax = 0` holds only for a freshly inserted row.
pub async fn grant(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    document_id: Uuid,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<Grant, sqlx::Error> {
    sqlx::query_as!(
        Grant,
        r#"INSERT INTO user_consents (id, user_id, document_id, ip, user_agent)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (user_id, document_id) WHERE revoked_at IS NULL
           DO UPDATE SET granted_at = user_consents.granted_at
           RETURNING id, granted_at, (xmax = 0) AS "created!""#,
        id,
        user_id,
        document_id,
        ip,
        user_agent,
    )
    .fetch_one(db)
    .await
}

pub async fn user_consents(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Vec<UserConsent>, sqlx::Error> {
    sqlx::query_as!(
        UserConsent,
        "SELECT uc.id, uc.document_id, d.kind, d.version, d.title, uc.granted_at, uc.revoked_at,
                uc.ip, uc.user_agent
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
         WHERE (uc.id = $1 OR uc.document_id = $1) AND uc.user_id = $2 AND uc.revoked_at IS NULL
           AND d.id = uc.document_id
         RETURNING d.kind",
        id,
        user_id,
    )
    .fetch_optional(db)
    .await
}
