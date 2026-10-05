use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::SearchStatus;

#[derive(Debug, Clone)]
pub struct CandidateProfile {
    pub user_id: Uuid,
    pub public_code: String,
    pub full_name_enc: Option<Vec<u8>>,
    pub contacts_enc: Option<Vec<u8>>,
    pub city: Option<String>,
    pub relocation: bool,
    pub work_formats: Vec<String>,
    pub salary_expectation_min: Option<i64>,
    pub about: String,
    pub experience_years: Option<f32>,
    pub roles: Vec<String>,
    pub soft_skills: Vec<String>,
    pub industries: Vec<String>,
    pub experience: Value,
    pub search_status: SearchStatus,
    pub visible: bool,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ProfileUpdate<'a> {
    pub full_name_enc: Option<&'a [u8]>,
    pub contacts_enc: Option<&'a [u8]>,
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
}

#[derive(Debug, Clone)]
pub struct CandidateSkill {
    pub skill: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ResumeUpload {
    pub filename: String,
    pub size_bytes: i32,
    pub recognized_fields: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CandidateInvitable {
    /// Visible, holding an unrevoked `pd_publication` grant and, while under 18, an unrevoked
    /// `guardian_consent`: employers may find and invite.
    pub published: bool,
    pub search_status: SearchStatus,
    pub specialization: Option<String>,
    pub grade: Option<String>,
}

pub async fn insert_candidate(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    public_code: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO candidates (user_id, public_code) VALUES ($1, $2)",
        user_id,
        public_code,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn candidate_profile(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<CandidateProfile>, sqlx::Error> {
    sqlx::query_as!(
        CandidateProfile,
        r#"SELECT user_id, public_code, full_name_enc, contacts_enc, city, relocation, work_formats,
                  salary_expectation_min, about, experience_years, roles, soft_skills, industries,
                  experience, search_status AS "search_status: SearchStatus", visible, show_name,
                  show_fsp, show_experience, specialization, grade, theta, category_assigned_at,
                  last_activity_at, updated_at
           FROM candidates
           WHERE user_id = $1"#,
        user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn update_candidate_profile(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
    p: &ProfileUpdate<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates
         SET full_name_enc = $1, contacts_enc = $2, city = $3, relocation = $4, work_formats = $5,
             salary_expectation_min = $6, about = $7, experience_years = $8, roles = $9,
             soft_skills = $10, industries = $11, experience = $12, updated_at = now(),
             last_activity_at = now()
         WHERE user_id = $13",
        p.full_name_enc,
        p.contacts_enc,
        p.city,
        p.relocation,
        p.work_formats,
        p.salary_expectation_min,
        p.about,
        p.experience_years,
        p.roles,
        p.soft_skills,
        p.industries,
        p.experience,
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn update_candidate_privacy(
    db: impl PgExecutor<'_>,
    visible: bool,
    show_name: bool,
    show_fsp: bool,
    show_experience: bool,
    search_status: SearchStatus,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates
         SET visible = $1, show_name = $2, show_fsp = $3, show_experience = $4,
             search_status = $5, updated_at = now()
         WHERE user_id = $6",
        visible,
        show_name,
        show_fsp,
        show_experience,
        search_status as SearchStatus,
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn candidate_skills(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CandidateSkill>, sqlx::Error> {
    sqlx::query_as!(
        CandidateSkill,
        "SELECT skill, source FROM candidate_skills WHERE candidate_id = $1 ORDER BY skill, source",
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn delete_candidate_skills_by_source(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    source: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM candidate_skills WHERE candidate_id = $1 AND source = $2",
        candidate_id,
        source,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_candidate_skill(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
    skill: &str,
    source: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO candidate_skills (candidate_id, skill, source) VALUES ($1, $2, $3)
         ON CONFLICT DO NOTHING",
        candidate_id,
        skill,
        source,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn insert_resume_upload(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
    filename: &str,
    size_bytes: i32,
    recognized_fields: &[String],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO resume_uploads (id, candidate_id, filename, size_bytes, recognized_fields)
         VALUES ($1, $2, $3, $4, $5)",
        id,
        candidate_id,
        filename,
        size_bytes,
        recognized_fields,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn resume_uploads(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<ResumeUpload>, sqlx::Error> {
    sqlx::query_as!(
        ResumeUpload,
        "SELECT filename, size_bytes, recognized_fields, created_at
         FROM resume_uploads
         WHERE candidate_id = $1
         ORDER BY created_at DESC",
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn touch_candidate_activity(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates SET last_activity_at = now() WHERE user_id = $1",
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn candidate_invitable(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<CandidateInvitable>, sqlx::Error> {
    sqlx::query_as!(
        CandidateInvitable,
        r#"SELECT c.visible
                  AND EXISTS (
                      SELECT 1 FROM user_consents uc
                      JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
                      WHERE uc.user_id = c.user_id AND uc.revoked_at IS NULL)
                  AND (c.birth_date IS NULL OR c.birth_date + interval '18 years' <= current_date
                       OR EXISTS (SELECT 1 FROM user_consents gc
                                   JOIN consent_documents gd ON gd.id = gc.document_id AND gd.kind = 'guardian_consent'
                                   WHERE gc.user_id = c.user_id AND gc.revoked_at IS NULL)) AS "published!",
                  c.search_status AS "search_status: SearchStatus", c.specialization, c.grade
           FROM candidates c
           WHERE c.user_id = $1"#,
        user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn set_search_status(
    db: impl PgExecutor<'_>,
    search_status: SearchStatus,
    user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates SET search_status = $1, updated_at = now() WHERE user_id = $2",
        search_status as SearchStatus,
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
