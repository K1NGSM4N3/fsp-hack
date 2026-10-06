use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::VacancyStatus;

/// Raw counters behind `matching::trust::TrustSignals`. Windows: 90 days for behaviour and flags,
/// 180 days for accepted invitations; complaints only from candidates, unresolved.
#[derive(Debug, Clone)]
pub struct CompanySignals {
    pub company_id: Uuid,
    pub name: String,
    pub owner_created_at: DateTime<Utc>,
    pub verified_domain: Option<String>,
    pub complaints: i64,
    pub declines: i64,
    pub bad_declines: i64,
    pub invitations: i64,
    pub withdrawn: i64,
    pub closed: i64,
    pub answered: i64,
    pub accepted: i64,
    pub salary_flags: i64,
    pub own_duplicates: i64,
    pub copies: i64,
}

#[derive(Debug, Clone)]
pub struct OwnerIdentity {
    pub company_id: Uuid,
    pub email_enc: Vec<u8>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub website: Option<String>,
    pub verified_domain: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ExpectationBand {
    pub samples: i64,
    pub median: Option<i64>,
    pub p90: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct VacancyText {
    pub id: Uuid,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct SignedVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub status: VacancyStatus,
    pub text_signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FlaggedVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub status: VacancyStatus,
    pub salary_min: i64,
    pub salary_max: i64,
    pub trust_flags: Value,
    pub flagged_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FlaggedInvitation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub trust_flags: Value,
    pub created_at: DateTime<Utc>,
}

/// One company (`Some(id)`) or all of them (`None`, for moderation).
pub async fn company_signals(
    db: impl PgExecutor<'_>,
    company_id: Option<Uuid>,
) -> Result<Vec<CompanySignals>, sqlx::Error> {
    sqlx::query_as!(
        CompanySignals,
        r#"SELECT co.id AS company_id, co.name, u.created_at AS owner_created_at, co.verified_domain,
                  rep.complaints AS "complaints!", inv.declines AS "declines!",
                  inv.bad_declines AS "bad_declines!", inv.invitations AS "invitations!",
                  inv.withdrawn AS "withdrawn!", inv.closed AS "closed!",
                  inv.answered AS "answered!", inv.accepted AS "accepted!",
                  inv.salary_flags + vac.salary_flags AS "salary_flags!",
                  vac.own_duplicates AS "own_duplicates!", vac.copies AS "copies!"
           FROM companies co
           JOIN users u ON u.id = co.owner_user_id
           CROSS JOIN LATERAL (
               SELECT count(*) FILTER (WHERE i.status = 'declined'
                                         AND i.responded_at >= now() - interval '90 days') AS declines,
                      count(*) FILTER (WHERE i.status = 'declined'
                                         AND i.responded_at >= now() - interval '90 days'
                                         AND i.decline_reason IN ('salary', 'company')) AS bad_declines,
                      count(*) FILTER (WHERE i.created_at >= now() - interval '90 days') AS invitations,
                      count(*) FILTER (WHERE i.created_at >= now() - interval '90 days'
                                         AND i.status = 'withdrawn') AS withdrawn,
                      count(*) FILTER (WHERE i.created_at >= now() - interval '90 days'
                                         AND i.status IN ('accepted', 'declined', 'expired')) AS closed,
                      count(*) FILTER (WHERE i.created_at >= now() - interval '90 days'
                                         AND i.status IN ('accepted', 'declined')) AS answered,
                      count(*) FILTER (WHERE i.status = 'accepted'
                                         AND i.responded_at >= now() - interval '180 days') AS accepted,
                      count(*) FILTER (WHERE i.created_at >= now() - interval '90 days'
                                         AND i.trust_flags <> '[]') AS salary_flags
               FROM invitations i
               WHERE i.company_id = co.id
           ) inv
           CROSS JOIN LATERAL (
               SELECT count(*) FILTER (WHERE jsonb_path_exists(v.trust_flags,
                          '$[*] ? (@.kind == "salary_above_market" || @.kind == "salary_wide_range")')) AS salary_flags,
                      count(*) FILTER (WHERE jsonb_path_exists(v.trust_flags,
                          '$[*] ? (@.kind == "duplicate_own")')) AS own_duplicates,
                      count(*) FILTER (WHERE jsonb_path_exists(v.trust_flags,
                          '$[*] ? (@.kind == "copy_of_other")')) AS copies
               FROM vacancies v
               WHERE v.company_id = co.id AND v.flagged_at >= now() - interval '90 days'
           ) vac
           CROSS JOIN LATERAL (
               SELECT count(DISTINCT r.reporter_user_id) AS complaints
               FROM reports r
               JOIN users ru ON ru.id = r.reporter_user_id AND ru.role = 'candidate'
               WHERE r.resolved_at IS NULL AND r.created_at >= now() - interval '90 days'
                 AND ((r.target_kind = 'company' AND r.target_id = co.id)
                   OR (r.target_kind = 'vacancy'
                       AND r.target_id IN (SELECT id FROM vacancies WHERE company_id = co.id))
                   OR (r.target_kind = 'invitation'
                       AND r.target_id IN (SELECT id FROM invitations WHERE company_id = co.id)))
           ) rep
           WHERE $1::uuid IS NULL OR co.id = $1
           ORDER BY co.created_at, co.id"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn owner_identity(
    db: impl PgExecutor<'_>,
    owner_user_id: Uuid,
) -> Result<Option<OwnerIdentity>, sqlx::Error> {
    sqlx::query_as!(
        OwnerIdentity,
        "SELECT co.id AS company_id, u.email_enc, u.email_verified_at, co.website, co.verified_domain
         FROM companies co
         JOIN users u ON u.id = co.owner_user_id
         WHERE co.owner_user_id = $1",
        owner_user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn set_verified_domain(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    domain: Option<&str>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE companies SET verified_domain = $2
         WHERE id = $1 AND verified_domain IS DISTINCT FROM $2",
        company_id,
        domain,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Stated expectations of the category's searchable candidates: the pool of the category cards,
/// under the same publication rule (`published_candidates`, minors only with the representative's
/// consent).
pub async fn expectation_band(
    db: impl PgExecutor<'_>,
    specialization: &str,
    grade: &str,
) -> Result<ExpectationBand, sqlx::Error> {
    sqlx::query_as!(
        ExpectationBand,
        r#"SELECT count(*) AS "samples!",
                  (percentile_cont(0.5) WITHIN GROUP (ORDER BY c.salary_expectation_min))::bigint AS median,
                  (percentile_cont(0.9) WITHIN GROUP (ORDER BY c.salary_expectation_min))::bigint AS p90
           FROM published_candidates c
           WHERE c.specialization = $1 AND c.grade = $2 AND c.salary_expectation_min IS NOT NULL
             AND c.search_status <> 'closed'"#,
        specialization,
        grade,
    )
    .fetch_one(db)
    .await
}

/// `signature` is `None` when the text is too short to compare; the row still counts as signed.
pub async fn set_vacancy_signature(
    db: impl PgExecutor<'_>,
    id: Uuid,
    signature: Option<&[u8]>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE vacancies SET text_signature = $2, signed_at = now() WHERE id = $1",
        id,
        signature,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn unsigned_vacancies(
    db: impl PgExecutor<'_>,
    limit: i64,
) -> Result<Vec<VacancyText>, sqlx::Error> {
    sqlx::query_as!(
        VacancyText,
        "SELECT id, title, description FROM vacancies WHERE signed_at IS NULL ORDER BY id LIMIT $1",
        limit,
    )
    .fetch_all(db)
    .await
}

/// Published vacancies of the last 90 days with a signature, newest first.
pub async fn recent_published_signatures(
    db: impl PgExecutor<'_>,
    exclude_id: Uuid,
    limit: i64,
) -> Result<Vec<SignedVacancy>, sqlx::Error> {
    sqlx::query_as!(
        SignedVacancy,
        r#"SELECT v.id, v.company_id, co.name AS company_name, v.title,
                  v.status AS "status: VacancyStatus", v.text_signature AS "text_signature!"
           FROM vacancies v
           JOIN companies co ON co.id = v.company_id
           WHERE v.status = 'published' AND v.published_at >= now() - interval '90 days'
             AND v.text_signature IS NOT NULL AND v.id <> $1
           ORDER BY v.published_at DESC
           LIMIT $2"#,
        exclude_id,
        limit,
    )
    .fetch_all(db)
    .await
}

/// The company's own drafts and published vacancies with a signature.
pub async fn company_signatures(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    exclude_id: Option<Uuid>,
) -> Result<Vec<SignedVacancy>, sqlx::Error> {
    sqlx::query_as!(
        SignedVacancy,
        r#"SELECT v.id, v.company_id, co.name AS company_name, v.title,
                  v.status AS "status: VacancyStatus", v.text_signature AS "text_signature!"
           FROM vacancies v
           JOIN companies co ON co.id = v.company_id
           WHERE v.company_id = $1 AND v.status <> 'closed' AND v.text_signature IS NOT NULL
             AND ($2::uuid IS NULL OR v.id <> $2)
           ORDER BY v.created_at DESC"#,
        company_id,
        exclude_id,
    )
    .fetch_all(db)
    .await
}

/// Replaces the flags; `flagged_at` keeps the first flagging time until the flags clear.
pub async fn set_vacancy_flags(
    db: impl PgExecutor<'_>,
    id: Uuid,
    flags: &Value,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE vacancies
         SET trust_flags = $2,
             flagged_at = CASE WHEN $2 = '[]'::jsonb THEN NULL ELSE coalesce(flagged_at, now()) END
         WHERE id = $1",
        id,
        flags,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn set_invitation_flags(
    db: impl PgExecutor<'_>,
    id: Uuid,
    flags: &Value,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE invitations SET trust_flags = $2 WHERE id = $1",
        id,
        flags,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn vacancy_flags(db: impl PgExecutor<'_>, id: Uuid) -> Result<Value, sqlx::Error> {
    sqlx::query_scalar!("SELECT trust_flags FROM vacancies WHERE id = $1", id)
        .fetch_one(db)
        .await
}

pub async fn invitation_flags(db: impl PgExecutor<'_>, id: Uuid) -> Result<Value, sqlx::Error> {
    sqlx::query_scalar!("SELECT trust_flags FROM invitations WHERE id = $1", id)
        .fetch_one(db)
        .await
}

pub async fn flagged_vacancies(
    db: impl PgExecutor<'_>,
    limit: i64,
) -> Result<Vec<FlaggedVacancy>, sqlx::Error> {
    sqlx::query_as!(
        FlaggedVacancy,
        r#"SELECT v.id, v.company_id, co.name AS company_name, v.title,
                  v.status AS "status: VacancyStatus", v.salary_min, v.salary_max, v.trust_flags,
                  v.flagged_at AS "flagged_at!"
           FROM vacancies v
           JOIN companies co ON co.id = v.company_id
           WHERE v.flagged_at IS NOT NULL
           ORDER BY v.flagged_at DESC
           LIMIT $1"#,
        limit,
    )
    .fetch_all(db)
    .await
}

pub async fn flagged_invitations(
    db: impl PgExecutor<'_>,
    limit: i64,
) -> Result<Vec<FlaggedInvitation>, sqlx::Error> {
    sqlx::query_as!(
        FlaggedInvitation,
        r#"SELECT i.id, i.company_id, co.name AS company_name, i.title, i.salary_min, i.salary_max,
                  i.trust_flags, i.created_at
           FROM invitations i
           JOIN companies co ON co.id = i.company_id
           WHERE i.trust_flags <> '[]'
           ORDER BY i.created_at DESC
           LIMIT $1"#,
        limit,
    )
    .fetch_all(db)
    .await
}
