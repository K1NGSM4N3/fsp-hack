use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    age::{self, AgeBracket},
    auth::{AuthUser, Role, clear_session_cookies, client_ip, user_agent},
    crypto::sha256,
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Path},
    guardian::{self, GuardianState},
    proctoring,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(me, delete_me))
        .routes(routes!(my_consents, grant_consent))
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
    pub age: AgeSummary,
}

/// Derived from the birth date today: a minor turns into an adult on the 18th birthday.
#[derive(Serialize, ToSchema)]
pub struct AgeSummary {
    pub bracket: AgeBracket,
    /// Minors only: the 18th birthday, when every restriction lifts.
    pub adult_from: Option<NaiveDate>,
    /// Minors only: where the legal representative's consent stands
    /// (details in `GET /candidate/guardian`).
    pub guardian: Option<GuardianState>,
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
    let row = db::users::user_by_id(&state.pool, user.id)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
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
            if let Some(c) = db::me::candidate_onboarding(&state.pool, user.id).await? {
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
                let guardian = if c.minor {
                    Some(
                        db::guardians::latest_request(&state.pool, user.id)
                            .await?
                            .map_or(GuardianState::NotSent, |r| {
                                guardian::request_state(&r, Utc::now())
                            }),
                    )
                } else {
                    None
                };
                res.candidate = Some(CandidateSummary {
                    public_code: c.public_code,
                    onboarding_step,
                    category,
                    active_session_id: c.active_session_id,
                    age: AgeSummary {
                        bracket: if c.minor {
                            AgeBracket::Minor
                        } else {
                            AgeBracket::Adult
                        },
                        adult_from: c.birth_date.filter(|_| c.minor).map(age::adult_from),
                        guardian,
                    },
                });
            }
        }
        Role::Employer => {
            res.company = db::me::company_by_owner(&state.pool, user.id)
                .await?
                .map(|c| CompanySummary {
                    id: c.id,
                    name: c.name,
                });
        }
        Role::Admin => {}
    }
    Ok(Json(res))
}

/// Delete the account and personal data (152-ФЗ), including a minor's birth date and everything
/// about their legal representative
#[utoipa::path(delete, path = "", tag = "account",
    responses((status = 204), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn delete_me(
    State(state): State<AppState>,
    user: AuthUser,
    jar: CookieJar,
) -> ApiResult<(CookieJar, StatusCode)> {
    let mut tx = state.pool.begin().await?;
    match user.role {
        Role::Candidate => {
            db::guardians::erase_grant_trail(&mut *tx, user.id).await?;
            db::me::delete_candidate(&mut *tx, user.id).await?;
        }
        Role::Employer => {
            db::me::delete_company_of(&mut *tx, user.id).await?;
        }
        Role::Admin => {}
    }
    // The users row stays as an anonymized tombstone: audit logs keep pointing at a real id.
    let tombstone = sha256(format!("deleted:{}:{}", user.id, Uuid::now_v7()).as_bytes());
    db::users::anonymize_user(&mut *tx, &tombstone, user.id).await?;
    db::users::revoke_user_refresh_tokens(&mut *tx, user.id).await?;
    tx.commit().await?;
    state.candidates.invalidate().await;
    if user.role == Role::Candidate
        && let Err(e) = state.recordings.remove_candidate(user.id).await
    {
        tracing::warn!(error = %e, user = %user.id, "proctoring files left for the retention sweep");
    }
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
async fn my_consents(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<UserConsent>>> {
    let rows = db::consents::user_consents(&state.pool, user.id).await?;
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

#[derive(Deserialize, ToSchema)]
pub struct GrantConsent {
    /// Id of the current version from `GET /api/v1/consents/current`.
    pub document_id: Uuid,
}

/// Give a consent: the current version of a document that applies to your role. Recorded with
/// time, IP and User-Agent like at registration; a document already in force is not granted twice
#[utoipa::path(post, path = "/consents", tag = "account",
    request_body = GrantConsent,
    responses(
        (status = 201, description = "Granted", body = UserConsent),
        (status = 200, description = "Already in force: the existing grant", body = UserConsent),
        (status = 401, body = Problem),
        (status = 422, description = "`validation_failed`: unknown document, not its current version, not for this role, or `guardian_consent` (only the legal representative gives it)", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn grant_consent(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<GrantConsent>,
) -> ApiResult<(StatusCode, Json<UserConsent>)> {
    let doc = db::consents::document_version(&state.pool, req.document_id)
        .await?
        .ok_or_else(|| ApiError::field("document_id", "Такого документа нет"))?;
    if !doc.current {
        return Err(ApiError::field(
            "document_id",
            "Это не действующая редакция документа — обновите страницу",
        ));
    }
    if !doc.applies_to.contains(&user.role.into()) {
        return Err(ApiError::field(
            "document_id",
            "Этот документ не относится к вашему аккаунту",
        ));
    }
    if doc.kind == db::guardians::KIND {
        return Err(ApiError::field(
            "document_id",
            "Это согласие даёт законный представитель — по ссылке из письма",
        ));
    }
    let grant = db::consents::grant(
        &state.pool,
        Uuid::now_v7(),
        user.id,
        doc.id,
        client_ip(&headers).as_deref(),
        user_agent(&headers).as_deref(),
    )
    .await?;
    state.candidates.invalidate().await;
    let status = if grant.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        Json(UserConsent {
            id: grant.id,
            document_id: doc.id,
            kind: doc.kind,
            version: doc.version,
            title: doc.title,
            granted_at: grant.granted_at,
            revoked_at: None,
        }),
    ))
}

/// Revoke a consent. Revoking publication or processing hides the candidate from search;
/// revoking the video consent deletes the candidate's recordings right away. `guardian_consent`
/// belongs to the legal representative and is revoked only by the link from their letter (404).
#[utoipa::path(post, path = "/consents/{id}/revoke", tag = "account",
    params(("id" = Uuid, Path, description = "user consent id or consent document id")),
    responses((status = 204), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn revoke_consent(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let kind = db::consents::revoke_user_consent(&state.pool, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if user.role == Role::Candidate && matches!(kind.as_str(), "pd_publication" | "pd_processing") {
        db::me::hide_candidate(&state.pool, user.id).await?;
    }
    state.candidates.invalidate().await;
    if user.role == Role::Candidate && kind == proctoring::CONSENT_KIND {
        let recordings = db::proctoring::candidate_recordings(&state.pool, user.id).await?;
        proctoring::purge(&state.pool, &state.recordings, &recordings).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}
