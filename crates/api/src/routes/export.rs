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
    assessment::{GradeHistoryEntry, SessionModeration, grade_history_views, parse_kind},
    candidate_profile::{Privacy, ProfileResponse, access_log, load_profile},
    company::{CompanyResponse, company_view},
    fsp::{FspStatus, load_status},
    invitations::{ReceivedInvitation, SentInvitation, received_views, sent_views},
    needs::{HiringNeed, company_need_views},
    notifications::NotificationExport,
    survey::{SurveyResponse, survey_views},
    tasks::{EmployerTask, MyTask, company_task_views, load_my_tasks},
    vacancies::{MyApplication, Vacancy, company_vacancy_views, my_apps},
};
use crate::{
    age::{self, AgeBracket},
    auth::{AuthUser, CandidateUser, Role},
    contacts::ContactAccessEntry,
    crypto::Crypto,
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
    /// The feed, email settings per kind and the letters sent.
    pub notifications: NotificationExport,
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

impl From<db::consents::UserConsent> for ConsentRecord {
    fn from(r: db::consents::UserConsent) -> Self {
        Self {
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
        }
    }
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
    let guardian_requests = db::guardians::requests(&state.pool, candidate)
        .await?
        .into_iter()
        .map(|r| guardian_record(r, &state.crypto, now))
        .collect::<ApiResult<_>>()?;
    Ok(age_record_of(age, guardian_requests))
}

/// Unlike the cabinet, the export shows the 18th birthday as long as the birth date is on record,
/// also once it has passed.
fn age_record_of(
    age: db::guardians::CandidateAge,
    guardian_requests: Vec<GuardianRecord>,
) -> AgeRecord {
    AgeRecord {
        bracket: if age.minor {
            AgeBracket::Minor
        } else {
            AgeBracket::Adult
        },
        birth_date: age.birth_date,
        adult_from: age.birth_date.map(age::adult_from),
        guardian_requests,
    }
}

fn guardian_record(
    r: db::guardians::GuardianRequest,
    crypto: &Crypto,
    now: DateTime<Utc>,
) -> ApiResult<GuardianRecord> {
    let decrypt = |b: Option<Vec<u8>>| b.map(|b| crypto.decrypt_str(&b)).transpose();
    Ok(GuardianRecord {
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
    /// This test assigned the category, and no moderator annulled it.
    pub assigned: bool,
    /// Person-fit statistic `l_z` and whether it flagged an aberrant pattern.
    pub person_fit_lz: Option<f64>,
    pub aberrant: Option<bool>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    /// Video recording summary and integrity events; the video is not part of the export.
    pub proctoring: Option<ProctoringSummary>,
    /// A moderator's decision on the result, with the comment.
    pub review: Option<SessionModeration>,
    /// When a moderator played the recording.
    pub recording_views: Vec<DateTime<Utc>>,
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
    let reviews = db::moderation::candidate_reviews(&state.pool, candidate).await?;
    let views = db::moderation::candidate_recording_views(&state.pool, candidate).await?;
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
            review: reviews
                .iter()
                .find(|v| v.session_id == r.id)
                .map(|v| SessionModeration {
                    decision: v.decision.into(),
                    comment: v.comment.clone(),
                    decided_at: v.decided_at,
                }),
            recording_views: views
                .iter()
                .filter(|v| v.session_id == r.id)
                .map(|v| v.at)
                .collect(),
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
        .map(ConsentRecord::from)
        .collect();
    let notifications = super::notifications::export(state, user).await?;
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
        notifications,
    })
}

/// Download everything the platform stores about you as a JSON file (ст. 14 152-ФЗ): account,
/// consents with the declared age, profile, age (birth date and legal representative for 14–17),
/// tests with summaries, category history, invitations, applications, who opened your contacts,
/// proctoring summaries; for an employer — the account and the company's data; for both —
/// notifications, email settings and the letters sent
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
    let disposition = attachment(now);
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

fn attachment(now: DateTime<Utc>) -> String {
    format!(
        "attachment; filename=\"fsp-talents-{}.json\"",
        now.format("%Y-%m-%d")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::Validator, guardian::GuardianContact};

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn keys(value: &impl Serialize) -> Vec<String> {
        let mut keys: Vec<String> = serde_json::to_value(value)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn consent_record_keeps_the_trail_of_the_grant() {
        let record = ConsentRecord::from(db::consents::UserConsent {
            id: Uuid::from_u128(1),
            document_id: Uuid::from_u128(2),
            kind: "guardian_consent".into(),
            version: 3,
            title: "Согласие законного представителя".into(),
            granted_at: at("2026-09-01T10:00:00Z"),
            revoked_at: Some(at("2026-09-20T10:00:00Z")),
            ip: Some("203.0.113.7".into()),
            user_agent: Some("Mozilla/5.0".into()),
            age_bracket: Some(db::types::AgeBracket::Minor),
        });
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["ip"], "203.0.113.7");
        assert_eq!(json["user_agent"], "Mozilla/5.0");
        assert_eq!(json["age_bracket"], "minor");
        assert_eq!(json["version"], 3);
        assert_eq!(json["revoked_at"], "2026-09-20T10:00:00Z");
    }

    fn request(crypto: &Crypto, email: Option<&str>) -> db::guardians::GuardianRequest {
        db::guardians::GuardianRequest {
            id: Uuid::from_u128(3),
            full_name_enc: email.map(|_| crypto.encrypt_str("Петрова Ольга Викторовна").unwrap()),
            email_enc: email.map(|e| crypto.encrypt_str(e).unwrap()),
            email_bidx: email.map(|e| crypto.blind_index(e)),
            sent_at: Some(at("2026-09-01T10:00:00Z")),
            confirm_expires_at: Some(at("2026-09-08T10:00:00Z")),
            decision: Some(db::types::GuardianDecision::Granted),
            answered_at: Some(at("2026-09-02T10:00:00Z")),
            consent_id: Some(Uuid::from_u128(4)),
            granted_at: Some(at("2026-09-02T10:00:00Z")),
            revoked_at: None,
            erased_at: None,
            created_at: at("2026-09-01T09:00:00Z"),
        }
    }

    #[test]
    fn guardian_record_shows_the_address_as_registered_in_lowercase() {
        let crypto = Crypto::new(&[1; 32], &[2; 32]);
        let mut v = Validator::default();
        let contact = guardian::clean_contact(
            &crypto,
            &GuardianContact {
                full_name: "Петрова Ольга Викторовна".into(),
                email: " O.Petrova@Example.RU ".into(),
            },
            "misha.petrov@example.ru",
            &mut v,
            ("full_name", "email"),
        );
        assert!(v.finish().is_ok());
        let now = at("2026-10-07T12:00:00Z");
        let r = guardian_record(request(&crypto, Some(&contact.email)), &crypto, now).unwrap();
        assert_eq!(r.email.as_deref(), Some("o.petrova@example.ru"));
        assert_eq!(r.full_name.as_deref(), Some("Петрова Ольга Викторовна"));
        assert_eq!(r.state, GuardianState::Granted);
        assert_eq!(r.requested_at, at("2026-09-01T09:00:00Z"));
        assert_eq!(r.last_letter_at, Some(at("2026-09-01T10:00:00Z")));
        assert_eq!(r.consent_id, Some(Uuid::from_u128(4)));
    }

    #[test]
    fn erased_guardian_request_keeps_only_dates_and_the_answer() {
        let crypto = Crypto::new(&[1; 32], &[2; 32]);
        let erased = db::guardians::GuardianRequest {
            erased_at: Some(at("2026-10-01T00:00:00Z")),
            ..request(&crypto, None)
        };
        let r = guardian_record(erased, &crypto, at("2026-10-07T12:00:00Z")).unwrap();
        assert_eq!((r.full_name, r.email), (None, None));
        assert_eq!(r.erased_at, Some(at("2026-10-01T00:00:00Z")));
        assert_eq!(r.state, GuardianState::Granted);
    }

    #[test]
    fn age_record_shows_the_18th_birthday_while_a_birth_date_is_on_record() {
        let birth = NaiveDate::from_ymd_opt(2010, 3, 14).unwrap();
        let minor = age_record_of(
            db::guardians::CandidateAge {
                birth_date: Some(birth),
                minor: true,
            },
            Vec::new(),
        );
        assert_eq!(minor.bracket, AgeBracket::Minor);
        assert_eq!(minor.birth_date, Some(birth));
        assert_eq!(minor.adult_from, Some(age::adult_from(birth)));

        let turned_18 = age_record_of(
            db::guardians::CandidateAge {
                birth_date: Some(birth),
                minor: false,
            },
            Vec::new(),
        );
        assert_eq!(turned_18.bracket, AgeBracket::Adult);
        assert_eq!(turned_18.adult_from, Some(age::adult_from(birth)));

        let adult = age_record_of(
            db::guardians::CandidateAge {
                birth_date: None,
                minor: false,
            },
            Vec::new(),
        );
        assert_eq!(adult.bracket, AgeBracket::Adult);
        assert_eq!((adult.birth_date, adult.adult_from), (None, None));
    }

    #[test]
    fn export_file_is_named_by_its_date() {
        assert_eq!(
            attachment(at("2026-10-07T23:59:59Z")),
            r#"attachment; filename="fsp-talents-2026-10-07.json""#
        );
    }

    #[test]
    fn assessment_record_leaves_the_test_internals_out() {
        let record = AssessmentRecord {
            id: Uuid::from_u128(5),
            specialization: Specialization::Backend,
            claimed_grade: Grade::Middle,
            attempt_kind: AttemptKind::Placement,
            status: AssessmentStatus::Passed,
            theta: Some(0.4),
            se: Some(0.3),
            measured_grade: Some(Grade::Middle),
            items_count: 18,
            assigned: true,
            person_fit_lz: Some(-0.5),
            aberrant: Some(false),
            started_at: at("2026-09-01T10:00:00Z"),
            finished_at: Some(at("2026-09-01T10:40:00Z")),
            proctoring: None,
            review: None,
            recording_views: Vec::new(),
        };
        let keys = keys(&record);
        assert_eq!(
            keys,
            [
                "aberrant",
                "assigned",
                "attempt_kind",
                "claimed_grade",
                "finished_at",
                "id",
                "items_count",
                "measured_grade",
                "person_fit_lz",
                "proctoring",
                "recording_views",
                "review",
                "se",
                "specialization",
                "started_at",
                "status",
                "theta",
            ]
        );
        for internal in ["state", "template_id", "seed", "current_item"] {
            assert!(!keys.iter().any(|k| k == internal), "{internal}");
        }
    }
}
