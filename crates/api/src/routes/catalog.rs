use axum::{Json, extract::State};
use catalog::{Direction, Grade, Industry, Specialization, WorkFormat};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{auth::Role, error::ApiResult, state::AppState};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_catalog)).routes(routes!(current_consents))
}

#[derive(Serialize, ToSchema)]
pub struct Named<T> {
    pub slug: T,
    pub name: &'static str,
}

#[derive(Serialize, ToSchema)]
pub struct DirectionInfo {
    pub slug: Direction,
    pub name: &'static str,
    pub specializations: Vec<Named<Specialization>>,
}

#[derive(Serialize, ToSchema)]
pub struct CatalogResponse {
    pub directions: Vec<DirectionInfo>,
    pub grades: Vec<Named<Grade>>,
    pub work_formats: Vec<Named<WorkFormat>>,
    pub industries: Vec<Named<Industry>>,
}

/// Directions, specializations, grades, formats and industries
#[utoipa::path(get, path = "/catalog", tag = "catalog", responses((status = 200, body = CatalogResponse)))]
async fn get_catalog() -> Json<CatalogResponse> {
    Json(CatalogResponse {
        directions: Direction::ALL
            .iter()
            .map(|&d| DirectionInfo {
                slug: d,
                name: d.name_ru(),
                specializations: d.specializations().map(|s| Named { slug: s, name: s.name_ru() }).collect(),
            })
            .collect(),
        grades: Grade::ALL.iter().map(|&g| Named { slug: g, name: g.name_ru() }).collect(),
        work_formats: WorkFormat::ALL.iter().map(|&f| Named { slug: f, name: f.name_ru() }).collect(),
        industries: Industry::ALL.iter().map(|&i| Named { slug: i, name: i.name_ru() }).collect(),
    })
}

#[derive(Serialize, ToSchema)]
pub struct ConsentDocument {
    pub id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub body: String,
    pub required_for: Vec<Role>,
}

/// Current versions of consent documents shown at registration
#[utoipa::path(get, path = "/consents/current", tag = "catalog", responses((status = 200, body = Vec<ConsentDocument>)))]
async fn current_consents(State(state): State<AppState>) -> ApiResult<Json<Vec<ConsentDocument>>> {
    let client = state.pool.get().await?;
    let docs = db::queries::consents::current_consent_documents().bind(&client).all().await?;
    Ok(Json(
        docs.into_iter()
            .map(|d| ConsentDocument {
                id: d.id,
                kind: d.kind,
                version: d.version,
                title: d.title,
                body: d.body,
                required_for: d.required_for.into_iter().map(Role::from).collect(),
            })
            .collect(),
    ))
}
