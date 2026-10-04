//! Classic flow kept alongside invitations (C-6, E-6): a candidate without invitations can still
//! take the initiative. Applying grants the company access to the candidate's contacts.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use catalog::{Category, Grade, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::{CandidateUser, EmployerUser},
    dto::{ApplicationStatus, Page, SalaryRange, VacancyStatus},
    error::{ApiError, ApiResult, Problem, Validator},
    skills,
    state::AppState,
};

pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_published))
        .routes(routes!(get_published))
        .routes(routes!(apply))
}

pub fn employer_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_own, create))
        .routes(routes!(get_own, update))
        .routes(routes!(list_applications))
        .routes(routes!(set_application_status))
}

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(my_applications))
        .routes(routes!(withdraw_application))
}

#[derive(Deserialize, ToSchema)]
pub struct VacancyInput {
    pub title: String,
    pub description: String,
    pub specialization: Specialization,
    pub grade: Grade,
    #[serde(default)]
    pub skills: Vec<String>,
    pub salary: SalaryRange,
    pub work_format: WorkFormat,
    pub city: Option<String>,
    pub status: VacancyStatus,
}

#[derive(Serialize, ToSchema)]
pub struct Vacancy {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub specialization: Specialization,
    pub grade: Grade,
    pub skills: Vec<String>,
    pub salary: SalaryRange,
    pub work_format: WorkFormat,
    pub city: Option<String>,
    pub status: VacancyStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub applications: i64,
}

macro_rules! vacancy_from_row {
    ($r:expr) => {{
        let r = $r;
        Vacancy {
            id: r.id,
            title: r.title,
            description: r.description,
            specialization: r.specialization.parse()?,
            grade: r.grade.parse()?,
            skills: r.skills,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            work_format: r.work_format.parse()?,
            city: r.city,
            status: r.status.into(),
            published_at: r.published_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
            applications: r.applications,
        }
    }};
}

fn validate(v: &VacancyInput) -> ApiResult<()> {
    let unknown: Vec<&str> = v
        .skills
        .iter()
        .map(String::as_str)
        .filter(|k| !skills::is_known(k))
        .collect();
    let mut val = Validator::default();
    val.check(!v.title.trim().is_empty(), "title", "Укажите название")
        .check(v.title.len() <= 200, "title", "Не длиннее 200 символов")
        .check(
            v.description.len() <= 10_000,
            "description",
            "Не длиннее 10 000 символов",
        )
        .check(v.salary.min > 0, "salary.min", "Зарплата «от» обязательна")
        .check(
            v.salary.max >= v.salary.min,
            "salary.max",
            "«До» не меньше «от»",
        )
        .check_with(unknown.is_empty(), "skills", || {
            format!("Неизвестные навыки: {}", unknown.join(", "))
        });
    val.finish()
}

/// Company vacancies
#[utoipa::path(get, path = "/vacancies", tag = "vacancies",
    responses((status = 200, body = Vec<Vacancy>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_own(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<Vacancy>>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let rows = db::queries::vacancies::company_vacancies()
        .bind(&client, &company)
        .all()
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(vacancy_from_row!(r));
    }
    Ok(Json(out))
}

/// Create a vacancy (salary range is mandatory)
#[utoipa::path(post, path = "/vacancies", tag = "vacancies",
    request_body = VacancyInput,
    responses((status = 201, body = Vacancy), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn create(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(v): Json<VacancyInput>,
) -> ApiResult<(StatusCode, Json<Vacancy>)> {
    validate(&v)?;
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let id = Uuid::now_v7();
    let status: db::types::VacancyStatus = v.status.into();
    let published_at = (v.status == VacancyStatus::Published).then(Utc::now);
    db::queries::vacancies::insert_vacancy()
        .bind(
            &client,
            &id,
            &company,
            &v.title.trim(),
            &v.description.trim(),
            &v.specialization.slug(),
            &v.grade.slug(),
            &v.skills,
            &v.salary.min,
            &v.salary.max,
            &v.work_format.slug(),
            &v.city.as_deref().map(str::trim).filter(|c| !c.is_empty()),
            &status,
            &published_at,
        )
        .await?;
    let row = db::queries::vacancies::company_vacancy()
        .bind(&client, &company, &id)
        .one()
        .await?;
    Ok((StatusCode::CREATED, Json(vacancy_from_row!(row))))
}

/// One own vacancy
#[utoipa::path(get, path = "/vacancies/{id}", tag = "vacancies",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vacancy), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_own(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vacancy>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let row = db::queries::vacancies::company_vacancy()
        .bind(&client, &company, &id)
        .opt()
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(vacancy_from_row!(row)))
}

/// Edit, publish or close a vacancy
#[utoipa::path(put, path = "/vacancies/{id}", tag = "vacancies",
    params(("id" = Uuid, Path)),
    request_body = VacancyInput,
    responses((status = 200, body = Vacancy), (status = 404, body = Problem), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn update(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(v): Json<VacancyInput>,
) -> ApiResult<Json<Vacancy>> {
    validate(&v)?;
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let status: db::types::VacancyStatus = v.status.into();
    let updated = db::queries::vacancies::update_vacancy()
        .bind(
            &client,
            &v.title.trim(),
            &v.description.trim(),
            &v.specialization.slug(),
            &v.grade.slug(),
            &v.skills,
            &v.salary.min,
            &v.salary.max,
            &v.work_format.slug(),
            &v.city.as_deref().map(str::trim).filter(|c| !c.is_empty()),
            &status,
            &id,
            &company,
        )
        .await?;
    if updated == 0 {
        return Err(ApiError::not_found());
    }
    let row = db::queries::vacancies::company_vacancy()
        .bind(&client, &company, &id)
        .one()
        .await?;
    Ok(Json(vacancy_from_row!(row)))
}

#[derive(Serialize, ToSchema)]
pub struct IncomingApplication {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub candidate_code: String,
    pub candidate_category: Option<Category>,
    pub cover_letter: String,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Applications to a vacancy. Contacts of applicants are available via `/employer/candidates/{id}/contacts`.
#[utoipa::path(get, path = "/vacancies/{id}/applications", tag = "vacancies",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vec<IncomingApplication>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_applications(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<IncomingApplication>>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let rows = db::queries::vacancies::vacancy_applications()
        .bind(&client, &id, &company)
        .all()
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(IncomingApplication {
            id: r.id,
            candidate_id: r.candidate_id,
            candidate_code: r.public_code,
            candidate_category: match (r.specialization.as_deref(), r.grade.as_deref()) {
                (Some(s), Some(g)) => Some(Category::new(s.parse()?, g.parse()?)),
                _ => None,
            },
            cover_letter: r.cover_letter,
            status: r.status.into(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        });
    }
    Ok(Json(out))
}

#[derive(Deserialize, ToSchema)]
pub struct ApplicationStatusInput {
    pub status: ApplicationStatus,
}

/// Move an application: viewed, accepted or rejected
#[utoipa::path(post, path = "/applications/{id}/status", tag = "vacancies",
    params(("id" = Uuid, Path)),
    request_body = ApplicationStatusInput,
    responses((status = 204), (status = 404, body = Problem), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn set_application_status(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ApplicationStatusInput>,
) -> ApiResult<StatusCode> {
    if matches!(
        input.status,
        ApplicationStatus::Sent | ApplicationStatus::Withdrawn
    ) {
        return Err(ApiError::field(
            "status",
            "Допустимо: viewed, accepted, rejected",
        ));
    }
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let status: db::types::ApplicationStatus = input.status.into();
    if db::queries::vacancies::set_application_status()
        .bind(&client, &status, &id, &company)
        .await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, IntoParams)]
pub struct VacancyQuery {
    pub specialization: Option<Specialization>,
    pub grade: Option<Grade>,
    pub work_format: Option<WorkFormat>,
    /// Full-text search over title and description (Russian morphology).
    pub q: Option<String>,
    /// Vacancies whose upper salary bound reaches this value.
    pub salary_from: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct PublicVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_description: Option<String>,
    pub title: String,
    pub description: String,
    pub specialization: Specialization,
    pub grade: Grade,
    pub skills: Vec<String>,
    pub salary: SalaryRange,
    pub work_format: WorkFormat,
    pub city: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Published vacancies
#[utoipa::path(get, path = "/vacancies", tag = "vacancies",
    params(VacancyQuery),
    responses((status = 200, body = Page<PublicVacancy>)))]
async fn list_published(
    State(state): State<AppState>,
    Query(q): Query<VacancyQuery>,
) -> ApiResult<Json<Page<PublicVacancy>>> {
    let client = state.pool.get().await?;
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);
    let text = q.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let rows = db::queries::vacancies::published_vacancies()
        .bind(
            &client,
            &q.specialization.map(Specialization::slug),
            &q.grade.map(Grade::slug),
            &q.work_format.map(WorkFormat::slug),
            &q.salary_from,
            &text,
            &limit,
            &offset,
        )
        .all()
        .await?;
    let total = rows.first().map_or(0, |r| r.total);
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        items.push(PublicVacancy {
            id: r.id,
            company_id: r.company_id,
            company_name: r.company_name,
            company_city: r.company_city,
            company_description: None,
            title: r.title,
            description: r.description,
            specialization: r.specialization.parse()?,
            grade: r.grade.parse()?,
            skills: r.skills,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            work_format: r.work_format.parse()?,
            city: r.city,
            published_at: r.published_at,
        });
    }
    Ok(Json(Page { items, total }))
}

/// One published vacancy
#[utoipa::path(get, path = "/vacancies/{id}", tag = "vacancies",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = PublicVacancy), (status = 404, body = Problem)))]
async fn get_published(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<PublicVacancy>> {
    let client = state.pool.get().await?;
    let r = db::queries::vacancies::published_vacancy()
        .bind(&client, &id)
        .opt()
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(PublicVacancy {
        id: r.id,
        company_id: r.company_id,
        company_name: r.company_name,
        company_city: r.company_city,
        company_description: Some(r.company_description),
        title: r.title,
        description: r.description,
        specialization: r.specialization.parse()?,
        grade: r.grade.parse()?,
        skills: r.skills,
        salary: SalaryRange {
            min: r.salary_min,
            max: r.salary_max,
        },
        work_format: r.work_format.parse()?,
        city: r.city,
        published_at: r.published_at,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct ApplyInput {
    #[serde(default)]
    pub cover_letter: String,
}

#[derive(Serialize, ToSchema)]
pub struct MyApplication {
    pub id: Uuid,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub company_name: String,
    pub salary: SalaryRange,
    pub cover_letter: String,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Apply to a vacancy: the company gets access to your contacts
#[utoipa::path(post, path = "/vacancies/{id}/applications", tag = "vacancies",
    params(("id" = Uuid, Path)),
    request_body = ApplyInput,
    responses((status = 201, body = MyApplication), (status = 404, body = Problem), (status = 409, description = "`already_applied`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn apply(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(vacancy_id): Path<Uuid>,
    Json(input): Json<ApplyInput>,
) -> ApiResult<(StatusCode, Json<MyApplication>)> {
    if input.cover_letter.len() > 4000 {
        return Err(ApiError::field("cover_letter", "Не длиннее 4000 символов"));
    }
    let client = state.pool.get().await?;
    db::queries::vacancies::published_vacancy()
        .bind(&client, &vacancy_id)
        .opt()
        .await?
        .ok_or_else(ApiError::not_found)?;
    let letter = input.cover_letter.trim();
    let reopened = db::queries::vacancies::reopen_application()
        .bind(&client, &letter, &vacancy_id, &user.id)
        .opt()
        .await?;
    let id = match reopened {
        Some(id) => id,
        None => {
            let id = Uuid::now_v7();
            let inserted = db::queries::vacancies::insert_application()
                .bind(&client, &id, &vacancy_id, &user.id, &letter)
                .await;
            match inserted {
                Ok(_) => id,
                Err(e) if e.code() == Some(&tokio_postgres::error::SqlState::UNIQUE_VIOLATION) => {
                    return Err(ApiError::conflict(
                        "already_applied",
                        "Вы уже откликнулись на эту вакансию",
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }
    };
    db::queries::candidates::touch_candidate_activity()
        .bind(&client, &user.id)
        .await?;
    let app = my_apps(&client, user.id)
        .await?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(ApiError::not_found)?;
    Ok((StatusCode::CREATED, Json(app)))
}

async fn my_apps(
    client: &deadpool_postgres::Object,
    candidate: Uuid,
) -> ApiResult<Vec<MyApplication>> {
    Ok(db::queries::vacancies::candidate_applications()
        .bind(client, &candidate)
        .all()
        .await?
        .into_iter()
        .map(|r| MyApplication {
            id: r.id,
            vacancy_id: r.vacancy_id,
            vacancy_title: r.vacancy_title,
            company_name: r.company_name,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            cover_letter: r.cover_letter,
            status: r.status.into(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Own applications and their statuses
#[utoipa::path(get, path = "/applications", tag = "vacancies",
    responses((status = 200, body = Vec<MyApplication>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn my_applications(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Vec<MyApplication>>> {
    let client = state.pool.get().await?;
    Ok(Json(my_apps(&client, user.id).await?))
}

/// Withdraw an application before the company decides
#[utoipa::path(post, path = "/applications/{id}/withdraw", tag = "vacancies",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 409, description = "`application_closed`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn withdraw_application(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let client = state.pool.get().await?;
    if db::queries::vacancies::withdraw_application()
        .bind(&client, &id, &user.id)
        .await?
        == 0
    {
        return Err(ApiError::conflict(
            "application_closed",
            "Отклик уже рассмотрен или отозван",
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
