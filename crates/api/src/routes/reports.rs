//! Complaints about vacancies, invitations and companies (X-1). Unresolved complaints from
//! candidates lower the company's trust level and invitation quota (architecture.md §11.1).

use std::time::Duration;

use axum::{extract::State, http::StatusCode};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult, Problem},
    extract::Json,
    state::AppState,
};

const DAILY_LIMIT: i64 = 20;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create_report))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReportTarget {
    Vacancy,
    Invitation,
    Company,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReportReason {
    FakeVacancy,
    SalaryMismatch,
    Spam,
    Abuse,
    Other,
}

#[derive(Deserialize, ToSchema)]
#[schema(example = json!({
    "target_kind": "vacancy",
    "target_id": "019a8b10-3c4d-7a5b-8e9f-0a1b2c3d4e5f",
    "reason": "salary_mismatch",
    "comment": "В вакансии 250–300 тыс. ₽, а на созвоне назвали 120 тыс. и «остальное премией»."
}))]
pub struct ReportInput {
    pub target_kind: ReportTarget,
    pub target_id: Uuid,
    pub reason: ReportReason,
    pub comment: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ReportCreated {
    pub id: Uuid,
}

/// Report a fake vacancy, a misleading invitation or an abusive company
#[utoipa::path(post, path = "/reports", tag = "account",
    request_body = ReportInput,
    responses((status = 201, body = ReportCreated), (status = 429, body = Problem), (status = 422, description = "`validation_failed`: `comment`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn create_report(
    State(state): State<AppState>,
    user: AuthUser,
    Json(r): Json<ReportInput>,
) -> ApiResult<(StatusCode, Json<ReportCreated>)> {
    let comment = r
        .comment
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty());
    if comment.is_some_and(|c| c.chars().count() > 2000) {
        return Err(ApiError::field("comment", "Не длиннее 2000 символов"));
    }
    let recent = db::reports::recent_reports_by(&state.pool, user.id).await?;
    if recent.count >= DAILY_LIMIT {
        let wait = recent
            .first_slot_frees_at
            .map_or(0, |at| (at - Utc::now()).num_seconds());
        return Err(ApiError::rate_limited(
            "reports",
            Duration::from_secs(wait.max(1).unsigned_abs()),
        ));
    }
    let kind = match r.target_kind {
        ReportTarget::Vacancy => "vacancy",
        ReportTarget::Invitation => "invitation",
        ReportTarget::Company => "company",
    };
    let reason = match r.reason {
        ReportReason::FakeVacancy => "fake_vacancy",
        ReportReason::SalaryMismatch => "salary_mismatch",
        ReportReason::Spam => "spam",
        ReportReason::Abuse => "abuse",
        ReportReason::Other => "other",
    };
    let id = Uuid::now_v7();
    db::reports::insert_report(&state.pool, id, user.id, kind, r.target_id, reason, comment)
        .await?;
    Ok((StatusCode::CREATED, Json(ReportCreated { id })))
}
