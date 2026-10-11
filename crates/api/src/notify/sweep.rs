//! Notifications no request triggers: the reminder 48 hours before an unanswered invitation
//! expires, and a change of a company's trust level or quota (it moves with time and with what
//! candidates do, not with the company's own requests). Safe with several replicas: a reminder is
//! unique by `(user_id, dedup_key)`, a trust change is written under the row lock of
//! `company_trust_seen`.

use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use matching::trust::{TrustAssessment, TrustLevel, assess};
use sqlx::PgPool;

use super::{NotificationKind, Notifier, TrustNote};
use crate::state::AppState;

pub const REMIND_BEFORE: Duration = Duration::hours(48);
const SWEEP_EVERY: StdDuration = StdDuration::from_secs(300);
const REMINDER_BATCH: i64 = 500;

pub fn level_slug(level: TrustLevel) -> &'static str {
    match level {
        TrustLevel::Low => "low",
        TrustLevel::New => "new",
        TrustLevel::Standard => "standard",
        TrustLevel::High => "high",
        TrustLevel::Frozen => "frozen",
    }
}

fn parse_level(s: &str) -> Option<TrustLevel> {
    [
        TrustLevel::Low,
        TrustLevel::New,
        TrustLevel::Standard,
        TrustLevel::High,
        TrustLevel::Frozen,
    ]
    .into_iter()
    .find(|l| level_slug(*l) == s)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TrustStep {
    /// Nothing to compare with — the first look, or a level this build does not know: the current
    /// assessment is recorded silently.
    Baseline,
    Same,
    Announce(TrustNote),
}

/// `seen`: the level slug and quota the company was last told about.
pub(crate) fn trust_step(seen: Option<(&str, i16)>, current: &TrustAssessment) -> TrustStep {
    let Some((level, quota)) = seen else {
        return TrustStep::Baseline;
    };
    if level == level_slug(current.level) && quota == current.quota {
        return TrustStep::Same;
    }
    match parse_level(level) {
        Some(before) => TrustStep::Announce(TrustNote {
            level_before: before,
            level: current.level,
            quota_before: quota,
            quota: current.quota,
        }),
        None => TrustStep::Baseline,
    }
}

/// Writes the missing reminders; returns how many this call wrote. Two sweepers racing over the
/// same invitation write one: the second insert hits the unique key and does nothing.
pub async fn remind_expiring(pool: &PgPool, notifier: Notifier) -> Result<usize, sqlx::Error> {
    let due = db::notifications::expiring_invitations(
        pool,
        REMIND_BEFORE.num_seconds() as f64,
        REMINDER_BATCH,
    )
    .await?;
    if due.is_empty() {
        return Ok(0);
    }
    let mut conn = pool.acquire().await?;
    let mut written = 0;
    for b in &due {
        if notifier
            .invitation_from(&mut conn, NotificationKind::InvitationExpiring, b)
            .await?
        {
            written += 1;
        }
    }
    Ok(written)
}

/// Compares every company's current assessment with the one it was last told about. The first
/// look only records the baseline.
pub async fn check_trust(
    pool: &PgPool,
    notifier: Notifier,
    now: DateTime<Utc>,
) -> Result<usize, sqlx::Error> {
    let mut changed = 0;
    for row in db::trust::company_signals(pool, None).await? {
        let current = assess(&crate::trust::signals(&row), now.timestamp());
        let level = level_slug(current.level);
        let mut tx = pool.begin().await?;
        let seen = db::notifications::trust_seen_for_update(&mut *tx, row.company_id).await?;
        match trust_step(seen.as_ref().map(|s| (s.level.as_str(), s.quota)), &current) {
            TrustStep::Same => continue,
            TrustStep::Announce(note) => {
                notifier
                    .trust_changed(&mut *tx, row.company_id, note)
                    .await?;
                changed += 1;
            }
            TrustStep::Baseline => {}
        }
        db::notifications::upsert_trust_seen(&mut *tx, row.company_id, level, current.quota)
            .await?;
        tx.commit().await?;
    }
    Ok(changed)
}

/// At startup and every 5 minutes: a reminder 48 hours ahead tolerates minutes of delay.
pub fn spawn(state: &AppState) {
    let pool = state.pool.clone();
    let notifier = Notifier::new(state);
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(SWEEP_EVERY);
        loop {
            tick.tick().await;
            match remind_expiring(&pool, notifier).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(reminders = n, "invitation expiry reminders written"),
                Err(e) => tracing::warn!(error = %e, "invitation reminder sweep failed"),
            }
            match check_trust(&pool, notifier, Utc::now()).await {
                Ok(0) => {}
                Ok(n) => tracing::info!(companies = n, "trust level changes notified"),
                Err(e) => tracing::warn!(error = %e, "trust change sweep failed"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_slugs_match_the_api() {
        for l in [
            TrustLevel::Low,
            TrustLevel::New,
            TrustLevel::Standard,
            TrustLevel::High,
        ] {
            assert_eq!(serde_json::to_value(l).unwrap(), level_slug(l));
            assert_eq!(parse_level(level_slug(l)), Some(l));
        }
    }

    fn assessed(level: TrustLevel, quota: i16) -> TrustAssessment {
        TrustAssessment {
            level,
            quota,
            base: 10,
            bonus: 0,
            penalty: 0,
            reasons: Vec::new(),
            advice: Vec::new(),
        }
    }

    #[test]
    fn first_look_records_the_baseline_without_announcing() {
        assert_eq!(
            trust_step(None, &assessed(TrustLevel::New, 10)),
            TrustStep::Baseline
        );
    }

    #[test]
    fn unchanged_level_and_quota_announce_nothing() {
        assert_eq!(
            trust_step(Some(("new", 10)), &assessed(TrustLevel::New, 10)),
            TrustStep::Same
        );
    }

    #[test]
    fn a_quota_change_is_announced_from_what_the_company_was_told() {
        assert_eq!(
            trust_step(Some(("new", 10)), &assessed(TrustLevel::New, 15)),
            TrustStep::Announce(TrustNote {
                level_before: TrustLevel::New,
                level: TrustLevel::New,
                quota_before: 10,
                quota: 15,
            })
        );
        assert_eq!(
            trust_step(Some(("standard", 20)), &assessed(TrustLevel::Low, 20)),
            TrustStep::Announce(TrustNote {
                level_before: TrustLevel::Standard,
                level: TrustLevel::Low,
                quota_before: 20,
                quota: 20,
            })
        );
    }

    #[test]
    fn an_unknown_stored_level_is_replaced_silently() {
        assert_eq!(
            trust_step(Some(("legacy", 10)), &assessed(TrustLevel::New, 15)),
            TrustStep::Baseline
        );
    }
}
