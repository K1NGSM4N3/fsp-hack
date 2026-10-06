use axum::extract::State;
use chrono::{DateTime, Utc};
use matching::trust::display_domain;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::EmployerUser,
    error::{ApiError, ApiResult, Problem, Validator},
    extract::Json,
    state::AppState,
    trust,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_company, put_company))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CompanyProfile {
    pub name: String,
    pub description: String,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub city: Option<String>,
    pub size: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_telegram: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CompanyResponse {
    pub id: Uuid,
    #[serde(flatten)]
    pub profile: CompanyProfile,
    /// Website and login email share this domain; candidates see «Домен подтверждён».
    pub verified_domain: Option<String>,
    pub updated_at: DateTime<Utc>,
}

pub async fn company_id(state: &AppState, owner: Uuid) -> ApiResult<Uuid> {
    state
        .candidates
        .company_of(&state.pool, owner)
        .await?
        .ok_or_else(|| {
            ApiError::conflict(
                "company_profile_required",
                "Сначала заполните профиль компании",
            )
        })
}

/// Company profile
#[utoipa::path(get, path = "/company", tag = "employer",
    responses((status = 200, body = CompanyResponse), (status = 403, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_company(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<CompanyResponse>> {
    company_view(&state.pool, user.id)
        .await?
        .map(Json)
        .ok_or_else(ApiError::not_found)
}

pub(crate) async fn company_view(pool: &PgPool, owner: Uuid) -> ApiResult<Option<CompanyResponse>> {
    Ok(db::companies::company_of_owner(pool, owner)
        .await?
        .map(|c| CompanyResponse {
            id: c.id,
            profile: CompanyProfile {
                name: c.name,
                description: c.description,
                industry: c.industry,
                website: c.website,
                city: c.city,
                size: c.size,
                contact_email: c.contact_email,
                contact_phone: c.contact_phone,
                contact_telegram: c.contact_telegram,
            },
            verified_domain: c.verified_domain.as_deref().map(display_domain),
            updated_at: c.updated_at,
        }))
}

fn blank_to_none(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

/// Update the company profile
#[utoipa::path(put, path = "/company", tag = "employer",
    request_body = CompanyProfile,
    responses((status = 200, body = CompanyResponse), (status = 422, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn put_company(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(p): Json<CompanyProfile>,
) -> ApiResult<Json<CompanyResponse>> {
    let mut v = Validator::default();
    v.check(!p.name.trim().is_empty(), "name", "Укажите название")
        .check(p.name.len() <= 200, "name", "Слишком длинное название")
        .check(
            p.description.len() <= 5000,
            "description",
            "Слишком длинное описание",
        );
    v.finish()?;
    db::companies::update_company(
        &state.pool,
        user.id,
        &db::companies::CompanyUpdate {
            name: p.name.trim(),
            description: p.description.trim(),
            industry: blank_to_none(p.industry).as_deref(),
            website: blank_to_none(p.website).as_deref(),
            city: blank_to_none(p.city).as_deref(),
            size: blank_to_none(p.size).as_deref(),
            contact_email: blank_to_none(p.contact_email).as_deref(),
            contact_phone: blank_to_none(p.contact_phone).as_deref(),
            contact_telegram: blank_to_none(p.contact_telegram).as_deref(),
        },
    )
    .await?;
    trust::refresh_domain(&state.pool, &state.crypto, user.id).await?;
    get_company(State(state), user).await
}
