use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{InvitationStatus, SearchStatus};

#[derive(Debug, Clone)]
pub struct PoolCandidate {
    pub user_id: Uuid,
    pub public_code: String,
    pub specialization: String,
    pub grade: String,
    pub theta: Option<f64>,
    pub salary_expectation_min: Option<i64>,
    pub work_formats: Vec<String>,
    pub city: Option<String>,
    pub relocation: bool,
    pub last_activity_at: DateTime<Utc>,
    pub industries: Vec<String>,
    pub show_fsp: bool,
    pub achievements: Option<Value>,
    pub skills: Vec<String>,
    pub skill_sources: Vec<String>,
    pub text_rank: f32,
}

#[derive(Debug, Clone)]
pub struct CandidateCard {
    pub user_id: Uuid,
    pub public_code: String,
    pub full_name_enc: Option<Vec<u8>>,
    pub show_name: bool,
    pub show_fsp: bool,
    pub show_experience: bool,
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
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub category_assigned_at: Option<DateTime<Utc>>,
    pub last_activity_at: DateTime<Utc>,
    pub visible: bool,
    pub achievements: Option<Value>,
    pub skills: Vec<String>,
    pub skill_sources: Vec<String>,
    pub task_rating: Option<f32>,
    pub tasks_done: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct CompanyRelation {
    pub candidate_id: Uuid,
    pub invitation_status: Option<InvitationStatus>,
    pub applied: bool,
}

/// Searchable candidates of one specialization with everything the matching engine needs.
/// `skills`/`skill_sources` are parallel arrays; `text_rank` scores the "about" text against an
/// OR-query built from the need description.
pub async fn candidate_pool(
    db: impl PgExecutor<'_>,
    text_query: Option<&str>,
    specialization: &str,
    grade: Option<&str>,
) -> Result<Vec<PoolCandidate>, sqlx::Error> {
    sqlx::query_as!(
        PoolCandidate,
        r#"SELECT c.user_id, c.public_code, c.specialization AS "specialization!", c.grade AS "grade!",
                  c.theta, c.salary_expectation_min, c.work_formats, c.city, c.relocation,
                  c.last_activity_at, c.industries, c.show_fsp, f.achievements AS "achievements?",
                  coalesce(array_agg(s.skill ORDER BY s.skill, s.source)
                           FILTER (WHERE s.skill IS NOT NULL), '{}') AS "skills!",
                  coalesce(array_agg(s.source ORDER BY s.skill, s.source)
                           FILTER (WHERE s.skill IS NOT NULL), '{}') AS "skill_sources!",
                  CASE WHEN $1::text IS NULL THEN 0::real
                       ELSE ts_rank_cd(c.about_tsv, to_tsquery('russian', $1))::real END AS "text_rank!"
           FROM candidates c
           JOIN users u ON u.id = c.user_id AND u.deleted_at IS NULL
           LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
           LEFT JOIN candidate_skills s ON s.candidate_id = c.user_id
           WHERE c.visible
             AND c.search_status <> 'closed'
             AND c.specialization = $2
             AND c.grade IS NOT NULL
             AND ($3::text IS NULL OR c.grade = $3)
           GROUP BY c.user_id, f.achievements"#,
        text_query,
        specialization,
        grade,
    )
    .fetch_all(db)
    .await
}

pub async fn candidate_cards(
    db: impl PgExecutor<'_>,
    ids: &[Uuid],
) -> Result<Vec<CandidateCard>, sqlx::Error> {
    sqlx::query_as!(
        CandidateCard,
        r#"SELECT c.user_id, c.public_code, c.full_name_enc, c.show_name, c.show_fsp, c.show_experience,
                  c.city, c.relocation, c.work_formats, c.salary_expectation_min, c.about,
                  c.experience_years, c.roles, c.soft_skills, c.industries, c.experience,
                  c.search_status AS "search_status: SearchStatus", c.specialization, c.grade, c.theta,
                  c.category_assigned_at, c.last_activity_at, c.visible, f.achievements AS "achievements?",
                  coalesce(array_agg(s.skill ORDER BY s.skill, s.source)
                           FILTER (WHERE s.skill IS NOT NULL), '{}') AS "skills!",
                  coalesce(array_agg(s.source ORDER BY s.skill, s.source)
                           FILTER (WHERE s.skill IS NOT NULL), '{}') AS "skill_sources!",
                  (SELECT avg(a.rating)::real FROM task_assignments a
                    WHERE a.candidate_id = c.user_id AND a.status = 'reviewed') AS task_rating,
                  (SELECT count(*) FROM task_assignments a
                    WHERE a.candidate_id = c.user_id AND a.status IN ('submitted', 'reviewed')) AS "tasks_done!"
           FROM candidates c
           LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
           LEFT JOIN candidate_skills s ON s.candidate_id = c.user_id
           WHERE c.user_id = ANY ($1)
           GROUP BY c.user_id, f.achievements"#,
        ids,
    )
    .fetch_all(db)
    .await
}

pub async fn company_relations(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    ids: &[Uuid],
) -> Result<Vec<CompanyRelation>, sqlx::Error> {
    sqlx::query_as!(
        CompanyRelation,
        r#"SELECT c.user_id AS candidate_id,
                  (SELECT i.status FROM invitations i WHERE i.company_id = $1 AND i.candidate_id = c.user_id
                    ORDER BY i.created_at DESC LIMIT 1) AS "invitation_status: InvitationStatus",
                  EXISTS (SELECT 1 FROM applications a JOIN vacancies v ON v.id = a.vacancy_id
                           WHERE v.company_id = $1 AND a.candidate_id = c.user_id
                             AND a.status <> 'withdrawn') AS "applied!"
           FROM candidates c
           WHERE c.user_id = ANY ($2)"#,
        company_id,
        ids,
    )
    .fetch_all(db)
    .await
}
