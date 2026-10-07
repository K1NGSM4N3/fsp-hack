//! Email delivery of notifications. Every API replica runs one dispatcher; a pass has two steps.
//!
//! 1. **Assemble.** For each recipient with a due item, under a per-recipient advisory lock, all
//!    queued items (including those still in their hold) become one letter — unless the rate
//!    rule says wait, in which case they are postponed together and keep collecting. A new
//!    invitation is not postponed for the gap between letters: when its hold ends, the letter goes
//!    with everything queued so far.
//! 2. **Send.** Pending letters are leased with `FOR UPDATE SKIP LOCKED`, rendered and sent; the
//!    letter id is the Message-ID, so a retry after a crash is the same letter. Kinds the user
//!    turned off after the item was queued are dropped at this point.

use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::{
    Audience, EmailMask, Event, NotificationKind,
    render::{self, LetterContext},
    token::{self, Scope},
};
use crate::{auth::Role, mail::Email, state::AppState};

const LEASE: StdDuration = StdDuration::from_secs(60);
const BATCH: i64 = 16;
/// After the n-th failed send: 1 m, 5 m, 30 m, 2 h; the fifth failure is final.
const RETRY_SECS: [i64; 4] = [60, 300, 1_800, 7_200];
const MAX_ATTEMPTS: i16 = 5;
/// Kinds that skip the gap between letters (`NotificationKind::skips_gap`).
const URGENT: [&str; 1] = [NotificationKind::InvitationReceived.slug()];

/// Per recipient: one letter per `min_gap`, at most `daily_cap` in any 24 hours. A letter that
/// carries a new invitation skips the gap, not the cap.
#[derive(Debug, Clone, Copy)]
pub struct RateRule {
    pub min_gap: Duration,
    pub daily_cap: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Now,
    Wait(DateTime<Utc>),
}

/// When the next letter may go: after the gap from the last one, and — once the cap of the last
/// 24 hours is reached — after the oldest of them leaves the window.
pub fn slot(rule: RateRule, now: DateTime<Utc>, history: &db::notifications::EmailHistory) -> Slot {
    let mut at = now;
    if let Some(last) = history.last_at {
        at = at.max(last + rule.min_gap);
    }
    if history.sent >= rule.daily_cap
        && let Some(oldest) = history.oldest_at
    {
        at = at.max(oldest + Duration::hours(24));
    }
    if at <= now { Slot::Now } else { Slot::Wait(at) }
}

/// What a pass does with one recipient's queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Plan {
    NotDue,
    Letter,
    /// Everything queued moves to `until`; the kinds that skip the gap keep their own time unless
    /// the daily cap pushes them to `urgent_until`.
    Defer {
        until: DateTime<Utc>,
        urgent_until: Option<DateTime<Utc>>,
    },
}

pub(crate) fn plan(
    rule: RateRule,
    now: DateTime<Utc>,
    queued: &db::notifications::Queued,
    history: &db::notifications::EmailHistory,
) -> Plan {
    if queued.due.is_none_or(|d| d > now) {
        return Plan::NotDue;
    }
    let cap_only = RateRule {
        min_gap: Duration::zero(),
        ..rule
    };
    let urgent_due = queued.urgent_due.is_some_and(|d| d <= now);
    let rule = if urgent_due { cap_only } else { rule };
    match slot(rule, now, history) {
        Slot::Now => Plan::Letter,
        Slot::Wait(until) => Plan::Defer {
            until,
            urgent_until: match slot(cap_only, now, history) {
                Slot::Now => None,
                Slot::Wait(cap) => Some(cap),
            },
        },
    }
}

/// What a letter still carries when it is sent: the recipient's own kinds whose email is on, and
/// what its unsubscribe link turns off — that one kind, or every email when kinds are mixed.
/// `None`: nothing left to send.
pub(crate) fn letter_items(
    events: impl IntoIterator<Item = Event>,
    mask: EmailMask,
    audience: Audience,
) -> Option<(Vec<Event>, Scope)> {
    let items: Vec<Event> = events
        .into_iter()
        .filter(|e| mask.allows(e.kind()) && e.kind().audience() == audience)
        .collect();
    let first = items.first()?.kind();
    let scope = if items.iter().all(|e| e.kind() == first) {
        Scope::Kind(first)
    } else {
        Scope::All
    };
    Some((items, scope))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assembled {
    Letter(Uuid),
    Deferred(DateTime<Utc>),
    /// Nothing due: another replica took it, or every item is still in its hold.
    NotDue,
    /// Another replica holds this recipient right now.
    Busy,
}

/// A letter ready for SMTP.
#[derive(Debug, Clone)]
pub struct Outgoing {
    pub to: String,
    pub letter: render::Letter,
    pub message_id: String,
    /// API endpoint for the one-click POST.
    pub list_unsubscribe: String,
}

#[derive(Debug, Clone)]
pub enum Composed {
    Ready(Box<Outgoing>),
    /// Nothing to send any more: the reason goes to `last_error`.
    Skip(&'static str),
}

fn host(public_url: &str) -> &str {
    let rest = public_url
        .split_once("://")
        .map_or(public_url, |(_, rest)| rest);
    let end = rest.find(['/', ':']).unwrap_or(rest.len());
    let host = &rest[..end];
    if host.is_empty() { "localhost" } else { host }
}

pub fn unsubscribe_links(public_url: &str, token: &str) -> (String, String) {
    (
        format!("{public_url}/notifications/unsubscribe?token={token}"),
        format!("{public_url}/api/v1/notifications/unsubscribe?token={token}"),
    )
}

#[derive(Clone)]
pub struct EmailDispatcher {
    state: AppState,
    rule: RateRule,
    interval: StdDuration,
}

impl EmailDispatcher {
    pub fn new(state: &AppState) -> Self {
        let c = &state.config.notifications;
        Self {
            state: state.clone(),
            rule: RateRule {
                min_gap: Duration::from_std(c.min_gap).unwrap_or(Duration::minutes(10)),
                daily_cap: i64::from(c.daily_cap),
            },
            interval: c.interval,
        }
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.run_once().await {
                    Ok((_, sent)) if sent as i64 == BATCH => continue,
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, "notification email pass failed"),
                }
                tokio::time::sleep(self.interval).await;
            }
        })
    }

    /// Assembles due letters, then sends what is pending; returns both counts.
    pub async fn run_once(&self) -> Result<(usize, usize), sqlx::Error> {
        let assembled = self.assemble_due(Utc::now()).await?;
        let sent = self.send_due().await?;
        Ok((assembled, sent))
    }

    pub async fn assemble_due(&self, now: DateTime<Utc>) -> Result<usize, sqlx::Error> {
        let users = db::notifications::users_with_due_email(&self.state.pool, 64).await?;
        let mut letters = 0;
        for user in users {
            if let Assembled::Letter(_) = self.assemble(user, now).await? {
                letters += 1;
            }
        }
        Ok(letters)
    }

    pub async fn assemble(&self, user: Uuid, now: DateTime<Utc>) -> Result<Assembled, sqlx::Error> {
        let mut tx = self.state.pool.begin().await?;
        if !db::notifications::try_lock_recipient(&mut *tx, user).await? {
            return Ok(Assembled::Busy);
        }
        let queued = db::notifications::queued(&mut *tx, user, &URGENT).await?;
        let history =
            db::notifications::email_history(&mut *tx, user, now - Duration::hours(24)).await?;
        let out = match plan(self.rule, now, &queued, &history) {
            Plan::NotDue => return Ok(Assembled::NotDue),
            Plan::Defer {
                until,
                urgent_until,
            } => {
                db::notifications::defer_queued(&mut *tx, user, until, &URGENT, urgent_until)
                    .await?;
                Assembled::Deferred(until)
            }
            Plan::Letter => {
                let id = Uuid::now_v7();
                db::notifications::insert_email(&mut *tx, id, user).await?;
                db::notifications::attach_queued(&mut *tx, user, id).await?;
                Assembled::Letter(id)
            }
        };
        tx.commit().await?;
        Ok(out)
    }

    /// Renders a letter exactly as it would be sent, without sending it.
    pub async fn compose(&self, email_id: Uuid, user: Uuid) -> Result<Composed, ComposeError> {
        let pool = &self.state.pool;
        let Some(r) = db::notifications::recipient(pool, user).await? else {
            return Ok(Composed::Skip("Аккаунт удалён"));
        };
        if !r.deliverable {
            return Ok(Composed::Skip("Аккаунт удалён или почта не подтверждена"));
        }
        let audience = match Role::from(r.role) {
            Role::Candidate => Audience::Candidate,
            Role::Employer => Audience::Employer,
            Role::Admin => return Ok(Composed::Skip("Администратору письма не отправляются")),
        };
        let rows = db::notifications::email_items(pool, email_id).await?;
        let events = rows.iter().filter_map(|row| Event::parse(&row.payload));
        let Some((items, scope)) = letter_items(events, EmailMask(r.email_off), audience) else {
            return Ok(Composed::Skip("Письма этого типа отключены"));
        };
        let key = self.state.crypto.derive_key(token::KEY_PURPOSE);
        let public_url = &self.state.config.public_url;
        let (page, endpoint) = unsubscribe_links(public_url, &token::sign(&key, user, scope));
        let letter = render::letter(
            &items,
            &LetterContext {
                public_url,
                audience,
                unsubscribe_url: &page,
                unsubscribe_label: scope.label(),
            },
        );
        Ok(Composed::Ready(Box::new(Outgoing {
            to: self.state.crypto.decrypt_str(&r.email_enc)?,
            letter,
            message_id: format!("<{email_id}@{}>", host(public_url)),
            list_unsubscribe: endpoint,
        })))
    }

    pub async fn send_due(&self) -> Result<usize, sqlx::Error> {
        let due = db::notifications::claim_due_emails(&self.state.pool, BATCH, LEASE.as_secs_f64())
            .await?;
        let n = due.len();
        for email in due {
            self.deliver(email).await?;
        }
        Ok(n)
    }

    async fn deliver(&self, due: db::notifications::DueEmail) -> Result<(), sqlx::Error> {
        let pool = &self.state.pool;
        let out = match self.compose(due.id, due.user_id).await {
            Ok(Composed::Ready(out)) => out,
            Ok(Composed::Skip(reason)) => {
                db::notifications::settle_email(
                    pool,
                    due.id,
                    db::types::NotificationEmailStatus::Skipped,
                    reason,
                )
                .await?;
                return Ok(());
            }
            Err(ComposeError::Db(e)) => return Err(e),
            Err(ComposeError::Crypto(e)) => {
                tracing::error!(email = %due.id, error = %e, "notification recipient unreadable");
                db::notifications::settle_email(
                    pool,
                    due.id,
                    db::types::NotificationEmailStatus::Failed,
                    "Не удалось прочитать адрес",
                )
                .await?;
                return Ok(());
            }
        };
        let sent = self
            .state
            .mailer
            .send_email(&Email {
                to: &out.to,
                subject: &out.letter.subject,
                text: &out.letter.text,
                html: &out.letter.html,
                message_id: &out.message_id,
                list_unsubscribe: Some(&out.list_unsubscribe),
            })
            .await;
        match sent {
            Ok(()) => {
                db::notifications::mark_email_sent(pool, due.id).await?;
            }
            Err(e) => {
                let error = e.to_string();
                let failed = due.attempts.saturating_add(1);
                tracing::warn!(email = %due.id, attempt = failed, error = %error, "notification email not sent");
                if failed >= MAX_ATTEMPTS {
                    db::notifications::settle_email(
                        pool,
                        due.id,
                        db::types::NotificationEmailStatus::Failed,
                        &error,
                    )
                    .await?;
                } else {
                    let i = usize::try_from(failed - 1)
                        .unwrap_or(0)
                        .min(RETRY_SECS.len() - 1);
                    let at = Utc::now() + Duration::seconds(RETRY_SECS[i]);
                    db::notifications::schedule_email_retry(pool, due.id, at, &error).await?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ComposeError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Crypto(#[from] crate::crypto::CryptoError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notify::{AcceptedNote, InvitationNote};

    fn at(min: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_790_000_000, 0).unwrap() + Duration::minutes(min)
    }

    const RULE: RateRule = RateRule {
        min_gap: Duration::minutes(10),
        daily_cap: 3,
    };

    fn history(
        sent: i64,
        last: Option<i64>,
        oldest: Option<i64>,
    ) -> db::notifications::EmailHistory {
        db::notifications::EmailHistory {
            sent,
            last_at: last.map(at),
            oldest_at: oldest.map(at),
        }
    }

    #[test]
    fn first_letter_goes_at_once() {
        assert_eq!(slot(RULE, at(0), &history(0, None, None)), Slot::Now);
    }

    #[test]
    fn gap_between_letters() {
        assert_eq!(
            slot(RULE, at(5), &history(1, Some(0), Some(0))),
            Slot::Wait(at(10))
        );
        assert_eq!(slot(RULE, at(10), &history(1, Some(0), Some(0))), Slot::Now);
    }

    #[test]
    fn daily_cap_waits_for_the_oldest_to_leave_the_window() {
        let h = history(3, Some(100), Some(20));
        assert_eq!(slot(RULE, at(200), &h), Slot::Wait(at(20 + 24 * 60)));
        assert_eq!(slot(RULE, at(20 + 24 * 60), &h), Slot::Now);
    }

    #[test]
    fn urgent_kinds_are_the_ones_that_skip_the_gap() {
        let skipping: Vec<&str> = NotificationKind::ALL
            .iter()
            .filter(|k| k.skips_gap())
            .map(|k| k.slug())
            .collect();
        assert_eq!(skipping, URGENT);
    }

    #[test]
    fn message_id_host() {
        assert_eq!(host("https://talents.example.ru"), "talents.example.ru");
        assert_eq!(host("http://localhost:3000"), "localhost");
        assert_eq!(host("https://x.ru/base"), "x.ru");
        assert_eq!(host(""), "localhost");
    }

    fn queued(due: Option<i64>, urgent: Option<i64>) -> db::notifications::Queued {
        db::notifications::Queued {
            due: due.map(at),
            urgent_due: urgent.map(at),
        }
    }

    #[test]
    fn nothing_goes_while_every_item_is_in_its_hold() {
        let empty = history(0, None, None);
        assert_eq!(
            plan(RULE, at(0), &queued(Some(2), Some(2)), &empty),
            Plan::NotDue
        );
        assert_eq!(plan(RULE, at(0), &queued(None, None), &empty), Plan::NotDue);
    }

    #[test]
    fn first_due_item_makes_a_letter_at_once() {
        let empty = history(0, None, None);
        assert_eq!(
            plan(RULE, at(2), &queued(Some(2), None), &empty),
            Plan::Letter
        );
    }

    #[test]
    fn within_the_gap_the_queue_waits_for_it_and_urgent_items_keep_their_time() {
        let h = history(1, Some(0), Some(0));
        assert_eq!(
            plan(RULE, at(5), &queued(Some(5), None), &h),
            Plan::Defer {
                until: at(10),
                urgent_until: None
            }
        );
    }

    #[test]
    fn a_due_invitation_skips_the_gap_under_the_cap() {
        let h = history(1, Some(0), Some(0));
        assert_eq!(
            plan(RULE, at(5), &queued(Some(3), Some(5)), &h),
            Plan::Letter
        );
    }

    #[test]
    fn an_invitation_still_in_its_hold_does_not_lift_the_gap() {
        let h = history(1, Some(0), Some(0));
        assert_eq!(
            plan(RULE, at(5), &queued(Some(3), Some(8)), &h),
            Plan::Defer {
                until: at(10),
                urgent_until: None
            }
        );
    }

    #[test]
    fn daily_cap_holds_back_invitations_too_until_the_oldest_letter_leaves_the_window() {
        let h = history(3, Some(100), Some(20));
        let window_ends = at(20 + 24 * 60);
        assert_eq!(
            plan(RULE, at(200), &queued(Some(200), Some(200)), &h),
            Plan::Defer {
                until: window_ends,
                urgent_until: Some(window_ends)
            }
        );
        assert_eq!(
            plan(RULE, at(200), &queued(Some(200), None), &h),
            Plan::Defer {
                until: window_ends,
                urgent_until: Some(window_ends)
            }
        );
    }

    #[test]
    fn with_both_gap_and_cap_the_queue_waits_for_the_later_one() {
        let window_ends = at(20 + 24 * 60);
        let cap_later = history(3, Some(100), Some(20));
        assert_eq!(
            plan(RULE, at(105), &queued(Some(105), None), &cap_later),
            Plan::Defer {
                until: window_ends,
                urgent_until: Some(window_ends)
            }
        );
        let gap_later = history(3, Some(20 + 24 * 60 - 5), Some(20));
        assert_eq!(
            plan(
                RULE,
                at(20 + 24 * 60 - 4),
                &queued(Some(0), None),
                &gap_later
            ),
            Plan::Defer {
                until: at(20 + 24 * 60 + 5),
                urgent_until: Some(window_ends)
            }
        );
    }

    fn invitation(kind: NotificationKind) -> Event {
        let note = InvitationNote {
            invitation_id: Uuid::nil(),
            company_name: "Рога и копыта".into(),
            title: "Backend".into(),
            salary_min: 200_000,
            salary_max: 250_000,
            work_format: None,
            expires_at: at(0),
        };
        match kind {
            NotificationKind::InvitationReceived => Event::InvitationReceived(note),
            NotificationKind::InvitationExpiring => Event::InvitationExpiring(note),
            _ => Event::InvitationWithdrawn(note),
        }
    }

    fn accepted() -> Event {
        Event::InvitationAccepted(AcceptedNote {
            invitation_id: Uuid::nil(),
            candidate_id: Uuid::nil(),
            candidate_code: "K-7F3A2C".into(),
            title: "Backend".into(),
        })
    }

    fn kinds(items: &[Event]) -> Vec<NotificationKind> {
        items.iter().map(Event::kind).collect()
    }

    #[test]
    fn a_letter_of_one_kind_unsubscribes_from_that_kind() {
        let (items, scope) = letter_items(
            [
                invitation(NotificationKind::InvitationReceived),
                invitation(NotificationKind::InvitationReceived),
            ],
            EmailMask::default(),
            Audience::Candidate,
        )
        .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(scope, Scope::Kind(NotificationKind::InvitationReceived));
        assert_eq!(scope.label(), "Новые приглашения");
    }

    #[test]
    fn a_mixed_letter_unsubscribes_from_every_email() {
        let (items, scope) = letter_items(
            [
                invitation(NotificationKind::InvitationReceived),
                invitation(NotificationKind::InvitationWithdrawn),
            ],
            EmailMask::default(),
            Audience::Candidate,
        )
        .unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(scope, Scope::All);
        assert_eq!(scope.label(), "все письма");
    }

    #[test]
    fn kinds_turned_off_after_queueing_and_other_roles_items_are_dropped() {
        let (items, scope) = letter_items(
            [
                invitation(NotificationKind::InvitationWithdrawn),
                accepted(),
                invitation(NotificationKind::InvitationExpiring),
            ],
            EmailMask(NotificationKind::InvitationWithdrawn.bit()),
            Audience::Candidate,
        )
        .unwrap();
        assert_eq!(kinds(&items), [NotificationKind::InvitationExpiring]);
        assert_eq!(scope, Scope::Kind(NotificationKind::InvitationExpiring));
    }

    #[test]
    fn a_letter_with_every_kind_turned_off_is_skipped() {
        let events = [
            invitation(NotificationKind::InvitationReceived),
            invitation(NotificationKind::InvitationWithdrawn),
        ];
        assert_eq!(
            letter_items(
                events.clone(),
                EmailMask(EmailMask::all_of(Audience::Candidate)),
                Audience::Candidate
            ),
            None
        );
        assert_eq!(
            letter_items(events, EmailMask::default(), Audience::Employer),
            None
        );
    }

    #[test]
    fn unsubscribe_links_lead_to_the_page_and_to_the_one_click_endpoint() {
        let (page, endpoint) = unsubscribe_links("https://talents.example", "T0k-en_");
        assert_eq!(
            page,
            "https://talents.example/notifications/unsubscribe?token=T0k-en_"
        );
        assert_eq!(
            endpoint,
            "https://talents.example/api/v1/notifications/unsubscribe?token=T0k-en_"
        );
    }
}
