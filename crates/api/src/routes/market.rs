//! Two-sided market transparency (architecture.md §6.2): employers see supply per category,
//! candidates see anonymous demand for theirs.

use axum::{Json, extract::State};
use catalog::{Category, Grade, Specialization};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{CandidateUser, EmployerUser},
    error::{ApiError, ApiResult, Problem},
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

pub async fn category_cards(client: &deadpool_postgres::Object) -> ApiResult<Vec<CategoryCard>> {
    let rows = db::queries::market::category_overview()
        .bind(client)
        .all()
        .await?;
    let mut cards = Vec::with_capacity(rows.len());
    for r in rows {
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
    let client = state.pool.get().await?;
    Ok(Json(category_cards(&client).await?))
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
    let client = state.pool.get().await?;
    let c = db::queries::candidates::candidate_invitable()
        .bind(&client, &user.id)
        .one()
        .await?;
    let (Some(spec), Some(grade)) = (c.specialization, c.grade) else {
        return Err(ApiError::conflict(
            "no_category",
            "Категория появится после теста",
        ));
    };
    let m = db::queries::market::candidate_market()
        .bind(&client, &user.id, &spec, &grade)
        .one()
        .await?;
    Ok(Json(MarketStats {
        category: Category::new(spec.parse()?, grade.parse()?),
        companies_viewed_you_30d: m.companies_viewed_you,
        invitations_in_category_30d: m.invitations_in_category,
        open_needs: m.open_needs,
        peers_in_category: m.peers,
        median_offer_min: m.median_salary_min,
        median_offer_max: m.median_salary_max,
    }))
}
