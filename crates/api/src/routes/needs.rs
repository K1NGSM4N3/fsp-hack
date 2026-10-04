//! Employer needs → recommended categories → persisted shortlist with explanations (§6.1).
//! Refining filters re-ranks but keeps pinned and dismissed candidates (M-4).

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use catalog::{Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{CategoryStats, Need, ParsedNeed, ScoreBreakdown, Weights};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::EmployerUser,
    cards::{self, CandidateCard},
    dto::{SalaryRange, parse_slugs, slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    pool::{self, PoolFilter},
    skills,
    state::AppState,
};

const SHORTLIST_DEFAULT: usize = 30;
const SHORTLIST_MAX: usize = 100;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_needs, create_need))
        .routes(routes!(parse_need))
        .routes(routes!(get_need, update_need, delete_need))
        .routes(routes!(need_categories))
        .routes(routes!(get_shortlist, build_shortlist))
        .routes(routes!(patch_shortlist_item))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NeedStatus {
    Active,
    Closed,
}

#[derive(Deserialize, ToSchema)]
pub struct NeedInput {
    pub title: String,
    /// What the team does and whom it needs; also used for text relevance.
    #[serde(default)]
    pub description: String,
    pub specialization: Specialization,
    pub grades: Vec<Grade>,
    #[serde(default)]
    pub allow_adjacent_grades: bool,
    #[serde(default)]
    pub must_skills: Vec<String>,
    #[serde(default)]
    pub nice_skills: Vec<String>,
    pub salary: SalaryRange,
    #[serde(default)]
    pub work_formats: Vec<WorkFormat>,
    pub city: Option<String>,
    #[serde(default)]
    pub industries: Vec<Industry>,
    pub status: Option<NeedStatus>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct ShortlistFilters {
    pub grades: Option<Vec<Grade>>,
    pub allow_adjacent_grades: Option<bool>,
    pub must_skills: Option<Vec<String>>,
    pub nice_skills: Option<Vec<String>>,
    pub work_formats: Option<Vec<WorkFormat>>,
    pub city: Option<String>,
    #[serde(default)]
    pub fsp_only: bool,
    /// Drop candidates whose expectations exceed this.
    pub max_expectation: Option<i64>,
    pub limit: Option<usize>,
}

#[derive(Serialize, ToSchema)]
pub struct HiringNeed {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub specialization: Specialization,
    pub grades: Vec<Grade>,
    pub allow_adjacent_grades: bool,
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub salary: SalaryRange,
    pub work_formats: Vec<WorkFormat>,
    pub city: Option<String>,
    pub industries: Vec<Industry>,
    pub status: NeedStatus,
    pub filters: ShortlistFilters,
    pub shortlisted: i64,
    pub invited: i64,
    pub shortlist_generated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

macro_rules! need_from_row {
    ($r:expr) => {{
        let r = $r;
        HiringNeed {
            id: r.id,
            title: r.title,
            description: r.description,
            specialization: r.specialization.parse()?,
            grades: parse_slugs(&r.grades),
            allow_adjacent_grades: r.allow_adjacent_grades,
            must_skills: r.must_skills,
            nice_skills: r.nice_skills,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            work_formats: parse_slugs(&r.work_formats),
            city: r.city,
            industries: parse_slugs(&r.industries),
            status: if r.status == "closed" {
                NeedStatus::Closed
            } else {
                NeedStatus::Active
            },
            filters: serde_json::from_value(r.filters).unwrap_or_default(),
            shortlisted: r.shortlisted,
            invited: r.invited,
            shortlist_generated_at: r.shortlist_generated_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }};
}

impl HiringNeed {
    fn to_need(&self, filters: &ShortlistFilters) -> Need {
        Need {
            specialization: self.specialization,
            grades: filters
                .grades
                .clone()
                .unwrap_or_else(|| self.grades.clone()),
            allow_adjacent_grades: filters
                .allow_adjacent_grades
                .unwrap_or(self.allow_adjacent_grades),
            must_skills: filters
                .must_skills
                .clone()
                .unwrap_or_else(|| self.must_skills.clone()),
            nice_skills: filters
                .nice_skills
                .clone()
                .unwrap_or_else(|| self.nice_skills.clone()),
            salary_min: self.salary.min,
            salary_max: self.salary.max,
            work_formats: filters
                .work_formats
                .clone()
                .unwrap_or_else(|| self.work_formats.clone()),
            city: filters.city.clone().or_else(|| self.city.clone()),
            industries: self.industries.clone(),
            description: format!("{}\n{}", self.title, self.description),
        }
    }
}

struct CleanNeed {
    must: Vec<String>,
    nice: Vec<String>,
    city: Option<String>,
}

fn validate(n: &NeedInput) -> ApiResult<CleanNeed> {
    let must = skills::canonicalize(&n.must_skills);
    let nice = skills::canonicalize(&n.nice_skills);
    let mut v = Validator::default();
    v.check(
        !n.title.trim().is_empty() && n.title.len() <= 200,
        "title",
        "Название до 200 символов",
    )
    .check(
        n.description.len() <= 10_000,
        "description",
        "Описание до 10 000 символов",
    )
    .check(
        !n.grades.is_empty(),
        "grades",
        "Выберите хотя бы один грейд",
    )
    .check(n.salary.min > 0, "salary.min", "Зарплата «от» обязательна")
    .check(
        n.salary.max >= n.salary.min,
        "salary.max",
        "«До» не меньше «от»",
    )
    .check(
        n.must_skills.len() <= 20 && n.nice_skills.len() <= 20,
        "must_skills",
        "Не больше 20 навыков",
    );
    if let Err(unknown) = &must {
        v.check_with(false, "must_skills", || {
            skills::unknown_message(unknown).into_owned()
        });
    }
    if let Err(unknown) = &nice {
        v.check_with(false, "nice_skills", || {
            skills::unknown_message(unknown).into_owned()
        });
    }
    v.finish()?;
    Ok(CleanNeed {
        must: must.unwrap_or_default(),
        nice: nice.unwrap_or_default(),
        city: n
            .city
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(str::to_owned),
    })
}

pub async fn effective_need(
    client: &deadpool_postgres::Object,
    company: Uuid,
    id: Uuid,
) -> ApiResult<Need> {
    let need = load_need(client, company, id).await?;
    Ok(need.to_need(&need.filters))
}

async fn load_need(
    client: &deadpool_postgres::Object,
    company: Uuid,
    id: Uuid,
) -> ApiResult<HiringNeed> {
    let row = db::queries::needs::company_need()
        .bind(client, &company, &id)
        .opt()
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(need_from_row!(row))
}

/// Company needs
#[utoipa::path(get, path = "/needs", tag = "matching",
    responses((status = 200, body = Vec<HiringNeed>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_needs(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<HiringNeed>>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let rows = db::queries::needs::company_needs()
        .bind(&client, &company)
        .all()
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(need_from_row!(r));
    }
    Ok(Json(out))
}

/// Describe a need: specialization, grades, stack, salary range
#[utoipa::path(post, path = "/needs", tag = "matching",
    request_body = NeedInput,
    responses((status = 201, body = HiringNeed), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn create_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(n): Json<NeedInput>,
) -> ApiResult<(StatusCode, Json<HiringNeed>)> {
    let clean = validate(&n)?;
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let id = Uuid::now_v7();
    db::queries::needs::insert_need()
        .bind(
            &client,
            &id,
            &company,
            &n.title.trim(),
            &n.description.trim(),
            &n.specialization.slug(),
            &slugs(&n.grades),
            &n.allow_adjacent_grades,
            &clean.must,
            &clean.nice,
            &n.salary.min,
            &n.salary.max,
            &slugs(&n.work_formats),
            &clean.city.as_deref(),
            &slugs(&n.industries),
        )
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(load_need(&client, company, id).await?),
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct ParseInput {
    pub text: String,
}

/// Recognize specialization, grade, stack, salary and format in a free-text vacancy
#[utoipa::path(post, path = "/needs/parse", tag = "matching",
    request_body = ParseInput,
    responses((status = 200, body = ParsedNeed)),
    security(("cookie" = []), ("bearer" = [])))]
async fn parse_need(
    _user: EmployerUser,
    Json(input): Json<ParseInput>,
) -> ApiResult<Json<ParsedNeed>> {
    if input.text.len() > 20_000 {
        return Err(ApiError::field("text", "Не длиннее 20 000 символов"));
    }
    Ok(Json(matching::parse_need(&input.text)))
}

/// One need
#[utoipa::path(get, path = "/needs/{id}", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = HiringNeed), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<HiringNeed>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    Ok(Json(load_need(&client, company, id).await?))
}

/// Edit or close a need
#[utoipa::path(put, path = "/needs/{id}", tag = "matching",
    params(("id" = Uuid, Path)),
    request_body = NeedInput,
    responses((status = 200, body = HiringNeed), (status = 404, body = Problem), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn update_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(n): Json<NeedInput>,
) -> ApiResult<Json<HiringNeed>> {
    let clean = validate(&n)?;
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let status = match n.status.unwrap_or(NeedStatus::Active) {
        NeedStatus::Active => "active",
        NeedStatus::Closed => "closed",
    };
    let updated = db::queries::needs::update_need()
        .bind(
            &client,
            &n.title.trim(),
            &n.description.trim(),
            &n.specialization.slug(),
            &slugs(&n.grades),
            &n.allow_adjacent_grades,
            &clean.must,
            &clean.nice,
            &n.salary.min,
            &n.salary.max,
            &slugs(&n.work_formats),
            &clean.city.as_deref(),
            &slugs(&n.industries),
            &status,
            &id,
            &company,
        )
        .await?;
    if updated == 0 {
        return Err(ApiError::not_found());
    }
    Ok(Json(load_need(&client, company, id).await?))
}

/// Delete a need with its shortlist
#[utoipa::path(delete, path = "/needs/{id}", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn delete_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    if db::queries::needs::delete_need()
        .bind(&client, &id, &company)
        .await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Recommended categories for the need: every grade of its specialization with supply and stack fit
#[utoipa::path(get, path = "/needs/{id}/categories", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vec<CategoryStats>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn need_categories(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<CategoryStats>>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let need = load_need(&client, company, id).await?;
    let now = Utc::now().timestamp();
    let loaded = pool::load(&client, need.specialization, &PoolFilter::default(), now).await?;
    Ok(Json(matching::category_stats(
        &need.to_need(&need.filters),
        &loaded.pool,
    )))
}

#[derive(Serialize, ToSchema)]
pub struct ShortlistItem {
    pub candidate: CandidateCard,
    pub score: f64,
    pub rank: i32,
    pub breakdown: ScoreBreakdown,
    /// Matches the current filters; pinned candidates stay listed even when they don't.
    pub in_current: bool,
    pub pinned: bool,
    pub dismissed: bool,
    /// Appeared in the latest rebuild.
    pub is_new: bool,
    pub first_seen_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct Shortlist {
    pub need: HiringNeed,
    pub items: Vec<ShortlistItem>,
}

async fn shortlist_view(
    state: &AppState,
    client: &deadpool_postgres::Object,
    company: Uuid,
    need: HiringNeed,
) -> ApiResult<Shortlist> {
    let rows = db::queries::needs::shortlist_items()
        .bind(client, &need.id)
        .all()
        .await?;
    let visible: Vec<_> = rows
        .into_iter()
        .filter(|r| r.in_current || r.pinned)
        .collect();
    let ids: Vec<Uuid> = visible.iter().map(|r| r.candidate_id).collect();
    let mut cards =
        cards::load(client, &state.crypto, company, &ids, Utc::now().timestamp()).await?;
    let generated = need.shortlist_generated_at;
    let had_previous = generated.is_some_and(|g| visible.iter().any(|r| r.first_seen_at < g));
    let mut items = Vec::with_capacity(visible.len());
    for r in visible {
        let Some(pos) = cards.iter().position(|c| c.id == r.candidate_id) else {
            continue;
        };
        items.push(ShortlistItem {
            candidate: cards.swap_remove(pos),
            score: r.score,
            rank: r.rank,
            breakdown: serde_json::from_value(r.breakdown)?,
            in_current: r.in_current,
            pinned: r.pinned,
            dismissed: r.dismissed,
            is_new: had_previous && generated.is_some_and(|g| r.first_seen_at >= g),
            first_seen_at: r.first_seen_at,
        });
    }
    items.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then(b.in_current.cmp(&a.in_current))
            .then(a.rank.cmp(&b.rank))
    });
    Ok(Shortlist { need, items })
}

/// Current shortlist with explanations
#[utoipa::path(get, path = "/needs/{id}/shortlist", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Shortlist), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_shortlist(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Shortlist>> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let need = load_need(&client, company, id).await?;
    Ok(Json(shortlist_view(&state, &client, company, need).await?))
}

/// Build or refine the shortlist; pinned and dismissed marks survive, new candidates are flagged
#[utoipa::path(post, path = "/needs/{id}/shortlist", tag = "matching",
    params(("id" = Uuid, Path)),
    request_body = ShortlistFilters,
    responses((status = 200, body = Shortlist), (status = 404, body = Problem), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn build_shortlist(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(mut filters): Json<ShortlistFilters>,
) -> ApiResult<Json<Shortlist>> {
    for list in [&mut filters.must_skills, &mut filters.nice_skills]
        .into_iter()
        .flatten()
    {
        *list = skills::canonicalize(list)
            .map_err(|unknown| ApiError::field("skills", skills::unknown_message(&unknown)))?;
    }
    let limit = filters
        .limit
        .unwrap_or(SHORTLIST_DEFAULT)
        .clamp(1, SHORTLIST_MAX);
    let mut client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    let need = load_need(&client, company, id).await?;
    let now = Utc::now().timestamp();
    let effective = need.to_need(&filters);
    let loaded = pool::load(
        &client,
        need.specialization,
        &PoolFilter {
            text: Some(&effective.description),
            fsp_only: filters.fsp_only,
            max_expectation: filters.max_expectation,
            ..PoolFilter::default()
        },
        now,
    )
    .await?;
    let ranked = matching::rank(&effective, &loaded.pool, &Weights::default(), now, limit);

    let tx = client.transaction().await?;
    db::queries::needs::start_shortlist_generation()
        .bind(&tx, &serde_json::to_value(&filters)?, &id)
        .one()
        .await?;
    let mut keep = Vec::with_capacity(limit);
    for (pos, r) in ranked.into_iter().take(limit).enumerate() {
        let candidate = loaded.ids[r.index];
        keep.push(candidate);
        db::queries::needs::upsert_shortlist_item()
            .bind(
                &tx,
                &id,
                &candidate,
                &f64::from(r.total),
                &(pos as i32 + 1),
                &serde_json::to_value(&r.breakdown)?,
            )
            .await?;
    }
    db::queries::needs::mark_shortlist_stale()
        .bind(&tx, &id, &keep)
        .await?;
    tx.commit().await?;

    let need = load_need(&client, company, id).await?;
    Ok(Json(shortlist_view(&state, &client, company, need).await?))
}

#[derive(Deserialize, ToSchema)]
pub struct ShortlistPatch {
    pub pinned: Option<bool>,
    pub dismissed: Option<bool>,
}

/// Pin or dismiss a candidate in the shortlist
#[utoipa::path(patch, path = "/needs/{id}/shortlist/{candidate_id}", tag = "matching",
    params(("id" = Uuid, Path), ("candidate_id" = Uuid, Path)),
    request_body = ShortlistPatch,
    responses((status = 204), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn patch_shortlist_item(
    State(state): State<AppState>,
    user: EmployerUser,
    Path((id, candidate_id)): Path<(Uuid, Uuid)>,
    Json(p): Json<ShortlistPatch>,
) -> ApiResult<StatusCode> {
    let client = state.pool.get().await?;
    let company = company_id(&client, user.id).await?;
    if db::queries::needs::patch_shortlist_item()
        .bind(
            &client,
            &p.pinned,
            &p.dismissed,
            &company,
            &id,
            &candidate_id,
        )
        .await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}
