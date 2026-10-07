//! Search over the whole candidate bank (E-4) and the anonymous profile with an explanation.

use axum::extract::State;
use catalog::{Grade, Specialization, WorkFormat};
use chrono::Utc;
use matching::{ScoreBreakdown, Weights};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::EmployerUser,
    cards::{self, CandidateCard},
    dto::Page,
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Path, Query},
    pool::{self, PoolFilter},
    skills,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(search))
        .routes(routes!(profile))
}

#[derive(Deserialize, IntoParams)]
pub struct SearchQuery {
    pub specialization: Specialization,
    pub grade: Option<Grade>,
    /// Comma-separated skills (aliases accepted); candidates must have all of them.
    pub skills: Option<String>,
    #[serde(default)]
    pub fsp_only: bool,
    /// «Только 18+»: leave out candidates aged 14–17.
    #[serde(default)]
    pub adults_only: bool,
    pub work_format: Option<WorkFormat>,
    pub city: Option<String>,
    pub max_expectation: Option<i64>,
    /// Free text over "about"; a single skill name is treated as a skill filter.
    pub q: Option<String>,
    /// Rank by fit to this need instead of profile strength.
    pub need_id: Option<Uuid>,
    /// 1–100, 20 by default; out-of-range values are clamped.
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub limit: Option<usize>,
    #[param(minimum = 0, default = 0)]
    pub offset: Option<usize>,
}

#[derive(Serialize, ToSchema)]
pub struct BankItem {
    pub candidate: CandidateCard,
    /// 0..1 strength of the verified profile (test, FSP, freshness).
    pub strength: f32,
    /// 0..100 fit to the need, when `need_id` is given.
    pub score: Option<f32>,
    pub reasons: Vec<String>,
    pub gaps: Vec<String>,
}

/// Search the candidate bank: specialization, grade, stack, FSP achievements, format, city,
/// expectations, adults only
#[utoipa::path(get, path = "/candidates", tag = "matching",
    params(SearchQuery),
    responses((status = 200, body = Page<BankItem>), (status = 404, description = "`need_id` is not a need of this company", body = Problem), (status = 422, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn search(
    State(state): State<AppState>,
    user: EmployerUser,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<Page<BankItem>>> {
    let mut required: Vec<String> = q
        .skills
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let mut text = q.q.as_deref().map(str::trim).filter(|t| !t.is_empty());
    if let Some(t) = text
        && let Some(def) = matching::normalize(t)
    {
        required.push(def.slug.to_owned());
        text = None;
    }
    let required = skills::canonicalize(&required)
        .map_err(|u| ApiError::field("skills", skills::unknown_message(&u)))?;
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let offset = q.offset.unwrap_or(0);

    let company = company_id(&state, user.id).await?;
    let now = Utc::now().timestamp();
    let need = match q.need_id {
        Some(id) => Some(super::needs::effective_need(&state.pool, company, id).await?),
        None => None,
    };
    let text_for_rank = need.as_ref().map(|n| n.description.as_str()).or(text);
    let loaded = pool::load(
        &state,
        q.specialization,
        &PoolFilter {
            grade: q.grade,
            text: text_for_rank,
            fsp_only: q.fsp_only,
            max_expectation: q.max_expectation,
            required_skills: &required,
            adults_only: q.adults_only,
            only: None,
        },
    )
    .await?;

    let mut ordered: Vec<(usize, f32, Option<f32>, Option<ScoreBreakdown>)> = match &need {
        Some(n) => {
            let mut n = n.clone();
            if let Some(g) = q.grade {
                n.grades = vec![g];
            }
            if let Some(f) = q.work_format {
                n.work_formats = vec![f];
            }
            if q.city.is_some() {
                n.city = q.city.clone();
            }
            matching::rank(&n, &loaded.pool, &Weights::default(), now, offset + limit)
                .into_iter()
                .map(|r| (r.index, r.strength, Some(r.total), r.breakdown))
                .collect()
        }
        None => matching::rank_by_strength(&loaded.pool, None, now)
            .into_iter()
            .filter(|r| {
                let p = loaded.pool.profile(r.index);
                q.work_format
                    .is_none_or(|f| p.is_some_and(|p| p.accepts_format(f)))
            })
            .map(|r| (r.index, r.strength, None, None))
            .collect(),
    };
    if need.is_none() && text.is_some() {
        ordered.retain(|(i, ..)| {
            loaded
                .pool
                .profile(*i)
                .is_some_and(|p| p.text_relevance() > 0.0)
        });
    }
    let total = ordered.len() as i64;
    let page: Vec<_> = ordered.into_iter().skip(offset).take(limit).collect();
    let ids: Vec<Uuid> = page.iter().map(|(i, ..)| loaded.ids[*i]).collect();
    let mut cards = cards::load(&state, company, &ids, now).await?;
    // The pool may be up to a generation old on another replica; the page is checked live.
    cards.retain(|c| c.searchable);
    let mut items = Vec::with_capacity(page.len());
    for (i, strength, score, breakdown) in page {
        let Some(pos) = cards.iter().position(|c| c.card.id == loaded.ids[i]) else {
            continue;
        };
        let (reasons, gaps) = breakdown.map(|b| (b.reasons, b.gaps)).unwrap_or_default();
        items.push(BankItem {
            candidate: cards.swap_remove(pos).card,
            strength,
            score,
            reasons,
            gaps,
        });
    }
    Ok(Json(Page { items, total }))
}

#[derive(Deserialize, IntoParams)]
pub struct ProfileQuery {
    /// Explain the fit to this need.
    pub need_id: Option<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct CandidateProfileView {
    pub candidate: CandidateCard,
    pub breakdown: Option<ScoreBreakdown>,
}

/// Anonymous candidate profile; with `need_id` — why the candidate fits the need
#[utoipa::path(get, path = "/candidates/{id}", tag = "matching",
    params(("id" = Uuid, Path), ProfileQuery),
    responses((status = 200, body = CandidateProfileView), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn profile(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
    Query(q): Query<ProfileQuery>,
) -> ApiResult<Json<CandidateProfileView>> {
    let company = company_id(&state, user.id).await?;
    let now = Utc::now().timestamp();
    let found = cards::load(&state, company, &[id], now)
        .await?
        .pop()
        .ok_or_else(ApiError::not_found)?;
    if !found.visible_to_company() {
        return Err(ApiError::not_found());
    }
    let candidate = found.card;
    db::market::record_profile_view(&state.pool, company, id).await?;

    let breakdown = match (q.need_id, candidate.category) {
        (Some(need_id), Some(category)) => {
            let need = super::needs::effective_need(&state.pool, company, need_id).await?;
            let loaded = pool::load(
                &state,
                category.specialization,
                &PoolFilter {
                    text: Some(&need.description),
                    only: Some(id),
                    ..PoolFilter::default()
                },
            )
            .await?;
            (loaded.ids.first() == Some(&id))
                .then(|| matching::explain_at(&need, &loaded.pool, 0, &Weights::default(), now))
        }
        _ => None,
    };
    Ok(Json(CandidateProfileView {
        candidate,
        breakdown,
    }))
}
