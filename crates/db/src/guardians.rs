//! Age gate: minors' birth dates and the requests to their legal representatives. Whether a
//! candidate is a minor is always derived here from the birth date at query time
//! (`birth_date + interval '18 years' > current_date`), so restrictions lift on the 18th birthday
//! without a job.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{consents::MissingConsent, types::GuardianDecision};

pub const KIND: &str = "guardian_consent";

#[derive(Debug, Clone, Copy)]
pub struct CandidateAge {
    /// Only collected from those who registered as 14–17; cleared 30 days after the 18th birthday.
    pub birth_date: Option<NaiveDate>,
    pub minor: bool,
}

#[derive(Debug, Clone)]
pub struct GuardianRequest {
    pub id: Uuid,
    pub full_name_enc: Option<Vec<u8>>,
    pub email_enc: Option<Vec<u8>>,
    pub email_bidx: Option<Vec<u8>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub confirm_expires_at: Option<DateTime<Utc>>,
    pub decision: Option<GuardianDecision>,
    pub answered_at: Option<DateTime<Utc>>,
    pub consent_id: Option<Uuid>,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub erased_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// A link from a representative's letter resolved to its request.
#[derive(Debug, Clone)]
pub struct LinkTarget {
    pub request_id: Uuid,
    pub candidate_id: Uuid,
    /// The one-time answer link; otherwise the revocation link of the same letter.
    pub confirm_link: bool,
    pub full_name_enc: Option<Vec<u8>>,
    pub account_email_enc: Vec<u8>,
    pub sent_at: Option<DateTime<Utc>>,
    pub confirm_expires_at: Option<DateTime<Utc>>,
    pub decision: Option<GuardianDecision>,
    pub answered_at: Option<DateTime<Utc>>,
    pub consent_id: Option<Uuid>,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub minor: bool,
}

pub async fn candidate_age(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<CandidateAge>, sqlx::Error> {
    sqlx::query_as!(
        CandidateAge,
        r#"SELECT birth_date,
                  (birth_date IS NOT NULL AND birth_date + interval '18 years' > current_date) AS "minor!"
           FROM candidates
           WHERE user_id = $1"#,
        user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn set_birth_date(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    birth_date: NaiveDate,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates SET birth_date = $1 WHERE user_id = $2",
        birth_date,
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_request(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
    full_name_enc: &[u8],
    email_enc: &[u8],
    email_bidx: &[u8],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO guardian_requests (id, candidate_id, full_name_enc, email_enc, email_bidx)
         VALUES ($1, $2, $3, $4, $5)",
        id,
        candidate_id,
        full_name_enc,
        email_enc,
        email_bidx,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Newest first; the first one is the current request.
pub async fn requests(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<GuardianRequest>, sqlx::Error> {
    sqlx::query_as!(
        GuardianRequest,
        r#"SELECT g.id, g.full_name_enc, g.email_enc, g.email_bidx, g.sent_at, g.confirm_expires_at,
                  g.decision AS "decision: GuardianDecision", g.answered_at, g.consent_id,
                  uc.granted_at AS "granted_at?", uc.revoked_at AS "revoked_at?", g.erased_at,
                  g.created_at
           FROM guardian_requests g
           LEFT JOIN user_consents uc ON uc.id = g.consent_id
           WHERE g.candidate_id = $1
           ORDER BY g.created_at DESC, g.id DESC"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn latest_request(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<GuardianRequest>, sqlx::Error> {
    sqlx::query_as!(
        GuardianRequest,
        r#"SELECT g.id, g.full_name_enc, g.email_enc, g.email_bidx, g.sent_at, g.confirm_expires_at,
                  g.decision AS "decision: GuardianDecision", g.answered_at, g.consent_id,
                  uc.granted_at AS "granted_at?", uc.revoked_at AS "revoked_at?", g.erased_at,
                  g.created_at
           FROM guardian_requests g
           LEFT JOIN user_consents uc ON uc.id = g.consent_id
           WHERE g.candidate_id = $1
           ORDER BY g.created_at DESC, g.id DESC
           LIMIT 1"#,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

/// New address for a request nobody has answered yet; links of earlier letters stop working at once.
pub async fn update_contact(
    db: impl PgExecutor<'_>,
    id: Uuid,
    full_name_enc: &[u8],
    email_enc: &[u8],
    email_bidx: &[u8],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE guardian_requests
         SET full_name_enc = $2, email_enc = $3, email_bidx = $4,
             confirm_hash = NULL, revoke_hash = NULL, confirm_expires_at = NULL, sent_at = NULL
         WHERE id = $1 AND decision IS NULL AND erased_at IS NULL",
        id,
        full_name_enc,
        email_enc,
        email_bidx,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Store the link hashes of a new letter; the previous letter's links stop working.
pub async fn set_letter(
    db: impl PgExecutor<'_>,
    id: Uuid,
    confirm_hash: &[u8],
    revoke_hash: &[u8],
    confirm_expires_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE guardian_requests
         SET confirm_hash = $2, revoke_hash = $3, confirm_expires_at = $4, sent_at = now()
         WHERE id = $1 AND decision IS NULL AND erased_at IS NULL",
        id,
        confirm_hash,
        revoke_hash,
        confirm_expires_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn link_target(
    db: impl PgExecutor<'_>,
    token_hash: &[u8],
) -> Result<Option<LinkTarget>, sqlx::Error> {
    sqlx::query_as!(
        LinkTarget,
        r#"SELECT g.id AS request_id, g.candidate_id,
                  (g.confirm_hash IS NOT DISTINCT FROM $1) AS "confirm_link!",
                  g.full_name_enc, u.email_enc AS account_email_enc, g.sent_at, g.confirm_expires_at,
                  g.decision AS "decision: GuardianDecision", g.answered_at, g.consent_id,
                  uc.granted_at AS "granted_at?", uc.revoked_at AS "revoked_at?",
                  (c.birth_date IS NOT NULL AND c.birth_date + interval '18 years' > current_date) AS "minor!"
           FROM guardian_requests g
           JOIN candidates c ON c.user_id = g.candidate_id
           JOIN users u ON u.id = g.candidate_id
           LEFT JOIN user_consents uc ON uc.id = g.consent_id
           WHERE g.confirm_hash = $1 OR g.revoke_hash = $1"#,
        token_hash,
    )
    .fetch_optional(db)
    .await
}

/// The representative's answer; only the first one counts.
pub async fn answer(
    db: impl PgExecutor<'_>,
    id: Uuid,
    decision: GuardianDecision,
    consent_id: Option<Uuid>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE guardian_requests SET decision = $2, answered_at = now(), consent_id = $3
         WHERE id = $1 AND decision IS NULL",
        id,
        decision as GuardianDecision,
        consent_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn revoke_grant(db: impl PgExecutor<'_>, consent_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE user_consents SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL",
        consent_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// The current `guardian_consent` document when the candidate is a minor without an active grant.
pub async fn missing_guardian_consent(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<MissingConsent>, sqlx::Error> {
    sqlx::query_as!(
        MissingConsent,
        r#"SELECT d.id AS "id!", d.kind AS "kind!", d.title AS "title!"
           FROM candidates c
           CROSS JOIN LATERAL (SELECT id, kind, title FROM consent_documents
                               WHERE kind = 'guardian_consent'
                               ORDER BY version DESC LIMIT 1) d
           WHERE c.user_id = $1
             AND c.birth_date + interval '18 years' > current_date
             AND NOT EXISTS (SELECT 1 FROM user_consents uc
                              JOIN consent_documents g ON g.id = uc.document_id AND g.kind = 'guardian_consent'
                              WHERE uc.user_id = c.user_id AND uc.revoked_at IS NULL)"#,
        user_id,
    )
    .fetch_optional(db)
    .await
}

/// Account deletion: the representative's IP and browser go with the account; the grant row keeps
/// only the document version and dates. The requests themselves cascade with the candidate row.
pub async fn erase_grant_trail(db: impl PgExecutor<'_>, user_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE user_consents uc SET ip = NULL, user_agent = NULL
         FROM consent_documents d
         WHERE uc.user_id = $1 AND d.id = uc.document_id AND d.kind = 'guardian_consent'
           AND (uc.ip IS NOT NULL OR uc.user_agent IS NOT NULL)",
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Retention: `days` after the 18th birthday the representative's name, email and links are erased.
pub async fn erase_expired_requests(
    db: impl PgExecutor<'_>,
    days: i32,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE guardian_requests g
         SET full_name_enc = NULL, email_enc = NULL, email_bidx = NULL,
             confirm_hash = NULL, revoke_hash = NULL, erased_at = now()
         FROM candidates c
         WHERE c.user_id = g.candidate_id AND g.erased_at IS NULL
           AND c.birth_date + interval '18 years' + make_interval(days => $1) <= current_date",
        days,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Retention: the representative's IP and browser on the grant, on the same day as the request data.
pub async fn erase_expired_grant_trails(
    db: impl PgExecutor<'_>,
    days: i32,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE user_consents uc SET ip = NULL, user_agent = NULL
         FROM consent_documents d, candidates c
         WHERE d.id = uc.document_id AND d.kind = 'guardian_consent' AND c.user_id = uc.user_id
           AND (uc.ip IS NOT NULL OR uc.user_agent IS NOT NULL)
           AND c.birth_date + interval '18 years' + make_interval(days => $1) <= current_date",
        days,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Retention, last step: without a birth date the account is an ordinary adult one.
pub async fn clear_expired_birth_dates(
    db: impl PgExecutor<'_>,
    days: i32,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates SET birth_date = NULL
         WHERE birth_date + interval '18 years' + make_interval(days => $1) <= current_date",
        days,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
