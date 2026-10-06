use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

pub struct NewGradeAssignment<'a> {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub specialization: &'a str,
    pub grade: &'a str,
    pub theta: f64,
    pub se: f64,
    pub session_id: Option<Uuid>,
    pub reason: &'a str,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GradeHistory {
    pub id: Uuid,
    pub specialization: String,
    pub grade: String,
    pub theta: f64,
    pub se: f64,
    pub session_id: Option<Uuid>,
    pub reason: String,
    pub assigned_at: DateTime<Utc>,
    /// Voided by a moderator; the row stays in the history.
    pub annulled_at: Option<DateTime<Utc>>,
}

pub async fn insert_grade_assignment(
    db: impl PgExecutor<'_>,
    g: &NewGradeAssignment<'_>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO grade_assignments (id, candidate_id, specialization, grade, theta, se, session_id,
                                        reason, assigned_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        g.id,
        g.candidate_id,
        g.specialization,
        g.grade,
        g.theta,
        g.se,
        g.session_id,
        g.reason,
        g.assigned_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn set_candidate_category(
    db: impl PgExecutor<'_>,
    specialization: &str,
    grade: &str,
    theta: f64,
    assigned_at: DateTime<Utc>,
    candidate_id: Uuid,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE candidates
         SET specialization = $1, grade = $2, theta = $3, category_assigned_at = $4,
             last_activity_at = greatest(last_activity_at, $4), updated_at = now()
         WHERE user_id = $5",
        specialization,
        grade,
        theta,
        assigned_at,
        candidate_id,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn grade_history(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<GradeHistory>, sqlx::Error> {
    sqlx::query_as!(
        GradeHistory,
        "SELECT id, specialization, grade, theta, se, session_id, reason, assigned_at, annulled_at
         FROM grade_assignments
         WHERE candidate_id = $1
         ORDER BY assigned_at DESC",
        candidate_id,
    )
    .fetch_all(db)
    .await
}
