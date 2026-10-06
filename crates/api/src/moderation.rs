//! Moderation (architecture.md §5.7, §11.1, §9.3): decisions on disputed test results and what
//! they do to the grade, anti-fake actions, and the audit log every action and every view of a
//! test recording lands in. Each write goes into the same transaction as its log row: an action
//! that was not logged did not happen.

use catalog::Category;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgConnection;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    notify::{
        self, ComplaintAbout, ComplaintNote, Event, FrozenNote, LiftedNote, Notifier, ReviewNote,
        UnpublishedNote,
    },
    proctoring::ProctoringMode,
    skills,
    state::AppState,
};

pub const COMMENT_MAX_CHARS: usize = 2000;
pub const FREEZE_MAX_DAYS: u16 = 90;
pub const PAGE_DEFAULT: i64 = 30;
pub const PAGE_MAX: i64 = 100;

/// The moderator's decision on a finished session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    /// «Подтвердить результат»: the flag clears, employers see «Результат подтверждён».
    Confirm,
    /// «Аннулировать результат»: the category this test gave is removed, the previous one returns.
    Annul,
    /// «Назначить пересдачу с видеозаписью»: annulled, plus an immediate retest that must use
    /// the camera.
    Retest,
}

impl From<ReviewDecision> for db::types::ReviewDecision {
    fn from(d: ReviewDecision) -> Self {
        match d {
            ReviewDecision::Confirm => Self::Confirm,
            ReviewDecision::Annul => Self::Annul,
            ReviewDecision::Retest => Self::Retest,
        }
    }
}

impl From<db::types::ReviewDecision> for ReviewDecision {
    fn from(d: db::types::ReviewDecision) -> Self {
        match d {
            db::types::ReviewDecision::Confirm => Self::Confirm,
            db::types::ReviewDecision::Annul => Self::Annul,
            db::types::ReviewDecision::Retest => Self::Retest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModerationAction {
    ConfirmResult,
    AnnulResult,
    OrderRetest,
    ViewRecording,
    Dismiss,
    UnpublishVacancy,
    FreezeInvitations,
    Restore,
}

impl From<ModerationAction> for db::types::ModerationAction {
    fn from(a: ModerationAction) -> Self {
        use db::types::ModerationAction as D;
        match a {
            ModerationAction::ConfirmResult => D::ConfirmResult,
            ModerationAction::AnnulResult => D::AnnulResult,
            ModerationAction::OrderRetest => D::OrderRetest,
            ModerationAction::ViewRecording => D::ViewRecording,
            ModerationAction::Dismiss => D::Dismiss,
            ModerationAction::UnpublishVacancy => D::UnpublishVacancy,
            ModerationAction::FreezeInvitations => D::FreezeInvitations,
            ModerationAction::Restore => D::Restore,
        }
    }
}

impl From<db::types::ModerationAction> for ModerationAction {
    fn from(a: db::types::ModerationAction) -> Self {
        use db::types::ModerationAction as D;
        match a {
            D::ConfirmResult => Self::ConfirmResult,
            D::AnnulResult => Self::AnnulResult,
            D::OrderRetest => Self::OrderRetest,
            D::ViewRecording => Self::ViewRecording,
            D::Dismiss => Self::Dismiss,
            D::UnpublishVacancy => Self::UnpublishVacancy,
            D::FreezeInvitations => Self::FreezeInvitations,
            D::Restore => Self::Restore,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModerationTarget {
    Session,
    Vacancy,
    Invitation,
    Company,
    Complaint,
}

impl From<ModerationTarget> for db::types::ModerationTarget {
    fn from(t: ModerationTarget) -> Self {
        use db::types::ModerationTarget as D;
        match t {
            ModerationTarget::Session => D::Session,
            ModerationTarget::Vacancy => D::Vacancy,
            ModerationTarget::Invitation => D::Invitation,
            ModerationTarget::Company => D::Company,
            ModerationTarget::Complaint => D::Complaint,
        }
    }
}

impl From<db::types::ModerationTarget> for ModerationTarget {
    fn from(t: db::types::ModerationTarget) -> Self {
        use db::types::ModerationTarget as D;
        match t {
            D::Session => Self::Session,
            D::Vacancy => Self::Vacancy,
            D::Invitation => Self::Invitation,
            D::Company => Self::Company,
            D::Complaint => Self::Complaint,
        }
    }
}

/// What a moderator does with an item of the anti-fake queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustAction {
    /// A false alarm: the flag stops counting against the company, the complaint is closed as
    /// unfounded, the company leaves the queue until its next signal.
    Dismiss,
    /// Take a published vacancy down; the employer cannot republish it.
    Unpublish,
    /// Quota 0 for `days` days for the company behind the item.
    Freeze,
    /// Undo a sanction: publish the taken-down vacancy again, or lift the freeze early.
    Restore,
}

/// Keyset position as an opaque string: `<unix microseconds>_<uuid>`.
pub fn encode_cursor(at: DateTime<Utc>, id: Uuid) -> String {
    format!("{}_{}", at.timestamp_micros(), id.simple())
}

pub fn decode_cursor(raw: Option<&str>) -> ApiResult<Option<db::moderation::Cursor>> {
    let Some(raw) = raw.filter(|r| !r.is_empty()) else {
        return Ok(None);
    };
    let invalid = || {
        ApiError::bad_request("invalid_query", "Некорректные параметры запроса")
            .with_detail("cursor: возьмите значение `next_cursor` из предыдущего ответа")
    };
    let (micros, id) = raw.split_once('_').ok_or_else(invalid)?;
    let micros: i64 = micros.parse().map_err(|_| invalid())?;
    let at = Utc.timestamp_micros(micros).single().ok_or_else(invalid)?;
    let id: Uuid = id.parse().map_err(|_| invalid())?;
    Ok(Some(db::moderation::Cursor { at, id }))
}

pub fn page_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(PAGE_DEFAULT).clamp(1, PAGE_MAX)
}

/// A comment is mandatory: the candidate or the company reads it, and the log keeps it.
pub fn comment(raw: &str) -> ApiResult<&str> {
    let c = raw.trim();
    if c.is_empty() {
        return Err(ApiError::field("comment", "Напишите комментарий к решению"));
    }
    if c.chars().count() > COMMENT_MAX_CHARS {
        return Err(ApiError::field("comment", "Не длиннее 2000 символов"));
    }
    Ok(c)
}

fn already_decided() -> ApiError {
    ApiError::conflict("already_decided", "По этому тесту уже есть решение")
}

/// Records a decision and applies it. `annul` and `retest` void the grade assignment of the
/// session: the category falls back to the latest assignment still in force (or none), and the
/// skills only this test confirmed are dropped. Returns whether the category changed.
pub async fn decide(
    state: &AppState,
    moderator: Uuid,
    session_id: Uuid,
    decision: ReviewDecision,
    comment: &str,
) -> ApiResult<bool> {
    let now = Utc::now();
    let mut tx = state.pool.begin().await?;
    let locked = db::moderation::lock_session(&mut *tx, session_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !matches!(
        locked.status,
        db::types::AssessmentStatus::Passed | db::types::AssessmentStatus::Failed
    ) {
        return Err(ApiError::conflict(
            "session_not_finished",
            "Решение принимается по завершённому тесту",
        ));
    }
    let s = db::moderation::review_session(&mut *tx, session_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if s.decision.is_some() {
        return Err(already_decided());
    }
    let in_force = s.assigned_at.is_some() && s.annulled_at.is_none();
    match decision {
        ReviewDecision::Confirm => {}
        ReviewDecision::Annul if !in_force => {
            return Err(ApiError::conflict(
                "nothing_to_annul",
                "Этот тест не присвоил грейд — аннулировать нечего",
            ));
        }
        ReviewDecision::Retest if !s.assignment_current => {
            return Err(ApiError::conflict(
                "retest_not_applicable",
                "Пересдача назначается по тесту, который дал текущую категорию",
            ));
        }
        ReviewDecision::Retest if state.config.proctoring.mode == ProctoringMode::Off => {
            return Err(ApiError::conflict(
                "proctoring_off",
                "Видеозапись отключена — пересдачу с камерой назначить нельзя",
            ));
        }
        ReviewDecision::Annul | ReviewDecision::Retest => {}
    }
    let inserted = db::moderation::insert_review(
        &mut *tx,
        session_id,
        s.candidate_id,
        decision.into(),
        comment,
        moderator,
        now,
    )
    .await?;
    if inserted == 0 {
        return Err(already_decided());
    }
    let category = format!("{}:{}", s.specialization, s.claimed_grade);
    let mut details = json!({ "category": category, "status": status_slug(s.status) });
    let mut changed = false;
    let mut category_now = match (&s.current_specialization, &s.current_grade) {
        (Some(spec), Some(grade)) => Category::parse(&format!("{spec}:{grade}")),
        _ => None,
    };
    if decision != ReviewDecision::Confirm {
        db::moderation::annul_assignment(&mut *tx, session_id, now).await?;
        let restored = restore_category(&mut tx, s.candidate_id).await?;
        changed = s.assignment_current;
        drop_voided_skills(&mut tx, s.candidate_id, session_id).await?;
        category_now = restored.as_deref().and_then(Category::parse);
        details["restored_category"] = json!(restored);
    }
    let action = match decision {
        ReviewDecision::Confirm => ModerationAction::ConfirmResult,
        ReviewDecision::Annul => ModerationAction::AnnulResult,
        ReviewDecision::Retest => ModerationAction::OrderRetest,
    };
    db::moderation::insert_action(
        &mut *tx,
        &db::moderation::NewAction {
            id: Uuid::now_v7(),
            moderator_id: moderator,
            action: action.into(),
            target_kind: db::types::ModerationTarget::Session,
            target_id: session_id,
            candidate_id: Some(s.candidate_id),
            company_id: None,
            comment: Some(comment),
            details: &details,
            at: now,
        },
    )
    .await?;
    let note = ReviewNote {
        session_id,
        category: Category::new(s.specialization.parse()?, s.claimed_grade.parse()?),
        comment: comment.to_owned(),
        category_now,
        badge: decision == ReviewDecision::Confirm && in_force && s.assignment_current,
    };
    let event = match decision {
        ReviewDecision::Confirm => Event::ResultConfirmed(note),
        ReviewDecision::Annul => Event::ResultAnnulled(note),
        ReviewDecision::Retest => Event::RetestOrdered(note),
    };
    Notifier::new(state)
        .to_candidate(&mut *tx, s.candidate_id, &event)
        .await?;
    tx.commit().await?;
    // Every decision changes a card (grade, badge or «результат подтверждён»), and cached cards must follow.
    state.candidates.invalidate().await;
    Ok(changed)
}

fn status_slug(s: db::types::AssessmentStatus) -> &'static str {
    match s {
        db::types::AssessmentStatus::InProgress => "in_progress",
        db::types::AssessmentStatus::Passed => "passed",
        db::types::AssessmentStatus::Failed => "failed",
        db::types::AssessmentStatus::Expired => "expired",
    }
}

/// The category is denormalized into `candidates` for search; it follows the latest assignment
/// that is still in force. Returns that category's key, if any.
async fn restore_category(tx: &mut PgConnection, candidate: Uuid) -> ApiResult<Option<String>> {
    match db::moderation::latest_live_assignment(&mut *tx, candidate).await? {
        Some(g) => {
            db::grades::set_candidate_category(
                &mut *tx,
                &g.specialization,
                &g.grade,
                g.theta,
                g.assigned_at,
                candidate,
            )
            .await?;
            Ok(Some(format!("{}:{}", g.specialization, g.grade)))
        }
        None => {
            db::moderation::clear_candidate_category(&mut *tx, candidate).await?;
            Ok(None)
        }
    }
}

fn template_skills(templates: &[String]) -> Vec<&'static str> {
    let bank = assessment::bank();
    let mut out: Vec<&'static str> = Vec::new();
    for t in templates.iter().filter_map(|id| bank.template(id)) {
        for s in t.skills.iter().copied().filter(|s| skills::is_known(s)) {
            if !out.contains(&s) {
                out.push(s);
            }
        }
    }
    out
}

/// Skills confirmed by the voided test and by no test still in force lose their `test` source.
async fn drop_voided_skills(
    tx: &mut PgConnection,
    candidate: Uuid,
    session: Uuid,
) -> ApiResult<()> {
    let voided = db::moderation::correct_templates(&mut *tx, session).await?;
    let kept = db::moderation::live_correct_templates(&mut *tx, candidate).await?;
    let kept = template_skills(&kept);
    let drop: Vec<String> = template_skills(&voided)
        .into_iter()
        .filter(|s| !kept.contains(s))
        .map(str::to_owned)
        .collect();
    if !drop.is_empty() {
        db::moderation::delete_test_skills(&mut *tx, candidate, &drop).await?;
    }
    Ok(())
}

/// The audit row for a recording a moderator is about to play. Written before the first byte
/// leaves: a view that was not logged must not happen.
pub async fn log_recording_view(
    state: &AppState,
    moderator: Uuid,
    session_id: Uuid,
    candidate_id: Uuid,
) -> ApiResult<()> {
    db::moderation::insert_action(
        &state.pool,
        &db::moderation::NewAction {
            id: Uuid::now_v7(),
            moderator_id: moderator,
            action: db::types::ModerationAction::ViewRecording,
            target_kind: db::types::ModerationTarget::Session,
            target_id: session_id,
            candidate_id: Some(candidate_id),
            company_id: None,
            comment: None,
            details: &json!({}),
            at: Utc::now(),
        },
    )
    .await?;
    Ok(())
}

/// The kind of item an anti-fake action targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustItemKind {
    Company,
    Vacancy,
    Invitation,
    Complaint,
}

impl From<TrustItemKind> for db::types::ModerationTarget {
    fn from(k: TrustItemKind) -> Self {
        match k {
            TrustItemKind::Company => Self::Company,
            TrustItemKind::Vacancy => Self::Vacancy,
            TrustItemKind::Invitation => Self::Invitation,
            TrustItemKind::Complaint => Self::Complaint,
        }
    }
}

fn not_applicable(text: &'static str) -> ApiError {
    ApiError::conflict("action_not_applicable", text)
}

/// What an anti-fake action changed, for the response and the log.
pub struct ActOutcome {
    pub company_id: Option<Uuid>,
    pub complaints_resolved: u64,
    pub frozen_until: Option<DateTime<Utc>>,
}

struct Logged<'a> {
    moderator: Uuid,
    comment: &'a str,
    now: DateTime<Utc>,
}

impl Logged<'_> {
    async fn write(
        &self,
        tx: &mut PgConnection,
        action: ModerationAction,
        kind: db::types::ModerationTarget,
        target: Uuid,
        company: Option<Uuid>,
        details: serde_json::Value,
    ) -> ApiResult<()> {
        db::moderation::insert_action(
            &mut *tx,
            &db::moderation::NewAction {
                id: Uuid::now_v7(),
                moderator_id: self.moderator,
                action: action.into(),
                target_kind: kind,
                target_id: target,
                candidate_id: None,
                company_id: company,
                comment: Some(self.comment),
                details: &details,
                at: self.now,
            },
        )
        .await?;
        Ok(())
    }
}

/// What the company hears about an action, built once the action is applied.
enum Notice {
    Unpublished {
        vacancy_id: Uuid,
        title: String,
        complaints: u64,
    },
    Frozen {
        until: DateTime<Utc>,
        complaints: u64,
    },
    VacancyRestored {
        vacancy_id: Uuid,
        title: String,
    },
    FreezeLifted,
    ComplaintDismissed {
        about: ComplaintAbout,
        vacancy: Option<(Uuid, String)>,
    },
}

impl Notice {
    fn event(self, comment: &str, quota: i16) -> Event {
        let comment = comment.to_owned();
        let upheld = |n: u64| u32::try_from(n).unwrap_or(u32::MAX);
        match self {
            Notice::Unpublished {
                vacancy_id,
                title,
                complaints,
            } => Event::VacancyUnpublished(UnpublishedNote {
                vacancy_id,
                title,
                comment,
                complaints_upheld: upheld(complaints),
            }),
            Notice::Frozen { until, complaints } => Event::InvitationsFrozen(FrozenNote {
                until,
                comment,
                complaints_upheld: upheld(complaints),
            }),
            Notice::VacancyRestored { vacancy_id, title } => Event::SanctionLifted(LiftedNote {
                vacancy_id: Some(vacancy_id),
                title: Some(title),
                comment,
                quota,
            }),
            Notice::FreezeLifted => Event::SanctionLifted(LiftedNote {
                vacancy_id: None,
                title: None,
                comment,
                quota,
            }),
            Notice::ComplaintDismissed { about, vacancy } => {
                let (vacancy_id, vacancy_title) = vacancy.unzip();
                Event::ComplaintResolved(ComplaintNote {
                    about,
                    vacancy_id,
                    vacancy_title,
                    quota,
                })
            }
        }
    }
}

fn complaint_about(target_kind: &str) -> ComplaintAbout {
    match target_kind {
        "vacancy" => ComplaintAbout::Vacancy,
        "invitation" => ComplaintAbout::Invitation,
        _ => ComplaintAbout::Company,
    }
}

/// Applies an anti-fake action to a queue item. Complaints a sanction answers are closed with
/// it: taking a vacancy down upholds the complaints about that vacancy, a freeze upholds those
/// about the company and its invitations (and the complaint it was taken from).
pub async fn act(
    state: &AppState,
    moderator: Uuid,
    kind: TrustItemKind,
    id: Uuid,
    action: TrustAction,
    comment: &str,
    days: Option<u16>,
) -> ApiResult<ActOutcome> {
    use db::types::{FlagReview, ModerationTarget as T};
    let now = Utc::now();
    let log = Logged {
        moderator,
        comment,
        now,
    };
    let mut tx = state.pool.begin().await?;
    let mut out = ActOutcome {
        company_id: None,
        complaints_resolved: 0,
        frozen_until: None,
    };
    let mut notice: Option<Notice> = None;
    match (action, kind) {
        (TrustAction::Dismiss, TrustItemKind::Vacancy) => {
            let v = db::moderation::vacancy_target(&mut *tx, id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            if db::moderation::review_vacancy_flags(&mut *tx, id, FlagReview::Dismissed, now)
                .await?
                == 0
            {
                return Err(not_applicable("У вакансии нет непросмотренных отметок"));
            }
            out.company_id = Some(v.company_id);
            log.write(
                &mut tx,
                ModerationAction::Dismiss,
                T::Vacancy,
                id,
                Some(v.company_id),
                json!({ "title": v.title }),
            )
            .await?;
        }
        (TrustAction::Dismiss, TrustItemKind::Invitation) => {
            let i = db::moderation::invitation_target(&mut *tx, id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            if db::moderation::review_invitation_flags(&mut *tx, id, FlagReview::Dismissed, now)
                .await?
                == 0
            {
                return Err(not_applicable("У приглашения нет непросмотренных отметок"));
            }
            out.company_id = Some(i.company_id);
            log.write(
                &mut tx,
                ModerationAction::Dismiss,
                T::Invitation,
                id,
                Some(i.company_id),
                json!({ "title": i.title }),
            )
            .await?;
        }
        (TrustAction::Dismiss, TrustItemKind::Company) => {
            let c = db::moderation::company_target(&mut *tx, id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            db::moderation::dismiss_company(&mut *tx, id, now).await?;
            out.company_id = Some(id);
            log.write(
                &mut tx,
                ModerationAction::Dismiss,
                T::Company,
                id,
                Some(id),
                json!({ "title": c.name }),
            )
            .await?;
        }
        (TrustAction::Dismiss, TrustItemKind::Complaint) => {
            let r = db::moderation::complaint_target(&mut *tx, id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            if db::moderation::resolve_complaint(&mut *tx, id, "dismissed", comment, moderator, now)
                .await?
                == 0
            {
                return Err(not_applicable("Жалоба уже рассмотрена"));
            }
            out.company_id = r.company_id;
            out.complaints_resolved = 1;
            let title = complaint_title(&mut tx, &r).await?;
            let about = complaint_about(&r.target_kind);
            notice = Some(Notice::ComplaintDismissed {
                about,
                vacancy: match (about, &title) {
                    (ComplaintAbout::Vacancy, Some(t)) => Some((r.target_id, t.clone())),
                    _ => None,
                },
            });
            log.write(
                &mut tx,
                ModerationAction::Dismiss,
                T::Complaint,
                id,
                r.company_id,
                json!({ "title": title, "complaint_target": r.target_kind }),
            )
            .await?;
        }
        (TrustAction::Unpublish, TrustItemKind::Vacancy | TrustItemKind::Complaint) => {
            let vacancy = match kind {
                TrustItemKind::Complaint => {
                    let r = db::moderation::complaint_target(&mut *tx, id)
                        .await?
                        .ok_or_else(ApiError::not_found)?;
                    if r.resolved {
                        return Err(not_applicable("Жалоба уже рассмотрена"));
                    }
                    if r.target_kind != "vacancy" {
                        return Err(not_applicable(
                            "Снять с публикации можно только вакансию: жалоба не на вакансию",
                        ));
                    }
                    r.target_id
                }
                _ => id,
            };
            let v = db::moderation::vacancy_target(&mut *tx, vacancy)
                .await?
                .ok_or_else(ApiError::not_found)?;
            if db::moderation::unpublish_vacancy(&mut *tx, vacancy, now).await? == 0 {
                return Err(not_applicable("Снять можно только опубликованную вакансию"));
            }
            out.company_id = Some(v.company_id);
            out.complaints_resolved = db::moderation::uphold_vacancy_complaints(
                &mut *tx,
                vacancy,
                "vacancy_unpublished",
                comment,
                moderator,
                now,
            )
            .await?;
            log.write(
                &mut tx,
                ModerationAction::UnpublishVacancy,
                T::Vacancy,
                vacancy,
                Some(v.company_id),
                json!({ "title": v.title, "complaints_resolved": out.complaints_resolved }),
            )
            .await?;
            notice = Some(Notice::Unpublished {
                vacancy_id: vacancy,
                title: v.title,
                complaints: out.complaints_resolved,
            });
        }
        (TrustAction::Freeze, _) => {
            let days = days
                .filter(|d| (1..=FREEZE_MAX_DAYS).contains(d))
                .ok_or_else(|| ApiError::field("days", "Срок приостановки — от 1 до 90 дней"))?;
            let company = match kind {
                TrustItemKind::Company => id,
                TrustItemKind::Vacancy => {
                    let v = db::moderation::vacancy_target(&mut *tx, id)
                        .await?
                        .ok_or_else(ApiError::not_found)?;
                    db::moderation::review_vacancy_flags(&mut *tx, id, FlagReview::Upheld, now)
                        .await?;
                    v.company_id
                }
                TrustItemKind::Invitation => {
                    let i = db::moderation::invitation_target(&mut *tx, id)
                        .await?
                        .ok_or_else(ApiError::not_found)?;
                    db::moderation::review_invitation_flags(&mut *tx, id, FlagReview::Upheld, now)
                        .await?;
                    i.company_id
                }
                TrustItemKind::Complaint => {
                    let r = db::moderation::complaint_target(&mut *tx, id)
                        .await?
                        .ok_or_else(ApiError::not_found)?;
                    if r.resolved {
                        return Err(not_applicable("Жалоба уже рассмотрена"));
                    }
                    let company = r
                        .company_id
                        .ok_or_else(|| not_applicable("Компании, на которую жалоба, больше нет"))?;
                    out.complaints_resolved += db::moderation::resolve_complaint(
                        &mut *tx,
                        id,
                        "invitations_frozen",
                        comment,
                        moderator,
                        now,
                    )
                    .await?;
                    company
                }
            };
            let c = db::moderation::company_target(&mut *tx, company)
                .await?
                .ok_or_else(ApiError::not_found)?;
            let until = now + Duration::days(i64::from(days));
            db::moderation::freeze_company(&mut *tx, company, now, until).await?;
            db::moderation::dismiss_company(&mut *tx, company, now).await?;
            out.complaints_resolved += db::moderation::uphold_company_complaints(
                &mut *tx,
                company,
                "invitations_frozen",
                comment,
                moderator,
                now,
            )
            .await?;
            out.company_id = Some(company);
            out.frozen_until = Some(until);
            log.write(
                &mut tx,
                ModerationAction::FreezeInvitations,
                T::Company,
                company,
                Some(company),
                json!({
                    "title": c.name,
                    "days": days,
                    "until": until,
                    "from": kind_slug(kind),
                    "item_id": id,
                    "complaints_resolved": out.complaints_resolved,
                }),
            )
            .await?;
            notice = Some(Notice::Frozen {
                until,
                complaints: out.complaints_resolved,
            });
        }
        (TrustAction::Restore, TrustItemKind::Vacancy) => {
            let v = db::moderation::vacancy_target(&mut *tx, id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            if db::moderation::restore_vacancy(&mut *tx, id, now).await? == 0 {
                return Err(not_applicable("Вакансия не снята модератором"));
            }
            out.company_id = Some(v.company_id);
            log.write(
                &mut tx,
                ModerationAction::Restore,
                T::Vacancy,
                id,
                Some(v.company_id),
                json!({ "title": v.title }),
            )
            .await?;
            notice = Some(Notice::VacancyRestored {
                vacancy_id: id,
                title: v.title,
            });
        }
        (TrustAction::Restore, TrustItemKind::Company) => {
            let c = db::moderation::company_target(&mut *tx, id)
                .await?
                .ok_or_else(ApiError::not_found)?;
            if db::moderation::unfreeze_company(&mut *tx, id).await? == 0 {
                return Err(not_applicable("Приглашения компании не приостановлены"));
            }
            out.company_id = Some(id);
            log.write(
                &mut tx,
                ModerationAction::Restore,
                T::Company,
                id,
                Some(id),
                json!({ "title": c.name }),
            )
            .await?;
            notice = Some(Notice::FreezeLifted);
        }
        (TrustAction::Unpublish, _) => {
            return Err(not_applicable(
                "Снять с публикации можно вакансию или вакансию из жалобы",
            ));
        }
        (TrustAction::Restore, _) => {
            return Err(not_applicable(
                "Вернуть можно снятую вакансию или приглашения компании",
            ));
        }
    }
    if let (Some(notice), Some(company)) = (notice, out.company_id) {
        let trust = notify::trust_now(&mut tx, company, now).await?;
        let event = notice.event(comment, trust.as_ref().map_or(0, |t| t.quota));
        Notifier::new(state)
            .moderation_to_company(&mut tx, company, &event, trust.as_ref())
            .await?;
    }
    tx.commit().await?;
    Ok(out)
}

fn kind_slug(k: TrustItemKind) -> &'static str {
    match k {
        TrustItemKind::Company => "company",
        TrustItemKind::Vacancy => "vacancy",
        TrustItemKind::Invitation => "invitation",
        TrustItemKind::Complaint => "complaint",
    }
}

async fn complaint_title(
    tx: &mut PgConnection,
    r: &db::moderation::ComplaintTarget,
) -> ApiResult<Option<String>> {
    Ok(match r.target_kind.as_str() {
        "vacancy" => db::moderation::vacancy_target(&mut *tx, r.target_id)
            .await?
            .map(|v| v.title),
        "invitation" => db::moderation::invitation_target(&mut *tx, r.target_id)
            .await?
            .map(|i| i.title),
        _ => db::moderation::company_target(&mut *tx, r.target_id)
            .await?
            .map(|c| c.name),
    })
}

/// What a decision means for the company, as its trust page says it. `details.title` is the
/// vacancy, invitation or company name at the time of the action.
pub fn company_notice(
    action: ModerationAction,
    target: ModerationTarget,
    details: &serde_json::Value,
) -> String {
    let title = details["title"].as_str().unwrap_or("");
    match (action, target) {
        (ModerationAction::UnpublishVacancy, _) => {
            format!("Вакансия «{title}» снята с публикации модератором")
        }
        (ModerationAction::Restore, ModerationTarget::Vacancy) => {
            format!("Вакансия «{title}» снова опубликована: модератор отменил снятие")
        }
        (ModerationAction::Restore, _) => "Приостановка приглашений снята досрочно".to_owned(),
        (ModerationAction::FreezeInvitations, _) => {
            let until = details["until"]
                .as_str()
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .map(|t| t.format("%d.%m.%Y").to_string())
                .unwrap_or_default();
            format!("Приглашения приостановлены до {until}: квота на это время — 0")
        }
        (ModerationAction::Dismiss, ModerationTarget::Vacancy) => {
            format!(
                "Отметка у вакансии «{title}» снята: нарушений нет, квоту она больше не снижает"
            )
        }
        (ModerationAction::Dismiss, ModerationTarget::Invitation) => {
            format!(
                "Отметка у приглашения «{title}» снята: нарушений нет, квоту она больше не снижает"
            )
        }
        (ModerationAction::Dismiss, ModerationTarget::Complaint) if title.is_empty() => {
            "Жалоба не подтвердилась и больше не снижает квоту".to_owned()
        }
        (ModerationAction::Dismiss, ModerationTarget::Complaint) => {
            format!("Жалоба на «{title}» не подтвердилась и больше не снижает квоту")
        }
        _ => "Решение модератора".to_owned(),
    }
}

/// A category key from the log details in words, e.g. «Бэкенд-разработка · Middle».
pub fn category_name(key: &str) -> Option<String> {
    Category::parse(key).map(Category::name_ru)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trip() {
        let at = Utc
            .timestamp_micros(1_790_000_000_123_456)
            .single()
            .unwrap();
        let id = Uuid::now_v7();
        let raw = encode_cursor(at, id);
        let c = decode_cursor(Some(&raw)).unwrap().unwrap();
        assert_eq!((c.at, c.id), (at, id));
        assert!(decode_cursor(None).unwrap().is_none());
        assert!(decode_cursor(Some("")).unwrap().is_none());
        for bad in ["x", "12_nope", "abc_0191f0000000700080000000000000a1"] {
            assert!(decode_cursor(Some(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn comment_is_required_and_bounded() {
        assert!(comment("   ").is_err());
        assert_eq!(comment("  Запись чистая  ").unwrap(), "Запись чистая");
        assert!(comment(&"я".repeat(2001)).is_err());
        assert!(comment(&"я".repeat(2000)).is_ok());
    }
}
