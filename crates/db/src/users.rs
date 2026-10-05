use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::UserRole;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub role: UserRole,
    pub email_enc: Vec<u8>,
    pub password_hash: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// What the account row holds about its owner, for the data export.
#[derive(Debug, Clone)]
pub struct AccountRecord {
    pub id: Uuid,
    pub role: UserRole,
    pub email_enc: Vec<u8>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub family_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn insert_user(
    db: impl PgExecutor<'_>,
    id: Uuid,
    role: UserRole,
    email_enc: &[u8],
    email_bidx: &[u8],
    password_hash: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO users (id, role, email_enc, email_bidx, password_hash)
         VALUES ($1, $2, $3, $4, $5)",
        id,
        role as UserRole,
        email_enc,
        email_bidx,
        password_hash,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn user_by_email_bidx(
    db: impl PgExecutor<'_>,
    email_bidx: &[u8],
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"SELECT id, role AS "role: UserRole", email_enc, password_hash, email_verified_at, deleted_at
           FROM users
           WHERE email_bidx = $1"#,
        email_bidx,
    )
    .fetch_optional(db)
    .await
}

pub async fn user_by_id(db: impl PgExecutor<'_>, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"SELECT id, role AS "role: UserRole", email_enc, password_hash, email_verified_at, deleted_at
           FROM users
           WHERE id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn account_record(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<AccountRecord>, sqlx::Error> {
    sqlx::query_as!(
        AccountRecord,
        r#"SELECT id, role AS "role: UserRole", email_enc, email_verified_at, created_at, last_login_at
           FROM users
           WHERE id = $1 AND deleted_at IS NULL"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn touch_last_login(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!("UPDATE users SET last_login_at = now() WHERE id = $1", id)
        .execute(db)
        .await
        .map(|r| r.rows_affected())
}

pub async fn mark_email_verified(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE users SET email_verified_at = now() WHERE id = $1 AND email_verified_at IS NULL",
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Account deletion (152-ФЗ): wipe personal data, keep an anonymized row for referential history.
pub async fn anonymize_user(
    db: impl PgExecutor<'_>,
    tombstone_bidx: &[u8],
    id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        r"UPDATE users
          SET email_enc = '\x'::bytea,
              email_bidx = $1,
              password_hash = '!',
              deleted_at = now()
          WHERE id = $2",
        tombstone_bidx,
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_email_verification(
    db: impl PgExecutor<'_>,
    token_hash: &[u8],
    user_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO email_verifications (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
        token_hash,
        user_id,
        expires_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn consume_email_verification(
    db: impl PgExecutor<'_>,
    token_hash: &[u8],
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "UPDATE email_verifications
         SET used_at = now()
         WHERE token_hash = $1 AND used_at IS NULL AND expires_at > now()
         RETURNING user_id",
        token_hash,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_refresh_token(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    family_id: Uuid,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO refresh_tokens (id, user_id, family_id, token_hash, expires_at)
         VALUES ($1, $2, $3, $4, $5)",
        id,
        user_id,
        family_id,
        token_hash,
        expires_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn refresh_token_by_hash(
    db: impl PgExecutor<'_>,
    token_hash: &[u8],
) -> Result<Option<RefreshToken>, sqlx::Error> {
    sqlx::query_as!(
        RefreshToken,
        "SELECT id, user_id, family_id, expires_at, rotated_at, revoked_at
         FROM refresh_tokens
         WHERE token_hash = $1",
        token_hash,
    )
    .fetch_optional(db)
    .await
}

/// Atomic rotation: only one concurrent refresh wins.
pub async fn rotate_refresh_token(db: impl PgExecutor<'_>, id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE refresh_tokens SET rotated_at = now()
         WHERE id = $1 AND rotated_at IS NULL AND revoked_at IS NULL",
        id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn revoke_refresh_family(
    db: impl PgExecutor<'_>,
    family_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE family_id = $1 AND revoked_at IS NULL",
        family_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn revoke_user_refresh_tokens(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
