use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::TaskAssignmentStatus;

pub struct NewTask<'a> {
    pub id: Uuid,
    pub company_id: Uuid,
    pub specialization: &'a str,
    pub grades: &'a [String],
    pub skills: &'a [String],
    pub title: &'a str,
    pub body: &'a str,
    pub kind: &'a str,
}

#[derive(Debug, Clone)]
pub struct CompanyTask {
    pub id: Uuid,
    pub specialization: String,
    pub grades: Vec<String>,
    pub skills: Vec<String>,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub submissions: i64,
    pub reviewed: i64,
}

#[derive(Debug, Clone)]
pub struct TaskSubmission {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub public_code: String,
    pub specialization: Option<String>,
    pub grade: Option<String>,
    pub status: TaskAssignmentStatus,
    pub answer: Option<String>,
    pub rating: Option<i16>,
    pub review_comment: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CandidateAssignment {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub skills: Vec<String>,
    pub company_name: Option<String>,
    pub status: TaskAssignmentStatus,
    pub answer: Option<String>,
    pub rating: Option<i16>,
    pub review_comment: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

pub async fn insert_task(db: impl PgExecutor<'_>, t: &NewTask<'_>) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO tasks (id, company_id, specialization, grades, skills, title, body, kind)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        t.id,
        t.company_id,
        t.specialization,
        t.grades,
        t.skills,
        t.title,
        t.body,
        t.kind,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn company_tasks(
    db: impl PgExecutor<'_>,
    company_id: Uuid,
) -> Result<Vec<CompanyTask>, sqlx::Error> {
    sqlx::query_as!(
        CompanyTask,
        r#"SELECT t.id, t.specialization, t.grades, t.skills, t.title, t.body, t.kind, t.active,
                  t.created_at,
                  count(a.id) FILTER (WHERE a.status IN ('submitted', 'reviewed')) AS "submissions!",
                  count(a.id) FILTER (WHERE a.status = 'reviewed') AS "reviewed!"
           FROM tasks t
           LEFT JOIN task_assignments a ON a.task_id = t.id
           WHERE t.company_id = $1
           GROUP BY t.id
           ORDER BY t.created_at DESC"#,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn set_task_active(
    db: impl PgExecutor<'_>,
    active: bool,
    id: Uuid,
    company_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE tasks SET active = $1 WHERE id = $2 AND company_id = $3",
        active,
        id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn task_submissions(
    db: impl PgExecutor<'_>,
    task_id: Uuid,
    company_id: Uuid,
) -> Result<Vec<TaskSubmission>, sqlx::Error> {
    sqlx::query_as!(
        TaskSubmission,
        r#"SELECT a.id, a.candidate_id, c.public_code, c.specialization, c.grade,
                  a.status AS "status: TaskAssignmentStatus", a.answer, a.rating, a.review_comment,
                  a.submitted_at
           FROM task_assignments a
           JOIN tasks t ON t.id = a.task_id
           JOIN candidates c ON c.user_id = a.candidate_id
           WHERE t.id = $1 AND t.company_id = $2 AND a.status IN ('submitted', 'reviewed')
           ORDER BY a.submitted_at DESC"#,
        task_id,
        company_id,
    )
    .fetch_all(db)
    .await
}

pub async fn review_submission(
    db: impl PgExecutor<'_>,
    rating: i16,
    review_comment: Option<&str>,
    id: Uuid,
    task_id: Uuid,
    company_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE task_assignments a
         SET status = 'reviewed', rating = $1, review_comment = $2, reviewed_at = now()
         FROM tasks t
         WHERE a.id = $3 AND a.task_id = t.id AND t.id = $4 AND t.company_id = $5
           AND a.status IN ('submitted', 'reviewed')",
        rating,
        review_comment,
        id,
        task_id,
        company_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn expire_assignments(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE task_assignments SET status = 'expired'
         WHERE candidate_id = $1 AND status = 'assigned' AND due_at < now()",
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn last_assignment_at(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT max(assigned_at) AS assigned_at FROM task_assignments WHERE candidate_id = $1",
        candidate_id,
    )
    .fetch_one(db)
    .await
}

pub async fn has_open_assignment(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "open!" FROM task_assignments
           WHERE candidate_id = $1 AND status = 'assigned'"#,
        candidate_id,
    )
    .fetch_one(db)
    .await
}

/// Oldest active task for the candidate's category that they have not seen yet; the most
/// overlapping skills first. Overlap counts distinct skills (one skill held from several sources
/// is still one), and FSP-confirmed skills only while the candidate shows FSP data — the rule of
/// [`crate::pool::searchable_pool`].
pub async fn pick_task_for(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT t.id
         FROM tasks t, candidates c
         WHERE c.user_id = $1
           AND t.active
           AND t.specialization = c.specialization
           AND c.grade = ANY (t.grades)
           AND NOT EXISTS (SELECT 1 FROM task_assignments a WHERE a.task_id = t.id AND a.candidate_id = c.user_id)
         ORDER BY (SELECT count(DISTINCT s.skill) FROM candidate_skills s
                    WHERE s.candidate_id = c.user_id AND s.skill = ANY (t.skills)
                      AND (s.source <> 'fsp' OR c.show_fsp)) DESC,
                  t.created_at
         LIMIT 1",
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_assignment(
    db: impl PgExecutor<'_>,
    id: Uuid,
    task_id: Uuid,
    candidate_id: Uuid,
    due_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO task_assignments (id, task_id, candidate_id, due_at) VALUES ($1, $2, $3, $4)",
        id,
        task_id,
        candidate_id,
        due_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn candidate_assignments(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<CandidateAssignment>, sqlx::Error> {
    sqlx::query_as!(
        CandidateAssignment,
        r#"SELECT a.id, t.title, t.body, t.kind, t.skills, co.name AS "company_name?",
                  a.status AS "status: TaskAssignmentStatus", a.answer, a.rating, a.review_comment,
                  a.assigned_at, a.due_at, a.submitted_at, a.reviewed_at
           FROM task_assignments a
           JOIN tasks t ON t.id = a.task_id
           LEFT JOIN companies co ON co.id = t.company_id
           WHERE a.candidate_id = $1
           ORDER BY a.assigned_at DESC"#,
        candidate_id,
    )
    .fetch_all(db)
    .await
}

pub async fn submit_assignment(
    db: impl PgExecutor<'_>,
    answer: &str,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE task_assignments SET status = 'submitted', answer = $1, submitted_at = now()
         WHERE id = $2 AND candidate_id = $3 AND status = 'assigned' AND due_at >= now()",
        answer,
        id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn skip_assignment(
    db: impl PgExecutor<'_>,
    id: Uuid,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE task_assignments SET status = 'skipped'
         WHERE id = $1 AND candidate_id = $2 AND status = 'assigned'",
        id,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}
