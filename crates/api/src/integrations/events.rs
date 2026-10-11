//! Integration events: the catalog, payload shapes and the transactional outbox writer.
//!
//! 152-ФЗ, enforced by construction: an [`AtsCandidate`] made before the candidate accepted an
//! invitation (or applied) carries only the anonymous code and category. The name, contacts and
//! profile links appear only through [`AtsCandidate::disclosed`], which consumes
//! [`CandidateContacts`] — and those come only from a [`ContactGrant`], which checks the basis
//! and writes the contact access log (channel `webhook`).

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    contacts::{
        AccessChannel, AccessContext, CandidateContacts, ContactBasis, ContactGrant, Contacts,
    },
    crypto::{Crypto, CryptoError},
    dto::{ApplicationStatus, DeclineReason, InvitationStatus, SalaryRange},
    error::ApiResult,
    state::AppState,
};
use catalog::{Category, WorkFormat};

/// Delivery is retried for a day; after that the event is marked failed.
pub const DELIVERY_WINDOW_HOURS: i64 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum EventType {
    #[serde(rename = "invitation.sent")]
    InvitationSent,
    #[serde(rename = "invitation.viewed")]
    InvitationViewed,
    #[serde(rename = "invitation.accepted")]
    InvitationAccepted,
    #[serde(rename = "invitation.declined")]
    InvitationDeclined,
    #[serde(rename = "invitation.expired")]
    InvitationExpired,
    #[serde(rename = "invitation.withdrawn")]
    InvitationWithdrawn,
    #[serde(rename = "application.created")]
    ApplicationCreated,
    #[serde(rename = "application.status_changed")]
    ApplicationStatusChanged,
    #[serde(rename = "contacts.opened")]
    ContactsOpened,
    #[serde(rename = "integration.test")]
    Test,
}

impl EventType {
    pub const ALL: [EventType; 10] = [
        EventType::InvitationSent,
        EventType::InvitationViewed,
        EventType::InvitationAccepted,
        EventType::InvitationDeclined,
        EventType::InvitationExpired,
        EventType::InvitationWithdrawn,
        EventType::ApplicationCreated,
        EventType::ApplicationStatusChanged,
        EventType::ContactsOpened,
        EventType::Test,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            EventType::InvitationSent => "invitation.sent",
            EventType::InvitationViewed => "invitation.viewed",
            EventType::InvitationAccepted => "invitation.accepted",
            EventType::InvitationDeclined => "invitation.declined",
            EventType::InvitationExpired => "invitation.expired",
            EventType::InvitationWithdrawn => "invitation.withdrawn",
            EventType::ApplicationCreated => "application.created",
            EventType::ApplicationStatusChanged => "application.status_changed",
            EventType::ContactsOpened => "contacts.opened",
            EventType::Test => "integration.test",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

/// Name, contacts and links to the full profile. Present only after the candidate accepted the
/// invitation or applied to the company's vacancy.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Disclosure {
    full_name: Option<String>,
    #[serde(flatten)]
    contacts: Contacts,
    basis: ContactBasis,
    /// `GET` with the same API key (`candidates:read`).
    profile_url: String,
    /// Standardized PDF profile, same key.
    profile_pdf_url: String,
}

/// Who the event is about. Before acceptance: the anonymous code and category only.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AtsCandidate {
    /// Anonymous code shown to employers, e.g. `K-7F3A2C`.
    code: String,
    category: Option<Category>,
    /// Platform id for `GET /integrations/candidates/{id}`; after acceptance or application.
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disclosed: Option<Disclosure>,
}

pub fn category_of(specialization: Option<&str>, grade: Option<&str>) -> Option<Category> {
    match (specialization, grade) {
        (Some(s), Some(g)) => Some(Category::new(s.parse().ok()?, g.parse().ok()?)),
        _ => None,
    }
}

impl AtsCandidate {
    pub fn anonymous(code: String, category: Option<Category>) -> Self {
        Self {
            code,
            category,
            id: None,
            disclosed: None,
        }
    }

    /// The id without contacts: the company already holds a basis for this candidate.
    pub fn identified(code: String, category: Option<Category>, grant: &ContactGrant) -> Self {
        Self {
            code,
            category,
            id: Some(grant.candidate_id()),
            disclosed: None,
        }
    }

    /// The id of an accepted invitation in the polling feed; contacts stay behind
    /// `GET /integrations/candidates/{id}`, which goes through `ContactGrant`.
    pub fn accepted(code: String, category: Option<Category>, id: Uuid) -> Self {
        Self {
            code,
            category,
            id: Some(id),
            disclosed: None,
        }
    }

    pub fn disclosed(
        code: String,
        category: Option<Category>,
        contacts: CandidateContacts,
        public_url: &str,
    ) -> Self {
        let id = contacts.candidate_id;
        let profile_url = format!("{public_url}/api/v1/integrations/candidates/{id}");
        Self {
            code,
            category,
            id: Some(id),
            disclosed: Some(Disclosure {
                full_name: contacts.full_name,
                contacts: contacts.contacts,
                basis: contacts.basis,
                profile_pdf_url: format!("{profile_url}/profile.pdf"),
                profile_url,
            }),
        }
    }

    pub fn is_disclosed(&self) -> bool {
        self.disclosed.is_some()
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AtsInvitation {
    pub id: Uuid,
    pub status: InvitationStatus,
    pub title: String,
    pub message: String,
    pub salary: SalaryRange,
    pub work_format: Option<WorkFormat>,
    pub need_id: Option<Uuid>,
    pub vacancy_id: Option<Uuid>,
    /// Only for `declined`.
    pub decline_reason: Option<DeclineReason>,
    pub decline_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    /// Latest status change; the polling cursor.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AtsApplication {
    pub id: Uuid,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub status: ApplicationStatus,
    pub cover_letter: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InvitationEventData {
    pub invitation: AtsInvitation,
    pub candidate: AtsCandidate,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApplicationEventData {
    pub application: AtsApplication,
    pub candidate: AtsCandidate,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ContactsOpenedData {
    pub candidate: AtsCandidate,
    pub basis: ContactBasis,
    pub basis_id: Uuid,
    pub opened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TestEventData {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(untagged)]
pub enum EventData {
    Invitation(InvitationEventData),
    Application(ApplicationEventData),
    ContactsOpened(ContactsOpenedData),
    Test(TestEventData),
}

/// Body of every webhook delivery. `id` repeats the `FSP-Event-Id` header: deliveries are
/// at-least-once, so the receiver deduplicates by it.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WebhookEvent {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub created_at: DateTime<Utc>,
    pub company_id: Uuid,
    pub data: EventData,
}

fn payload_context(id: Uuid) -> Vec<u8> {
    let mut ctx = b"integration_event:".to_vec();
    ctx.extend_from_slice(id.as_bytes());
    ctx
}

pub fn seal_payload(crypto: &Crypto, id: Uuid, body: &[u8]) -> Result<Vec<u8>, CryptoError> {
    crypto.fields.encrypt_bound(body, &payload_context(id))
}

pub fn open_payload(crypto: &Crypto, id: Uuid, sealed: &[u8]) -> Result<Vec<u8>, CryptoError> {
    crypto.fields.decrypt_bound(sealed, &payload_context(id))
}

pub fn invitation_from_snapshot(s: &db::integrations::InvitationSnapshot) -> AtsInvitation {
    AtsInvitation {
        id: s.id,
        status: s.status.into(),
        title: s.title.clone(),
        message: s.message.clone(),
        salary: SalaryRange {
            min: s.salary_min,
            max: s.salary_max,
        },
        work_format: s.work_format.as_deref().and_then(WorkFormat::from_slug),
        need_id: s.need_id,
        vacancy_id: s.vacancy_id,
        decline_reason: s
            .decline_reason
            .as_deref()
            .and_then(DeclineReason::from_slug),
        decline_comment: s.decline_comment.clone(),
        created_at: s.created_at,
        viewed_at: s.viewed_at,
        responded_at: s.responded_at,
        expires_at: s.expires_at,
        updated_at: s.updated_at,
    }
}

/// Writes integration events inside the caller's transaction. Nothing is written for a company
/// without a webhook: no event means no disclosure and no access-log entry.
pub struct Outbox<'a> {
    crypto: &'a Crypto,
    public_url: &'a str,
}

impl<'a> Outbox<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self {
            crypto: &state.crypto,
            public_url: &state.config.public_url,
        }
    }

    pub async fn invitation(
        &self,
        conn: &mut PgConnection,
        kind: EventType,
        invitation_id: Uuid,
    ) -> ApiResult<Option<Uuid>> {
        let Some(s) = db::integrations::invitation_snapshot(&mut *conn, invitation_id).await?
        else {
            return Ok(None);
        };
        if !s.webhook_on {
            return Ok(None);
        }
        let category = category_of(s.specialization.as_deref(), s.grade.as_deref());
        let candidate = if s.status == db::types::InvitationStatus::Accepted {
            self.disclose(
                conn,
                s.company_id,
                s.owner_user_id,
                s.candidate_id,
                s.public_code.clone(),
                category,
            )
            .await?
        } else {
            AtsCandidate::anonymous(s.public_code.clone(), category)
        };
        let data = EventData::Invitation(InvitationEventData {
            invitation: invitation_from_snapshot(&s),
            candidate,
        });
        self.write(
            conn,
            s.company_id,
            Some(s.candidate_id),
            kind,
            data,
            Utc::now(),
        )
        .await
        .map(Some)
    }

    pub async fn application(
        &self,
        conn: &mut PgConnection,
        kind: EventType,
        application_id: Uuid,
    ) -> ApiResult<Option<Uuid>> {
        let Some(s) = db::integrations::application_snapshot(&mut *conn, application_id).await?
        else {
            return Ok(None);
        };
        if !s.webhook_on {
            return Ok(None);
        }
        let category = category_of(s.specialization.as_deref(), s.grade.as_deref());
        let candidate = self
            .disclose(
                conn,
                s.company_id,
                s.owner_user_id,
                s.candidate_id,
                s.public_code.clone(),
                category,
            )
            .await?;
        let data = EventData::Application(ApplicationEventData {
            application: AtsApplication {
                id: s.id,
                vacancy_id: s.vacancy_id,
                vacancy_title: s.vacancy_title,
                status: s.status.into(),
                cover_letter: s.cover_letter,
                created_at: s.created_at,
                updated_at: s.updated_at,
            },
            candidate,
        });
        self.write(
            conn,
            s.company_id,
            Some(s.candidate_id),
            kind,
            data,
            Utc::now(),
        )
        .await
        .map(Some)
    }

    /// Someone in the cabinet opened the contacts: the ATS learns the candidate id and the basis,
    /// and fetches the contacts itself if it needs them.
    pub async fn contacts_opened(
        &self,
        conn: &mut PgConnection,
        company_id: Uuid,
        grant: &ContactGrant,
    ) -> ApiResult<Option<Uuid>> {
        if !db::integrations::webhook_enabled(&mut *conn, company_id).await? {
            return Ok(None);
        }
        let Some(brief) =
            db::integrations::candidate_brief(&mut *conn, grant.candidate_id()).await?
        else {
            return Ok(None);
        };
        let category = category_of(brief.specialization.as_deref(), brief.grade.as_deref());
        let data = EventData::ContactsOpened(ContactsOpenedData {
            candidate: AtsCandidate::identified(brief.public_code, category, grant),
            basis: grant.basis(),
            basis_id: grant.basis_id(),
            opened_at: Utc::now(),
        });
        self.write(
            conn,
            company_id,
            Some(grant.candidate_id()),
            EventType::ContactsOpened,
            data,
            Utc::now(),
        )
        .await
        .map(Some)
    }

    /// The «Тестовое событие» button. Written already leased, so the dispatcher leaves it to the
    /// request that delivers it inline.
    pub async fn test(
        &self,
        conn: &mut PgConnection,
        company_id: Uuid,
        lease: Duration,
    ) -> ApiResult<Uuid> {
        let data = EventData::Test(TestEventData {
            message: "Тестовое событие ФСП Таланты: вебхук настроен правильно".into(),
        });
        self.write(
            conn,
            company_id,
            None,
            EventType::Test,
            data,
            Utc::now() + lease,
        )
        .await
    }

    async fn disclose(
        &self,
        conn: &mut PgConnection,
        company_id: Uuid,
        owner_user_id: Uuid,
        candidate_id: Uuid,
        code: String,
        category: Option<Category>,
    ) -> ApiResult<AtsCandidate> {
        let ctx = AccessContext {
            employer_user_id: owner_user_id,
            company_id,
            ip: None,
            user_agent: None,
            channel: AccessChannel::Webhook,
            api_key_id: None,
        };
        Ok(
            match ContactGrant::obtain(conn, &ctx, candidate_id).await? {
                Some(grant) => AtsCandidate::disclosed(
                    code,
                    category,
                    CandidateContacts::reveal(&mut *conn, self.crypto, grant).await?,
                    self.public_url,
                ),
                None => AtsCandidate::anonymous(code, category),
            },
        )
    }

    async fn write(
        &self,
        conn: &mut PgConnection,
        company_id: Uuid,
        candidate_id: Option<Uuid>,
        kind: EventType,
        data: EventData,
        next_attempt_at: DateTime<Utc>,
    ) -> ApiResult<Uuid> {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let body = serde_json::to_vec(&WebhookEvent {
            id,
            event_type: kind,
            created_at,
            company_id,
            data,
        })?;
        db::integrations::insert_event(
            &mut *conn,
            &db::integrations::NewEvent {
                id,
                company_id,
                candidate_id,
                event_type: kind.as_str(),
                payload_enc: &seal_payload(self.crypto, id, &body)?,
                next_attempt_at,
                deadline_at: created_at + Duration::hours(DELIVERY_WINDOW_HOURS),
            },
        )
        .await?;
        Ok(id)
    }
}

/// Lazy expiry with `invitation.expired` events in the same transaction. Called wherever
/// invitations are read, like the plain `expire_stale_invitations` it replaces in handlers.
pub async fn expire_invitations(state: &AppState) -> ApiResult<()> {
    let mut tx = state.pool.begin().await?;
    let expired = db::integrations::expire_stale_invitations(&mut *tx).await?;
    if expired.is_empty() {
        return Ok(());
    }
    let outbox = Outbox::new(state);
    for e in expired {
        outbox
            .invitation(&mut tx, EventType::InvitationExpired, e.id)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog::{Grade, Specialization};

    fn invitation(status: InvitationStatus) -> AtsInvitation {
        let at = DateTime::from_timestamp(1_790_000_000, 0).unwrap();
        AtsInvitation {
            id: Uuid::nil(),
            status,
            title: "Backend-разработчик".into(),
            message: "Здравствуйте!".into(),
            salary: SalaryRange {
                min: 250_000,
                max: 320_000,
            },
            work_format: Some(WorkFormat::Remote),
            need_id: None,
            vacancy_id: None,
            decline_reason: None,
            decline_comment: None,
            created_at: at,
            viewed_at: None,
            responded_at: None,
            expires_at: at,
            updated_at: at,
        }
    }

    fn contacts(id: Uuid) -> CandidateContacts {
        CandidateContacts {
            candidate_id: id,
            full_name: Some("Анна Смирнова".into()),
            contacts: Contacts {
                phone: Some("+7 900 000-00-00".into()),
                telegram: Some("@anna".into()),
                email: Some("anna@example.ru".into()),
            },
            basis: ContactBasis::Invitation,
            basis_id: Uuid::nil(),
        }
    }

    fn envelope(kind: EventType, data: EventData) -> serde_json::Value {
        serde_json::to_value(WebhookEvent {
            id: Uuid::nil(),
            event_type: kind,
            created_at: DateTime::from_timestamp(1_790_000_000, 0).unwrap(),
            company_id: Uuid::nil(),
            data,
        })
        .unwrap()
    }

    #[test]
    fn payload_before_acceptance_is_anonymous() {
        let category = Some(Category::new(Specialization::Backend, Grade::Middle));
        let v = envelope(
            EventType::InvitationSent,
            EventData::Invitation(InvitationEventData {
                invitation: invitation(InvitationStatus::Sent),
                candidate: AtsCandidate::anonymous("K-7F3A2C".into(), category),
            }),
        );
        assert_eq!(v["type"], "invitation.sent");
        let candidate = v["data"]["candidate"].as_object().unwrap();
        let mut keys: Vec<&str> = candidate.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["category", "code"], "only the code and category");
        assert_eq!(candidate["code"], "K-7F3A2C");
        assert_eq!(candidate["category"]["specialization"], "backend");
        let text = v.to_string();
        for leak in ["Анна", "anna", "+7 900", "profile", "full_name"] {
            assert!(!text.contains(leak), "{leak} leaked: {text}");
        }
    }

    #[test]
    fn payload_after_acceptance_carries_contacts_and_links() {
        let id = Uuid::now_v7();
        let v = envelope(
            EventType::InvitationAccepted,
            EventData::Invitation(InvitationEventData {
                invitation: invitation(InvitationStatus::Accepted),
                candidate: AtsCandidate::disclosed(
                    "K-7F3A2C".into(),
                    None,
                    contacts(id),
                    "https://talents.example.ru",
                ),
            }),
        );
        let c = &v["data"]["candidate"];
        assert_eq!(c["id"], id.to_string());
        let d = &c["disclosed"];
        assert_eq!(d["full_name"], "Анна Смирнова");
        assert_eq!(d["phone"], "+7 900 000-00-00");
        assert_eq!(d["telegram"], "@anna");
        assert_eq!(d["email"], "anna@example.ru");
        assert_eq!(d["basis"], "invitation");
        assert_eq!(
            d["profile_url"],
            format!("https://talents.example.ru/api/v1/integrations/candidates/{id}")
        );
        assert_eq!(
            d["profile_pdf_url"],
            format!("https://talents.example.ru/api/v1/integrations/candidates/{id}/profile.pdf")
        );
    }

    #[test]
    fn decline_carries_the_reason_but_no_identity() {
        let mut inv = invitation(InvitationStatus::Declined);
        inv.decline_reason = Some(DeclineReason::Salary);
        inv.decline_comment = Some("рассматриваю от 350 000".into());
        let v = envelope(
            EventType::InvitationDeclined,
            EventData::Invitation(InvitationEventData {
                invitation: inv,
                candidate: AtsCandidate::anonymous("K-1".into(), None),
            }),
        );
        assert_eq!(v["data"]["invitation"]["decline_reason"], "salary");
        assert_eq!(
            v["data"]["invitation"]["decline_comment"],
            "рассматриваю от 350 000"
        );
        assert!(v["data"]["candidate"].get("id").is_none());
        assert!(v["data"]["candidate"].get("disclosed").is_none());
    }

    #[test]
    fn sealed_payload_is_bound_to_its_event() {
        let crypto = Crypto::new(&[3; 32], &[4; 32]);
        let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
        let sealed = seal_payload(&crypto, a, b"{\"x\":1}").unwrap();
        assert_eq!(open_payload(&crypto, a, &sealed).unwrap(), b"{\"x\":1}");
        assert!(open_payload(&crypto, b, &sealed).is_err());
    }

    #[test]
    fn event_catalog_round_trips() {
        for t in EventType::ALL {
            assert_eq!(EventType::parse(t.as_str()), Some(t));
            assert_eq!(serde_json::to_value(t).unwrap(), t.as_str());
        }
    }
}
