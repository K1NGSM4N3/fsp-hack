//! Email delivery of notifications. Every API replica runs one dispatcher; a pass has two steps.
//!
//! 1. **Assemble.** For each recipient with a due item, under a per-recipient advisory lock, all
//!    queued items (including those still in their hold) become one letter — unless the rate
//!    rule says wait, in which case they are postponed together and keep collecting.
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

/// Per recipient: one letter per `min_gap`, at most `daily_cap` in any 24 hours.
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
    pub kinds: Vec<NotificationKind>,
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
        let due = db::notifications::earliest_queued(&mut *tx, user).await?;
        if due.is_none_or(|d| d > now) {
            return Ok(Assembled::NotDue);
        }
        let history =
            db::notifications::email_history(&mut *tx, user, now - Duration::hours(24)).await?;
        let out = match slot(self.rule, now, &history) {
            Slot::Wait(at) => {
                db::notifications::defer_queued(&mut *tx, user, at).await?;
                Assembled::Deferred(at)
            }
            Slot::Now => {
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
        let mask = EmailMask(r.email_off);
        let items: Vec<Event> = db::notifications::email_items(pool, email_id)
            .await?
            .iter()
            .filter_map(|row| Event::parse(&row.payload))
            .filter(|e| mask.allows(e.kind()) && e.kind().audience() == audience)
            .collect();
        let Some(first) = items.first() else {
            return Ok(Composed::Skip("Письма этого типа отключены"));
        };
        let mut kinds: Vec<NotificationKind> = items.iter().map(Event::kind).collect();
        kinds.sort_unstable_by_key(|k| k.index());
        kinds.dedup();
        let scope = if items.iter().all(|e| e.kind() == first.kind()) {
            Scope::Kind(first.kind())
        } else {
            Scope::All
        };
        let label = match scope {
            Scope::Kind(k) => k.title(),
            Scope::All => "все письма",
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
                unsubscribe_label: label,
            },
        );
        Ok(Composed::Ready(Box::new(Outgoing {
            to: self.state.crypto.decrypt_str(&r.email_enc)?,
            letter,
            message_id: format!("<{email_id}@{}>", host(public_url)),
            list_unsubscribe: endpoint,
            kinds,
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
    fn message_id_host() {
        assert_eq!(host("https://talents.example.ru"), "talents.example.ru");
        assert_eq!(host("http://localhost:3000"), "localhost");
        assert_eq!(host("https://x.ru/base"), "x.ru");
        assert_eq!(host(""), "localhost");
    }
}
