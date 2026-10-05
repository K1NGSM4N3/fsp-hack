//! Right of access (ст. 14 152-ФЗ): everything the platform stores about the caller as one JSON
//! file. Built from the same views the cabinet shows. Task internals (templates, seeds, answers)
//! and the video itself stay out; the proctoring summary goes in.

use assessment::AttemptKind;
use axum::{
    extract::State,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use catalog::{Grade, Specialization};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    assessment::{GradeHistoryEntry, grade_history_views, parse_kind},
    candidate_profile::{Privacy, ProfileResponse, access_log, load_profile},
    company::{CompanyResponse, company_view},
    fsp::{FspStatus, load_status},
    invitations::{ReceivedInvitation, SentInvitation, received_views, sent_views},
    needs::{HiringNeed, company_need_views},
    survey::{SurveyResponse, survey_views},
    tasks::{EmployerTask, MyTask, company_task_views, load_my_tasks},
    vacancies::{MyApplication, Vacancy, company_vacancy_views, my_apps},
};
use crate::{
    age::{self, AgeBracket},
    auth::{AuthUser, CandidateUser, Role},
    contacts::ContactAccessEntry,
    dto::AssessmentStatus,
    error::{ApiError, ApiResult, Problem},
    guardian::{self, GuardianState},
    proctoring::{self, ProctoringSummary},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(export))
}

pub const FORMAT: &str = "fsp-talents-export/1";

#[derive(Serialize, ToSchema)]
pub struct DataExport {
    /// File format; a new number means an incompatible change.
    pub format: &'static str,
    pub generated_at: DateTime<Utc>,
    pub account: AccountRecord,
    /// Every grant and revocation: document version, time, IP and User-Agent.
    pub consents: Vec<ConsentRecord>,
    pub candidate: Option<CandidateData>,
    pub employer: Option<EmployerData>,
}

#[derive(Serialize, ToSchema)]
pub struct AccountRecord {
    pub id: Uuid,
    pub role: Role,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct ConsentRecord {
    pub id: Uuid,
    pub document_id: Uuid,
    pub kind: String,
    pub version: i32,
    pub title: String,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    /// For `guardian_consent`: the legal representative's IP and browser, until erased.
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    /// The age declared with this grant at registration.
    pub age_bracket: Option<AgeBracket>,
}

#[derive(Serialize, ToSchema)]
pub struct CandidateData {
    pub age: AgeRecord,
    pub profile: ProfileResponse,
    pub privacy: Privacy,
    pub surveys: Vec<SurveyResponse>,
    pub assessments: Vec<AssessmentRecord>,
    pub category_history: Vec<GradeHistoryEntry>,
    pub invitations: Vec<ReceivedInvitation>,
    pub applications: Vec<MyApplication>,
    /// Which companies received your contacts and on what basis.
    pub contact_access: Vec<ContactAccessEntry>,
    pub fsp: FspStatus,
    pub tasks: Vec<MyTask>,
    pub resume_uploads: Vec<ResumeUploadRecord>,
}

#[derive(Serialize, ToSchema)]
pub struct AgeRecord {
    /// Today, derived from the birth date; adults gave no birth date, only the declaration
    /// recorded with their consents.
    pub bracket: AgeBracket,
    /// Only for those who registered as 14–17; erased 30 days after the 18th birthday.
    pub birth_date: Option<NaiveDate>,
    pub adult_from: Option<NaiveDate>,
    /// Requests to legal representatives and their answers, newest first.
    pub guardian_requests: Vec<GuardianRecord>,
}

#[derive(Serialize, ToSchema)]
pub struct GuardianRecord {
    /// `None` once erased (30 days after the 18th birthday).
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub state: GuardianState,
    pub requested_at: DateTime<Utc>,
    pub last_letter_at: Option<DateTime<Utc>>,
    pub answered_at: Option<DateTime<Utc>>,
    /// The `guardian_consent` grant in `consents`, when given.
    pub consent_id: Option<Uuid>,
    pub granted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub erased_at: Option<DateTime<Utc>>,
}

async fn age_record(state: &AppState, candidate: Uuid) -> ApiResult<AgeRecord> {
    let age = db::guardians::candidate_age(&state.pool, candidate)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let now = Utc::now();
    let decrypt = |b: Option<Vec<u8>>| b.map(|b| state.crypto.decrypt_str(&b)).transpose();
    let mut guardian_requests = Vec::new();
    for r in db::guardians::requests(&state.pool, candidate).await? {
        guardian_requests.push(GuardianRecord {
            state: guardian::request_state(&r, now),
            full_name: decrypt(r.full_name_enc)?,
            email: decrypt(r.email_enc)?,
            requested_at: r.created_at,
            last_letter_at: r.sent_at,
            answered_at: r.answered_at,
            consent_id: r.consent_id,
            granted_at: r.granted_at,
            revoked_at: r.revoked_at,
            erased_at: r.erased_at,
        });
    }
    Ok(AgeRecord {
        bracket: if age.minor {
            AgeBracket::Minor
        } else {
            AgeBracket::Adult
        },
        birth_date: age.birth_date,
        adult_from: age.birth_date.map(age::adult_from),
        guardian_requests,
    })
}

/// One test session: the outcome without the tasks shown or the answers given.
#[derive(Serialize, ToSchema)]
pub struct AssessmentRecord {
    pub id: Uuid,
    pub specialization: Specialization,
    pub claimed_grade: Grade,
    pub attempt_kind: AttemptKind,
    pub status: AssessmentStatus,
    pub theta: Option<f64>,
    pub se: Option<f64>,
    pub measured_grade: Option<Grade>,
    pub items_count: i32,
    /// This test assigned the category.
    pub assigned: bool,
    /// Person-fit statistic `l_z` and whether it flagged an aberrant pattern.
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    /// Video recording summary and integrity events; the video is not part of the export.
    pub proctoring: Option<ProctoringSummary>,
}

#[derive(Serialize, ToSchema)]
pub struct ResumeUploadRecord {
    pub filename: String,
    pub size_bytes: i32,
    pub recognized_fields: Vec<String>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct EmployerData {
    pub company: Option<CompanyResponse>,
    pub needs: Vec<HiringNeed>,
    pub vacancies: Vec<Vacancy>,
    pub invitations: Vec<SentInvitation>,
    pub tasks: Vec<EmployerTask>,
}

async fn assessments(state: &AppState, candidate: Uuid) -> ApiResult<Vec<AssessmentRecord>> {
    let rows = db::assessment::candidate_sessions(&state.pool, candidate).await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let status = AssessmentStatus::from(r.status);
        let proctoring = db::proctoring::summary(&state.pool, r.id, Some(candidate))
            .await?
            .map(|p| {
                let facts = proctoring::facts(&p, status == AssessmentStatus::InProgress);
                proctoring::summary_view(&p, &facts, state.config.proctoring.retention_days)
            });
        out.push(AssessmentRecord {
            id: r.id,
            specialization: r.specialization.parse()?,
            claimed_grade: r.claimed_grade.parse()?,
            attempt_kind: parse_kind(&r.attempt_kind),
            status,
            theta: r.theta,
            se: r.se,
            measured_grade: r.measured_grade.as_deref().and_then(Grade::from_slug),
            items_count: r.items_count,
            assigned: r.assigned,
            person_fit_lz: r.person_fit_lz,
            aberrant: r.aberrant,
            started_at: r.started_at,
            finished_at: r.finished_at,
            proctoring,
        });
    }
    Ok(out)
}

async fn candidate_data(state: &AppState, user: AuthUser) -> ApiResult<CandidateData> {
    let pool = &state.pool;
    let c = db::candidates::candidate_profile(pool, user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok(CandidateData {
        age: age_record(state, user.id).await?,
        profile: load_profile(state, &CandidateUser(user)).await?,
        privacy: Privacy {
            visible: c.visible,
            show_name: c.show_name,
            show_fsp: c.show_fsp,
            show_experience: c.show_experience,
            search_status: c.search_status.into(),
        },
        surveys: survey_views(pool, user.id).await?,
        assessments: assessments(state, user.id).await?,
        category_history: grade_history_views(pool, user.id).await?,
        invitations: received_views(pool, user.id).await?,
        applications: my_apps(pool, user.id).await?,
        contact_access: access_log(pool, user.id).await?,
        fsp: load_status(pool, user.id).await?,
        tasks: load_my_tasks(pool, user.id).await?,
        resume_uploads: db::candidates::resume_uploads(pool, user.id)
            .await?
            .into_iter()
            .map(|r| ResumeUploadRecord {
                filename: r.filename,
                size_bytes: r.size_bytes,
                recognized_fields: r.recognized_fields,
                uploaded_at: r.created_at,
            })
            .collect(),
    })
}

async fn employer_data(state: &AppState, owner: Uuid) -> ApiResult<EmployerData> {
    let pool = &state.pool;
    let company = company_view(pool, owner).await?;
    let Some(id) = company.as_ref().map(|c| c.id) else {
        return Ok(EmployerData {
            company,
            needs: Vec::new(),
            vacancies: Vec::new(),
            invitations: Vec::new(),
            tasks: Vec::new(),
        });
    };
    Ok(EmployerData {
        needs: company_need_views(pool, id).await?,
        vacancies: company_vacancy_views(pool, id).await?,
        invitations: sent_views(pool, id).await?,
        tasks: company_task_views(pool, id).await?,
        company,
    })
}

pub async fn build(state: &AppState, user: AuthUser, now: DateTime<Utc>) -> ApiResult<DataExport> {
    let account = db::users::account_record(&state.pool, user.id)
        .await?
        .ok_or_else(ApiError::unauthorized)?;
    let consents = db::consents::user_consents(&state.pool, user.id)
        .await?
        .into_iter()
        .map(|r| ConsentRecord {
            id: r.id,
            document_id: r.document_id,
            kind: r.kind,
            version: r.version,
            title: r.title,
            granted_at: r.granted_at,
            revoked_at: r.revoked_at,
            ip: r.ip,
            user_agent: r.user_agent,
            age_bracket: r.age_bracket.map(AgeBracket::from),
        })
        .collect();
    let (candidate, employer) = match user.role {
        Role::Candidate => (Some(candidate_data(state, user).await?), None),
        Role::Employer => (None, Some(employer_data(state, user.id).await?)),
        Role::Admin => (None, None),
    };
    Ok(DataExport {
        format: FORMAT,
        generated_at: now,
        account: AccountRecord {
            id: account.id,
            role: account.role.into(),
            email: state.crypto.decrypt_str(&account.email_enc)?,
            email_verified_at: account.email_verified_at,
            created_at: account.created_at,
            last_login_at: account.last_login_at,
        },
        consents,
        candidate,
        employer,
    })
}

/// Download everything the platform stores about you as a JSON file (ст. 14 152-ФЗ): account,
/// consents with the declared age, profile, age (birth date and legal representative for 14–17),
/// tests with summaries, category history, invitations, applications, who opened your contacts,
/// proctoring summaries; for an employer — the account and the company's data
#[utoipa::path(get, path = "/export", tag = "account",
    responses(
        (status = 200, description = "JSON attachment `fsp-talents-<date>.json`", body = DataExport, content_type = "application/json",
            headers(("Content-Disposition" = String, description = "attachment; filename=\"fsp-talents-<date>.json\""))),
        (status = 401, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn export(State(state): State<AppState>, user: AuthUser) -> ApiResult<Response> {
    let now = Utc::now();
    let data = build(&state, user, now).await?;
    let body = serde_json::to_vec_pretty(&data)?;
    let disposition = format!(
        "attachment; filename=\"fsp-talents-{}.json\"",
        now.format("%Y-%m-%d")
    );
    Ok((
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json; charset=utf-8"),
            ),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&disposition)
                    .unwrap_or(HeaderValue::from_static("attachment")),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, no-store"),
            ),
        ],
        body,
    )
        .into_response())
}
