//! Trust and anti-fake (X-1): the employer's own trust level and quota with reasons, soft checks
//! while filling a vacancy or an invitation, and the moderation list for administrators.

use axum::{
    Json,
    extract::{Query, State},
};
use catalog::{Category, Grade, Specialization};
use chrono::{DateTime, Duration, Utc};
use matching::trust::{
    DomainCheck, MAX_QUOTA, MIN_QUOTA, TrustFlag, TrustLevel, TrustReason, display_domain,
    quota_rule,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::{AdminUser, EmployerUser},
    dto::{SalaryRange, VacancyStatus},
    error::{ApiError, ApiResult, Problem, Validator},
    state::AppState,
    trust,
};

const MODERATION_LIMIT: i64 = 200;

pub fn employer_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(trust_status))
        .routes(routes!(salary_check))
        .routes(routes!(similar_vacancies))
}

pub fn admin_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(moderation))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WarningKind {
    SalaryAboveMarket,
    SalaryWideRange,
    DuplicateOwn,
    CopyOfOther,
}

impl From<&TrustFlag> for WarningKind {
    fn from(f: &TrustFlag) -> Self {
        match f {
            TrustFlag::SalaryAboveMarket { .. } => WarningKind::SalaryAboveMarket,
            TrustFlag::SalaryWideRange { .. } => WarningKind::SalaryWideRange,
            TrustFlag::DuplicateOwn { .. } => WarningKind::DuplicateOwn,
            TrustFlag::CopyOfOther { .. } => WarningKind::CopyOfOther,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct TrustWarning {
    pub kind: WarningKind,
    pub text: String,
}

#[derive(Serialize, ToSchema)]
pub struct DomainStatus {
    /// Shared domain of the login email and the website, in display form (Unicode for IDN).
    pub domain: Option<String>,
    pub check: DomainCheck,
    /// The result in words and, if it failed, how to fix it.
    pub hint: String,
}

#[derive(Serialize, ToSchema)]
pub struct TrustStatus {
    pub level: TrustLevel,
    /// Invitations allowed per 7 days right now.
    pub quota_limit: i64,
    pub quota_used: i64,
    pub quota_base: i64,
    pub quota_min: i64,
    pub quota_max: i64,
    /// Sum of positive reasons, capped at `quota_max − quota_base`.
    pub bonus: i64,
    /// Sum of negative reasons (≤ 0).
    pub penalty: i64,
    pub reasons: Vec<TrustReason>,
    /// What raises the quota, most effective first.
    pub advice: Vec<String>,
    pub domain: DomainStatus,
    /// The quota rule in plain words.
    pub rule: Vec<String>,
}

/// The company's trust level, weekly invitation quota, the reasons behind it and how to raise it
#[utoipa::path(get, path = "/trust", tag = "trust",
    responses(
        (status = 200, body = TrustStatus),
        (status = 403, description = "`wrong_role`", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn trust_status(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<TrustStatus>> {
    let company = company_id(&state.pool, user.id).await?;
    let check = trust::refresh_domain(&state.pool, &state.crypto, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let (a, _) = trust::company_assessment(&state.pool, company).await?;
    let used = db::invitations::company_invitations_since(
        &state.pool,
        company,
        Utc::now() - Duration::days(7),
    )
    .await?;
    Ok(Json(TrustStatus {
        level: a.level,
        quota_limit: i64::from(a.quota),
        quota_used: used,
        quota_base: i64::from(a.base),
        quota_min: i64::from(MIN_QUOTA),
        quota_max: i64::from(MAX_QUOTA),
        bonus: i64::from(a.bonus),
        penalty: i64::from(a.penalty),
        reasons: a.reasons,
        advice: a.advice,
        domain: DomainStatus {
            domain: check.verified_domain().map(display_domain),
            hint: check.hint(),
            check,
        },
        rule: quota_rule(),
    }))
}

#[derive(Deserialize, IntoParams)]
pub struct SalaryQuery {
    pub specialization: Specialization,
    pub grade: Grade,
    pub min: i64,
    pub max: i64,
}

/// Stated expectations of the category's candidates.
#[derive(Serialize, ToSchema)]
pub struct MarketView {
    pub samples: i64,
    pub median: i64,
    pub p90: i64,
}

#[derive(Serialize, ToSchema)]
pub struct SalaryCheck {
    pub category: Category,
    /// Absent when the category has too few stated expectations.
    pub market: Option<MarketView>,
    /// Soft warnings: publishing and sending are never blocked.
    pub warnings: Vec<TrustWarning>,
}

/// Compare a salary range with the category market before publishing or inviting
#[utoipa::path(get, path = "/salary-check", tag = "trust",
    params(SalaryQuery),
    responses(
        (status = 200, body = SalaryCheck),
        (status = 403, description = "`wrong_role`", body = Problem),
        (status = 422, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn salary_check(
    State(state): State<AppState>,
    _user: EmployerUser,
    Query(q): Query<SalaryQuery>,
) -> ApiResult<Json<SalaryCheck>> {
    let mut v = Validator::default();
    v.check(q.min > 0, "min", "Зарплата «от» обязательна")
        .check(q.max >= q.min, "max", "«До» не меньше «от»")
        .check(q.max <= 100_000_000, "max", "Проверьте сумму");
    v.finish()?;
    let category = Category::new(q.specialization, q.grade);
    let (band, flags) = trust::salary_check(&state.pool, category, q.min, q.max).await?;
    Ok(Json(SalaryCheck {
        category,
        market: band
            .filter(|b| b.samples >= matching::trust::salary::MIN_MARKET_SAMPLES)
            .map(|b| MarketView {
                samples: i64::from(b.samples),
                median: b.median,
                p90: b.p90,
            }),
        warnings: flags
            .iter()
            .map(|f| TrustWarning {
                kind: f.into(),
                text: f.employer_text(),
            })
            .collect(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct SimilarInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    /// The vacancy being edited, so it does not match itself.
    pub exclude_id: Option<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct SimilarVacancy {
    pub id: Uuid,
    pub title: String,
    pub status: VacancyStatus,
    /// Share of common word pairs, 0..1 (MinHash estimate); listed from 0.6.
    pub similarity: f32,
}

/// Own drafts and published vacancies with nearly the same text — offer to edit one instead
#[utoipa::path(post, path = "/vacancies/similar", tag = "trust",
    request_body = SimilarInput,
    responses(
        (status = 200, body = Vec<SimilarVacancy>),
        (status = 403, description = "`wrong_role`", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
        (status = 422, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn similar_vacancies(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(input): Json<SimilarInput>,
) -> ApiResult<Json<Vec<SimilarVacancy>>> {
    let mut v = Validator::default();
    v.check(input.title.len() <= 200, "title", "Не длиннее 200 символов")
        .check(
            input.description.len() <= 10_000,
            "description",
            "Не длиннее 10 000 символов",
        );
    v.finish()?;
    let company = company_id(&state.pool, user.id).await?;
    let similar = trust::similar_own(
        &state.pool,
        company,
        input.exclude_id,
        input.title.trim(),
        input.description.trim(),
    )
    .await?;
    Ok(Json(
        similar
            .into_iter()
            .map(|s| SimilarVacancy {
                id: s.id,
                title: s.title,
                status: s.status.into(),
                similarity: s.similarity,
            })
            .collect(),
    ))
}

#[derive(Serialize, ToSchema)]
pub struct FlaggedCompany {
    pub id: Uuid,
    pub name: String,
    pub level: TrustLevel,
    pub quota: i64,
    pub verified_domain: Option<String>,
    pub reasons: Vec<TrustReason>,
}

#[derive(Serialize, ToSchema)]
pub struct FlaggedVacancy {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub status: VacancyStatus,
    pub salary: SalaryRange,
    pub reasons: Vec<String>,
    pub flagged_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct FlaggedInvitation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub salary: SalaryRange,
    pub reasons: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct ModerationQueue {
    /// Companies with at least one negative signal, lowest quota first.
    pub companies: Vec<FlaggedCompany>,
    pub vacancies: Vec<FlaggedVacancy>,
    pub invitations: Vec<FlaggedInvitation>,
}

fn reasons(flags: serde_json::Value) -> ApiResult<Vec<String>> {
    let flags: Vec<TrustFlag> = serde_json::from_value(flags)?;
    Ok(flags.iter().map(TrustFlag::moderator_text).collect())
}

/// Flagged companies, vacancies and invitations with their reasons (moderation)
#[utoipa::path(get, path = "/trust", tag = "trust",
    responses(
        (status = 200, body = ModerationQueue),
        (status = 403, description = "`wrong_role`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn moderation(
    State(state): State<AppState>,
    _user: AdminUser,
) -> ApiResult<Json<ModerationQueue>> {
    let now = Utc::now().timestamp();
    let mut companies: Vec<FlaggedCompany> = db::trust::company_signals(&state.pool, None)
        .await?
        .into_iter()
        .filter_map(|row| {
            let a = matching::trust::assess(&trust::signals(&row), now);
            a.has_negative().then(|| FlaggedCompany {
                id: row.company_id,
                name: row.name,
                level: a.level,
                quota: i64::from(a.quota),
                verified_domain: row.verified_domain.as_deref().map(display_domain),
                reasons: a.reasons,
            })
        })
        .collect();
    companies.sort_by_key(|c| c.quota);

    let mut vacancies = Vec::new();
    for r in db::trust::flagged_vacancies(&state.pool, MODERATION_LIMIT).await? {
        vacancies.push(FlaggedVacancy {
            id: r.id,
            company_id: r.company_id,
            company_name: r.company_name,
            title: r.title,
            status: r.status.into(),
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            reasons: reasons(r.trust_flags)?,
            flagged_at: r.flagged_at,
        });
    }
    let mut invitations = Vec::new();
    for r in db::trust::flagged_invitations(&state.pool, MODERATION_LIMIT).await? {
        invitations.push(FlaggedInvitation {
            id: r.id,
            company_id: r.company_id,
            company_name: r.company_name,
            title: r.title,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            reasons: reasons(r.trust_flags)?,
            created_at: r.created_at,
        });
    }
    Ok(Json(ModerationQueue {
        companies,
        vacancies,
        invitations,
    }))
}
