//! Classic flow kept alongside invitations (C-6, E-6): a candidate without invitations can still
//! take the initiative. Applying grants the company access to the candidate's contacts.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use catalog::{Category, Grade, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::trust::{TrustFlag, display_domain};
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
    extract::{Created, Json, Path, Query},
    integrations::{
        events::{EventType, Outbox},
        outcomes::{HiringOutcome, OutcomeIndex},
    },
    notify::Notifier,
    page::{self, CursorPage, PageQuery},
    routes::trust::TrustWarning,
    skills,
    state::AppState,
    trust,
};

/// Anonymous reads of published vacancies: no personal data, cached publicly.
pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_published))
        .routes(routes!(get_published))
        .routes(routes!(similar_published))
}

pub fn apply_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(apply))
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
    /// Taken down by a moderator: publishing again is refused (409 `vacancy_unpublished`); the
    /// reason is on the «Доверие и квота» page.
    pub unpublished_by_moderator_at: Option<DateTime<Utc>>,
    /// What the anti-fake checks noticed about this vacancy, softly worded; nothing is blocked.
    /// The paraphrase check runs a few seconds after publishing, once the text's embedding is
    /// computed.
    pub warnings: Vec<TrustWarning>,
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
            unpublished_by_moderator_at: r.unpublished_at,
            warnings: warnings(&r.trust_flags),
        }
    }};
}

fn warnings(flags: &serde_json::Value) -> Vec<TrustWarning> {
    serde_json::from_value::<Vec<TrustFlag>>(flags.clone())
        .unwrap_or_default()
        .iter()
        .map(|f| TrustWarning {
            kind: f.into(),
            text: f.employer_text(),
        })
        .collect()
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
        .check(
            v.title.chars().count() <= 200,
            "title",
            "Не длиннее 200 символов",
        )
        .check(
            v.description.chars().count() <= 10_000,
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
    responses((status = 200, body = Vec<Vacancy>), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_own(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<Vacancy>>> {
    let company = company_id(&state, user.id).await?;
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

/// Create a vacancy (salary range is mandatory). With `Idempotency-Key` a repeated click or
/// retry returns the vacancy already created instead of a duplicate
#[utoipa::path(post, path = "/vacancies", tag = "vacancies",
    request_body = VacancyInput,
    params(("Idempotency-Key" = Option<String>, Header, description = "До 100 символов, в пределах компании. Повтор с тем же ключом — и одновременная отправка — вернёт уже созданную вакансию", example = "vac-2026-10-07-0001")),
    responses(
        (status = 201, body = Vacancy, headers(("Location" = String, description = "`/api/v1/employer/vacancies/{id}`"))),
        (status = 200, description = "Повтор по Idempotency-Key: вакансия уже создана", body = Vacancy),
        (status = 422, description = "`validation_failed`; `idempotency_key` — длиннее 100 символов", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn create(
    State(state): State<AppState>,
    user: EmployerUser,
    headers: HeaderMap,
    Json(v): Json<VacancyInput>,
) -> ApiResult<Response> {
    validate(&v)?;
    let key = idempotency_key(&headers)?;
    let company = company_id(&state, user.id).await?;
    if let Some(key) = key.as_deref()
        && let Some(existing) = replay(&state, company, key).await?
    {
        return Ok((StatusCode::OK, Json(existing)).into_response());
    }
    let id = Uuid::now_v7();
    let status: db::types::VacancyStatus = v.status.into();
    let published_at = (v.status == VacancyStatus::Published).then(Utc::now);
    let inserted = db::vacancies::insert_vacancy(
        &state.pool,
        id,
        company,
        status,
        published_at,
        &fields(&v),
        key.as_deref(),
    )
    .await;
    if let Err(e) = inserted {
        // The same key sent twice at once: the second insert lost to the first.
        if db::is_unique_violation(&e)
            && let Some(key) = key.as_deref()
            && let Some(existing) = replay(&state, company, key).await?
        {
            return Ok((StatusCode::OK, Json(existing)).into_response());
        }
        return Err(e.into());
    }
    check_trust(&state, company, id, &v).await?;
    let row = db::vacancies::company_vacancy(&state.pool, company, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok(Created(
        format!("/api/v1/employer/vacancies/{id}"),
        vacancy_from_row!(row),
    )
    .into_response())
}

/// `Idempotency-Key`: 1–100 characters after trimming.
pub(crate) fn idempotency_key(headers: &HeaderMap) -> ApiResult<Option<String>> {
    let Some(v) = headers.get("idempotency-key") else {
        return Ok(None);
    };
    v.to_str()
        .ok()
        .map(str::trim)
        .filter(|k| !k.is_empty() && k.chars().count() <= 100)
        .map(|k| Some(k.to_owned()))
        .ok_or_else(|| ApiError::field("idempotency_key", "Idempotency-Key — от 1 до 100 символов"))
}

/// The vacancy an `Idempotency-Key` already created.
async fn replay(state: &AppState, company: Uuid, key: &str) -> ApiResult<Option<Vacancy>> {
    let Some(id) = db::vacancies::vacancy_by_idempotency_key(&state.pool, company, key).await?
    else {
        return Ok(None);
    };
    let row = db::vacancies::company_vacancy(&state.pool, company, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok(Some(vacancy_from_row!(row)))
}

/// One own vacancy
#[utoipa::path(get, path = "/vacancies/{id}", tag = "vacancies",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vacancy), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_own(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vacancy>> {
    let company = company_id(&state, user.id).await?;
    let row = db::vacancies::company_vacancy(&state.pool, company, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(vacancy_from_row!(row)))
}

/// Edit, publish or close a vacancy
#[utoipa::path(put, path = "/vacancies/{id}", tag = "vacancies",
    params(("id" = Uuid, Path)),
    request_body = VacancyInput,
    responses((status = 200, body = Vacancy), (status = 404, body = Problem), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`; `vacancy_unpublished` — a moderator took the vacancy down, it can be edited but not published", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn update(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(v): Json<VacancyInput>,
) -> ApiResult<Json<Vacancy>> {
    validate(&v)?;
    let company = company_id(&state, user.id).await?;
    let status: db::types::VacancyStatus = v.status.into();
    let updated =
        db::vacancies::update_vacancy(&state.pool, id, company, status, &fields(&v)).await?;
    if updated == 0 {
        return Err(
            match db::vacancies::company_vacancy(&state.pool, company, id).await? {
                Some(row) if row.unpublished_at.is_some() => ApiError::conflict(
                    "vacancy_unpublished",
                    "Вакансию снял с публикации модератор",
                )
                .with_detail(
                    "Её можно править, но не публиковать. Причина — в разделе «Доверие и квота»",
                ),
                _ => ApiError::not_found(),
            },
        );
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

/// Applications to a vacancy, newest first. Contacts of applicants are available via
/// `/employer/candidates/{id}/contacts`. Keyset pagination: pass `next_cursor` as `cursor`
#[utoipa::path(get, path = "/vacancies/{id}/applications", tag = "vacancies",
    params(("id" = Uuid, Path), PageQuery),
    responses(
        (status = 200, body = CursorPage<IncomingApplication>),
        (status = 400, description = "`invalid_query` — `cursor` не из ответа", body = Problem),
        (status = 404, description = "No such vacancy of this company", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_applications(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Query(q): Query<PageQuery>,
) -> ApiResult<Json<CursorPage<IncomingApplication>>> {
    let before = page::decode(q.cursor.as_deref())?;
    let limit = page::limit(q.limit, page::DEFAULT, page::MAX);
    let company = company_id(&state, user.id).await?;
    db::vacancies::company_vacancy(&state.pool, company, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let rows =
        db::vacancies::vacancy_applications(&state.pool, id, company, before, limit + 1).await?;
    let (rows, next_cursor) = page::split(rows, limit, |r| (r.created_at, r.id));
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
    Ok(Json(CursorPage {
        items: out,
        next_cursor,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct ApplicationStatusInput {
    pub status: ApplicationStatus,
}

/// Move an application: viewed, accepted or rejected
#[utoipa::path(post, path = "/applications/{id}/status", tag = "vacancies",
    params(("id" = Uuid, Path)),
    request_body = ApplicationStatusInput,
    responses((status = 204), (status = 404, body = Problem), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
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
    let company = company_id(&state, user.id).await?;
    let status: db::types::ApplicationStatus = input.status.into();
    let mut tx = state.pool.begin().await?;
    let before = db::integrations::application_status(&mut *tx, company, id).await?;
    if db::vacancies::set_application_status(&mut *tx, status, id, company).await? == 0 {
        return Err(ApiError::not_found());
    }
    if before != Some(status) {
        Outbox::new(&state)
            .application(&mut tx, EventType::ApplicationStatusChanged, id)
            .await?;
        Notifier::new(&state)
            .application(&mut tx, id, input.status)
            .await?;
    }
    tx.commit().await?;
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
    /// 1–100, 20 by default; out-of-range values are clamped.
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub limit: Option<i64>,
    #[param(minimum = 0, default = 0)]
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

/// How «Похожие вакансии» were found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelatedMethod {
    /// Nearest texts by the cosine of their embeddings.
    Meaning,
    /// Embeddings off or not computed yet: the same specialization, most shared skills first.
    Skills,
}

#[derive(Serialize, ToSchema)]
pub struct RelatedVacancy {
    pub vacancy: PublicVacancy,
    /// Meaning: cosine of the texts' embeddings; skills: share of this vacancy's skills it has.
    pub similarity: f32,
}

#[derive(Serialize, ToSchema)]
pub struct RelatedVacancies {
    pub method: RelatedMethod,
    pub items: Vec<RelatedVacancy>,
}

#[derive(Deserialize, IntoParams)]
pub struct SimilarQuery {
    /// 1–12, 4 by default.
    #[param(minimum = 1, maximum = 12, default = 4)]
    pub limit: Option<i64>,
}

fn public_vacancy(r: db::vacancies::PublishedVacancy) -> ApiResult<PublicVacancy> {
    Ok(PublicVacancy {
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
    })
}

/// Published vacancies close to this one: by meaning of the texts when their embeddings are
/// computed, otherwise by shared skills within the specialization
#[utoipa::path(get, path = "/vacancies/{id}/similar", tag = "vacancies",
    params(("id" = Uuid, Path), SimilarQuery),
    responses((status = 200, body = RelatedVacancies), (status = 404, description = "No published vacancy with this id", body = Problem)))]
async fn similar_published(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<SimilarQuery>,
) -> ApiResult<Json<RelatedVacancies>> {
    let limit = q.limit.unwrap_or(4).clamp(1, 12);
    let source = db::vacancies::published_vacancy(&state.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let stored = match state.embedder {
        Some(_) => db::embeddings::vacancy_vector(&state.pool, id).await?,
        None => None,
    };
    let (method, ranked): (RelatedMethod, Vec<(Uuid, f64)>) = match stored {
        Some(v) => {
            let mut tx = state.pool.begin().await?;
            db::embeddings::relaxed_scan(&mut *tx).await?;
            let near = db::embeddings::nearest_published(
                &mut *tx,
                &v.embedding,
                &v.model,
                id,
                None,
                None,
                limit * NEIGHBOUR_POOL,
            )
            .await?;
            tx.commit().await?;
            let near: Vec<(Uuid, bool, f64)> = near
                .into_iter()
                .map(|n| {
                    (
                        n.id,
                        n.specialization == source.specialization,
                        n.similarity,
                    )
                })
                .collect();
            (RelatedMethod::Meaning, rank_neighbours(near, limit))
        }
        None => (
            RelatedMethod::Skills,
            db::embeddings::similar_by_skills(&state.pool, id, limit).await?,
        ),
    };
    let ids: Vec<Uuid> = ranked.iter().map(|(id, _)| *id).collect();
    let mut rows = db::vacancies::published_by_ids(&state.pool, &ids).await?;
    let mut items = Vec::with_capacity(ranked.len());
    for (vid, similarity) in ranked {
        if let Some(pos) = rows.iter().position(|r| r.id == vid) {
            items.push(RelatedVacancy {
                vacancy: public_vacancy(rows.swap_remove(pos))?,
                similarity: similarity as f32,
            });
        }
    }
    Ok(Json(RelatedVacancies { method, items }))
}

/// Nearest neighbours fetched per shown one: the same specialization is preferred among them.
const NEIGHBOUR_POOL: i64 = 3;

/// Neighbours of the same specialization first, closest first within each group: a Go vacancy is
/// more useful next to other backend vacancies than next to a slightly closer text of another
/// trade. An index scan in relaxed order may return them slightly out of order, hence the sort.
fn rank_neighbours(mut near: Vec<(Uuid, bool, f64)>, limit: i64) -> Vec<(Uuid, f64)> {
    near.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.total_cmp(&a.2)).then(a.0.cmp(&b.0)));
    near.into_iter()
        .take(usize::try_from(limit).unwrap_or(0))
        .map(|(id, _, similarity)| (id, similarity))
        .collect()
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({"cover_letter": "Здравствуйте! Три года пишу на Go сервисы платежей (PostgreSQL, Kafka), на платформе подтверждён Middle. Готов выйти через две недели."}))]
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
    /// Stages the company's hiring system reported.
    pub outcomes: Vec<HiringOutcome>,
}

/// Apply to a vacancy: the company gets access to your contacts. Under 18 this needs the legal
/// representative's consent first
#[utoipa::path(post, path = "/vacancies/{id}/applications", tag = "vacancies",
    params(("id" = Uuid, Path)),
    request_body = ApplyInput,
    responses((status = 201, body = MyApplication), (status = 404, body = Problem), (status = 409, description = "`already_applied`; `guardian_consent_required`", body = Problem), (status = 422, description = "`validation_failed`: `cover_letter`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn apply(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(vacancy_id): Path<Uuid>,
    Json(input): Json<ApplyInput>,
) -> ApiResult<(StatusCode, Json<MyApplication>)> {
    if input.cover_letter.chars().count() > 4000 {
        return Err(ApiError::field("cover_letter", "Не длиннее 4000 символов"));
    }
    db::vacancies::published_vacancy(&state.pool, vacancy_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if let Some(doc) = db::guardians::missing_guardian_consent(&state.pool, user.id).await? {
        return Err(ApiError::GuardianConsentRequired {
            document_id: doc.id,
        });
    }
    let letter = input.cover_letter.trim();
    let mut tx = state.pool.begin().await?;
    let reopened = db::vacancies::reopen_application(&mut *tx, letter, vacancy_id, user.id).await?;
    let id = match reopened {
        Some(id) => id,
        None => {
            let id = Uuid::now_v7();
            let inserted =
                db::vacancies::insert_application(&mut *tx, id, vacancy_id, user.id, letter).await;
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
    db::candidates::touch_candidate_activity(&mut *tx, user.id).await?;
    Outbox::new(&state)
        .application(&mut tx, EventType::ApplicationCreated, id)
        .await?;
    Notifier::new(&state)
        .application(&mut tx, id, ApplicationStatus::Sent)
        .await?;
    tx.commit().await?;
    let app = my_apps(&state.pool, user.id)
        .await?
        .into_iter()
        .find(|a| a.id == id)
        .ok_or_else(ApiError::not_found)?;
    Ok((StatusCode::CREATED, Json(app)))
}

pub(crate) async fn my_apps(pool: &PgPool, candidate: Uuid) -> ApiResult<Vec<MyApplication>> {
    let outcomes = OutcomeIndex::for_candidate(pool, candidate).await?;
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
            outcomes: outcomes.of(r.id),
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
    responses((status = 204), (status = 404, description = "Not the caller's application", body = Problem), (status = 409, description = "`application_closed`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn withdraw_application(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let mut tx = state.pool.begin().await?;
    if db::vacancies::withdraw_application(&mut *tx, id, user.id).await? == 0 {
        let own = db::vacancies::candidate_applications(&mut *tx, user.id).await?;
        if !own.iter().any(|a| a.id == id) {
            return Err(ApiError::not_found());
        }
        return Err(ApiError::conflict(
            "application_closed",
            "Отклик уже рассмотрен или отозван",
        ));
    }
    Outbox::new(&state)
        .application(&mut tx, EventType::ApplicationStatusChanged, id)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn key(value: &[u8]) -> ApiResult<Option<String>> {
        let mut headers = HeaderMap::new();
        headers.insert("idempotency-key", HeaderValue::from_bytes(value).unwrap());
        idempotency_key(&headers)
    }

    fn rejected(result: ApiResult<Option<String>>) {
        let e = result.unwrap_err();
        assert_eq!(
            e.summary(),
            (422, "validation_failed", vec!["idempotency_key"])
        );
        let ApiError::Validation(errors) = e else {
            unreachable!()
        };
        assert_eq!(
            serde_json::to_value(&errors).unwrap(),
            serde_json::json!({ "idempotency_key": ["Idempotency-Key — от 1 до 100 символов"] })
        );
    }

    #[test]
    fn neighbours_of_the_same_specialization_come_first_then_by_closeness() {
        let id = Uuid::from_u128;
        let near = vec![
            (id(1), false, 0.96),
            (id(2), true, 0.91),
            (id(3), true, 0.93),
            (id(4), false, 0.99),
            (id(5), true, 0.91),
        ];
        assert_eq!(
            rank_neighbours(near.clone(), 4),
            [(id(3), 0.93), (id(2), 0.91), (id(5), 0.91), (id(4), 0.99)]
        );
        assert_eq!(rank_neighbours(near, 0), []);
    }

    #[test]
    fn stored_flags_become_soft_warnings_and_junk_is_ignored() {
        let flags = serde_json::json!([
            {"kind": "salary_wide_range", "min": 100000, "max": 500000},
            {"kind": "paraphrase_of_other", "vacancy_id": "x", "company": "Северный Код", "title": "Go", "cosine": 0.96, "threshold": 0.953},
        ]);
        let w = warnings(&flags);
        assert_eq!(w.len(), 2);
        assert_eq!(
            w[1].kind,
            crate::routes::trust::WarningKind::ParaphraseOfOther
        );
        assert!(!w[1].text.contains("Северный Код"));
        assert!(warnings(&serde_json::json!([{"kind": "unknown"}])).is_empty());
        assert!(warnings(&serde_json::json!({})).is_empty());
    }

    #[test]
    fn idempotency_key_is_optional() {
        assert_eq!(idempotency_key(&HeaderMap::new()).unwrap(), None);
    }

    #[test]
    fn idempotency_key_is_trimmed() {
        assert_eq!(key(b" k ").unwrap().as_deref(), Some("k"));
        assert_eq!(
            key(b"inv-2026-10-06-0001").unwrap().as_deref(),
            Some("inv-2026-10-06-0001")
        );
    }

    #[test]
    fn empty_or_blank_idempotency_key_is_rejected() {
        rejected(key(b""));
        rejected(key(b"   "));
    }

    #[test]
    fn idempotency_key_takes_up_to_100_characters_after_trimming() {
        let hundred = "a".repeat(100);
        assert_eq!(key(hundred.as_bytes()).unwrap(), Some(hundred.clone()));
        assert_eq!(
            key(format!("  {hundred}  ").as_bytes()).unwrap(),
            Some(hundred)
        );
        rejected(key("a".repeat(101).as_bytes()));
    }

    #[test]
    fn idempotency_key_outside_visible_ascii_is_rejected() {
        rejected(key("ключ".as_bytes()));
        rejected(key(b"k\xff"));
    }
}
