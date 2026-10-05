use axum::{
    Json,
    extract::{Query, State},
};
use catalog::{Direction, Grade, Industry, Specialization, WorkFormat};
use matching::{Discipline, SkillView};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{auth::Role, error::ApiResult, state::AppState};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_catalog))
        .routes(routes!(skills))
        .routes(routes!(current_consents))
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
    pub fsp_disciplines: Vec<Named<Discipline>>,
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
                specializations: d
                    .specializations()
                    .map(|s| Named {
                        slug: s,
                        name: s.name_ru(),
                    })
                    .collect(),
            })
            .collect(),
        grades: Grade::ALL
            .iter()
            .map(|&g| Named {
                slug: g,
                name: g.name_ru(),
            })
            .collect(),
        work_formats: WorkFormat::ALL
            .iter()
            .map(|&f| Named {
                slug: f,
                name: f.name_ru(),
            })
            .collect(),
        industries: Industry::ALL
            .iter()
            .map(|&i| Named {
                slug: i,
                name: i.name_ru(),
            })
            .collect(),
        fsp_disciplines: Discipline::ALL
            .iter()
            .map(|&d| Named {
                slug: d,
                name: d.name_ru(),
            })
            .collect(),
    })
}

#[derive(Deserialize, IntoParams)]
pub struct SkillQuery {
    /// Prefix or alias in Russian or English ("постг", "k8s").
    pub q: Option<String>,
    /// Without `q`: the typical stack of this specialization.
    pub specialization: Option<Specialization>,
}

/// Skill autocomplete with aliases
#[utoipa::path(get, path = "/catalog/skills", tag = "catalog",
    params(SkillQuery),
    responses((status = 200, body = Vec<SkillView>)))]
async fn skills(Query(q): Query<SkillQuery>) -> Json<Vec<SkillView>> {
    let q_text = q.q.as_deref().map(str::trim).unwrap_or("");
    let defs: Vec<&matching::SkillDef> = if !q_text.is_empty() {
        matching::search(q_text).into_iter().take(20).collect()
    } else if let Some(spec) = q.specialization {
        matching::typical_skills(spec)
            .iter()
            .filter_map(|s| matching::lookup(s))
            .collect()
    } else {
        Vec::new()
    };
    Json(defs.into_iter().map(SkillView::from).collect())
}

#[derive(Serialize, ToSchema)]
pub struct ConsentDocument {
    pub id: Uuid,
    /// `pd_processing`, `pd_publication`, `terms`, `proctoring_video`.
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub body: String,
    /// Registration of these roles requires the document.
    pub required_for: Vec<Role>,
    /// Roles that may grant the document at all (`POST /me/consents`).
    pub applies_to: Vec<Role>,
}

impl From<db::consents::ConsentDocument> for ConsentDocument {
    fn from(d: db::consents::ConsentDocument) -> Self {
        Self {
            id: d.id,
            kind: d.kind,
            version: d.version,
            title: d.title,
            body: d.body,
            required_for: d.required_for.into_iter().map(Role::from).collect(),
            applies_to: d.applies_to.into_iter().map(Role::from).collect(),
        }
    }
}

/// Current versions of consent documents: registration, the privacy policy page, re-granting
#[utoipa::path(get, path = "/consents/current", tag = "catalog", responses((status = 200, body = Vec<ConsentDocument>)))]
async fn current_consents(State(state): State<AppState>) -> ApiResult<Json<Vec<ConsentDocument>>> {
    let docs = db::consents::current_consent_documents(&state.pool).await?;
    Ok(Json(docs.into_iter().map(ConsentDocument::from).collect()))
}
