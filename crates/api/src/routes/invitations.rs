//! The core contact mechanic (architecture.md §6): the employer invites, the candidate sees the
//! salary range before any conversation, contacts open only after acceptance.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
use catalog::{Category, WorkFormat};
use chrono::{DateTime, Duration, Utc};
use matching::trust::{MAX_QUOTA, TrustLevel, display_domain};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::{CandidateUser, EmployerUser, client_ip, user_agent},
    contacts::{AccessChannel, AccessContext, CandidateContacts, ContactGrant},
    dto::{DeclineReason, HiringStage, InvitationStatus, SalaryRange},
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Json, Path},
    integrations::{
        events::{EventType, Outbox, expire_invitations},
        outcomes::{HiringOutcome, OutcomeIndex},
    },
    notify::{NotificationKind, Notifier},
    state::AppState,
    trust,
};

pub const TTL_DAYS: i64 = 14;

pub fn employer_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_sent, send))
        .routes(routes!(invitation_stats))
        .routes(routes!(get_sent))
        .routes(routes!(withdraw))
        .routes(routes!(candidate_contacts))
}

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_received))
        .routes(routes!(get_received))
        .routes(routes!(accept))
        .routes(routes!(decline))
}

#[derive(Deserialize, ToSchema)]
pub struct SendInvitation {
    pub candidate_id: Uuid,
    pub need_id: Option<Uuid>,
    pub vacancy_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub salary: SalaryRange,
    pub work_format: Option<WorkFormat>,
    /// How the candidate can reach the company (shown in the invitation).
    pub contact: String,
}

#[derive(Serialize, ToSchema)]
pub struct StatusEvent {
    pub status: InvitationStatus,
    pub at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct SentInvitation {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub candidate_code: String,
    pub candidate_category: Option<Category>,
    pub need_id: Option<Uuid>,
    pub vacancy_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub salary: SalaryRange,
    pub work_format: Option<WorkFormat>,
    pub contact: String,
    pub status: InvitationStatus,
    pub decline_reason: Option<DeclineReason>,
    pub decline_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<StatusEvent>>,
}

macro_rules! sent_from_row {
    ($r:expr) => {{
        let r = $r;
        SentInvitation {
            id: r.id,
            candidate_id: r.candidate_id,
            candidate_code: r.public_code,
            candidate_category: match (r.specialization.as_deref(), r.grade.as_deref()) {
                (Some(s), Some(g)) => Some(Category::new(s.parse()?, g.parse()?)),
                _ => None,
            },
            need_id: r.need_id,
            vacancy_id: r.vacancy_id,
            title: r.title,
            message: r.message,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            work_format: r.work_format.as_deref().and_then(WorkFormat::from_slug),
            contact: r.contact,
            status: r.status.into(),
            decline_reason: r
                .decline_reason
                .as_deref()
                .and_then(DeclineReason::from_slug),
            decline_comment: r.decline_comment,
            created_at: r.created_at,
            viewed_at: r.viewed_at,
            responded_at: r.responded_at,
            expires_at: r.expires_at,
            events: None,
        }
    }};
}

fn validate(req: &SendInvitation) -> ApiResult<()> {
    let mut v = Validator::default();
    v.check(
        !req.title.trim().is_empty(),
        "title",
        "Укажите, что предлагаете",
    )
    .check(req.title.len() <= 200, "title", "Не длиннее 200 символов")
    .check(
        !req.message.trim().is_empty(),
        "message",
        "Опишите предложение",
    )
    .check(
        req.message.len() <= 4000,
        "message",
        "Не длиннее 4000 символов",
    )
    .check(
        req.salary.min > 0,
        "salary.min",
        "Зарплата «от» обязательна",
    )
    .check(
        req.salary.max >= req.salary.min,
        "salary.max",
        "«До» не меньше «от»",
    )
    .check(
        req.salary.max <= 100_000_000,
        "salary.max",
        "Проверьте сумму",
    )
    .check(
        !req.contact.trim().is_empty(),
        "contact",
        "Укажите способ связи",
    );
    v.finish()
}

/// Send an invitation to a candidate (salary range is mandatory, vacancy is optional)
#[utoipa::path(post, path = "/invitations", tag = "invitations",
    request_body = SendInvitation,
    params(("Idempotency-Key" = Option<String>, Header, description = "Повтор с тем же ключом вернёт уже созданное приглашение")),
    responses(
        (status = 201, body = SentInvitation),
        (status = 200, description = "Повтор по Idempotency-Key", body = SentInvitation),
        (status = 404, description = "Кандидат не найден или скрыт", body = Problem),
        (status = 409, description = "`invitation_quota_exceeded`, `invitation_already_active`, `candidate_not_available`; `company_profile_required`", body = Problem),
        (status = 422, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn send(
    State(state): State<AppState>,
    user: EmployerUser,
    headers: HeaderMap,
    Json(req): Json<SendInvitation>,
) -> ApiResult<(StatusCode, Json<SentInvitation>)> {
    validate(&req)?;
    let company = company_id(&state, user.id).await?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.chars().take(100).collect::<String>());

    if let Some(key) = idempotency_key.as_deref()
        && let Some(existing) =
            db::invitations::invitation_by_idempotency_key(&state.pool, company, key).await?
    {
        let row = db::invitations::employer_invitation(&state.pool, company, existing)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        return Ok((StatusCode::OK, Json(sent_from_row!(row))));
    }

    let candidate = db::candidates::candidate_invitable(&state.pool, req.candidate_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !candidate.published {
        return Err(ApiError::not_found());
    }
    let category = match (
        candidate.specialization.as_deref(),
        candidate.grade.as_deref(),
    ) {
        (Some(s), Some(g)) if candidate.search_status != db::types::SearchStatus::Closed => {
            Category::new(s.parse()?, g.parse()?)
        }
        _ => {
            return Err(ApiError::conflict(
                "candidate_not_available",
                "Кандидат сейчас не принимает приглашения",
            ));
        }
    };

    let quota = trust::weekly_quota(&state.pool, company).await?;
    let since = Utc::now() - Duration::days(7);
    let sent = db::invitations::company_invitations_since(&state.pool, company, since).await?;
    if sent >= quota {
        return Err(ApiError::conflict(
            "invitation_quota_exceeded",
            format!("Лимит {quota} приглашений за 7 дней исчерпан"),
        )
        .with_detail(
            "Квота зависит от уровня доверия к компании — как её поднять, видно в разделе «Доверие и квота»",
        ));
    }

    let id = Uuid::now_v7();
    let mut tx = state.pool.begin().await?;
    let inserted = db::invitations::insert_invitation(
        &mut *tx,
        &db::invitations::NewInvitation {
            id,
            company_id: company,
            candidate_id: req.candidate_id,
            need_id: req.need_id,
            vacancy_id: req.vacancy_id,
            title: req.title.trim(),
            message: req.message.trim(),
            salary_min: req.salary.min,
            salary_max: req.salary.max,
            work_format: req.work_format.map(|f| f.slug()),
            contact: req.contact.trim(),
            idempotency_key: idempotency_key.as_deref(),
            expires_at: Utc::now() + Duration::days(TTL_DAYS),
        },
    )
    .await;
    if let Err(e) = inserted {
        if db::is_unique_violation(&e) {
            return Err(ApiError::conflict(
                "invitation_already_active",
                "Этому кандидату уже отправлено действующее приглашение",
            ));
        }
        return Err(e.into());
    }
    db::invitations::insert_invitation_event(
        &mut *tx,
        Uuid::now_v7(),
        id,
        db::types::InvitationStatus::Sent,
        Some(user.id),
    )
    .await?;
    Outbox::new(&state)
        .invitation(&mut tx, EventType::InvitationSent, id)
        .await?;
    Notifier::new(&state)
        .invitation(&mut tx, NotificationKind::InvitationReceived, id)
        .await?;
    tx.commit().await?;
    trust::flag_invitation(&state.pool, id, category, (req.salary.min, req.salary.max)).await?;
    let row = db::invitations::employer_invitation(&state.pool, company, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok((StatusCode::CREATED, Json(sent_from_row!(row))))
}

/// Invitations sent by the company
#[utoipa::path(get, path = "/invitations", tag = "invitations",
    responses((status = 200, body = Vec<SentInvitation>), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_sent(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<SentInvitation>>> {
    let company = company_id(&state, user.id).await?;
    expire_invitations(&state).await?;
    Ok(Json(sent_views(&state.pool, company).await?))
}

pub(crate) async fn sent_views(pool: &PgPool, company: Uuid) -> ApiResult<Vec<SentInvitation>> {
    let rows = db::invitations::employer_invitations(pool, company).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(sent_from_row!(row));
    }
    Ok(out)
}

/// One sent invitation with its status history
#[utoipa::path(get, path = "/invitations/{id}", tag = "invitations",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = SentInvitation), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_sent(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<SentInvitation>> {
    let company = company_id(&state, user.id).await?;
    expire_invitations(&state).await?;
    let row = db::invitations::employer_invitation(&state.pool, company, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let mut inv = sent_from_row!(row);
    inv.events = Some(events(&state.pool, id).await?);
    db::notifications::mark_subject_read(&state.pool, user.id, id).await?;
    Ok(Json(inv))
}

async fn events(pool: &PgPool, id: Uuid) -> ApiResult<Vec<StatusEvent>> {
    Ok(db::invitations::invitation_events(pool, id)
        .await?
        .into_iter()
        .map(|e| StatusEvent {
            status: e.status.into(),
            at: e.at,
        })
        .collect())
}

/// Withdraw an invitation that has not been answered yet
#[utoipa::path(post, path = "/invitations/{id}/withdraw", tag = "invitations",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 409, description = "`invitation_closed`; `company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn withdraw(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    let mut tx = state.pool.begin().await?;
    if db::invitations::withdraw_invitation(&mut *tx, id, company).await? == 0 {
        return Err(ApiError::conflict(
            "invitation_closed",
            "Приглашение уже закрыто",
        ));
    }
    db::invitations::insert_invitation_event(
        &mut *tx,
        Uuid::now_v7(),
        id,
        db::types::InvitationStatus::Withdrawn,
        Some(user.id),
    )
    .await?;
    Outbox::new(&state)
        .invitation(&mut tx, EventType::InvitationWithdrawn, id)
        .await?;
    Notifier::new(&state)
        .invitation(&mut tx, NotificationKind::InvitationWithdrawn, id)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, ToSchema)]
pub struct StatusCount {
    pub status: InvitationStatus,
    pub count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct ReasonCount {
    pub reason: Option<DeclineReason>,
    pub count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct InvitationStats {
    /// Weekly quota set by the company's trust level (`GET /employer/trust`).
    pub quota_limit: i64,
    /// The quota a trusted company gets.
    pub quota_max: i64,
    pub trust_level: TrustLevel,
    pub quota_used: i64,
    pub quota_resets_in_days: i64,
    pub by_status: Vec<StatusCount>,
    /// Share of answered (accepted or declined) among non-withdrawn invitations.
    pub response_rate: f64,
    pub decline_reasons: Vec<ReasonCount>,
    /// Hiring stages reported by the company's ATS (`POST /integrations/outcomes`): how many
    /// invitations and applications reached each one.
    pub hiring: Vec<StageCount>,
}

#[derive(Serialize, ToSchema)]
pub struct StageCount {
    pub stage: HiringStage,
    pub count: i64,
}

/// Quota, funnel and aggregated decline reasons
#[utoipa::path(get, path = "/invitations/stats", tag = "invitations",
    responses((status = 200, body = InvitationStats), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn invitation_stats(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<InvitationStats>> {
    let company = company_id(&state, user.id).await?;
    expire_invitations(&state).await?;
    let used = db::invitations::company_invitations_since(
        &state.pool,
        company,
        Utc::now() - Duration::days(7),
    )
    .await?;
    let rate = db::invitations::company_response_rate(&state.pool, company).await?;
    let (trust, _) = trust::company_assessment(&state.pool, company).await?;
    Ok(Json(InvitationStats {
        quota_limit: i64::from(trust.quota),
        quota_max: i64::from(MAX_QUOTA),
        trust_level: trust.level,
        quota_used: used,
        quota_resets_in_days: 7,
        by_status: db::invitations::company_invitation_status_counts(&state.pool, company)
            .await?
            .into_iter()
            .map(|r| StatusCount {
                status: r.status.into(),
                count: r.count,
            })
            .collect(),
        response_rate: if rate.total > 0 {
            rate.answered as f64 / rate.total as f64
        } else {
            0.0
        },
        decline_reasons: db::invitations::company_decline_reasons(&state.pool, company)
            .await?
            .into_iter()
            .map(|r| ReasonCount {
                reason: r.reason.as_deref().and_then(DeclineReason::from_slug),
                count: r.count,
            })
            .collect(),
        hiring: db::integrations::company_stage_counts(&state.pool, company)
            .await?
            .into_iter()
            .map(|r| StageCount {
                stage: r.stage.into(),
                count: r.count,
            })
            .collect(),
    }))
}

/// Candidate contacts: only after the candidate accepted an invitation or applied to a vacancy.
/// Every successful call is written to the access log.
#[utoipa::path(get, path = "/candidates/{id}/contacts", tag = "invitations",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = CandidateContacts), (status = 403, description = "`contacts_not_granted`", body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn candidate_contacts(
    State(state): State<AppState>,
    user: EmployerUser,
    headers: HeaderMap,
    Path(candidate_id): Path<Uuid>,
) -> ApiResult<Json<CandidateContacts>> {
    let company = company_id(&state, user.id).await?;
    let ip = client_ip(&headers);
    let ua = user_agent(&headers);
    let ctx = AccessContext {
        employer_user_id: user.id,
        company_id: company,
        ip: ip.as_deref(),
        user_agent: ua.as_deref(),
        channel: AccessChannel::Cabinet,
        api_key_id: None,
    };
    let mut tx = state.pool.begin().await?;
    let grant = ContactGrant::obtain(&mut tx, &ctx, candidate_id)
        .await?
        .ok_or_else(|| {
            ApiError::forbidden(
                "contacts_not_granted",
                "Контакты откроются, когда кандидат примет приглашение или откликнется сам",
            )
        })?;
    Outbox::new(&state)
        .contacts_opened(&mut tx, company, &grant)
        .await?;
    let contacts = CandidateContacts::reveal(&mut *tx, &state.crypto, grant).await?;
    tx.commit().await?;
    Ok(Json(contacts))
}

#[derive(Serialize, ToSchema)]
pub struct ReceivedInvitation {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_description: String,
    pub company_industry: Option<String>,
    pub company_city: Option<String>,
    pub company_website: Option<String>,
    /// «Домен подтверждён»: the company's login email and website share this domain.
    pub company_verified_domain: Option<String>,
    pub vacancy_id: Option<Uuid>,
    pub title: String,
    pub message: String,
    pub salary: SalaryRange,
    pub work_format: Option<WorkFormat>,
    pub contact: String,
    pub status: InvitationStatus,
    pub decline_reason: Option<DeclineReason>,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    /// Stages the company's hiring system reported after acceptance.
    pub outcomes: Vec<HiringOutcome>,
}

macro_rules! received_from_row {
    ($r:expr) => {{
        let r = $r;
        ReceivedInvitation {
            id: r.id,
            company_id: r.company_id,
            company_name: r.company_name,
            company_description: r.company_description,
            company_industry: r.company_industry,
            company_city: r.company_city,
            company_website: r.company_website,
            company_verified_domain: r.company_verified_domain.as_deref().map(display_domain),
            vacancy_id: r.vacancy_id,
            title: r.title,
            message: r.message,
            salary: SalaryRange {
                min: r.salary_min,
                max: r.salary_max,
            },
            work_format: r.work_format.as_deref().and_then(WorkFormat::from_slug),
            contact: r.contact,
            status: r.status.into(),
            decline_reason: r
                .decline_reason
                .as_deref()
                .and_then(DeclineReason::from_slug),
            created_at: r.created_at,
            viewed_at: r.viewed_at,
            responded_at: r.responded_at,
            expires_at: r.expires_at,
            outcomes: Vec::new(),
        }
    }};
}

async fn received_view(
    pool: &PgPool,
    candidate: Uuid,
    row: db::invitations::CandidateInvitation,
) -> ApiResult<ReceivedInvitation> {
    let mut inv = received_from_row!(row);
    inv.outcomes = OutcomeIndex::for_candidate(pool, candidate)
        .await?
        .of(inv.id);
    Ok(inv)
}

/// Incoming invitations
#[utoipa::path(get, path = "/invitations", tag = "invitations",
    responses((status = 200, body = Vec<ReceivedInvitation>)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_received(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<Vec<ReceivedInvitation>>> {
    expire_invitations(&state).await?;
    Ok(Json(received_views(&state.pool, user.id).await?))
}

pub(crate) async fn received_views(
    pool: &PgPool,
    candidate: Uuid,
) -> ApiResult<Vec<ReceivedInvitation>> {
    let rows = db::invitations::candidate_invitations(pool, candidate).await?;
    let outcomes = OutcomeIndex::for_candidate(pool, candidate).await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let mut inv = received_from_row!(r);
            inv.outcomes = outcomes.of(inv.id);
            inv
        })
        .collect())
}

/// One invitation; the first open marks it as viewed
#[utoipa::path(get, path = "/invitations/{id}", tag = "invitations",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = ReceivedInvitation), (status = 404, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn get_received(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ReceivedInvitation>> {
    expire_invitations(&state).await?;
    let mut tx = state.pool.begin().await?;
    if db::invitations::mark_invitation_viewed(&mut *tx, id, user.id).await? > 0 {
        db::invitations::insert_invitation_event(
            &mut *tx,
            Uuid::now_v7(),
            id,
            db::types::InvitationStatus::Viewed,
            Some(user.id),
        )
        .await?;
        Outbox::new(&state)
            .invitation(&mut tx, EventType::InvitationViewed, id)
            .await?;
    }
    db::notifications::mark_subject_read(&mut *tx, user.id, id).await?;
    tx.commit().await?;
    let row = db::invitations::candidate_invitation(&state.pool, user.id, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(received_view(&state.pool, user.id, row).await?))
}

async fn respond(
    state: &AppState,
    user: CandidateUser,
    id: Uuid,
    status: db::types::InvitationStatus,
    reason: Option<DeclineReason>,
    comment: Option<String>,
) -> ApiResult<Json<ReceivedInvitation>> {
    let mut tx = state.pool.begin().await?;
    let updated = db::invitations::respond_invitation(
        &mut *tx,
        status,
        reason.map(DeclineReason::slug),
        comment.as_deref(),
        id,
        user.id,
    )
    .await?;
    if updated == 0 {
        drop(tx);
        return match db::invitations::candidate_invitation(&state.pool, user.id, id).await? {
            None => Err(ApiError::not_found()),
            Some(_) => Err(ApiError::conflict(
                "invitation_closed",
                "Приглашение уже закрыто или истекло",
            )),
        };
    }
    db::invitations::insert_invitation_event(&mut *tx, Uuid::now_v7(), id, status, Some(user.id))
        .await?;
    db::candidates::touch_candidate_activity(&mut *tx, user.id).await?;
    let (kind, note) = if status == db::types::InvitationStatus::Accepted {
        (
            EventType::InvitationAccepted,
            NotificationKind::InvitationAccepted,
        )
    } else {
        (
            EventType::InvitationDeclined,
            NotificationKind::InvitationDeclined,
        )
    };
    Outbox::new(state).invitation(&mut tx, kind, id).await?;
    Notifier::new(state).invitation(&mut tx, note, id).await?;
    tx.commit().await?;
    let row = db::invitations::candidate_invitation(&state.pool, user.id, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok(Json(received_view(&state.pool, user.id, row).await?))
}

/// Accept: the company gets access to your contacts. Under 18 this needs the legal
/// representative's consent in force (it may have been revoked after the invitation came)
#[utoipa::path(post, path = "/invitations/{id}/accept", tag = "invitations",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = ReceivedInvitation), (status = 409, description = "`invitation_closed`; `guardian_consent_required`", body = Problem), (status = 404, description = "Not the caller's invitation", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn accept(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ReceivedInvitation>> {
    if let Some(doc) = db::guardians::missing_guardian_consent(&state.pool, user.id).await? {
        return Err(ApiError::GuardianConsentRequired {
            document_id: doc.id,
        });
    }
    respond(
        &state,
        user,
        id,
        db::types::InvitationStatus::Accepted,
        None,
        None,
    )
    .await
}

#[derive(Deserialize, ToSchema)]
pub struct DeclineRequest {
    pub reason: DeclineReason,
    pub comment: Option<String>,
}

/// Decline with a reason (aggregated reasons are shown to the employer)
#[utoipa::path(post, path = "/invitations/{id}/decline", tag = "invitations",
    params(("id" = Uuid, Path)),
    request_body = DeclineRequest,
    responses((status = 200, body = ReceivedInvitation), (status = 409, description = "`invitation_closed`", body = Problem), (status = 404, description = "Not the caller's invitation", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn decline(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
    Json(req): Json<DeclineRequest>,
) -> ApiResult<Json<ReceivedInvitation>> {
    let comment = req
        .comment
        .map(|c| c.chars().take(1000).collect::<String>())
        .filter(|c| !c.trim().is_empty());
    let declined = respond(
        &state,
        user,
        id,
        db::types::InvitationStatus::Declined,
        Some(req.reason),
        comment,
    )
    .await?;
    if req.reason == DeclineReason::NotLooking {
        db::candidates::set_search_status(&state.pool, db::types::SearchStatus::Closed, user.id)
            .await?;
        state.candidates.invalidate().await;
    }
    Ok(declined)
}
