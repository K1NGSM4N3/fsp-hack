use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{ApplicationStatus, VacancyStatus};

pub struct VacancyFields<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub skills: &'a [String],
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: &'a str,
    pub city: Option<&'a str>,
}

#[derive(Default)]
pub struct VacancyFilter<'a> {
    pub specialization: Option<&'a str>,
    pub grade: Option<&'a str>,
    pub work_format: Option<&'a str>,
    pub salary_from: Option<i64>,
    pub q: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct CompanyVacancy {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub status: VacancyStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub applications: i64,
}

#[derive(Debug, Clone)]
pub struct PublishedVacancyItem {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_city: Option<String>,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct PublishedVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_city: Option<String>,
    pub company_description: String,
    pub title: String,
    pub description: String,
    pub specialization: String,
    pub grade: String,
    pub skills: Vec<String>,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: String,
    pub city: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CandidateApplication {
    pub id: Uuid,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub company_name: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub cover_letter: String,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct VacancyApplication {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub theta: Option<f64>,
    pub cover_letter: String,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn insert_vacancy(
    db: impl PgExecutor<'_>,
    id: Uuid,
    company_id: Uuid,
    status: VacancyStatus,
    published_at: Option<DateTime<Utc>>,
    v: &VacancyFields<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO vacancies (id, company_id, title, description, specialization, grade, skills,
                                salary_min, salary_max, work_format, city, status, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        id,
        company_id,
        v.title,
        v.description,
        v.specialization,
        v.grade,
        v.skills,
        v.salary_min,
        v.salary_max,
        v.work_format,
        v.city,
        status as VacancyStatus,
        published_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn update_vacancy(
    db: impl PgExecutor<'_>,
    id: Uuid,
    company_id: Uuid,
    status: VacancyStatus,
    v: &VacancyFields<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE vacancies
         SET title = $1, description = $2, specialization = $3, grade = $4, skills = $5,
             salary_min = $6, salary_max = $7, work_format = $8, city = $9, status = $10,
             published_at = CASE WHEN $10 = 'published'::vacancy_status AND published_at IS NULL
                                 THEN now() ELSE published_at END,
             updated_at = now()
         WHERE id = $11 AND company_id = $12",
        v.title,
        v.description,
        v.specialization,
        v.grade,
        v.skills,
        v.salary_min,
        v.salary_max,
        v.work_format,
        v.city,
        status as VacancyStatus,
        id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_vacancies(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<CompanyVacancy>, sqlx::Error> {
    sqlx::query_as!(
        CompanyVacancy,
        r#"SELECT v.id, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min,
                  v.salary_max, v.work_format, v.city, v.status AS "status: VacancyStatus",
                  v.published_at, v.created_at, v.updated_at,
                  (SELECT count(*) FROM applications a
                    WHERE a.vacancy_id = v.id AND a.status <> 'withdrawn') AS "applications!"
           FROM vacancies v
           WHERE v.company_id = $1
           ORDER BY v.created_at DESC"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn company_vacancy(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
    id: Uuid,
) -> Result<Option<CompanyVacancy>, sqlx::Error> {
    sqlx::query_as!(
        CompanyVacancy,
        r#"SELECT v.id, v.title, v.description, v.specialization, v.grade, v.skills, v.salary_min,
                  v.salary_max, v.work_format, v.city, v.status AS "status: VacancyStatus",
                  v.published_at, v.created_at, v.updated_at,
                  (SELECT count(*) FROM applications a
                    WHERE a.vacancy_id = v.id AND a.status <> 'withdrawn') AS "applications!"
           FROM vacancies v
           WHERE v.company_id = $1 AND v.id = $2"#,
        company_id,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn published_vacancies(
    db: impl PgExecutor<'_>,
    f: &VacancyFilter<'_>,
) -> Result<Vec<PublishedVacancyItem>, sqlx::Error> {
    sqlx::query_as!(
        PublishedVacancyItem,
        r#"SELECT v.id, v.company_id, co.name AS company_name, co.city AS company_city, v.title,
                  v.description, v.specialization, v.grade, v.skills, v.salary_min, v.salary_max,
                  v.work_format, v.city, v.published_at, count(*) OVER () AS "total!"
           FROM vacancies v
           JOIN companies co ON co.id = v.company_id
           WHERE v.status = 'published'
             AND ($1::text IS NULL OR v.specialization = $1)
             AND ($2::text IS NULL OR v.grade = $2)
             AND ($3::text IS NULL OR v.work_format = $3)
             AND ($4::bigint IS NULL OR v.salary_max >= $4)
             AND ($5::text IS NULL OR v.search_tsv @@ websearch_to_tsquery('russian', $5))
           ORDER BY v.published_at DESC
           LIMIT $6 OFFSET $7"#,
        f.specialization,
        f.grade,
        f.work_format,
        f.salary_from,
        f.q,
        f.limit,
        f.offset,
    )
    .fetch_all(db)
    .await
}

pub async fn published_vacancy(
    db: impl PgExecutor<'_>,
    id: Uuid,
) -> Result<Option<PublishedVacancy>, sqlx::Error> {
    sqlx::query_as!(
        PublishedVacancy,
        "SELECT v.id, v.company_id, co.name AS company_name, co.city AS company_city,
                co.description AS company_description, v.title, v.description, v.specialization,
                v.grade, v.skills, v.salary_min, v.salary_max, v.work_format, v.city, v.published_at
         FROM vacancies v
         JOIN companies co ON co.id = v.company_id
         WHERE v.id = $1 AND v.status = 'published'",
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_application(
    db: impl PgExecutor<'_>,
    id: Uuid,
    vacancy_id: Uuid,
    candidate_id: Uuid,
    cover_letter: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO applications (id, vacancy_id, candidate_id, cover_letter) VALUES ($1, $2, $3, $4)",
        id,
        vacancy_id,
        candidate_id,
        cover_letter,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn reopen_application(
    db: impl PgExecutor<'_>,
    cover_letter: &str,
    vacancy_id: Uuid,
    candidate_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "UPDATE applications SET status = 'sent', cover_letter = $1, updated_at = now()
         WHERE vacancy_id = $2 AND candidate_id = $3 AND status = 'withdrawn'
         RETURNING id",
        cover_letter,
        vacancy_id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn candidate_applications(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CandidateApplication>, sqlx::Error> {
    sqlx::query_as!(
        CandidateApplication,
        r#"SELECT a.id, a.vacancy_id, v.title AS vacancy_title, co.name AS company_name, v.salary_min,
                  v.salary_max, a.cover_letter, a.status AS "status: ApplicationStatus", a.created_at,
                  a.updated_at
           FROM applications a
           JOIN vacancies v ON v.id = a.vacancy_id
           JOIN companies co ON co.id = v.company_id
           WHERE a.candidate_id = $1
           ORDER BY a.created_at DESC"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn withdraw_application(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE applications SET status = 'withdrawn', updated_at = now()
         WHERE id = $1 AND candidate_id = $2 AND status IN ('sent', 'viewed')",
        id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn vacancy_applications(
    db: impl PgExecutor<'_>,
    vacancy_id: Uuid,
    company_id: Uuid,
) -> Result<Vec<VacancyApplication>, sqlx::Error> {
    sqlx::query_as!(
        VacancyApplication,
        r#"SELECT a.id, a.candidate_id, c.public_code, c.specialization, c.grade, c.theta,
                  a.cover_letter, a.status AS "status: ApplicationStatus", a.created_at, a.updated_at
           FROM applications a
           JOIN vacancies v ON v.id = a.vacancy_id
           JOIN candidates c ON c.user_id = a.candidate_id
           WHERE v.id = $1 AND v.company_id = $2 AND a.status <> 'withdrawn'
           ORDER BY a.created_at DESC"#,
        vacancy_id,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn set_application_status(
    db: impl PgExecutor<'_>,
    status: ApplicationStatus,
    id: Uuid,
    company_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE applications a SET status = $1, updated_at = now()
         FROM vacancies v
         WHERE a.id = $2 AND v.id = a.vacancy_id AND v.company_id = $3 AND a.status NOT IN ('withdrawn')",
        status as ApplicationStatus,
        id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
