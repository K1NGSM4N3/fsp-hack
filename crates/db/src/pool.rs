use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{InvitationStatus, SearchStatus};

#[derive(Debug, Clone)]
pub struct PoolCandidate {
    pub user_id: Uuid,
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
    /// Under 18 today; such a candidate is here only with the representative's consent.
    pub minor: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TextRank {
    pub user_id: Uuid,
    pub rank: f32,
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
    /// Under 18 today; the birth date itself never leaves the database for employers.
    pub minor: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CompanyRelation {
    pub candidate_id: Uuid,
    pub invitation_status: Option<InvitationStatus>,
    pub applied: bool,
}

/// Searchable candidates of one specialization with everything the matching engine needs,
/// ordered by id. `skills`/`skill_sources` are parallel arrays. The API caches the result per
/// specialization (`api::pool`), so it carries no per-request filters: grade, «Только 18+» and
/// the rest are applied to the cached snapshot.
///
/// Searchable means visible, not closed, holding an unrevoked `pd_publication` grant and, while
/// under 18, an unrevoked `guardian_consent` given by the legal representative. The grant checks
/// repeat what `PUT /candidate/privacy` enforces, so a stale `visible` flag can never publish a
/// profile; both semi-joins read `user_consents_document_idx`. Skills are aggregated per
/// candidate in a lateral subquery: grouping the whole joined set was the slowest step of the
/// query at 10 000 candidates.
pub async fn searchable_pool(
    db: impl PgExecutor<'_>,
    specialization: &str,
) -> Result<Vec<PoolCandidate>, sqlx::Error> {
    sqlx::query_as!(
        PoolCandidate,
        r#"SELECT c.user_id, c.grade AS "grade!", c.theta, c.salary_expectation_min, c.work_formats,
                  c.city, c.relocation, c.last_activity_at, c.industries, c.show_fsp,
                  f.achievements AS "achievements?", sk.skills AS "skills!",
                  sk.sources AS "skill_sources!",
                  (c.birth_date IS NOT NULL AND c.birth_date + interval '18 years' > current_date) AS "minor!"
           FROM candidates c
           JOIN users u ON u.id = c.user_id AND u.deleted_at IS NULL
           LEFT JOIN fsp_links f ON f.candidate_id = c.user_id
           CROSS JOIN LATERAL (
               SELECT coalesce(array_agg(s.skill ORDER BY s.skill, s.source), '{}') AS skills,
                      coalesce(array_agg(s.source ORDER BY s.skill, s.source), '{}') AS sources
               FROM candidate_skills s
               WHERE s.candidate_id = c.user_id
           ) sk
           WHERE c.visible
             AND c.search_status <> 'closed'
             AND EXISTS (SELECT 1 FROM user_consents uc
                          JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
                          WHERE uc.user_id = c.user_id AND uc.revoked_at IS NULL)
             AND (c.birth_date IS NULL OR c.birth_date + interval '18 years' <= current_date
                  OR EXISTS (SELECT 1 FROM user_consents gc
                              JOIN consent_documents gd ON gd.id = gc.document_id AND gd.kind = 'guardian_consent'
                              WHERE gc.user_id = c.user_id AND gc.revoked_at IS NULL))
             AND c.specialization = $1
             AND c.grade IS NOT NULL
           ORDER BY c.user_id"#,
        specialization,
    )
    .fetch_all(db)
    .await
}

/// `ts_rank_cd` of each candidate's "about" against an OR-query, for the specialization's
/// candidates that match it at all (the rest rank 0), ordered by id. Visibility is not checked
/// here: ranks are only attached to candidates already in [`searchable_pool`].
pub async fn text_ranks(
    db: impl PgExecutor<'_>,
    specialization: &str,
    text_query: &str,
) -> Result<Vec<TextRank>, sqlx::Error> {
    sqlx::query_as!(
        TextRank,
        r#"SELECT c.user_id, ts_rank_cd(c.about_tsv, q)::real AS "rank!"
           FROM candidates c, to_tsquery('russian', $2) q
           WHERE c.specialization = $1 AND c.grade IS NOT NULL
             AND c.visible AND c.search_status <> 'closed'
             AND c.about_tsv @@ q
           ORDER BY c.user_id"#,
        specialization,
        text_query,
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
                    WHERE a.candidate_id = c.user_id AND a.status IN ('submitted', 'reviewed')) AS "tasks_done!",
                  (c.birth_date IS NOT NULL AND c.birth_date + interval '18 years' > current_date) AS "minor!"
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

/// Of `ids`, those an employer may currently find: the same rule as [`searchable_pool`].
/// Shown shortlists and search pages are re-checked with it against the live tables, so hiding a
/// profile takes effect on the next request whatever the age of the cached pool.
pub async fn searchable_among(
    db: impl PgExecutor<'_>,
    ids: &[Uuid],
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT c.user_id FROM candidates c
         WHERE c.user_id = ANY ($1)
           AND c.visible
           AND c.search_status <> 'closed'
           AND EXISTS (SELECT 1 FROM user_consents uc
                        JOIN consent_documents d ON d.id = uc.document_id AND d.kind = 'pd_publication'
                        WHERE uc.user_id = c.user_id AND uc.revoked_at IS NULL)
           AND (c.birth_date IS NULL OR c.birth_date + interval '18 years' <= current_date
                OR EXISTS (SELECT 1 FROM user_consents gc
                            JOIN consent_documents gd ON gd.id = gc.document_id AND gd.kind = 'guardian_consent'
                            WHERE gc.user_id = c.user_id AND gc.revoked_at IS NULL))",
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
