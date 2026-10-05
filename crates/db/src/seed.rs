use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{AssessmentStatus, SearchStatus, UserRole};

pub struct SeedCandidate<'a> {
    pub user_id: Uuid,
    pub public_code: &'a str,
    pub full_name_enc: &'a [u8],
    pub contacts_enc: &'a [u8],
    pub city: Option<&'a str>,
    pub relocation: bool,
    pub work_formats: &'a [String],
    pub salary_expectation_min: Option<i64>,
    pub about: &'a str,
    pub experience_years: Option<f32>,
    pub roles: &'a [String],
    pub soft_skills: &'a [String],
    pub industries: &'a [String],
    pub experience: &'a Value,
    pub search_status: SearchStatus,
    pub show_name: bool,
    pub last_activity_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct SeedCompany<'a> {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub name: &'a str,
    pub website: &'a str,
    pub description: &'a str,
    pub industry: &'a str,
    pub city: &'a str,
    pub size: &'a str,
    pub contact_email: &'a str,
    pub contact_telegram: &'a str,
}

pub struct SeedSession<'a> {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub attempt_kind: &'a str,
    pub status: AssessmentStatus,
    pub state: &'a Value,
    pub theta: f64,
    pub se: f64,
    pub measured_grade: Option<&'a str>,
    pub confident_above_next: bool,
    pub person_fit_lz: f64,
    pub aberrant: bool,
    pub items_count: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

pub async fn demo_seeded(db: impl PgExecutor<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT count(*) AS "count!" FROM companies"#)
        .fetch_one(db)
        .await
}

pub async fn seed_verified_user(
    db: impl PgExecutor<'_>,
    id: Uuid,
    role: UserRole,
    email_enc: &[u8],
    email_bidx: &[u8],
    password_hash: &str,
    created_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO users (id, role, email_enc, email_bidx, password_hash, email_verified_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $6)",
        id,
        role as UserRole,
        email_enc,
        email_bidx,
        password_hash,
        created_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn seed_candidate(
    db: impl PgExecutor<'_>,
    c: &SeedCandidate<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO candidates (user_id, public_code, full_name_enc, contacts_enc, city, relocation,
                                 work_formats, salary_expectation_min, about, experience_years, roles,
                                 soft_skills, industries, experience, search_status, show_name,
                                 last_activity_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
        c.user_id,
        c.public_code,
        c.full_name_enc,
        c.contacts_enc,
        c.city,
        c.relocation,
        c.work_formats,
        c.salary_expectation_min,
        c.about,
        c.experience_years,
        c.roles,
        c.soft_skills,
        c.industries,
        c.experience,
        c.search_status as SearchStatus,
        c.show_name,
        c.last_activity_at,
        c.created_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn seed_company(
    db: impl PgExecutor<'_>,
    c: &SeedCompany<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO companies (id, owner_user_id, name, description, industry, city, size,
                                contact_email, contact_telegram, website)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        c.id,
        c.owner_user_id,
        c.name,
        c.description,
        c.industry,
        c.city,
        c.size,
        c.contact_email,
        c.contact_telegram,
        c.website,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn seed_fsp_link(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    fsp_id: &str,
    achievements: &Value,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO fsp_links (candidate_id, fsp_id, linked_via, last_sync_at, achievements)
         VALUES ($1, $2, 'seed', now(), $3)",
        candidate_id,
        fsp_id,
        achievements,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn seed_session(
    db: impl PgExecutor<'_>,
    s: &SeedSession<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO assessment_sessions (id, candidate_id, specialization, claimed_grade, attempt_kind,
                                          status, state, theta, se, measured_grade, confident_above_next,
                                          person_fit_lz, aberrant, items_count, started_at, finished_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        s.id,
        s.candidate_id,
        s.specialization,
        s.claimed_grade,
        s.attempt_kind,
        s.status as AssessmentStatus,
        s.state,
        s.theta,
        s.se,
        s.measured_grade,
        s.confident_above_next,
        s.person_fit_lz,
        s.aberrant,
        s.items_count,
        s.started_at,
        s.finished_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn seed_consent(
    db: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    document_id: Uuid,
    granted_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO user_consents (id, user_id, document_id, granted_at, user_agent)
         VALUES ($1, $2, $3, $4, 'seed')",
        id,
        user_id,
        document_id,
        granted_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub struct SeedProctoring {
    pub session_id: Uuid,
    pub candidate_id: Uuid,
    pub camera: bool,
    pub consent_id: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub purged_at: Option<DateTime<Utc>>,
}

pub async fn seed_proctoring(
    db: impl PgExecutor<'_>,
    p: &SeedProctoring,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO proctoring_sessions (session_id, candidate_id, camera, consent_id, mime,
                                          started_at, purged_at)
         VALUES ($1, $2, $3, $4, CASE WHEN $3 THEN 'video/webm;codecs=vp8' END, $5, $6)",
        p.session_id,
        p.candidate_id,
        p.camera,
        p.consent_id,
        p.started_at,
        p.purged_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Chunk metadata of a recording whose files never existed (demo data): `count` chunks of
/// `size` bytes, one every 10 seconds from `from`.
pub async fn seed_chunks(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    count: i32,
    size: i32,
    from: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO proctoring_chunks (session_id, seq, size, received_at)
         SELECT $1, g, $3, $4::timestamptz + make_interval(secs => (g + 1) * 10)
         FROM generate_series(0, $2 - 1) g",
        session_id,
        count,
        size,
        from,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
