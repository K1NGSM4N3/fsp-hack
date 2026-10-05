//! FSP ID linking (architecture.md §7.1): OIDC Authorization Code + PKCE against the FSP
//! Keycloak realm, then a snapshot of achievements from the registry.

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use chrono::{DateTime, Duration, Utc};
use matching::{Achievement, FspScore};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::CandidateUser,
    error::{ApiError, ApiResult, Problem},
    fsp_client::{self, FspError},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(link_start))
        .routes(routes!(link_callback))
        .routes(routes!(link_dev))
        .routes(routes!(achievements))
        .routes(routes!(sync))
        .routes(routes!(unlink))
}

#[derive(Serialize, ToSchema)]
pub struct FspStatus {
    pub linked: bool,
    pub fsp_id: Option<String>,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    /// Snapshot from the registry (architecture.md §7.2). An empty list is a valid state.
    pub achievements: Vec<AchievementView>,
    /// Score for the candidate's specialization; "Нет данных ФСП" without history.
    pub score: Option<FspScore>,
}

#[derive(Serialize, ToSchema)]
pub struct AchievementView {
    #[serde(flatten)]
    pub achievement: Achievement,
    pub text: String,
}

impl FspStatus {
    fn unlinked() -> Self {
        Self {
            linked: false,
            fsp_id: None,
            linked_at: None,
            last_sync_at: None,
            achievements: Vec::new(),
            score: None,
        }
    }
}

fn settings_url(state: &AppState, outcome: &str) -> String {
    format!(
        "{}/candidate/settings?fsp={outcome}",
        state.config.public_url
    )
}

/// Start linking: redirects to the FSP ID login page
#[utoipa::path(get, path = "/link/start", tag = "fsp",
    responses((status = 303, description = "Redirect to FSP ID"), (status = 503, description = "`fsp_oidc_disabled`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn link_start(State(state): State<AppState>, user: CandidateUser) -> ApiResult<Redirect> {
    let request = fsp_client::authorization_request(&state.http, &state.config.fsp)
        .await
        .map_err(fsp_error)?;
    db::fsp::purge_oidc_states(&state.pool).await?;
    db::fsp::insert_oidc_state(
        &state.pool,
        &request.state,
        user.id,
        &request.pkce_verifier,
        &request.nonce,
    )
    .await?;
    Ok(Redirect::to(&request.url))
}

#[derive(Deserialize, IntoParams)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// OIDC redirect target: verifies the id_token, links `fsp_id`, syncs achievements, returns to settings
#[utoipa::path(get, path = "/link/callback", tag = "fsp",
    params(CallbackQuery),
    responses((status = 303, description = "Redirect to /candidate/settings?fsp=linked|error")))]
async fn link_callback(State(state): State<AppState>, Query(q): Query<CallbackQuery>) -> Redirect {
    match finish_link(&state, q).await {
        Ok(()) => Redirect::to(&settings_url(&state, "linked")),
        Err(reason) => {
            tracing::warn!(%reason, "fsp link failed");
            Redirect::to(&format!(
                "{}&reason={reason}",
                settings_url(&state, "error")
            ))
        }
    }
}

async fn finish_link(state: &AppState, q: CallbackQuery) -> Result<(), &'static str> {
    if q.error.is_some() {
        return Err("cancelled");
    }
    let (Some(code), Some(oidc_state)) = (q.code, q.state) else {
        return Err("bad_request");
    };
    let pending = db::fsp::take_oidc_state(&state.pool, &oidc_state)
        .await
        .map_err(|_| "internal")?
        .ok_or("expired")?;
    if pending.created_at < Utc::now() - Duration::minutes(10) {
        return Err("expired");
    }
    let fsp_id = fsp_client::complete(
        &state.http,
        &state.config.fsp,
        &code,
        &pending.pkce_verifier,
        &pending.nonce,
    )
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "fsp id_token exchange failed");
        match e {
            FspError::NoFspId => "no_fsp_id",
            _ => "verification_failed",
        }
    })?;
    link(state, pending.user_id, &fsp_id, "oidc")
        .await
        .map_err(|e| match e {
            ApiError::Status {
                code: "fsp_already_linked",
                ..
            } => "already_linked",
            _ => "internal",
        })
}

async fn link(state: &AppState, candidate: uuid::Uuid, fsp_id: &str, via: &str) -> ApiResult<()> {
    if let Some(owner) = db::fsp::fsp_link_owner(&state.pool, fsp_id).await?
        && owner != candidate
    {
        return Err(ApiError::conflict(
            "fsp_already_linked",
            "Этот ФСП ID уже привязан к другому профилю",
        ));
    }
    db::fsp::upsert_fsp_link(&state.pool, candidate, fsp_id, via).await?;
    if let Err(e) = sync_achievements(state, candidate, fsp_id).await {
        tracing::warn!(error = %e, "fsp registry sync failed, link kept");
    }
    Ok(())
}

async fn sync_achievements(state: &AppState, candidate: uuid::Uuid, fsp_id: &str) -> ApiResult<()> {
    let achievements =
        match fsp_client::fetch_achievements(&state.http, &state.config.fsp, fsp_id).await {
            Ok(a) => a,
            Err(FspError::NotFound) => Vec::new(),
            Err(e) => return Err(fsp_error(e)),
        };
    db::fsp::store_fsp_achievements(
        &state.pool,
        &serde_json::Value::Array(achievements),
        candidate,
    )
    .await?;
    db::candidates::touch_candidate_activity(&state.pool, candidate).await?;
    Ok(())
}

fn fsp_error(e: FspError) -> ApiError {
    match e {
        FspError::OidcDisabled => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "fsp_oidc_disabled",
            "Вход через ФСП ID не настроен",
        ),
        FspError::RegistryDisabled => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "fsp_registry_disabled",
            "Реестр ФСП не настроен",
        ),
        other => ApiError::new(
            StatusCode::BAD_GATEWAY,
            "fsp_unavailable",
            "Сервис ФСП недоступен",
        )
        .with_detail(other.to_string()),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct DevLinkInput {
    pub fsp_id: String,
}

/// Development only (`FSP_DEV_LINK=true`): link an FSP ID without Keycloak
#[utoipa::path(post, path = "/link/dev", tag = "fsp",
    request_body = DevLinkInput,
    responses((status = 200, body = FspStatus), (status = 404, description = "Disabled"), (status = 409, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn link_dev(
    State(state): State<AppState>,
    user: CandidateUser,
    Json(input): Json<DevLinkInput>,
) -> ApiResult<Json<FspStatus>> {
    if !state.config.fsp.dev_link {
        return Err(ApiError::not_found());
    }
    let fsp_id = input.fsp_id.trim().to_uppercase();
    if fsp_id.is_empty() || fsp_id.len() > 40 {
        return Err(ApiError::field("fsp_id", "Укажите ФСП ID"));
    }
    link(&state, user.id, &fsp_id, "dev").await?;
    load_status(&state.pool, user.id).await.map(Json)
}

pub(crate) async fn load_status(pool: &PgPool, candidate: uuid::Uuid) -> ApiResult<FspStatus> {
    let Some(l) = db::fsp::fsp_link(pool, candidate).await? else {
        return Ok(FspStatus::unlinked());
    };
    let list = crate::pool::achievements(Some(&l.achievements));
    let spec = db::candidates::candidate_invitable(pool, candidate)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?
        .specialization
        .and_then(|s| s.parse().ok());
    let score = spec.map(|s| matching::fsp_score(&list, s, Utc::now().timestamp()));
    Ok(FspStatus {
        linked: true,
        fsp_id: Some(l.fsp_id),
        linked_at: Some(l.linked_at),
        last_sync_at: l.last_sync_at,
        achievements: list
            .into_iter()
            .map(|a| AchievementView {
                text: a.describe_ru(),
                achievement: a,
            })
            .collect(),
        score,
    })
}

/// Linked FSP ID and the achievements snapshot
#[utoipa::path(get, path = "/achievements", tag = "fsp",
    responses((status = 200, body = FspStatus)),
    security(("cookie" = []), ("bearer" = [])))]
async fn achievements(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<FspStatus>> {
    load_status(&state.pool, user.id).await.map(Json)
}

/// Re-sync achievements from the registry
#[utoipa::path(post, path = "/sync", tag = "fsp",
    responses((status = 200, body = FspStatus), (status = 404, body = Problem), (status = 502, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn sync(State(state): State<AppState>, user: CandidateUser) -> ApiResult<Json<FspStatus>> {
    let link = db::fsp::fsp_link(&state.pool, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    sync_achievements(&state, user.id, &link.fsp_id).await?;
    load_status(&state.pool, user.id).await.map(Json)
}

/// Unlink FSP ID and drop the achievements snapshot
#[utoipa::path(delete, path = "/link", tag = "fsp",
    responses((status = 204)),
    security(("cookie" = []), ("bearer" = [])))]
async fn unlink(State(state): State<AppState>, user: CandidateUser) -> ApiResult<StatusCode> {
    db::fsp::delete_fsp_link(&state.pool, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
