//! Regular short tasks (T-8). Assignment is lazy: when a candidate opens their tasks and has
//! nothing open, and the last task was given more than a week ago, the next unseen task for
//! their category is assigned. No scheduler is needed and inactive candidates get nothing.

use axum::{extract::State, http::StatusCode};
use catalog::{Category, Grade, Specialization};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::{CandidateUser, EmployerUser},
    dto::{TaskStatus, parse_slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Json, Path, Query},
    page::{self, CursorPage, PageQuery},
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
    responses((status = 200, body = Vec<EmployerTask>), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_tasks(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<EmployerTask>>> {
    let company = company_id(&state, user.id).await?;
    Ok(Json(company_task_views(&state.pool, company).await?))
}

pub(crate) async fn company_task_views(
    pool: &PgPool,
    company: Uuid,
) -> ApiResult<Vec<EmployerTask>> {
    let rows = db::tasks::company_tasks(pool, company).await?;
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
    Ok(out)
}

/// Publish a short task for a category; candidates receive it in turn
#[utoipa::path(post, path = "/tasks", tag = "tasks",
    request_body = TaskInput,
    responses((status = 201, body = EmployerTask), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
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
        !t.title.trim().is_empty() && t.title.chars().count() <= 200,
        "title",
        "Название до 200 символов",
    )
    .check(
        !t.body.trim().is_empty() && t.body.chars().count() <= 5000,
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
    let company = company_id(&state, user.id).await?;
    let id = Uuid::now_v7();
    let grades: Vec<String> = t.grades.iter().map(|g| g.slug().to_owned()).collect();
    db::tasks::insert_task(
        &state.pool,
        &db::tasks::NewTask {
            id,
            company_id: company,
            specialization: t.specialization.slug(),
            grades: &grades,
            skills: &t.skills,
            title: t.title.trim(),
            body: t.body.trim(),
            kind: t.kind.slug(),
        },
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
    responses((status = 204), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn set_active(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ActiveInput>,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    if db::tasks::set_task_active(&state.pool, input.active, id, company).await? == 0 {
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

/// Anonymous submissions to a task, newest first (a strong one is a reason to send an
/// invitation). Keyset pagination: pass `next_cursor` as `cursor`
#[utoipa::path(get, path = "/tasks/{id}/submissions", tag = "tasks",
    params(("id" = Uuid, Path), PageQuery),
    responses(
        (status = 200, body = CursorPage<Submission>),
        (status = 400, description = "`invalid_query` — `cursor` не из ответа", body = Problem),
        (status = 404, description = "No such task of this company", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn submissions(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<CursorPage<Submission>>> {
    let before = page::decode(q.cursor.as_deref())?;
    let limit = page::limit(q.limit, page::DEFAULT, page::MAX);
    let company = company_id(&state, user.id).await?;
    if !db::tasks::company_tasks(&state.pool, company)
        .await?
        .iter()
        .any(|t| t.id == id)
    {
        return Err(ApiError::not_found());
    }
    let rows = db::tasks::task_submissions(&state.pool, id, company, before, limit + 1).await?;
    let (rows, next_cursor) =
        page::split(rows, limit, |r| (r.submitted_at.unwrap_or_default(), r.id));
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
    Ok(Json(CursorPage {
        items: out,
        next_cursor,
    }))
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
    responses((status = 204), (status = 404, body = Problem), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
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
    let company = company_id(&state, user.id).await?;
    let comment = input
        .comment
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty());
    if db::tasks::review_submission(&state.pool, input.rating, comment, sid, id, company).await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    // The rating is on the candidate's card, which employers' caches hold.
    state.candidates.invalidate().await;
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

/// One task at a time, at most one a week: `last` is when the previous one was handed out.
fn due_for_assignment(open: bool, last: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    !open && last.is_none_or(|at| at <= now - Duration::days(ASSIGN_EVERY_DAYS))
}

async fn assign_if_due(pool: &PgPool, candidate: Uuid) -> ApiResult<()> {
    db::tasks::expire_assignments(pool, candidate).await?;
    let open = db::tasks::has_open_assignment(pool, candidate).await? > 0;
    let last = if open {
        None
    } else {
        db::tasks::last_assignment_at(pool, candidate).await?
    };
    if !due_for_assignment(open, last, Utc::now()) {
        return Ok(());
    }
    if let Some(task) = db::tasks::pick_task_for(pool, candidate).await? {
        db::tasks::insert_assignment(
            pool,
            Uuid::now_v7(),
            task,
            candidate,
            Utc::now() + Duration::days(DUE_IN_DAYS),
        )
        .await?;
    }
    Ok(())
}

pub(crate) async fn load_my_tasks(pool: &PgPool, candidate: Uuid) -> ApiResult<Vec<MyTask>> {
    Ok(db::tasks::candidate_assignments(pool, candidate)
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
    assign_if_due(&state.pool, user.id).await?;
    Ok(Json(load_my_tasks(&state.pool, user.id).await?))
}

#[derive(Deserialize, ToSchema)]
pub struct SubmitInput {
    pub answer: String,
}

/// 404 unless the assignment is the candidate's: a closed one of theirs is a 409.
async fn own_task(state: &AppState, candidate: Uuid, id: Uuid) -> ApiResult<()> {
    let own = db::tasks::candidate_assignments(&state.pool, candidate).await?;
    if own.iter().any(|a| a.id == id) {
        Ok(())
    } else {
        Err(ApiError::not_found())
    }
}

/// Submit a solution or an approach (refreshes the profile)
#[utoipa::path(post, path = "/tasks/{id}/submit", tag = "tasks",
    params(("id" = Uuid, Path)),
    request_body = SubmitInput,
    responses((status = 200, body = Vec<MyTask>), (status = 404, description = "Not the caller's task", body = Problem), (status = 409, description = "`task_closed`", body = Problem), (status = 422, description = "`validation_failed`: `answer`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn submit(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
    Json(input): Json<SubmitInput>,
) -> ApiResult<Json<Vec<MyTask>>> {
    let answer = input.answer.trim();
    if answer.is_empty() || answer.chars().count() > 20_000 {
        return Err(ApiError::field("answer", "Ответ от 1 до 20 000 символов"));
    }
    if db::tasks::submit_assignment(&state.pool, answer, id, user.id).await? == 0 {
        own_task(&state, user.id, id).await?;
        return Err(ApiError::conflict(
            "task_closed",
            "Задание уже закрыто или просрочено",
        ));
    }
    db::candidates::touch_candidate_activity(&state.pool, user.id).await?;
    // The count of solved tasks is on the candidate's card, which employers' caches hold.
    state.candidates.invalidate().await;
    Ok(Json(load_my_tasks(&state.pool, user.id).await?))
}

/// Skip the current task
#[utoipa::path(post, path = "/tasks/{id}/skip", tag = "tasks",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 404, description = "Not the caller's task", body = Problem), (status = 409, description = "`task_closed`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn skip(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    if db::tasks::skip_assignment(&state.pool, id, user.id).await? == 0 {
        own_task(&state, user.id, id).await?;
        return Err(ApiError::conflict("task_closed", "Задание уже закрыто"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-10-07T12:00:00Z".parse().unwrap()
    }

    #[test]
    fn first_task_is_handed_out_at_once() {
        assert!(due_for_assignment(false, None, now()));
    }

    #[test]
    fn next_task_comes_a_week_after_the_previous_one() {
        let week = Duration::days(ASSIGN_EVERY_DAYS);
        assert!(due_for_assignment(false, Some(now() - week), now()));
        assert!(due_for_assignment(
            false,
            Some(now() - Duration::days(30)),
            now()
        ));
        assert!(!due_for_assignment(
            false,
            Some(now() - week + Duration::seconds(1)),
            now()
        ));
        assert!(!due_for_assignment(false, Some(now()), now()));
    }

    #[test]
    fn open_task_blocks_the_next_one() {
        assert!(!due_for_assignment(true, None, now()));
        assert!(!due_for_assignment(
            true,
            Some(now() - Duration::days(30)),
            now()
        ));
    }
}
