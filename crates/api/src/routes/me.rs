use axum::{Json, extract::{Path, State}, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, Role, clear_session_cookies},
    crypto::sha256,
    error::{ApiError, ApiResult, Problem},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(me, delete_me))
        .routes(routes!(my_consents))
        .routes(routes!(revoke_consent))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    Profile,
    Survey,
    Assessment,
    Done,
}

#[derive(Serialize, ToSchema)]
pub struct CandidateSummary {
    pub public_code: String,
    pub onboarding_step: OnboardingStep,
    pub category: Option<catalog::Category>,
    pub active_session_id: Option<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct CompanySummary {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct MeResponse {
    pub id: Uuid,
    pub role: Role,
    pub email: String,
    pub email_verified: bool,
    pub candidate: Option<CandidateSummary>,
    pub company: Option<CompanySummary>,
}

/// Current user with onboarding status
#[utoipa::path(get, path = "", tag = "account",
    responses((status = 200, body = MeResponse), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn me(State(state): State<AppState>, user: AuthUser) -> ApiResult<Json<MeResponse>> {
    let client = state.pool.get().await?;
    let row = db::queries::users::user_by_id().bind(&client, &user.id).opt().await?.ok_or_else(ApiError::unauthorized)?;
    let mut res = MeResponse {
        id: row.id,
        role: row.role.into(),
        email: state.crypto.decrypt_str(&row.email_enc)?,
        email_verified: row.email_verified_at.is_some(),
        candidate: None,
        company: None,
    };
    match user.role {
        Role::Candidate => {
            if let Some(c) = db::queries::me::candidate_onboarding().bind(&client, &user.id).opt().await? {
                let category = match (c.specialization.as_deref(), c.grade.as_deref()) {
                    (Some(s), Some(g)) => Some(catalog::Category::new(s.parse()?, g.parse()?)),
                    _ => None,
                };
                let onboarding_step = if !c.has_profile {
                    OnboardingStep::Profile
                } else if !c.has_survey {
                    OnboardingStep::Survey
                } else if category.is_none() {
                    OnboardingStep::Assessment
                } else {
                    OnboardingStep::Done
                };
                res.candidate = Some(CandidateSummary {
                    public_code: c.public_code,
                    onboarding_step,
                    category,
                    active_session_id: c.active_session_id,
                });
            }
        }
        Role::Employer => {
            res.company = db::queries::me::company_by_owner()
                .bind(&client, &user.id)
                .opt()
                .await?
                .map(|c| CompanySummary { id: c.id, name: c.name });
        }
        Role::Admin => {}
    }
    Ok(Json(res))
}

/// Delete the account and personal data (152-ФЗ)
#[utoipa::path(delete, path = "", tag = "account",
    responses((status = 204), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn delete_me(State(state): State<AppState>, user: AuthUser, jar: CookieJar) -> ApiResult<(CookieJar, StatusCode)> {
    let mut client = state.pool.get().await?;
    let tx = client.transaction().await?;
    match user.role {
        Role::Candidate => {
            db::queries::me::delete_candidate().bind(&tx, &user.id).await?;
        }
        Role::Employer => {
            db::queries::me::delete_company_of().bind(&tx, &user.id).await?;
        }
        Role::Admin => {}
    }
    // The users row stays as an anonymized tombstone: audit logs keep pointing at a real id.
    let tombstone = sha256(format!("deleted:{}:{}", user.id, Uuid::now_v7()).as_bytes());
    db::queries::users::anonymize_user().bind(&tx, &tombstone, &user.id).await?;
    db::queries::users::revoke_user_refresh_tokens().bind(&tx, &user.id).await?;
    tx.commit().await?;
    user.revoke(&state).await?;
    Ok((clear_session_cookies(jar), StatusCode::NO_CONTENT))
}

#[derive(Serialize, ToSchema)]
pub struct UserConsent {
    pub id: Uuid,
    pub document_id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Consents the user has given
#[utoipa::path(get, path = "/consents", tag = "account",
    responses((status = 200, body = Vec<UserConsent>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn my_consents(State(state): State<AppState>, user: AuthUser) -> ApiResult<Json<Vec<UserConsent>>> {
    let client = state.pool.get().await?;
    let rows = db::queries::consents::user_consents().bind(&client, &user.id).all().await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| UserConsent {
                id: r.id,
                document_id: r.document_id,
                kind: r.kind,
                version: r.version,
                title: r.title,
                granted_at: r.granted_at,
                revoked_at: r.revoked_at,
            })
            .collect(),
    ))
}

/// Revoke a consent. Revoking publication or processing hides the candidate from search.
#[utoipa::path(post, path = "/consents/{id}/revoke", tag = "account",
    params(("id" = Uuid, Path, description = "user consent id")),
    responses((status = 204), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn revoke_consent(State(state): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> ApiResult<StatusCode> {
    let client = state.pool.get().await?;
    let kind = db::queries::consents::revoke_user_consent()
        .bind(&client, &id, &user.id)
        .opt()
        .await?
        .ok_or_else(ApiError::not_found)?;
    if user.role == Role::Candidate && matches!(kind.as_str(), "pd_publication" | "pd_processing") {
        db::queries::me::hide_candidate().bind(&client, &user.id).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
