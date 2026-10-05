use sqlx::PgExecutor;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CandidateOnboarding {
    pub public_code: String,
    pub has_profile: bool,
    pub has_survey: bool,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub active_session_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct CompanyByOwner {
    pub id: Uuid,
    pub name: String,
}

pub async fn candidate_onboarding(
    db: impl PgExecutor<'_>,
    user_id: Uuid,
) -> Result<Option<CandidateOnboarding>, sqlx::Error> {
    sqlx::query_as!(
        CandidateOnboarding,
        r#"SELECT c.public_code,
                  c.full_name_enc IS NOT NULL AS "has_profile!",
                  EXISTS (SELECT 1 FROM survey_responses s WHERE s.candidate_id = c.user_id) AS "has_survey!",
                  c.specialization,
                  c.grade,
                  (SELECT a.id FROM assessment_sessions a
                    WHERE a.candidate_id = c.user_id AND a.status = 'in_progress') AS active_session_id
           FROM candidates c
           WHERE c.user_id = $1"#,
        user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn company_by_owner(
    db: impl PgExecutor<'_>,
    owner_user_id: Uuid,
) -> Result<Option<CompanyByOwner>, sqlx::Error> {
    sqlx::query_as!(
        CompanyByOwner,
        "SELECT id, name FROM companies WHERE owner_user_id = $1",
        owner_user_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn delete_candidate(db: impl PgExecutor<'_>, user_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!("DELETE FROM candidates WHERE user_id = $1", user_id)
        .execute(db)
        .await
        .map(|r| r.rows_affected())
}

pub async fn delete_company_of(
    db: impl PgExecutor<'_>,
    owner_user_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM companies WHERE owner_user_id = $1",
        owner_user_id
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn hide_candidate(db: impl PgExecutor<'_>, user_id: Uuid) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates SET visible = false, updated_at = now() WHERE user_id = $1",
        user_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
