//! Inbound API for employers' ATS, authenticated by company API keys only
//! (`Authorization: Bearer fsp_live_…`, see `integrations::keys`). Contacts and the full profile
//! go through `ContactGrant` exactly like in the cabinet, with channel `api` in the access log.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use catalog::{Category, Grade, Industry, Specialization, WorkFormat};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::candidate_profile::{
    ExperienceEntry, SkillSource, load_profile_of, pdf_response, profile_pdf_bytes,
};
use crate::{
    contacts::{
        AccessChannel, AccessContext, CandidateContacts, ContactBasis, ContactGrant, Contacts,
    },
    dto::{HiringStage, InvitationStatus, SalaryRange, slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    integrations::{
        events::{
            AtsCandidate, AtsInvitation, category_of, expire_invitations, invitation_from_snapshot,
        },
        keys::{ApiClient, Scope},
        outcomes::{HiringOutcome, OutcomeIndex},
    },
    state::AppState,
};

const TAG: &str = "Интеграции ATS";
const POLL_DEFAULT: i64 = 50;
const POLL_MAX: i64 = 200;
const MAX_IMPORT_TEXT: usize = 20_000;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(ats_whoami))
        .routes(routes!(ats_invitations))
        .routes(routes!(ats_candidate))
        .routes(routes!(ats_candidate_pdf))
        .routes(routes!(ats_outcome))
        .routes(routes!(ats_import_need))
}

#[derive(Serialize, ToSchema)]
pub struct AtsWhoami {
    pub company_id: Uuid,
    pub company_name: String,
    pub scopes: Vec<Scope>,
}

/// Check the key: which company it belongs to and what it may do
#[utoipa::path(get, path = "/whoami", tag = TAG,
    responses(
        (status = 200, body = AtsWhoami),
        (status = 401, description = "`api_key_required`, `api_key_invalid`", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_whoami(
    State(state): State<AppState>,
    client: ApiClient,
) -> ApiResult<Json<AtsWhoami>> {
    let company = db::companies::company_of_owner(&state.pool, client.owner_user_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(AtsWhoami {
        company_id: company.id,
        company_name: company.name,
        scopes: client.scopes.to_vec(),
    }))
}

#[derive(Deserialize, IntoParams)]
pub struct PollQuery {
    pub status: Option<InvitationStatus>,
    /// RFC 3339; only invitations whose status changed after this moment. Pass `next_since` from
    /// the previous page.
    pub since: Option<DateTime<Utc>>,
    /// Default 50, at most 200.
    pub limit: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct AtsInvitationItem {
    pub invitation: AtsInvitation,
    /// Before acceptance — anonymous code and category; after — also `id`.
    pub candidate: AtsCandidate,
    /// The candidate accepted: `GET /integrations/candidates/{candidate.id}` returns contacts.
    pub contacts_available: bool,
    pub outcomes: Vec<HiringOutcome>,
}

#[derive(Serialize, ToSchema)]
pub struct AtsInvitationPage {
    pub items: Vec<AtsInvitationItem>,
    /// Cursor for the next call; `null` when the page is empty.
    pub next_since: Option<DateTime<Utc>>,
}

/// Polling alternative to webhooks: invitations whose status changed after `since`, oldest first
#[utoipa::path(get, path = "/invitations", tag = TAG,
    params(PollQuery),
    responses(
        (status = 200, body = AtsInvitationPage),
        (status = 401, description = "`api_key_required`, `api_key_invalid`", body = Problem),
        (status = 403, description = "`insufficient_scope` — нужно `invitations:read`", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_invitations(
    State(state): State<AppState>,
    client: ApiClient,
    Query(q): Query<PollQuery>,
) -> ApiResult<Json<AtsInvitationPage>> {
    client.require(Scope::InvitationsRead)?;
    expire_invitations(&state).await?;
    let rows = db::integrations::polled_invitations(
        &state.pool,
        client.company_id,
        q.status.map(Into::into),
        q.since,
        q.limit.unwrap_or(POLL_DEFAULT).clamp(1, POLL_MAX),
    )
    .await?;
    let ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let outcomes = OutcomeIndex::from_rows(
        db::integrations::subject_outcomes(&state.pool, client.company_id, &ids).await?,
    );
    let next_since = rows.last().map(|r| r.updated_at);
    let items = rows
        .into_iter()
        .map(|r| {
            let category = category_of(r.specialization.as_deref(), r.grade.as_deref());
            let accepted = r.status == db::types::InvitationStatus::Accepted;
            AtsInvitationItem {
                invitation: invitation_from_snapshot(&r),
                candidate: if accepted {
                    AtsCandidate::accepted(r.public_code.clone(), category, r.candidate_id)
                } else {
                    AtsCandidate::anonymous(r.public_code.clone(), category)
                },
                contacts_available: accepted,
                outcomes: outcomes.of(r.id),
            }
        })
        .collect();
    Ok(Json(AtsInvitationPage { items, next_since }))
}

#[derive(Serialize, ToSchema)]
pub struct AtsSkill {
    pub slug: String,
    pub name: String,
    /// Confirmed by the test or FSP achievements, not only declared.
    pub verified: bool,
}

/// Full profile after the candidate accepted an invitation or applied to the company.
#[derive(Serialize, ToSchema)]
pub struct AtsCandidateProfile {
    pub id: Uuid,
    pub code: String,
    pub category: Option<Category>,
    pub category_assigned_at: Option<DateTime<Utc>>,
    pub full_name: Option<String>,
    #[serde(flatten)]
    pub contacts: Contacts,
    pub basis: ContactBasis,
    pub basis_id: Uuid,
    pub city: Option<String>,
    pub relocation: bool,
    pub work_formats: Vec<WorkFormat>,
    pub salary_expectation_min: Option<i64>,
    pub experience_years: Option<f32>,
    pub skills: Vec<AtsSkill>,
    pub roles: Vec<String>,
    pub industries: Vec<Industry>,
    pub about: String,
    pub experience: Vec<ExperienceEntry>,
    pub profile_pdf_url: String,
}

async fn grant_for(
    state: &AppState,
    client: &ApiClient,
    candidate_id: Uuid,
) -> ApiResult<ContactGrant> {
    let ctx = AccessContext {
        employer_user_id: client.owner_user_id,
        company_id: client.company_id,
        ip: None,
        user_agent: Some("ATS API"),
        channel: AccessChannel::Api,
        api_key_id: Some(client.key_id),
    };
    let mut conn = state.pool.acquire().await?;
    ContactGrant::obtain(&mut conn, &ctx, candidate_id)
        .await?
        .ok_or_else(ApiError::not_found)
}

/// Candidate profile with contacts — only if this company holds a basis: an accepted invitation
/// or the candidate's own application. Otherwise 404. Every call is written to the access log
#[utoipa::path(get, path = "/candidates/{id}", tag = TAG,
    params(("id" = Uuid, Path, description = "`candidate.id` from an event or the invitations feed")),
    responses(
        (status = 200, body = AtsCandidateProfile),
        (status = 401, description = "`api_key_required`, `api_key_invalid`", body = Problem),
        (status = 403, description = "`insufficient_scope` — нужно `candidates:read`", body = Problem),
        (status = 404, description = "Нет принятого приглашения или отклика", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_candidate(
    State(state): State<AppState>,
    client: ApiClient,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AtsCandidateProfile>> {
    client.require(Scope::CandidatesRead)?;
    let grant = grant_for(&state, &client, id).await?;
    let (basis_id, candidate_id) = (grant.basis_id(), grant.candidate_id());
    let disclosed = CandidateContacts::reveal(&state.pool, &state.crypto, grant).await?;
    let p = load_profile_of(&state, candidate_id).await?;
    Ok(Json(AtsCandidateProfile {
        id: candidate_id,
        code: p.public_code,
        category: p.category,
        category_assigned_at: p.category_assigned_at,
        full_name: disclosed.full_name,
        contacts: disclosed.contacts,
        basis: disclosed.basis,
        basis_id,
        city: p.input.city,
        relocation: p.input.relocation,
        work_formats: p.input.work_formats,
        salary_expectation_min: p.input.salary_expectation_min,
        experience_years: p.input.experience_years,
        skills: p
            .skill_details
            .into_iter()
            .map(|s| AtsSkill {
                verified: s
                    .sources
                    .iter()
                    .any(|x| matches!(x, SkillSource::Test | SkillSource::Fsp)),
                slug: s.slug,
                name: s.name,
            })
            .collect(),
        roles: p.input.roles,
        industries: p.input.industries,
        about: p.input.about,
        experience: p.input.experience,
        profile_pdf_url: format!(
            "{}/api/v1/integrations/candidates/{candidate_id}/profile.pdf",
            state.config.public_url
        ),
    }))
}

/// Standardized PDF profile of the candidate, on the same terms as the profile
#[utoipa::path(get, path = "/candidates/{id}/profile.pdf", tag = TAG,
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, content_type = "application/pdf", body = Vec<u8>),
        (status = 401, body = Problem),
        (status = 403, description = "`insufficient_scope` — нужно `candidates:read`", body = Problem),
        (status = 404, description = "Нет принятого приглашения или отклика", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_candidate_pdf(
    State(state): State<AppState>,
    client: ApiClient,
    Path(id): Path<Uuid>,
) -> ApiResult<impl IntoResponse> {
    client.require(Scope::CandidatesRead)?;
    let grant = grant_for(&state, &client, id).await?;
    let (code, bytes) = profile_pdf_bytes(&state, grant.candidate_id()).await?;
    Ok(pdf_response(&code, bytes))
}

#[derive(Deserialize, ToSchema)]
pub struct OutcomeInput {
    /// Exactly one of `invitation_id` and `application_id`.
    pub invitation_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub stage: HiringStage,
    /// Shown to the candidate. Up to 500 characters.
    pub reason: Option<String>,
    /// When it happened in the ATS.
    pub at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct OutcomeRecorded {
    pub id: Uuid,
    pub invitation_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub stage: HiringStage,
    pub reason: Option<String>,
    pub at: DateTime<Utc>,
}

/// Report a hiring stage: interview, offer, hired or rejected. The candidate sees it in the
/// invitation or application timeline; repeating the same stage updates it (safe to retry)
#[utoipa::path(post, path = "/outcomes", tag = TAG,
    request_body = OutcomeInput,
    responses(
        (status = 201, description = "Этап записан", body = OutcomeRecorded),
        (status = 200, description = "Этап уже был — обновлены причина и время", body = OutcomeRecorded),
        (status = 401, body = Problem),
        (status = 403, description = "`insufficient_scope` — нужно `outcomes:write`", body = Problem),
        (status = 404, description = "Приглашение или отклик не найдены у этой компании", body = Problem),
        (status = 409, description = "`outcome_not_allowed` — кандидат не принял приглашение или отозвал отклик", body = Problem),
        (status = 422, body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_outcome(
    State(state): State<AppState>,
    client: ApiClient,
    Json(input): Json<OutcomeInput>,
) -> ApiResult<(StatusCode, Json<OutcomeRecorded>)> {
    client.require(Scope::OutcomesWrite)?;
    let reason = input
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    let now = Utc::now();
    let mut v = Validator::default();
    v.check(
        input.invitation_id.is_some() != input.application_id.is_some(),
        "invitation_id",
        "Укажите ровно одно: invitation_id или application_id",
    )
    .check(
        reason.is_none_or(|r| r.chars().count() <= 500),
        "reason",
        "Не длиннее 500 символов",
    )
    .check(
        input.at <= now + Duration::minutes(5),
        "at",
        "Время этапа в будущем",
    )
    .check(
        input.at >= now - Duration::days(365),
        "at",
        "Этап старше года",
    );
    v.finish()?;

    let subject = match (input.invitation_id, input.application_id) {
        (Some(id), _) => {
            db::integrations::invitation_outcome_subject(&state.pool, client.company_id, id).await?
        }
        (None, Some(id)) => {
            db::integrations::application_outcome_subject(&state.pool, client.company_id, id)
                .await?
        }
        (None, None) => None,
    }
    .ok_or_else(ApiError::not_found)?;
    if !subject.open {
        return Err(ApiError::conflict(
            "outcome_not_allowed",
            if input.invitation_id.is_some() {
                "Кандидат ещё не принял приглашение — этап найма отмечается только после принятия"
            } else {
                "Кандидат отозвал отклик"
            },
        ));
    }
    let stored = db::integrations::upsert_outcome(
        &state.pool,
        &db::integrations::NewOutcome {
            id: Uuid::now_v7(),
            company_id: client.company_id,
            candidate_id: subject.candidate_id,
            invitation_id: input.invitation_id,
            application_id: input.application_id,
            stage: input.stage.into(),
            reason,
            occurred_at: input.at,
            api_key_id: Some(client.key_id),
        },
    )
    .await?;
    let status = if stored.inserted {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        Json(OutcomeRecorded {
            id: stored.id,
            invitation_id: input.invitation_id,
            application_id: input.application_id,
            stage: input.stage,
            reason: reason.map(str::to_owned),
            at: input.at,
        }),
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct NeedImport {
    /// Vacancy title in the ATS; the first line of `text` if absent.
    pub title: Option<String>,
    /// Vacancy text as is: the parser finds specialization, grade, stack, salary and format.
    pub text: String,
    /// Overrides the salary range found in the text.
    pub salary: Option<SalaryRange>,
}

#[derive(Serialize, ToSchema)]
pub struct ImportedNeed {
    pub need_id: Uuid,
    /// Page of the need with the shortlist in the employer cabinet.
    pub shortlist_url: String,
    pub title: String,
    pub specialization: Specialization,
    pub grades: Vec<Grade>,
    pub must_skills: Vec<String>,
    pub nice_skills: Vec<String>,
    pub salary: SalaryRange,
    pub work_formats: Vec<WorkFormat>,
    pub city: Option<String>,
    /// What was assumed because the text did not say it.
    pub notes: Vec<String>,
}

fn truncate(s: &str, max_chars: usize) -> &str {
    s.char_indices().nth(max_chars).map_or(s, |(i, _)| &s[..i])
}

/// Import a vacancy from the ATS: the text is parsed into a hiring need; the shortlist is built
/// when the need is opened in the cabinet
#[utoipa::path(post, path = "/needs", tag = TAG,
    request_body = NeedImport,
    responses(
        (status = 201, body = ImportedNeed),
        (status = 401, body = Problem),
        (status = 403, description = "`insufficient_scope` — нужно `needs:write`", body = Problem),
        (status = 422, description = "Нет текста, не распознана специализация или вилка", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_import_need(
    State(state): State<AppState>,
    client: ApiClient,
    Json(input): Json<NeedImport>,
) -> ApiResult<(StatusCode, Json<ImportedNeed>)> {
    client.require(Scope::NeedsWrite)?;
    let text = input.text.trim();
    let title_in = input
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    if text.is_empty() || text.len() > MAX_IMPORT_TEXT {
        return Err(ApiError::field(
            "text",
            "Текст вакансии — от 1 до 20 000 символов",
        ));
    }
    let parsed = match title_in {
        Some(t) => matching::parse_need(&format!("{t}\n{text}")),
        None => matching::parse_need(text),
    };
    let Some(specialization) = parsed.specialization.as_ref().map(|g| g.specialization) else {
        return Err(ApiError::field(
            "text",
            "Не удалось определить специализацию — добавьте роль, например «Backend-разработчик на Go»",
        ));
    };
    let mut notes = Vec::new();
    let mut grades = if parsed.grades.is_empty() {
        parsed.grade.into_iter().collect::<Vec<_>>()
    } else {
        parsed.grades.clone()
    };
    if grades.is_empty() {
        grades = vec![Grade::Junior, Grade::Middle, Grade::Senior];
        notes.push("Грейд не указан в тексте — подборка по всем грейдам".to_owned());
    }
    let salary = match (&input.salary, parsed.salary_min, parsed.salary_max) {
        (Some(s), _, _) => s.clone(),
        (None, Some(min), Some(max)) => SalaryRange { min, max },
        (None, Some(v), None) | (None, None, Some(v)) => {
            notes.push("В тексте одна граница вилки — вторая взята равной ей".to_owned());
            SalaryRange { min: v, max: v }
        }
        (None, None, None) => {
            return Err(ApiError::field(
                "salary",
                "В тексте нет зарплатной вилки — передайте salary {min, max}: без неё приглашения не отправляются",
            ));
        }
    };
    let mut v = Validator::default();
    v.check(salary.min > 0, "salary.min", "Зарплата «от» обязательна")
        .check(
            salary.max >= salary.min,
            "salary.max",
            "«До» не меньше «от»",
        );
    v.finish()?;

    let title = truncate(
        title_in.unwrap_or_else(|| text.lines().next().unwrap_or(text).trim()),
        200,
    )
    .to_owned();
    let description = truncate(text, 10_000);
    let grade_slugs = slugs(&grades);
    let formats = slugs(&parsed.work_formats);
    let industries = slugs(&parsed.industries);
    let id = Uuid::now_v7();
    db::needs::insert_need(
        &state.pool,
        id,
        client.company_id,
        &db::needs::NeedFields {
            title: &title,
            description,
            specialization: specialization.slug(),
            grades: &grade_slugs,
            allow_adjacent_grades: false,
            must_skills: &parsed.must_skills,
            nice_skills: &parsed.nice_skills,
            salary_min: salary.min,
            salary_max: salary.max,
            work_formats: &formats,
            city: parsed.city.as_deref(),
            industries: &industries,
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(ImportedNeed {
            need_id: id,
            shortlist_url: format!("{}/employer/needs/{id}", state.config.public_url),
            title,
            specialization,
            grades,
            must_skills: parsed.must_skills,
            nice_skills: parsed.nice_skills,
            salary,
            work_formats: parsed.work_formats,
            city: parsed.city,
            notes,
        }),
    ))
}
