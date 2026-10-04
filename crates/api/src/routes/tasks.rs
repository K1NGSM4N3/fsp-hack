//! Regular short tasks (T-8). Assignment is lazy: when a candidate opens their tasks and has
//! nothing open, and the last task was given more than a week ago, the next unseen task for
//! their category is assigned. No scheduler is needed and inactive candidates get nothing.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use catalog::{Category, Grade, Specialization};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::{CandidateUser, EmployerUser},
    dto::{TaskStatus, parse_slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    skills,
    state::AppState,
};

const ASSIGN_EVERY_DAYS: i64 = 7;
const DUE_IN_DAYS: i64 = 7;

pub fn employer_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_tasks, create_task))
        .routes(routes!(set_active))
        .routes(routes!(submissions))
        .routes(routes!(review))
}

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(my_tasks))
        .routes(routes!(submit))
        .routes(routes!(skip))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// Solve the task.
    Solution,
    /// Describe how you would approach it.
    Approach,
}

impl TaskKind {
    const fn slug(self) -> &'static str {
        match self {
            TaskKind::Solution => "solution",
            TaskKind::Approach => "approach",
        }
    }

    fn parse(s: &str) -> TaskKind {
        if s == "solution" {
            TaskKind::Solution
        } else {
            TaskKind::Approach
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct TaskInput {
    pub title: String,
    pub body: String,
    pub kind: TaskKind,
    pub specialization: Specialization,
    pub grades: Vec<Grade>,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct EmployerTask {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub kind: TaskKind,
    pub specialization: Specialization,
    pub grades: Vec<Grade>,
    pub skills: Vec<String>,
    pub active: bool,
    pub submissions: i64,
    pub reviewed: i64,
    pub created_at: DateTime<Utc>,
}

/// Company tasks with submission counters
#[utoipa::path(get, path = "/tasks", tag = "tasks",
    responses((status = 200, body = Vec<EmployerTask>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_tasks(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<EmployerTask>>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let rows = db::queries::tasks::company_tasks()
        .bind(&client, &company)
        .all()
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(EmployerTask {
            id: r.id,
            title: r.title,
            body: r.body,
            kind: TaskKind::parse(&r.kind),
            specialization: r.specialization.parse()?,
            grades: parse_slugs(&r.grades),
            skills: r.skills,
            active: r.active,
            submissions: r.submissions,
            reviewed: r.reviewed,
            created_at: r.created_at,
        });
    }
    Ok(Json(out))
}

/// Publish a short task for a category; candidates receive it in turn
#[utoipa::path(post, path = "/tasks", tag = "tasks",
    request_body = TaskInput,
    responses((status = 201, body = EmployerTask), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn create_task(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(t): Json<TaskInput>,
) -> ApiResult<(StatusCode, Json<EmployerTask>)> {
    let unknown: Vec<&str> = t
        .skills
        .iter()
        .map(String::as_str)
        .filter(|k| !skills::is_known(k))
        .collect();
    let mut v = Validator::default();
    v.check(
        !t.title.trim().is_empty() && t.title.len() <= 200,
        "title",
        "Название до 200 символов",
    )
    .check(
        !t.body.trim().is_empty() && t.body.len() <= 5000,
        "body",
        "Условие до 5000 символов",
    )
    .check(
        !t.grades.is_empty(),
        "grades",
        "Выберите хотя бы один грейд",
    )
    .check_with(unknown.is_empty(), "skills", || {
        format!("Неизвестные навыки: {}", unknown.join(", "))
    });
    v.finish()?;
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let id = Uuid::now_v7();
    let grades: Vec<&str> = t.grades.iter().map(|g| g.slug()).collect();
    db::queries::tasks::insert_task()
        .bind(
            &client,
            &id,
            &company,
            &t.specialization.slug(),
            &grades,
            &t.skills,
            &t.title.trim(),
            &t.body.trim(),
            &t.kind.slug(),
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(EmployerTask {
            id,
            title: t.title.trim().to_owned(),
            body: t.body.trim().to_owned(),
            kind: t.kind,
            specialization: t.specialization,
            grades: t.grades,
            skills: t.skills,
            active: true,
            submissions: 0,
            reviewed: 0,
            created_at: Utc::now(),
        }),
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct ActiveInput {
    pub active: bool,
}

/// Pause or resume handing out a task
#[utoipa::path(post, path = "/tasks/{id}/active", tag = "tasks",
    params(("id" = Uuid, Path)),
    request_body = ActiveInput,
    responses((status = 204), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn set_active(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ActiveInput>,
) -> ApiResult<StatusCode> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    if db::queries::tasks::set_task_active()
        .bind(&client, &input.active, &id, &company)
        .await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema)]
pub struct Submission {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub candidate_code: String,
    pub candidate_category: Option<Category>,
    pub status: TaskStatus,
    pub answer: Option<String>,
    pub rating: Option<i16>,
    pub review_comment: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
}

/// Anonymous submissions to a task (a strong one is a reason to send an invitation)
#[utoipa::path(get, path = "/tasks/{id}/submissions", tag = "tasks",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vec<Submission>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn submissions(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<Submission>>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let rows = db::queries::tasks::task_submissions()
        .bind(&client, &id, &company)
        .all()
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(Submission {
            id: r.id,
            candidate_id: r.candidate_id,
            candidate_code: r.public_code,
            candidate_category: match (r.specialization.as_deref(), r.grade.as_deref()) {
                (Some(s), Some(g)) => Some(Category::new(s.parse()?, g.parse()?)),
                _ => None,
            },
            status: r.status.into(),
            answer: r.answer,
            rating: r.rating,
            review_comment: r.review_comment,
            submitted_at: r.submitted_at,
        });
    }
    Ok(Json(out))
}

#[derive(Deserialize, ToSchema)]
pub struct ReviewInput {
    /// 1–5
    pub rating: i16,
    pub comment: Option<String>,
}

/// Rate a submission
#[utoipa::path(post, path = "/tasks/{id}/submissions/{sid}/review", tag = "tasks",
    params(("id" = Uuid, Path), ("sid" = Uuid, Path)),
    request_body = ReviewInput,
    responses((status = 204), (status = 404, body = Problem), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn review(
    State(state): State<AppState>,
    user: EmployerUser,
    Path((id, sid)): Path<(Uuid, Uuid)>,
    Json(input): Json<ReviewInput>,
) -> ApiResult<StatusCode> {
    if !(1..=5).contains(&input.rating) {
        return Err(ApiError::field("rating", "Оценка от 1 до 5"));
    }
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let comment = input
        .comment
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty());
    if db::queries::tasks::review_submission()
        .bind(&client, &input.rating, &comment, &sid, &id, &company)
        .await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema)]
pub struct MyTask {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub kind: TaskKind,
    pub skills: Vec<String>,
    pub company_name: Option<String>,
    pub status: TaskStatus,
    pub answer: Option<String>,
    pub rating: Option<i16>,
    pub review_comment: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub due_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

async fn assign_if_due(client: &deadpool_postgres::Object, candidate: Uuid) -> ApiResult<()> {
    db::queries::tasks::expire_assignments()
        .bind(client, &candidate)
        .await?;
    if db::queries::tasks::has_open_assignment()
        .bind(client, &candidate)
        .one()
        .await?
        > 0
    {
        return Ok(());
    }
    let last = db::queries::tasks::last_assignment_at()
        .bind(client, &candidate)
        .one()
        .await?;
    if last.is_some_and(|at| at > Utc::now() - Duration::days(ASSIGN_EVERY_DAYS)) {
        return Ok(());
    }
    if let Some(task) = db::queries::tasks::pick_task_for()
        .bind(client, &candidate)
        .opt()
        .await?
    {
        db::queries::tasks::insert_assignment()
            .bind(
                client,
                &Uuid::now_v7(),
                &task,
                &candidate,
                &(Utc::now() + Duration::days(DUE_IN_DAYS)),
            )
            .await?;
    }
    Ok(())
}

async fn load_my_tasks(
    client: &deadpool_postgres::Object,
    candidate: Uuid,
) -> ApiResult<Vec<MyTask>> {
    Ok(db::queries::tasks::candidate_assignments()
        .bind(client, &candidate)
        .all()
        .await?
        .into_iter()
        .map(|r| MyTask {
            id: r.id,
            title: r.title,
            body: r.body,
            kind: TaskKind::parse(&r.kind),
            skills: r.skills,
            company_name: r.company_name,
            status: r.status.into(),
            answer: r.answer,
            rating: r.rating,
            review_comment: r.review_comment,
            assigned_at: r.assigned_at,
            due_at: r.due_at,
            submitted_at: r.submitted_at,
            reviewed_at: r.reviewed_at,
        })
        .collect())
}

/// Current and past tasks; hands out a new one when due
#[utoipa::path(get, path = "/tasks", tag = "tasks",
    responses((status = 200, body = Vec<MyTask>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn my_tasks(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Vec<MyTask>>> {
    let client = state.pool.get().await?;
    assign_if_due(&client, user.id).await?;
    Ok(Json(load_my_tasks(&client, user.id).await?))
}

#[derive(Deserialize, ToSchema)]
pub struct SubmitInput {
    pub answer: String,
}

/// Submit a solution or an approach (refreshes the profile)
#[utoipa::path(post, path = "/tasks/{id}/submit", tag = "tasks",
    params(("id" = Uuid, Path)),
    request_body = SubmitInput,
    responses((status = 200, body = Vec<MyTask>), (status = 409, description = "`task_closed`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn submit(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
    Json(input): Json<SubmitInput>,
) -> ApiResult<Json<Vec<MyTask>>> {
    let answer = input.answer.trim();
    if answer.is_empty() || answer.len() > 20_000 {
        return Err(ApiError::field("answer", "Ответ от 1 до 20 000 символов"));
    }
    let client = state.pool.get().await?;
    if db::queries::tasks::submit_assignment()
        .bind(&client, &answer, &id, &user.id)
        .await?
        == 0
    {
        return Err(ApiError::conflict(
            "task_closed",
            "Задание уже закрыто или просрочено",
        ));
    }
    db::queries::candidates::touch_candidate_activity()
        .bind(&client, &user.id)
        .await?;
    Ok(Json(load_my_tasks(&client, user.id).await?))
}

/// Skip the current task
#[utoipa::path(post, path = "/tasks/{id}/skip", tag = "tasks",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 409, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn skip(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let client = state.pool.get().await?;
    if db::queries::tasks::skip_assignment()
        .bind(&client, &id, &user.id)
        .await?
        == 0
    {
        return Err(ApiError::conflict("task_closed", "Задание уже закрыто"));
    }
    Ok(StatusCode::NO_CONTENT)
}
