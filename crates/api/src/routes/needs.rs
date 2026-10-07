//! Employer needs → recommended categories → persisted shortlist with explanations (§6.1).
//! Refining filters re-ranks but keeps pinned and dismissed candidates (M-4).

use std::cmp::Reverse;

use axum::{extract::State, http::StatusCode};
use catalog::{Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Utc};
use matching::{CategoryStats, Need, ParsedNeed, ScoreBreakdown, Weights};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use sqlx::PgPool;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::EmployerUser,
    cards::{self, CandidateCard},
    dto::{SalaryRange, parse_slugs, slugs},
    embeddings,
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Created, Json, Path},
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
#[schema(example = json!({
    "title": "Go-разработчик в команду платежей",
    "description": "Процессинг платежей: идемпотентные API, outbox, шардирование PostgreSQL. Команда из 6 человек, релизы каждый день.",
    "specialization": "backend",
    "grades": ["middle", "senior"],
    "allow_adjacent_grades": false,
    "must_skills": ["go", "postgresql", "kafka"],
    "nice_skills": ["grpc", "docker"],
    "salary": {"min": 280000, "max": 360000},
    "work_formats": ["remote", "hybrid"],
    "city": "Москва",
    "industries": ["fintech"],
    "status": "active"
}))]
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
#[schema(example = json!({
    "grades": ["middle"],
    "must_skills": ["go", "postgresql"],
    "nice_skills": ["kafka"],
    "work_formats": ["remote"],
    "fsp_only": false,
    "adults_only": true,
    "max_expectation": 350000,
    "limit": 30
}))]
pub struct ShortlistFilters {
    pub grades: Option<Vec<Grade>>,
    pub allow_adjacent_grades: Option<bool>,
    pub must_skills: Option<Vec<String>>,
    pub nice_skills: Option<Vec<String>>,
    pub work_formats: Option<Vec<WorkFormat>>,
    pub city: Option<String>,
    #[serde(default)]
    pub fsp_only: bool,
    /// «Только 18+»: leave out candidates aged 14–17.
    #[serde(default)]
    pub adults_only: bool,
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
    grades: Vec<String>,
    work_formats: Vec<String>,
    industries: Vec<String>,
}

impl CleanNeed {
    fn fields<'a>(&'a self, n: &'a NeedInput) -> db::needs::NeedFields<'a> {
        db::needs::NeedFields {
            title: n.title.trim(),
            description: n.description.trim(),
            specialization: n.specialization.slug(),
            grades: &self.grades,
            allow_adjacent_grades: n.allow_adjacent_grades,
            must_skills: &self.must,
            nice_skills: &self.nice,
            salary_min: n.salary.min,
            salary_max: n.salary.max,
            work_formats: &self.work_formats,
            city: self.city.as_deref(),
            industries: &self.industries,
        }
    }
}

fn validate(n: &NeedInput) -> ApiResult<CleanNeed> {
    let must = skills::canonicalize(&n.must_skills);
    let nice = skills::canonicalize(&n.nice_skills);
    let mut v = Validator::default();
    v.check(
        !n.title.trim().is_empty() && n.title.chars().count() <= 200,
        "title",
        "Название до 200 символов",
    )
    .check(
        n.description.chars().count() <= 10_000,
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
        n.must_skills.len() <= 20,
        "must_skills",
        "Не больше 20 навыков",
    )
    .check(
        n.nice_skills.len() <= 20,
        "nice_skills",
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
        grades: slugs(&n.grades),
        work_formats: slugs(&n.work_formats),
        industries: slugs(&n.industries),
    })
}

pub async fn effective_need(pool: &PgPool, company: Uuid, id: Uuid) -> ApiResult<Need> {
    let need = load_need(pool, company, id).await?;
    Ok(need.to_need(&need.filters))
}

async fn load_need(pool: &PgPool, company: Uuid, id: Uuid) -> ApiResult<HiringNeed> {
    let row = db::needs::company_need(pool, company, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(need_from_row!(row))
}

/// Company needs
#[utoipa::path(get, path = "/needs", tag = "matching",
    responses((status = 200, body = Vec<HiringNeed>), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_needs(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<HiringNeed>>> {
    let company = company_id(&state, user.id).await?;
    Ok(Json(company_need_views(&state.pool, company).await?))
}

pub(crate) async fn company_need_views(pool: &PgPool, company: Uuid) -> ApiResult<Vec<HiringNeed>> {
    let rows = db::needs::company_needs(pool, company).await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(need_from_row!(r));
    }
    Ok(out)
}

/// Describe a need: specialization, grades, stack, salary range
#[utoipa::path(post, path = "/needs", tag = "matching",
    request_body = NeedInput,
    responses((status = 201, body = HiringNeed, headers(("Location" = String, description = "`/api/v1/employer/needs/{id}`"))), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn create_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(n): Json<NeedInput>,
) -> ApiResult<Created<HiringNeed>> {
    let clean = validate(&n)?;
    let company = company_id(&state, user.id).await?;
    let id = Uuid::now_v7();
    db::needs::insert_need(&state.pool, id, company, &clean.fields(&n)).await?;
    Ok(Created(
        format!("/api/v1/employer/needs/{id}"),
        load_need(&state.pool, company, id).await?,
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct ParseInput {
    pub text: String,
}

/// Recognize specialization, grade, stack, salary and format in a free-text vacancy
#[utoipa::path(post, path = "/needs/parse", tag = "matching",
    request_body = ParseInput,
    responses((status = 200, body = ParsedNeed), (status = 422, description = "`validation_failed`: `text`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn parse_need(
    _user: EmployerUser,
    Json(input): Json<ParseInput>,
) -> ApiResult<Json<ParsedNeed>> {
    if input.text.chars().count() > 20_000 {
        return Err(ApiError::field("text", "Не длиннее 20 000 символов"));
    }
    Ok(Json(matching::parse_need(&input.text)))
}

/// One need
#[utoipa::path(get, path = "/needs/{id}", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = HiringNeed), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<HiringNeed>> {
    let company = company_id(&state, user.id).await?;
    Ok(Json(load_need(&state.pool, company, id).await?))
}

/// Edit or close a need
#[utoipa::path(put, path = "/needs/{id}", tag = "matching",
    params(("id" = Uuid, Path)),
    request_body = NeedInput,
    responses((status = 200, body = HiringNeed), (status = 404, body = Problem), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn update_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(n): Json<NeedInput>,
) -> ApiResult<Json<HiringNeed>> {
    let clean = validate(&n)?;
    let company = company_id(&state, user.id).await?;
    let status = match n.status.unwrap_or(NeedStatus::Active) {
        NeedStatus::Active => "active",
        NeedStatus::Closed => "closed",
    };
    let updated =
        db::needs::update_need(&state.pool, id, company, status, &clean.fields(&n)).await?;
    if updated == 0 {
        return Err(ApiError::not_found());
    }
    Ok(Json(load_need(&state.pool, company, id).await?))
}

/// Delete a need with its shortlist
#[utoipa::path(delete, path = "/needs/{id}", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn delete_need(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    if db::needs::delete_need(&state.pool, id, company).await? == 0 {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Recommended categories for the need: every grade of its specialization with supply and stack fit
#[utoipa::path(get, path = "/needs/{id}/categories", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Vec<CategoryStats>), (status = 404, description = "No such need or not this company's", body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn need_categories(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<CategoryStats>>> {
    let company = company_id(&state, user.id).await?;
    let need = load_need(&state.pool, company, id).await?;
    let loaded = pool::load(&state, need.specialization, &PoolFilter::default()).await?;
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
    // The JSON the rebuild stored, passed through without parsing.
    #[schema(value_type = ScoreBreakdown)]
    pub breakdown: Box<RawValue>,
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

async fn shortlist_view(state: &AppState, company: Uuid, need: HiringNeed) -> ApiResult<Shortlist> {
    let visible = db::needs::shortlist_items(&state.pool, need.id).await?;
    let ids: Vec<Uuid> = visible.iter().map(|r| r.candidate_id).collect();
    // The snapshot holds the cached cards; loading it here keeps a shortlist cheap to reopen.
    state
        .candidates
        .pool(&state.pool, need.specialization)
        .await?;
    let mut cards = cards::load(state, company, &ids, Utc::now().timestamp()).await?;
    cards.retain(|c| c.searchable || c.related());
    let generated = need.shortlist_generated_at;
    let had_previous = has_earlier_build(generated, visible.iter().map(|r| r.first_seen_at));
    let mut items = Vec::with_capacity(visible.len());
    for r in visible {
        let Some(pos) = cards.iter().position(|c| c.card.id == r.candidate_id) else {
            continue;
        };
        items.push(ShortlistItem {
            candidate: cards.swap_remove(pos).card,
            score: r.score,
            rank: r.rank,
            breakdown: r.breakdown.0,
            in_current: r.in_current,
            pinned: r.pinned,
            dismissed: r.dismissed,
            is_new: is_new(generated, had_previous, r.first_seen_at),
            first_seen_at: r.first_seen_at,
        });
    }
    items.sort_by_key(|i| shortlist_order(i.pinned, i.in_current, i.rank));
    Ok(Shortlist { need, items })
}

/// Whether the shortlist was built before the latest rebuild: on the first build every candidate
/// would otherwise be «new».
fn has_earlier_build(
    generated: Option<DateTime<Utc>>,
    mut first_seen: impl Iterator<Item = DateTime<Utc>>,
) -> bool {
    generated.is_some_and(|g| first_seen.any(|at| at < g))
}

fn is_new(
    generated: Option<DateTime<Utc>>,
    had_previous: bool,
    first_seen_at: DateTime<Utc>,
) -> bool {
    had_previous && generated.is_some_and(|g| first_seen_at >= g)
}

/// Pinned first, then those matching the current filters, then by rank.
fn shortlist_order(
    pinned: bool,
    in_current: bool,
    rank: i32,
) -> (Reverse<bool>, Reverse<bool>, i32) {
    (Reverse(pinned), Reverse(in_current), rank)
}

/// Current shortlist with explanations
#[utoipa::path(get, path = "/needs/{id}/shortlist", tag = "matching",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Shortlist), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_shortlist(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Shortlist>> {
    let company = company_id(&state, user.id).await?;
    let need = load_need(&state.pool, company, id).await?;
    Ok(Json(shortlist_view(&state, company, need).await?))
}

/// Build or refine the shortlist; pinned and dismissed marks survive, new candidates are flagged
#[utoipa::path(post, path = "/needs/{id}/shortlist", tag = "matching",
    params(("id" = Uuid, Path)),
    request_body = ShortlistFilters,
    responses((status = 200, body = Shortlist), (status = 404, body = Problem), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn build_shortlist(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Json(mut filters): Json<ShortlistFilters>,
) -> ApiResult<Json<Shortlist>> {
    for (field, list) in [
        ("must_skills", &mut filters.must_skills),
        ("nice_skills", &mut filters.nice_skills),
    ] {
        if let Some(list) = list {
            *list = skills::canonicalize(list)
                .map_err(|unknown| ApiError::field(field, skills::unknown_message(&unknown)))?;
        }
    }
    let limit = filters
        .limit
        .unwrap_or(SHORTLIST_DEFAULT)
        .clamp(1, SHORTLIST_MAX);
    let company = company_id(&state, user.id).await?;
    let need = load_need(&state.pool, company, id).await?;
    let now = Utc::now().timestamp();
    let effective = need.to_need(&filters);
    let meaning = embeddings::need_vector(&state, id).await?;
    let loaded = pool::load(
        &state,
        need.specialization,
        &PoolFilter {
            text: Some(&effective.description),
            fsp_only: filters.fsp_only,
            max_expectation: filters.max_expectation,
            adults_only: filters.adults_only,
            meaning: meaning.as_deref(),
            ..PoolFilter::default()
        },
    )
    .await?;
    let ranked = matching::rank(&effective, &loaded.pool, &Weights::default(), now, limit);
    let mut keep = Vec::with_capacity(limit);
    let mut scores = Vec::with_capacity(limit);
    let mut ranks = Vec::with_capacity(limit);
    let mut breakdowns = Vec::with_capacity(limit);
    for (pos, r) in ranked.into_iter().take(limit).enumerate() {
        keep.push(loaded.ids[r.index]);
        scores.push(f64::from(r.total));
        ranks.push(pos as i32 + 1);
        breakdowns.push(serde_json::to_string(&r.breakdown)?);
    }
    let filters_json = serde_json::to_value(&filters)?;

    // Everything is computed before the need's row is locked: concurrent rebuilds of one need
    // queue on that lock, so it is held for three statements and the commit.
    let mut tx = state.pool.begin().await?;
    db::needs::start_shortlist_generation(&mut *tx, &filters_json, id).await?;
    db::needs::upsert_shortlist_items(&mut *tx, id, &keep, &scores, &ranks, &breakdowns).await?;
    db::needs::mark_shortlist_stale(&mut *tx, id, &keep).await?;
    tx.commit().await?;

    let need = load_need(&state.pool, company, id).await?;
    Ok(Json(shortlist_view(&state, company, need).await?))
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
    responses((status = 204), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn patch_shortlist_item(
    State(state): State<AppState>,
    user: EmployerUser,
    Path((id, candidate_id)): Path<(Uuid, Uuid)>,
    Json(p): Json<ShortlistPatch>,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    if db::needs::patch_shortlist_item(
        &state.pool,
        p.pinned,
        p.dismissed,
        company,
        id,
        candidate_id,
    )
    .await?
        == 0
    {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    #[test]
    fn shortlist_lists_pinned_then_matching_then_by_rank() {
        let mut items = [
            ("c", false, true, 1),
            ("e", false, false, 1),
            ("b", true, false, 3),
            ("d", false, true, 2),
            ("a", true, true, 5),
        ];
        items.sort_by_key(|&(_, pinned, in_current, rank)| {
            shortlist_order(pinned, in_current, rank)
        });
        let order: Vec<&str> = items.iter().map(|i| i.0).collect();
        assert_eq!(order, ["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn first_build_flags_nobody_as_new() {
        let built = at("2026-10-07T12:00:00Z");
        let first_seen = [built, built];
        assert!(!has_earlier_build(Some(built), first_seen.into_iter()));
        assert!(!has_earlier_build(None, first_seen.into_iter()));
        assert!(!is_new(Some(built), false, built));
    }

    #[test]
    fn rebuild_flags_candidates_seen_first_in_it() {
        let earlier = at("2026-10-01T12:00:00Z");
        let rebuilt = at("2026-10-07T12:00:00Z");
        let had_previous = has_earlier_build(Some(rebuilt), [earlier, rebuilt].into_iter());
        assert!(had_previous);
        assert!(is_new(Some(rebuilt), had_previous, rebuilt));
        assert!(is_new(
            Some(rebuilt),
            had_previous,
            at("2026-10-07T12:00:01Z")
        ));
        assert!(!is_new(Some(rebuilt), had_previous, earlier));
        assert!(!is_new(None, true, rebuilt), "never built, nothing is new");
    }
}
