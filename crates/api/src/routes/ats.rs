//! Inbound API for employers' ATS, authenticated by company API keys only
//! (`Authorization: Bearer fsp_live_…`, see `integrations::keys`). Contacts and the full profile
//! go through `ContactGrant` exactly like in the cabinet, with channel `api` in the access log.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
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
    dto::{HiringStage, InvitationStatus, SalaryRange, parse_slugs, slugs},
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Created, Json, Path, Query},
    integrations::{
        events::{
            AtsCandidate, AtsInvitation, category_of, expire_invitations, invitation_from_snapshot,
        },
        keys::{ApiClient, Scope},
        outcomes::{HiringOutcome, OutcomeIndex},
    },
    notify::Notifier,
    page,
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
        (status = 404, description = "The key's company no longer exists", body = Problem),
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
    /// `next_cursor` of the previous call; without it the feed starts from the first invitation.
    pub cursor: Option<String>,
    /// Deprecated, use `cursor`. RFC 3339: invitations whose status changed after this moment.
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
    /// Pass as `cursor` on the next call. An empty page repeats the cursor it was asked with:
    /// keep polling with it. A full page (`limit` items) means more are waiting — call again at once.
    pub next_cursor: Option<String>,
    /// Deprecated: `updated_at` of the last item, for clients still on `since`; `null` when the
    /// page is empty.
    pub next_since: Option<DateTime<Utc>>,
}

/// Polling alternative to webhooks: invitations whose status changed after `cursor`, oldest
/// first. Changes of the last few seconds come on the next call, so a change still being saved is
/// never skipped
#[utoipa::path(get, path = "/invitations", tag = TAG,
    params(PollQuery),
    responses(
        (status = 200, body = AtsInvitationPage),
        (status = 400, description = "`invalid_query` — `cursor` не из ответа", body = Problem),
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
    let after = match page::decode(q.cursor.as_deref())? {
        Some(c) => Some(c),
        None => q.since.map(|at| db::types::Cursor {
            at,
            id: Uuid::max(),
        }),
    };
    expire_invitations(&state).await?;
    let rows = db::integrations::polled_invitations(
        &state.pool,
        client.company_id,
        q.status.map(Into::into),
        after,
        state.config.integrations.poll_lag.as_secs_f64(),
        page::limit(q.limit, POLL_DEFAULT, POLL_MAX),
    )
    .await?;
    let ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let outcomes = OutcomeIndex::from_rows(
        db::integrations::subject_outcomes(&state.pool, client.company_id, &ids).await?,
    );
    let next_since = rows.last().map(|r| r.updated_at);
    let next_cursor = match rows.last() {
        Some(r) => Some(page::encode(r.updated_at, r.id)),
        None => after.map(|c| page::encode(c.at, c.id)),
    };
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
    Ok(Json(AtsInvitationPage {
        items,
        next_cursor,
        next_since,
    }))
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
        (status = 401, description = "`api_key_required`, `api_key_invalid`", body = Problem),
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
#[schema(example = json!({"invitation_id": "019a8d12-77e0-7c3a-b5f1-4d2e9a6c0b18", "application_id": null, "stage": "interview", "reason": null, "at": "2026-10-06T11:30:00Z"}))]
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
        (status = 401, description = "`api_key_required`, `api_key_invalid`", body = Problem),
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
    let mut tx = state.pool.begin().await?;
    let stored = db::integrations::upsert_outcome(
        &mut *tx,
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
    if stored.inserted {
        Notifier::new(&state)
            .hiring_stage(
                &mut tx,
                input.invitation_id,
                input.application_id,
                input.stage,
                reason,
            )
            .await?;
    }
    tx.commit().await?;
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
#[schema(example = json!({
    "external_id": "hh-98765432",
    "title": "Senior Python-разработчик (ML-платформа)",
    "text": "Senior Python-разработчик в команду ML-платформы\nМосква, гибрид, от 350 000 до 450 000 ₽\nСтек: Python, FastAPI, PostgreSQL, Kafka, Docker, Kubernetes.\nБудет плюсом: Airflow, ClickHouse. Опыт от 5 лет.",
    "salary": {"min": 350000, "max": 450000}
}))]
pub struct NeedImport {
    /// The vacancy id in your ATS, up to 200 characters. A repeat with the same `external_id`
    /// creates nothing and returns the need imported first (200) — safe to retry.
    pub external_id: Option<String>,
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
    pub external_id: Option<String>,
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
/// when the need is opened in the cabinet. With `external_id` the call is safe to retry
#[utoipa::path(post, path = "/needs", tag = TAG,
    request_body = NeedImport,
    responses(
        (status = 201, body = ImportedNeed, headers(("Location" = String, description = "`/api/v1/employer/needs/{id}`"))),
        (status = 200, description = "Потребность с этим `external_id` уже импортирована — она и возвращается", body = ImportedNeed),
        (status = 401, description = "`api_key_required`, `api_key_invalid`", body = Problem),
        (status = 403, description = "`insufficient_scope` — нужно `needs:write`", body = Problem),
        (status = 422, description = "Нет текста, не распознана специализация или вилка; `external_id` длиннее 200 символов", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("api_key" = [])))]
async fn ats_import_need(
    State(state): State<AppState>,
    client: ApiClient,
    Json(input): Json<NeedImport>,
) -> ApiResult<Response> {
    client.require(Scope::NeedsWrite)?;
    let external_id = input
        .external_id
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty());
    if external_id.is_some_and(|e| e.chars().count() > 200) {
        return Err(ApiError::field("external_id", "Не длиннее 200 символов"));
    }
    if let Some(ext) = external_id
        && let Some(existing) = imported(&state, client.company_id, ext).await?
    {
        return Ok((StatusCode::OK, Json(existing)).into_response());
    }
    let text = input.text.trim();
    let title_in = input
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty());
    if text.is_empty() || text.chars().count() > MAX_IMPORT_TEXT {
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
    let inserted = db::needs::insert_imported_need(
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
        external_id,
    )
    .await?;
    if !inserted {
        // The same `external_id` arrived twice at once and the other request won.
        let existing = match external_id {
            Some(ext) => imported(&state, client.company_id, ext).await?,
            None => None,
        };
        return Ok((
            StatusCode::OK,
            Json(existing.ok_or(sqlx::Error::RowNotFound)?),
        )
            .into_response());
    }
    Ok(Created(
        format!("/api/v1/employer/needs/{id}"),
        ImportedNeed {
            need_id: id,
            external_id: external_id.map(str::to_owned),
            shortlist_url: shortlist_url(&state, id),
            title,
            specialization,
            grades,
            must_skills: parsed.must_skills,
            nice_skills: parsed.nice_skills,
            salary,
            work_formats: parsed.work_formats,
            city: parsed.city,
            notes,
        },
    )
    .into_response())
}

fn shortlist_url(state: &AppState, id: Uuid) -> String {
    format!("{}/employer/needs/{id}", state.config.public_url)
}

/// The need this company already imported under `external_id`, as it is stored now.
async fn imported(
    state: &AppState,
    company: Uuid,
    external_id: &str,
) -> ApiResult<Option<ImportedNeed>> {
    let Some(id) = db::needs::need_by_external_id(&state.pool, company, external_id).await? else {
        return Ok(None);
    };
    let n = db::needs::company_need(&state.pool, company, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok(Some(ImportedNeed {
        need_id: n.id,
        external_id: Some(external_id.to_owned()),
        shortlist_url: shortlist_url(state, n.id),
        title: n.title,
        specialization: n.specialization.parse()?,
        grades: parse_slugs(&n.grades),
        must_skills: n.must_skills,
        nice_skills: n.nice_skills,
        salary: SalaryRange {
            min: n.salary_min,
            max: n.salary_max,
        },
        work_formats: parse_slugs(&n.work_formats),
        city: n.city,
        notes: Vec::new(),
    }))
}
