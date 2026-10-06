//! Two-sided market transparency (architecture.md §6.2): employers see supply per category,
//! candidates see anonymous demand for theirs.

use axum::extract::State;
use catalog::{Category, Grade, Specialization};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{CandidateUser, EmployerUser},
    error::{ApiError, ApiResult, Problem},
    extract::Json,
    state::AppState,
};

pub fn employer_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(categories))
}

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(market))
}

#[derive(Serialize, ToSchema)]
pub struct CategoryCard {
    pub category: Category,
    pub name: String,
    pub candidates: i64,
    pub with_fsp: i64,
    pub actively_looking: i64,
    pub median_expectation: Option<i64>,
}

pub async fn category_cards(state: &AppState) -> ApiResult<Vec<CategoryCard>> {
    let overview = state.candidates.overview(&state.pool).await?;
    let mut cards = Vec::with_capacity(overview.value.len());
    for r in &overview.value {
        let category = Category::new(
            r.specialization.parse::<Specialization>()?,
            r.grade.parse::<Grade>()?,
        );
        cards.push(CategoryCard {
            name: category.name_ru(),
            category,
            candidates: r.candidates,
            with_fsp: r.with_fsp,
            actively_looking: r.actively_looking,
            median_expectation: r.median_expectation,
        });
    }
    cards.sort_unstable_by_key(|c| (c.category.specialization, c.category.grade));
    Ok(cards)
}

/// All categories with supply statistics
#[utoipa::path(get, path = "/categories", tag = "matching",
    responses((status = 200, body = Vec<CategoryCard>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn categories(
    State(state): State<AppState>,
    _user: EmployerUser,
) -> ApiResult<Json<Vec<CategoryCard>>> {
    Ok(Json(category_cards(&state).await?))
}

#[derive(Serialize, ToSchema)]
pub struct MarketStats {
    pub category: Category,
    pub companies_viewed_you_30d: i64,
    pub invitations_in_category_30d: i64,
    pub open_needs: i64,
    pub peers_in_category: i64,
    pub median_offer_min: Option<i64>,
    pub median_offer_max: Option<i64>,
}

/// Anonymous demand in the candidate's category
#[utoipa::path(get, path = "/market", tag = "candidate",
    responses((status = 200, body = MarketStats), (status = 409, description = "`no_category`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn market(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<MarketStats>> {
    let c = db::candidates::candidate_invitable(&state.pool, user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let (Some(spec), Some(grade)) = (c.specialization, c.grade) else {
        return Err(ApiError::conflict(
            "no_category",
            "Категория появится после теста",
        ));
    };
    let category = Category::new(spec.parse()?, grade.parse()?);
    let demand = state
        .candidates
        .demand(&state.pool, category.specialization, category.grade)
        .await?;
    let d = &demand.value;
    Ok(Json(MarketStats {
        category,
        companies_viewed_you_30d: db::market::companies_viewed(&state.pool, user.id).await?,
        invitations_in_category_30d: d.invitations_in_category,
        open_needs: d.open_needs,
        peers_in_category: d.peers,
        median_offer_min: d.median_salary_min,
        median_offer_max: d.median_salary_max,
    }))
}
