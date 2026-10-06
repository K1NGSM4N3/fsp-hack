//! Notifications (architecture.md §13). A domain change writes its in-app item in the same
//! transaction ([`Notifier`]); the email dispatcher folds queued items into letters under a
//! per-recipient rate cap ([`email`]); a sweeper adds expiry reminders and trust-level changes
//! ([`sweep`]).
//!
//! 152-ФЗ by construction: a payload holds only what its recipient already sees in the cabinet.
//! Company payloads name a candidate by the anonymous code — there is no field for a name or
//! contacts, so no letter to a company can carry them.
//!
//! Adding a kind: a variant at the end of [`NotificationKind`] (its bit in the settings mask is
//! the discriminant, never renumbered), a payload in [`Event`], texts in [`render`], and a
//! migration that extends the `notifications.kind` check.

pub mod email;
pub mod render;
pub mod sweep;
pub mod token;

use std::time::Duration;

use catalog::{Category, WorkFormat};
use chrono::{DateTime, Utc};
use matching::trust::TrustLevel;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgExecutor};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    config::NotifyConfig,
    dto::{ApplicationStatus, DeclineReason, HiringStage},
    state::AppState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum NotificationKind {
    /// A company invited the candidate.
    InvitationReceived = 0,
    /// 48 hours before an unanswered invitation expires; once per invitation.
    InvitationExpiring = 1,
    InvitationWithdrawn = 2,
    /// The company or its ATS reported interview, offer, hire or rejection.
    HiringStage = 3,
    ApplicationStatus = 4,
    /// The test finished: category assigned or not.
    AssessmentResult = 5,
    /// The legal representative of a candidate aged 14–17 answered.
    GuardianDecision = 6,
    InvitationAccepted = 7,
    InvitationDeclined = 8,
    ApplicationReceived = 9,
    /// Deliveries to the company's ATS webhook keep failing.
    WebhookFailing = 10,
    /// The company's trust level or invitation quota moved.
    TrustChanged = 11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Candidate,
    Employer,
}

impl NotificationKind {
    pub const ALL: [NotificationKind; 12] = [
        NotificationKind::InvitationReceived,
        NotificationKind::InvitationExpiring,
        NotificationKind::InvitationWithdrawn,
        NotificationKind::HiringStage,
        NotificationKind::ApplicationStatus,
        NotificationKind::AssessmentResult,
        NotificationKind::GuardianDecision,
        NotificationKind::InvitationAccepted,
        NotificationKind::InvitationDeclined,
        NotificationKind::ApplicationReceived,
        NotificationKind::WebhookFailing,
        NotificationKind::TrustChanged,
    ];

    pub const fn slug(self) -> &'static str {
        match self {
            NotificationKind::InvitationReceived => "invitation_received",
            NotificationKind::InvitationExpiring => "invitation_expiring",
            NotificationKind::InvitationWithdrawn => "invitation_withdrawn",
            NotificationKind::HiringStage => "hiring_stage",
            NotificationKind::ApplicationStatus => "application_status",
            NotificationKind::AssessmentResult => "assessment_result",
            NotificationKind::GuardianDecision => "guardian_decision",
            NotificationKind::InvitationAccepted => "invitation_accepted",
            NotificationKind::InvitationDeclined => "invitation_declined",
            NotificationKind::ApplicationReceived => "application_received",
            NotificationKind::WebhookFailing => "webhook_failing",
            NotificationKind::TrustChanged => "trust_changed",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.slug() == s)
    }

    pub const fn index(self) -> u8 {
        self as u8
    }

    pub fn from_index(i: u8) -> Option<Self> {
        Self::ALL.get(usize::from(i)).copied()
    }

    pub const fn bit(self) -> i32 {
        1 << self as u8
    }

    pub const fn audience(self) -> Audience {
        match self {
            NotificationKind::InvitationReceived
            | NotificationKind::InvitationExpiring
            | NotificationKind::InvitationWithdrawn
            | NotificationKind::HiringStage
            | NotificationKind::ApplicationStatus
            | NotificationKind::AssessmentResult
            | NotificationKind::GuardianDecision => Audience::Candidate,
            NotificationKind::InvitationAccepted
            | NotificationKind::InvitationDeclined
            | NotificationKind::ApplicationReceived
            | NotificationKind::WebhookFailing
            | NotificationKind::TrustChanged => Audience::Employer,
        }
    }

    /// The settings row and the unsubscribe page name the kind this way.
    pub const fn title(self) -> &'static str {
        match self {
            NotificationKind::InvitationReceived => "Новые приглашения",
            NotificationKind::InvitationExpiring => "Напоминание о сроке ответа",
            NotificationKind::InvitationWithdrawn => "Компания отозвала приглашение",
            NotificationKind::HiringStage => "Этапы найма",
            NotificationKind::ApplicationStatus => "Ответы на отклики",
            NotificationKind::AssessmentResult => "Итоги теста",
            NotificationKind::GuardianDecision => "Решение законного представителя",
            NotificationKind::InvitationAccepted => "Кандидат принял приглашение",
            NotificationKind::InvitationDeclined => "Кандидат отклонил приглашение",
            NotificationKind::ApplicationReceived => "Новые отклики",
            NotificationKind::WebhookFailing => "Сбои доставки в ATS",
            NotificationKind::TrustChanged => "Уровень доверия и квота",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            NotificationKind::InvitationReceived => "Компания, должность, вилка и срок ответа",
            NotificationKind::InvitationExpiring => {
                "За 48 часов до конца срока, если вы ещё не ответили — один раз"
            }
            NotificationKind::InvitationWithdrawn => "Приглашение больше не ждёт ответа",
            NotificationKind::HiringStage => {
                "Компания отметила собеседование, оффер, найм или отказ"
            }
            NotificationKind::ApplicationStatus => {
                "Приглашение или отказ по отклику; «просмотрено» — только в кабинете"
            }
            NotificationKind::AssessmentResult => "Присвоена категория или тест не пройден",
            NotificationKind::GuardianDecision => {
                "Согласие дано, не дано или отозвано — для участников 14–17 лет"
            }
            NotificationKind::InvitationAccepted => {
                "Контакты кандидата откроются в кабинете; в письме их нет"
            }
            NotificationKind::InvitationDeclined => "С причиной отказа",
            NotificationKind::ApplicationReceived => "Отклик на опубликованную вакансию",
            NotificationKind::WebhookFailing => "Вебхук в вашу ATS не принимает события",
            NotificationKind::TrustChanged => "Изменилась квота приглашений за 7 дней",
        }
    }

    pub fn of(audience: Audience) -> impl Iterator<Item = NotificationKind> {
        Self::ALL
            .into_iter()
            .filter(move |k| k.audience() == audience)
    }
}

/// Kinds whose email the user turned off, one bit per kind. In-app is always on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EmailMask(pub i32);

impl EmailMask {
    pub fn allows(self, kind: NotificationKind) -> bool {
        self.0 & kind.bit() == 0
    }

    pub fn all_of(audience: Audience) -> i32 {
        NotificationKind::of(audience).fold(0, |m, k| m | k.bit())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitationNote {
    pub invitation_id: Uuid,
    pub company_name: String,
    pub title: String,
    pub salary_min: i64,
    pub salary_max: i64,
    pub work_format: Option<WorkFormat>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageNote {
    pub invitation_id: Option<Uuid>,
    pub application_id: Option<Uuid>,
    pub company_name: String,
    pub title: String,
    pub stage: HiringStage,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplicationNote {
    pub application_id: Uuid,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub company_name: String,
    pub status: ApplicationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssessmentNote {
    pub session_id: Uuid,
    pub category: Category,
    pub passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianOutcome {
    Granted,
    Declined,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardianNote {
    pub decision: GuardianOutcome,
}

/// The company learns the anonymous code and where the contacts are — not the contacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AcceptedNote {
    pub invitation_id: Uuid,
    pub candidate_id: Uuid,
    pub candidate_code: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeclinedNote {
    pub invitation_id: Uuid,
    pub candidate_code: String,
    pub title: String,
    pub reason: Option<DeclineReason>,
    /// The candidate's own words: shown in the cabinet, kept out of the letter.
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplicantNote {
    pub application_id: Uuid,
    pub vacancy_id: Uuid,
    pub vacancy_title: String,
    pub candidate_id: Uuid,
    pub candidate_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookNote {
    pub event_type: String,
    pub error: String,
    pub failed_attempts: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrustNote {
    pub level_before: TrustLevel,
    pub level: TrustLevel,
    pub quota_before: i16,
    pub quota: i16,
}

/// What happened, stored as the item's payload. Cold data: parsed only to render a page of the
/// feed or a letter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    InvitationReceived(InvitationNote),
    InvitationExpiring(InvitationNote),
    InvitationWithdrawn(InvitationNote),
    HiringStage(StageNote),
    ApplicationStatus(ApplicationNote),
    AssessmentResult(AssessmentNote),
    GuardianDecision(GuardianNote),
    InvitationAccepted(AcceptedNote),
    InvitationDeclined(DeclinedNote),
    ApplicationReceived(ApplicantNote),
    WebhookFailing(WebhookNote),
    TrustChanged(TrustNote),
}

impl Event {
    pub fn kind(&self) -> NotificationKind {
        match self {
            Event::InvitationReceived(_) => NotificationKind::InvitationReceived,
            Event::InvitationExpiring(_) => NotificationKind::InvitationExpiring,
            Event::InvitationWithdrawn(_) => NotificationKind::InvitationWithdrawn,
            Event::HiringStage(_) => NotificationKind::HiringStage,
            Event::ApplicationStatus(_) => NotificationKind::ApplicationStatus,
            Event::AssessmentResult(_) => NotificationKind::AssessmentResult,
            Event::GuardianDecision(_) => NotificationKind::GuardianDecision,
            Event::InvitationAccepted(_) => NotificationKind::InvitationAccepted,
            Event::InvitationDeclined(_) => NotificationKind::InvitationDeclined,
            Event::ApplicationReceived(_) => NotificationKind::ApplicationReceived,
            Event::WebhookFailing(_) => NotificationKind::WebhookFailing,
            Event::TrustChanged(_) => NotificationKind::TrustChanged,
        }
    }

    /// The invitation, application or test the item is about.
    pub fn subject(&self) -> Option<Uuid> {
        match self {
            Event::InvitationReceived(n)
            | Event::InvitationExpiring(n)
            | Event::InvitationWithdrawn(n) => Some(n.invitation_id),
            Event::HiringStage(n) => n.invitation_id.or(n.application_id),
            Event::ApplicationStatus(n) => Some(n.application_id),
            Event::AssessmentResult(n) => Some(n.session_id),
            Event::InvitationAccepted(n) => Some(n.invitation_id),
            Event::InvitationDeclined(n) => Some(n.invitation_id),
            Event::ApplicationReceived(n) => Some(n.application_id),
            Event::GuardianDecision(_) | Event::WebhookFailing(_) | Event::TrustChanged(_) => None,
        }
    }

    pub fn parse(payload: &str) -> Option<Event> {
        serde_json::from_str(payload).ok()
    }
}

/// Who gets the item.
#[derive(Debug, Clone, Copy)]
pub enum To {
    Candidate(Uuid),
    /// The company's owner account.
    Company(Uuid),
}

/// Who the item is about, beyond its recipient: it goes away with either account.
#[derive(Debug, Clone, Copy, Default)]
pub struct About {
    pub candidate_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
}

/// Writes items inside the caller's transaction, so an item exists exactly when the change it
/// reports was committed.
#[derive(Debug, Clone, Copy)]
pub struct Notifier {
    hold: Duration,
}

impl Default for Notifier {
    fn default() -> Self {
        Self::from_config(&NotifyConfig::default())
    }
}

impl Notifier {
    pub fn new(state: &AppState) -> Self {
        Self::from_config(&state.config.notifications)
    }

    pub fn from_config(config: &NotifyConfig) -> Self {
        Self { hold: config.hold }
    }

    /// `email: false` keeps the item in the cabinet only, whatever the settings say.
    pub async fn write(
        &self,
        db: impl PgExecutor<'_>,
        to: To,
        about: About,
        event: &Event,
        dedup_key: Option<&str>,
        email: bool,
    ) -> Result<bool, sqlx::Error> {
        let payload = serde_json::to_string(event).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
        let kind = event.kind();
        let (user_id, company_id) = match to {
            To::Candidate(id) => (Some(id), about.company_id),
            To::Company(id) => (None, Some(id)),
        };
        let candidate_id = match to {
            To::Candidate(id) => Some(id),
            To::Company(_) => about.candidate_id,
        };
        db::notifications::insert(
            db,
            &db::notifications::NewNotification {
                id: Uuid::now_v7(),
                user_id,
                kind: kind.slug(),
                kind_bit: kind.bit(),
                payload: &payload,
                subject_id: event.subject(),
                candidate_id,
                company_id,
                dedup_key,
                email,
                hold_secs: self.hold.as_secs_f64(),
            },
        )
        .await
    }

    /// Invitation kinds: the candidate hears about a new, expiring or withdrawn invitation, the
    /// company about an acceptance or a decline.
    pub async fn invitation(
        &self,
        conn: &mut PgConnection,
        kind: NotificationKind,
        invitation_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let Some(b) = db::notifications::invitation_brief(&mut *conn, invitation_id).await? else {
            return Ok(false);
        };
        self.invitation_from(conn, kind, &b).await
    }

    pub async fn invitation_from(
        &self,
        conn: &mut PgConnection,
        kind: NotificationKind,
        b: &db::notifications::InvitationBrief,
    ) -> Result<bool, sqlx::Error> {
        let about = About {
            candidate_id: Some(b.candidate_id),
            company_id: Some(b.company_id),
        };
        let note = || InvitationNote {
            invitation_id: b.id,
            company_name: b.company_name.clone(),
            title: b.title.clone(),
            salary_min: b.salary_min,
            salary_max: b.salary_max,
            work_format: b.work_format.as_deref().and_then(WorkFormat::from_slug),
            expires_at: b.expires_at,
        };
        let to_candidate = To::Candidate(b.candidate_id);
        let to_company = To::Company(b.company_id);
        let (to, event, dedup) = match kind {
            NotificationKind::InvitationReceived => {
                (to_candidate, Event::InvitationReceived(note()), None)
            }
            NotificationKind::InvitationExpiring => (
                to_candidate,
                Event::InvitationExpiring(note()),
                Some(format!("expiring:{}", b.id)),
            ),
            NotificationKind::InvitationWithdrawn => {
                (to_candidate, Event::InvitationWithdrawn(note()), None)
            }
            NotificationKind::InvitationAccepted => (
                to_company,
                Event::InvitationAccepted(AcceptedNote {
                    invitation_id: b.id,
                    candidate_id: b.candidate_id,
                    candidate_code: b.public_code.clone(),
                    title: b.title.clone(),
                }),
                None,
            ),
            NotificationKind::InvitationDeclined => (
                to_company,
                Event::InvitationDeclined(DeclinedNote {
                    invitation_id: b.id,
                    candidate_code: b.public_code.clone(),
                    title: b.title.clone(),
                    reason: b
                        .decline_reason
                        .as_deref()
                        .and_then(DeclineReason::from_slug),
                    comment: b.decline_comment.clone(),
                }),
                None,
            ),
            _ => return Ok(false),
        };
        self.write(&mut *conn, to, about, &event, dedup.as_deref(), true)
            .await
    }

    /// A new application tells the company; a status change tells the candidate («просмотрено»
    /// only in the cabinet).
    pub async fn application(
        &self,
        conn: &mut PgConnection,
        application_id: Uuid,
        status: ApplicationStatus,
    ) -> Result<bool, sqlx::Error> {
        let Some(b) = db::notifications::application_brief(&mut *conn, application_id).await?
        else {
            return Ok(false);
        };
        let about = About {
            candidate_id: Some(b.candidate_id),
            company_id: Some(b.company_id),
        };
        match status {
            ApplicationStatus::Sent => {
                let event = Event::ApplicationReceived(ApplicantNote {
                    application_id: b.id,
                    vacancy_id: b.vacancy_id,
                    vacancy_title: b.vacancy_title,
                    candidate_id: b.candidate_id,
                    candidate_code: b.public_code,
                });
                self.write(conn, To::Company(b.company_id), about, &event, None, true)
                    .await
            }
            ApplicationStatus::Withdrawn => Ok(false),
            status => {
                let event = Event::ApplicationStatus(ApplicationNote {
                    application_id: b.id,
                    vacancy_id: b.vacancy_id,
                    vacancy_title: b.vacancy_title,
                    company_name: b.company_name,
                    status,
                });
                let email = status != ApplicationStatus::Viewed;
                self.write(
                    conn,
                    To::Candidate(b.candidate_id),
                    about,
                    &event,
                    None,
                    email,
                )
                .await
            }
        }
    }

    /// A stage the company's ATS reported on an accepted invitation or an application.
    pub async fn hiring_stage(
        &self,
        conn: &mut PgConnection,
        invitation_id: Option<Uuid>,
        application_id: Option<Uuid>,
        stage: HiringStage,
        reason: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let (candidate_id, company_id, company_name, title) = match (invitation_id, application_id)
        {
            (Some(id), _) => match db::notifications::invitation_brief(&mut *conn, id).await? {
                Some(b) => (b.candidate_id, b.company_id, b.company_name, b.title),
                None => return Ok(false),
            },
            (None, Some(id)) => match db::notifications::application_brief(&mut *conn, id).await? {
                Some(b) => (
                    b.candidate_id,
                    b.company_id,
                    b.company_name,
                    b.vacancy_title,
                ),
                None => return Ok(false),
            },
            (None, None) => return Ok(false),
        };
        let event = Event::HiringStage(StageNote {
            invitation_id,
            application_id,
            company_name,
            title,
            stage,
            reason: reason.map(str::to_owned),
        });
        let about = About {
            candidate_id: Some(candidate_id),
            company_id: Some(company_id),
        };
        self.write(conn, To::Candidate(candidate_id), about, &event, None, true)
            .await
    }

    pub async fn assessment(
        &self,
        db: impl PgExecutor<'_>,
        candidate: Uuid,
        session_id: Uuid,
        category: Category,
        passed: bool,
    ) -> Result<bool, sqlx::Error> {
        let event = Event::AssessmentResult(AssessmentNote {
            session_id,
            category,
            passed,
        });
        self.write(
            db,
            To::Candidate(candidate),
            About::default(),
            &event,
            None,
            true,
        )
        .await
    }

    pub async fn guardian(
        &self,
        db: impl PgExecutor<'_>,
        candidate: Uuid,
        decision: GuardianOutcome,
    ) -> Result<bool, sqlx::Error> {
        let event = Event::GuardianDecision(GuardianNote { decision });
        self.write(
            db,
            To::Candidate(candidate),
            About::default(),
            &event,
            None,
            true,
        )
        .await
    }

    /// At most one alert per company and day.
    pub async fn webhook_failing(
        &self,
        db: impl PgExecutor<'_>,
        company: Uuid,
        note: WebhookNote,
        now: DateTime<Utc>,
    ) -> Result<bool, sqlx::Error> {
        let key = format!("webhook_failing:{}", now.date_naive());
        self.write(
            db,
            To::Company(company),
            About::default(),
            &Event::WebhookFailing(note),
            Some(&key),
            true,
        )
        .await
    }

    pub async fn trust_changed(
        &self,
        db: impl PgExecutor<'_>,
        company: Uuid,
        note: TrustNote,
    ) -> Result<bool, sqlx::Error> {
        self.write(
            db,
            To::Company(company),
            About::default(),
            &Event::TrustChanged(note),
            None,
            true,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_catalog_is_stable() {
        for (i, k) in NotificationKind::ALL.iter().enumerate() {
            assert_eq!(
                usize::from(k.index()),
                i,
                "{k:?}: bit = position, never renumbered"
            );
            assert_eq!(NotificationKind::from_slug(k.slug()), Some(*k));
            assert_eq!(serde_json::to_value(k).unwrap(), k.slug());
            assert_eq!(NotificationKind::from_index(k.index()), Some(*k));
        }
        assert_eq!(NotificationKind::from_index(12), None);
        let candidate = EmailMask::all_of(Audience::Candidate);
        let employer = EmailMask::all_of(Audience::Employer);
        assert_eq!(candidate & employer, 0);
        assert_eq!(candidate | employer, (1 << 12) - 1);
    }

    #[test]
    fn mask_turns_off_one_kind() {
        let mask = EmailMask(NotificationKind::InvitationExpiring.bit());
        assert!(!mask.allows(NotificationKind::InvitationExpiring));
        assert!(mask.allows(NotificationKind::InvitationReceived));
        assert!(EmailMask::default().allows(NotificationKind::TrustChanged));
    }

    #[test]
    fn payload_round_trips_with_its_kind() {
        let event = Event::InvitationAccepted(AcceptedNote {
            invitation_id: Uuid::nil(),
            candidate_id: Uuid::nil(),
            candidate_code: "K-7F3A2C".into(),
            title: "Backend".into(),
        });
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains(r#""kind":"invitation_accepted""#), "{json}");
        assert_eq!(Event::parse(&json), Some(event.clone()));
        assert_eq!(event.kind(), NotificationKind::InvitationAccepted);
        let keys: Vec<String> =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&json)
                .unwrap()
                .keys()
                .cloned()
                .collect();
        assert_eq!(
            keys,
            [
                "candidate_code",
                "candidate_id",
                "invitation_id",
                "kind",
                "title"
            ],
            "no field could hold a name or contacts"
        );
    }
}
