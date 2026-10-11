use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

pub struct NewSurvey<'a> {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub direction: &'a str,
    pub specialization: &'a str,
    pub claimed_grade: &'a str,
    pub skills: &'a [String],
    pub industries: &'a [String],
    pub experience_years: Option<f32>,
    pub self_assessment: &'a Value,
}

#[derive(Debug, Clone)]
pub struct Survey {
    pub id: Uuid,
    pub direction: String,
    pub specialization: String,
    pub claimed_grade: String,
    pub skills: Vec<String>,
    pub industries: Vec<String>,
    pub experience_years: Option<f32>,
    pub self_assessment: Value,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_survey(db: impl PgExecutor<'_>, s: &NewSurvey<'_>) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO survey_responses (id, candidate_id, direction, specialization, claimed_grade, skills,
                                       industries, experience_years, self_assessment)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        s.id,
        s.candidate_id,
        s.direction,
        s.specialization,
        s.claimed_grade,
        s.skills,
        s.industries,
        s.experience_years,
        s.self_assessment,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn survey_history(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<Survey>, sqlx::Error> {
    sqlx::query_as!(
        Survey,
        "SELECT id, direction, specialization, claimed_grade, skills, industries, experience_years,
                self_assessment, created_at
         FROM survey_responses
         WHERE candidate_id = $1
         ORDER BY created_at DESC",
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn latest_survey(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<Survey>, sqlx::Error> {
    sqlx::query_as!(
        Survey,
        "SELECT id, direction, specialization, claimed_grade, skills, industries, experience_years,
                self_assessment, created_at
         FROM survey_responses
         WHERE candidate_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
        candidate_id,
    )
    .fetch_optional(db)
    .await
}
