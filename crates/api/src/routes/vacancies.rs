//! Classic flow kept alongside invitations (C-6, E-6): a candidate without invitations can still
//! take the initiative. Applying grants the company access to the candidate's contacts.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use catalog::{Category, Grade, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::trust::display_domain;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    trust,
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
    let company = company_id(&state.pool, user.id).await?;
    Ok(Json(company_vacancy_views(&state.pool, company).await?))
}

pub(crate) async fn company_vacancy_views(pool: &PgPool, company: Uuid) -> ApiResult<Vec<Vacancy>> {
    let rows = db::vacancies::company_vacancies(pool, company).await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(vacancy_from_row!(r));
    }
    Ok(out)
}

fn fields(v: &VacancyInput) -> db::vacancies::VacancyFields<'_> {
    db::vacancies::VacancyFields {
        title: v.title.trim(),
        description: v.description.trim(),
        specialization: v.specialization.slug(),
        grade: v.grade.slug(),
        skills: &v.skills,
        salary_min: v.salary.min,
        salary_max: v.salary.max,
        work_format: v.work_format.slug(),
        city: v.city.as_deref().map(str::trim).filter(|c| !c.is_empty()),
    }
}

/// Every save re-signs the text; a published vacancy also gets its trust flags (never blocking).
async fn check_trust(state: &AppState, company: Uuid, id: Uuid, v: &VacancyInput) -> ApiResult<()> {
    let signature =
        trust::sign_vacancy(&state.pool, id, v.title.trim(), v.description.trim()).await?;
    if v.status == VacancyStatus::Published {
        trust::flag_published_vacancy(
            &state.pool,
            company,
            id,
            Category::new(v.specialization, v.grade),
            (v.salary.min, v.salary.max),
            signature.as_ref(),
        )
        .await?;
    }
    Ok(())
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
    let company = company_id(&state.pool, user.id).await?;
    let id = Uuid::now_v7();
    let status: db::types::VacancyStatus = v.status.into();
    let published_at = (v.status == VacancyStatus::Published).then(Utc::now);
    db::vacancies::insert_vacancy(&state.pool, id, company, status, published_at, &fields(&v))
        .await?;
    check_trust(&state, company, id, &v).await?;
    let row = db::vacancies::company_vacancy(&state.pool, company, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
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
    let company = company_id(&state.pool, user.id).await?;
    let row = db::vacancies::company_vacancy(&state.pool, company, id)
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
    let company = company_id(&state.pool, user.id).await?;
    let status: db::types::VacancyStatus = v.status.into();
    let updated =
        db::vacancies::update_vacancy(&state.pool, id, company, status, &fields(&v)).await?;
    if updated == 0 {
        return Err(ApiError::not_found());
    }
    check_trust(&state, company, id, &v).await?;
    let row = db::vacancies::company_vacancy(&state.pool, company, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
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
    let company = company_id(&state.pool, user.id).await?;
    let rows = db::vacancies::vacancy_applications(&state.pool, id, company).await?;
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
    let company = company_id(&state.pool, user.id).await?;
    let status: db::types::ApplicationStatus = input.status.into();
    if db::vacancies::set_application_status(&state.pool, status, id, company).await? == 0 {
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
    /// «Домен подтверждён»: the company's login email and website share this domain.
    pub company_verified_domain: Option<String>,
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
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);
    let text = q.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let rows = db::vacancies::published_vacancies(
        &state.pool,
        &db::vacancies::VacancyFilter {
            specialization: q.specialization.map(Specialization::slug),
            grade: q.grade.map(Grade::slug),
            work_format: q.work_format.map(WorkFormat::slug),
            salary_from: q.salary_from,
            q: text,
            limit,
            offset,
        },
    )
    .await?;
    let total = rows.first().map_or(0, |r| r.total);
    let mut items = Vec::with_capacity(rows.len());
    for r in rows {
        items.push(PublicVacancy {
            id: r.id,
            company_id: r.company_id,
            company_name: r.company_name,
            company_city: r.company_city,
            company_verified_domain: r.company_verified_domain.as_deref().map(display_domain),
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
    let r = db::vacancies::published_vacancy(&state.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(PublicVacancy {
        id: r.id,
        company_id: r.company_id,
        company_name: r.company_name,
        company_city: r.company_city,
        company_verified_domain: r.company_verified_domain.as_deref().map(display_domain),
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
    db::vacancies::published_vacancy(&state.pool, vacancy_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let letter = input.cover_letter.trim();
    let reopened =
        db::vacancies::reopen_application(&state.pool, letter, vacancy_id, user.id).await?;
    let id = match reopened {
        Some(id) => id,
        None => {
            let id = Uuid::now_v7();
            let inserted =
                db::vacancies::insert_application(&state.pool, id, vacancy_id, user.id, letter)
                    .await;
            match inserted {
                Ok(_) => id,
                Err(e) if db::is_unique_violation(&e) => {
                    return Err(ApiError::conflict(
                        "already_applied",
                        "Вы уже откликнулись на эту вакансию",
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }
    };
    db::candidates::touch_candidate_activity(&state.pool, user.id).await?;
    let app = my_apps(&state.pool, user.id)
        .await?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(ApiError::not_found)?;
    Ok((StatusCode::CREATED, Json(app)))
}

pub(crate) async fn my_apps(pool: &PgPool, candidate: Uuid) -> ApiResult<Vec<MyApplication>> {
    Ok(db::vacancies::candidate_applications(pool, candidate)
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
    Ok(Json(my_apps(&state.pool, user.id).await?))
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
    if db::vacancies::withdraw_application(&state.pool, id, user.id).await? == 0 {
        return Err(ApiError::conflict(
            "application_closed",
            "Отклик уже рассмотрен или отозван",
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
