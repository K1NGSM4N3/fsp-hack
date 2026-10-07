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
use matching::trust::{TrustAssessment, TrustLevel};
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
    /// A moderator reviewed a disputed test and kept its result.
    ResultConfirmed = 12,
    /// A moderator voided the category a disputed test gave.
    ResultAnnulled = 13,
    /// Voided, plus a retest with the camera open right away.
    RetestOrdered = 14,
    VacancyUnpublished = 15,
    /// Invitations frozen by a moderator until a date.
    InvitationsFrozen = 16,
    /// A taken-down vacancy is published again, or a freeze ended early.
    SanctionLifted = 17,
    /// A complaint about the company, its vacancy or invitation did not hold up. Upheld ones
    /// arrive with the sanction they led to.
    ComplaintResolved = 18,
    /// The ФСП registry pushed a change of the candidate's achievements.
    FspAchievements = 19,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    Candidate,
    Employer,
}

impl NotificationKind {
    pub const ALL: [NotificationKind; 20] = [
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
        NotificationKind::ResultConfirmed,
        NotificationKind::ResultAnnulled,
        NotificationKind::RetestOrdered,
        NotificationKind::VacancyUnpublished,
        NotificationKind::InvitationsFrozen,
        NotificationKind::SanctionLifted,
        NotificationKind::ComplaintResolved,
        NotificationKind::FspAchievements,
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
            NotificationKind::ResultConfirmed => "result_confirmed",
            NotificationKind::ResultAnnulled => "result_annulled",
            NotificationKind::RetestOrdered => "retest_ordered",
            NotificationKind::VacancyUnpublished => "vacancy_unpublished",
            NotificationKind::InvitationsFrozen => "invitations_frozen",
            NotificationKind::SanctionLifted => "sanction_lifted",
            NotificationKind::ComplaintResolved => "complaint_resolved",
            NotificationKind::FspAchievements => "fsp_achievements",
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
            | NotificationKind::GuardianDecision
            | NotificationKind::ResultConfirmed
            | NotificationKind::ResultAnnulled
            | NotificationKind::RetestOrdered
            | NotificationKind::FspAchievements => Audience::Candidate,
            NotificationKind::InvitationAccepted
            | NotificationKind::InvitationDeclined
            | NotificationKind::ApplicationReceived
            | NotificationKind::WebhookFailing
            | NotificationKind::TrustChanged
            | NotificationKind::VacancyUnpublished
            | NotificationKind::InvitationsFrozen
            | NotificationKind::SanctionLifted
            | NotificationKind::ComplaintResolved => Audience::Employer,
        }
    }

    /// Goes out as soon as its hold ends, whatever the gap since the last letter: an invitation
    /// is the reason the candidate is here. The daily cap still holds.
    pub const fn skips_gap(self) -> bool {
        matches!(self, NotificationKind::InvitationReceived)
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
            NotificationKind::ResultConfirmed => "Модератор подтвердил результат теста",
            NotificationKind::ResultAnnulled => "Модератор аннулировал результат теста",
            NotificationKind::RetestOrdered => "Назначена пересдача с видеозаписью",
            NotificationKind::VacancyUnpublished => "Вакансия снята модератором",
            NotificationKind::InvitationsFrozen => "Приглашения приостановлены",
            NotificationKind::SanctionLifted => "Модератор отменил меру",
            NotificationKind::ComplaintResolved => "Жалобы на компанию",
            NotificationKind::FspAchievements => "Достижения ФСП",
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
            NotificationKind::ResultConfirmed => {
                "Спорный тест проверен, результат остаётся — с комментарием модератора"
            }
            NotificationKind::ResultAnnulled => {
                "Категория из теста снята — с комментарием модератора"
            }
            NotificationKind::RetestOrdered => {
                "Пройти тест снова можно сразу, но только с камерой — с комментарием модератора"
            }
            NotificationKind::VacancyUnpublished => {
                "С комментарием модератора; вернуть её может только модератор"
            }
            NotificationKind::InvitationsFrozen => {
                "До какой даты квота — 0, с комментарием модератора"
            }
            NotificationKind::SanctionLifted => {
                "Вакансия снова опубликована или приглашения доступны раньше срока"
            }
            NotificationKind::ComplaintResolved => {
                "Жалоба не подтвердилась и больше не снижает квоту; кто жаловался, не сообщаем"
            }
            NotificationKind::FspAchievements => {
                "Реестр ФСП прислал новые результаты — профиль обновился сам"
            }
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

/// A moderator's decision on the candidate's disputed test. The moderator is not named: the
/// candidate needs the decision and its reasons, not who took it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReviewNote {
    pub session_id: Uuid,
    /// The test: specialization × the grade it was taken for.
    pub category: Category,
    /// Written by the moderator for the candidate.
    pub comment: String,
    /// The candidate's category after the decision.
    pub category_now: Option<Category>,
    /// Confirmed, and the current category stands on this test: companies see «Результат
    /// подтверждён».
    pub badge: bool,
}

/// Company notes about moderation carry the company's own item, the moderator's comment and
/// counts — never who complained or what the complaint said.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnpublishedNote {
    pub vacancy_id: Uuid,
    pub title: String,
    pub comment: String,
    /// Complaints about the vacancy this decision upheld.
    pub complaints_upheld: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrozenNote {
    pub until: DateTime<Utc>,
    pub comment: String,
    pub complaints_upheld: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiftedNote {
    /// The vacancy published again; `None` — the freeze ended early.
    pub vacancy_id: Option<Uuid>,
    pub title: Option<String>,
    pub comment: String,
    /// Invitations per 7 days after the decision.
    pub quota: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplaintAbout {
    Company,
    Vacancy,
    Invitation,
}

/// A dismissed complaint. A vacancy is public and may be named; an invitation is not — it went
/// to one person, and naming it would name the complainant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplaintNote {
    pub about: ComplaintAbout,
    pub vacancy_id: Option<Uuid>,
    pub vacancy_title: Option<String>,
    pub quota: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FspNote {
    pub added: u16,
    pub removed: u16,
    /// The newest added achievement in words, e.g. «Призёр всероссийских соревнований по
    /// алгоритмическому программированию, 2026».
    pub latest: Option<String>,
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
    ResultConfirmed(ReviewNote),
    ResultAnnulled(ReviewNote),
    RetestOrdered(ReviewNote),
    VacancyUnpublished(UnpublishedNote),
    InvitationsFrozen(FrozenNote),
    SanctionLifted(LiftedNote),
    ComplaintResolved(ComplaintNote),
    FspAchievements(FspNote),
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
            Event::ResultConfirmed(_) => NotificationKind::ResultConfirmed,
            Event::ResultAnnulled(_) => NotificationKind::ResultAnnulled,
            Event::RetestOrdered(_) => NotificationKind::RetestOrdered,
            Event::VacancyUnpublished(_) => NotificationKind::VacancyUnpublished,
            Event::InvitationsFrozen(_) => NotificationKind::InvitationsFrozen,
            Event::SanctionLifted(_) => NotificationKind::SanctionLifted,
            Event::ComplaintResolved(_) => NotificationKind::ComplaintResolved,
            Event::FspAchievements(_) => NotificationKind::FspAchievements,
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
            Event::ResultConfirmed(n) | Event::ResultAnnulled(n) | Event::RetestOrdered(n) => {
                Some(n.session_id)
            }
            Event::VacancyUnpublished(n) => Some(n.vacancy_id),
            Event::SanctionLifted(n) => n.vacancy_id,
            Event::ComplaintResolved(n) => n.vacancy_id,
            Event::GuardianDecision(_)
            | Event::WebhookFailing(_)
            | Event::TrustChanged(_)
            | Event::InvitationsFrozen(_)
            | Event::FspAchievements(_) => None,
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

    /// A moderator's decision or a registry push about the candidate.
    pub async fn to_candidate(
        &self,
        db: impl PgExecutor<'_>,
        candidate: Uuid,
        event: &Event,
    ) -> Result<bool, sqlx::Error> {
        self.write(
            db,
            To::Candidate(candidate),
            About::default(),
            event,
            None,
            true,
        )
        .await
    }

    /// A moderator's action about the company. `trust` is the level and quota after the action,
    /// read in the same transaction: it becomes what the company was last told, so the sweeper
    /// does not announce the same change again as `trust_changed`.
    pub async fn moderation_to_company(
        &self,
        conn: &mut PgConnection,
        company: Uuid,
        event: &Event,
        trust: Option<&TrustAssessment>,
    ) -> Result<bool, sqlx::Error> {
        let about = About {
            candidate_id: None,
            company_id: Some(company),
        };
        let written = self
            .write(&mut *conn, To::Company(company), about, event, None, true)
            .await?;
        if let Some(t) = trust {
            db::notifications::upsert_trust_seen(
                &mut *conn,
                company,
                sweep::level_slug(t.level),
                t.quota,
            )
            .await?;
        }
        Ok(written)
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

/// The company's trust level and quota as this transaction sees them.
pub async fn trust_now(
    conn: &mut PgConnection,
    company: Uuid,
    now: DateTime<Utc>,
) -> Result<Option<TrustAssessment>, sqlx::Error> {
    Ok(db::trust::company_signals(&mut *conn, Some(company))
        .await?
        .first()
        .map(|row| matching::trust::assess(&crate::trust::signals(row), now.timestamp())))
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
        assert_eq!(NotificationKind::from_index(20), None);
        let candidate = EmailMask::all_of(Audience::Candidate);
        let employer = EmailMask::all_of(Audience::Employer);
        assert_eq!(candidate & employer, 0);
        assert_eq!(candidate | employer, (1 << 20) - 1);
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

    fn keys(event: &Event) -> Vec<String> {
        let mut keys: Vec<String> = serde_json::to_value(event)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        keys.sort_unstable();
        keys
    }

    const IDENTITY: [&str; 8] = [
        "name",
        "phone",
        "email",
        "telegram",
        "contact",
        "reporter",
        "complainant",
        "moderator",
    ];

    fn assert_no_identity(keys: &[String]) {
        for key in keys {
            assert!(
                IDENTITY.iter().all(|id| !key.contains(id)),
                "{key} could identify a person"
            );
        }
    }

    #[test]
    fn company_payloads_hold_no_candidate_or_complainant_identity() {
        let at = DateTime::from_timestamp(1_790_000_000, 0).unwrap();
        let cases: [(Event, &[&str]); 8] = [
            (
                Event::InvitationAccepted(AcceptedNote {
                    invitation_id: Uuid::nil(),
                    candidate_id: Uuid::nil(),
                    candidate_code: "K-7F3A2C".into(),
                    title: "Backend".into(),
                }),
                &[
                    "candidate_code",
                    "candidate_id",
                    "invitation_id",
                    "kind",
                    "title",
                ],
            ),
            (
                Event::InvitationDeclined(DeclinedNote {
                    invitation_id: Uuid::nil(),
                    candidate_code: "K-7F3A2C".into(),
                    title: "Backend".into(),
                    reason: Some(DeclineReason::Salary),
                    comment: None,
                }),
                &[
                    "candidate_code",
                    "comment",
                    "invitation_id",
                    "kind",
                    "reason",
                    "title",
                ],
            ),
            (
                Event::ApplicationReceived(ApplicantNote {
                    application_id: Uuid::nil(),
                    vacancy_id: Uuid::nil(),
                    vacancy_title: "Backend".into(),
                    candidate_id: Uuid::nil(),
                    candidate_code: "K-7F3A2C".into(),
                }),
                &[
                    "application_id",
                    "candidate_code",
                    "candidate_id",
                    "kind",
                    "vacancy_id",
                    "vacancy_title",
                ],
            ),
            (
                Event::ComplaintResolved(ComplaintNote {
                    about: ComplaintAbout::Invitation,
                    vacancy_id: None,
                    vacancy_title: None,
                    quota: 15,
                }),
                &["about", "kind", "quota", "vacancy_id", "vacancy_title"],
            ),
            (
                Event::VacancyUnpublished(UnpublishedNote {
                    vacancy_id: Uuid::nil(),
                    title: "Backend".into(),
                    comment: "Вилка не соответствует описанию".into(),
                    complaints_upheld: 2,
                }),
                &[
                    "comment",
                    "complaints_upheld",
                    "kind",
                    "title",
                    "vacancy_id",
                ],
            ),
            (
                Event::InvitationsFrozen(FrozenNote {
                    until: at,
                    comment: "Массовые приглашения".into(),
                    complaints_upheld: 3,
                }),
                &["comment", "complaints_upheld", "kind", "until"],
            ),
            (
                Event::SanctionLifted(LiftedNote {
                    vacancy_id: None,
                    title: None,
                    comment: "Ошибка модерации".into(),
                    quota: 20,
                }),
                &["comment", "kind", "quota", "title", "vacancy_id"],
            ),
            (
                Event::TrustChanged(TrustNote {
                    level_before: TrustLevel::New,
                    level: TrustLevel::Standard,
                    quota_before: 10,
                    quota: 20,
                }),
                &["kind", "level", "level_before", "quota", "quota_before"],
            ),
        ];
        for (event, expected) in cases {
            assert_eq!(event.kind().audience(), Audience::Employer);
            let keys = keys(&event);
            assert_eq!(keys, expected, "{:?}", event.kind());
            assert_no_identity(&keys);
        }
    }

    #[test]
    fn moderator_decisions_reach_the_candidate_without_the_moderator() {
        let note = ReviewNote {
            session_id: Uuid::nil(),
            category: Category::new(catalog::Specialization::Backend, catalog::Grade::Middle),
            comment: "Ответы совпадают с записью экрана".into(),
            category_now: None,
            badge: false,
        };
        for event in [
            Event::ResultConfirmed(note.clone()),
            Event::ResultAnnulled(note.clone()),
            Event::RetestOrdered(note),
        ] {
            assert_eq!(event.kind().audience(), Audience::Candidate);
            let keys = keys(&event);
            assert_eq!(
                keys,
                [
                    "badge",
                    "category",
                    "category_now",
                    "comment",
                    "kind",
                    "session_id"
                ]
            );
            assert_no_identity(&keys);
        }
    }
}
