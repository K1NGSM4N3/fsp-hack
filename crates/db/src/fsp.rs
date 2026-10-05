use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OidcState {
    pub user_id: Uuid,
    pub pkce_verifier: String,
    pub nonce: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FspLink {
    pub candidate_id: Uuid,
    pub fsp_id: String,
    pub linked_via: String,
    pub linked_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub achievements: Value,
    pub has_achievements: bool,
}

pub async fn insert_oidc_state(
    db: impl PgExecutor<'_>,
    state: &str,
    user_id: Uuid,
    pkce_verifier: &str,
    nonce: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO oidc_states (state, user_id, pkce_verifier, nonce) VALUES ($1, $2, $3, $4)",
        state,
        user_id,
        pkce_verifier,
        nonce,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn take_oidc_state(
    db: impl PgExecutor<'_>,
    state: &str,
) -> Result<Option<OidcState>, sqlx::Error> {
    sqlx::query_as!(
        OidcState,
        "DELETE FROM oidc_states WHERE state = $1 RETURNING user_id, pkce_verifier, nonce, created_at",
        state,
    )
    .fetch_optional(db)
    .await
}

pub async fn purge_oidc_states(db: impl PgExecutor<'_>) -> Result<u64, sqlx::Error> {
    sqlx::query!("DELETE FROM oidc_states WHERE created_at < now() - interval '15 minutes'")
        .execute(db)
        .await
        .map(|r| r.rows_affected())
}

pub async fn fsp_link(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<FspLink>, sqlx::Error> {
    sqlx::query_as!(
        FspLink,
        r#"SELECT candidate_id, fsp_id, linked_via, linked_at, last_sync_at, achievements,
                  has_achievements AS "has_achievements!"
           FROM fsp_links
           WHERE candidate_id = $1"#,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn fsp_link_owner(
    db: impl PgExecutor<'_>,
    fsp_id: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT candidate_id FROM fsp_links WHERE fsp_id = $1",
        fsp_id
    )
    .fetch_optional(db)
    .await
}

pub async fn upsert_fsp_link(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    fsp_id: &str,
    linked_via: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO fsp_links (candidate_id, fsp_id, linked_via)
         VALUES ($1, $2, $3)
         ON CONFLICT (candidate_id) DO UPDATE
         SET fsp_id = EXCLUDED.fsp_id, linked_via = EXCLUDED.linked_via, linked_at = now(),
             achievements = '[]', last_sync_at = NULL",
        candidate_id,
        fsp_id,
        linked_via,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn store_fsp_achievements(
    db: impl PgExecutor<'_>,
    achievements: &Value,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE fsp_links SET achievements = $1, last_sync_at = now() WHERE candidate_id = $2",
        achievements,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn delete_fsp_link(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM fsp_links WHERE candidate_id = $1",
        candidate_id
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
