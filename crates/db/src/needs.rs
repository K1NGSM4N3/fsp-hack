use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;
use uuid::Uuid;

pub struct NeedFields<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grades: &'a [String],
    pub allow_adjacent_grades: bool,
    pub must_skills: &'a [String],
    pub nice_skills: &'a [String],
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: &'a [String],
    pub city: Option<&'a str>,
    pub industries: &'a [String],
}

#[derive(Debug, Clone)]
pub struct CompanyNeed {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grades: Vec<String>,
    pub allow_adjacent_grades: bool,
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_formats: Vec<String>,
    pub city: Option<String>,
    pub industries: Vec<String>,
    pub filters: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub shortlist_generated_at: Option<DateTime<Utc>>,
    pub shortlisted: i64,
    pub invited: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct ShortlistGeneration {
    pub previous: Option<DateTime<Utc>>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ShortlistItem {
    pub candidate_id: Uuid,
    pub score: f64,
    pub rank: i32,
    pub breakdown: Value,
    pub in_current: bool,
    pub pinned: bool,
    pub dismissed: bool,
    pub first_seen_at: DateTime<Utc>,
}

pub async fn insert_need(
    db: impl PgExecutor<'_>,
    id: Uuid,
    company_id: Uuid,
    n: &NeedFields<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO hiring_needs (id, company_id, title, description, specialization, grades,
                                   allow_adjacent_grades, must_skills, nice_skills, salary_min,
                                   salary_max, work_formats, city, industries)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        id,
        company_id,
        n.title,
        n.description,
        n.specialization,
        n.grades,
        n.allow_adjacent_grades,
        n.must_skills,
        n.nice_skills,
        n.salary_min,
        n.salary_max,
        n.work_formats,
        n.city,
        n.industries,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn update_need(
    db: impl PgExecutor<'_>,
    id: Uuid,
    company_id: Uuid,
    status: &str,
    n: &NeedFields<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE hiring_needs
         SET title = $1, description = $2, specialization = $3, grades = $4,
             allow_adjacent_grades = $5, must_skills = $6, nice_skills = $7, salary_min = $8,
             salary_max = $9, work_formats = $10, city = $11, industries = $12, status = $13,
             updated_at = now()
         WHERE id = $14 AND company_id = $15",
        n.title,
        n.description,
        n.specialization,
        n.grades,
        n.allow_adjacent_grades,
        n.must_skills,
        n.nice_skills,
        n.salary_min,
        n.salary_max,
        n.work_formats,
        n.city,
        n.industries,
        status,
        id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_needs(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<CompanyNeed>, sqlx::Error> {
    sqlx::query_as!(
        CompanyNeed,
        r#"SELECT n.id, n.title, n.description, n.specialization, n.grades, n.allow_adjacent_grades,
                  n.must_skills, n.nice_skills, n.salary_min, n.salary_max, n.work_formats, n.city,
                  n.industries, n.filters, n.status, n.created_at, n.updated_at,
                  n.shortlist_generated_at,
                  (SELECT count(*) FROM shortlist_items s
                    WHERE s.need_id = n.id AND s.in_current AND NOT s.dismissed) AS "shortlisted!",
                  (SELECT count(*) FROM invitations i WHERE i.need_id = n.id) AS "invited!"
           FROM hiring_needs n
           WHERE n.company_id = $1
           ORDER BY n.created_at DESC"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn company_need(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<Option<CompanyNeed>, sqlx::Error> {
    sqlx::query_as!(
        CompanyNeed,
        r#"SELECT n.id, n.title, n.description, n.specialization, n.grades, n.allow_adjacent_grades,
                  n.must_skills, n.nice_skills, n.salary_min, n.salary_max, n.work_formats, n.city,
                  n.industries, n.filters, n.status, n.created_at, n.updated_at,
                  n.shortlist_generated_at,
                  (SELECT count(*) FROM shortlist_items s
                    WHERE s.need_id = n.id AND s.in_current AND NOT s.dismissed) AS "shortlisted!",
                  (SELECT count(*) FROM invitations i WHERE i.need_id = n.id) AS "invited!"
           FROM hiring_needs n
           WHERE n.company_id = $1 AND n.id = $2"#,
        company_id,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn delete_need(
    db: impl PgExecutor<'_>,
    id: Uuid,
    company_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM hiring_needs WHERE id = $1 AND company_id = $2",
        id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn start_shortlist_generation(
    db: impl PgExecutor<'_>,
    filters: &Value,
    id: Uuid,
) -> Result<ShortlistGeneration, sqlx::Error> {
    sqlx::query_as!(
        ShortlistGeneration,
        r#"UPDATE hiring_needs n
           SET filters = $1, shortlist_generated_at = now()
           FROM (SELECT shortlist_generated_at AS previous FROM hiring_needs WHERE id = $2 FOR UPDATE) prev
           WHERE n.id = $2
           RETURNING prev.previous, n.shortlist_generated_at AS "generated_at!""#,
        filters,
        id,
    )
    .fetch_one(db)
    .await
}

pub async fn mark_shortlist_stale(
    db: impl PgExecutor<'_>,
    need_id: Uuid,
    keep: &[Uuid],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE shortlist_items SET in_current = false, updated_at = now()
         WHERE need_id = $1 AND NOT (candidate_id = ANY ($2))",
        need_id,
        keep,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn upsert_shortlist_item(
    db: impl PgExecutor<'_>,
    need_id: Uuid,
    candidate_id: Uuid,
    score: f64,
    rank: i32,
    breakdown: &Value,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO shortlist_items (need_id, candidate_id, score, rank, breakdown, in_current)
         VALUES ($1, $2, $3, $4, $5, true)
         ON CONFLICT (need_id, candidate_id) DO UPDATE
         SET score = EXCLUDED.score, rank = EXCLUDED.rank, breakdown = EXCLUDED.breakdown,
             in_current = true, updated_at = now()",
        need_id,
        candidate_id,
        score,
        rank,
        breakdown,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn shortlist_items(
    db: impl PgExecutor<'_>,
    need_id: Uuid,
) -> Result<Vec<ShortlistItem>, sqlx::Error> {
    sqlx::query_as!(
        ShortlistItem,
        "SELECT candidate_id, score, rank, breakdown, in_current, pinned, dismissed, first_seen_at
         FROM shortlist_items
         WHERE need_id = $1
         ORDER BY in_current DESC, rank",
        need_id,
    )
    .fetch_all(db)
    .await
}

pub async fn patch_shortlist_item(
    db: impl PgExecutor<'_>,
    pinned: Option<bool>,
    dismissed: Option<bool>,
    company_id: Uuid,
    need_id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE shortlist_items s
         SET pinned = coalesce($1, s.pinned), dismissed = coalesce($2, s.dismissed), updated_at = now()
         FROM hiring_needs n
         WHERE s.need_id = n.id AND n.company_id = $3 AND s.need_id = $4 AND s.candidate_id = $5",
        pinned,
        dismissed,
        company_id,
        need_id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Demo seed: needs inserted in one transaction share `now()`, which makes their order unstable.
pub async fn set_need_created_at(
    db: impl PgExecutor<'_>,
    id: Uuid,
    at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE hiring_needs SET created_at = $2, updated_at = $2 WHERE id = $1",
        id,
        at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
