//! Delivers the outbox. Every API replica runs one dispatcher; `claim_due_events` leases rows
//! with `FOR UPDATE SKIP LOCKED`, so replicas never send the same event concurrently, and a
//! replica that dies mid-delivery only delays its batch until the lease runs out.

use std::{sync::Arc, time::Duration as StdDuration};

use chrono::{DateTime, Duration, Utc};
use futures_util::future::join_all;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    events::open_payload,
    open_secret, signature,
    webhook::{Attempt, SignedRequest, WebhookSender},
};
use crate::crypto::Crypto;

/// Delay after the n-th failed attempt: about 1 m, 5 m, 30 m, 2 h, then every 12 h until the
/// 24-hour window closes — six attempts in total for an endpoint that stays down.
const SCHEDULE_SECS: [i64; 5] = [60, 300, 1_800, 7_200, 43_200];
pub const LEASE: StdDuration = StdDuration::from_secs(60);
const BATCH: i64 = 16;
const DEFAULT_INTERVAL: StdDuration = StdDuration::from_secs(2);
/// Settled events (and their payloads with contacts) are kept this long for the delivery log.
pub const RETENTION_DAYS: i64 = 30;
const PURGE_EVERY: StdDuration = StdDuration::from_secs(3_600);
/// How often the pending-events gauge is refreshed: the Prometheus scrape interval.
const GAUGE_EVERY: StdDuration = StdDuration::from_secs(15);
/// After a rotation the old secret keeps signing for this long.
pub const ROTATION_OVERLAP_HOURS: i64 = 24;

pub fn backoff(failed_attempts: u16) -> Duration {
    let i = usize::from(failed_attempts.max(1) - 1).min(SCHEDULE_SECS.len() - 1);
    Duration::seconds(SCHEDULE_SECS[i])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    RetryAt(DateTime<Utc>),
    GiveUp,
}

pub fn next_step(failed_attempts: u16, now: DateTime<Utc>, deadline: DateTime<Utc>) -> Next {
    let at = now + backoff(failed_attempts);
    if at > deadline {
        Next::GiveUp
    } else {
        Next::RetryAt(at)
    }
}

#[derive(Clone)]
pub struct Dispatcher {
    pub pool: PgPool,
    pub crypto: Arc<Crypto>,
    pub sender: WebhookSender,
    pub interval: StdDuration,
}

impl Dispatcher {
    pub fn new(
        pool: PgPool,
        crypto: Arc<Crypto>,
        sender: WebhookSender,
        interval: Option<StdDuration>,
    ) -> Self {
        Self {
            pool,
            crypto,
            sender,
            interval: interval.unwrap_or(DEFAULT_INTERVAL),
        }
    }

    pub fn spawn(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut last_purge: Option<tokio::time::Instant> = None;
            let mut last_gauge: Option<tokio::time::Instant> = None;
            loop {
                if last_gauge.is_none_or(|t| t.elapsed() >= GAUGE_EVERY) {
                    last_gauge = Some(tokio::time::Instant::now());
                    match db::integrations::pending_events(&self.pool).await {
                        Ok(n) => metrics::gauge!("webhook_outbox_pending").set(n as f64),
                        Err(e) => tracing::warn!(error = %e, "pending events count failed"),
                    }
                }
                if last_purge.is_none_or(|t| t.elapsed() >= PURGE_EVERY) {
                    last_purge = Some(tokio::time::Instant::now());
                    let before = Utc::now() - Duration::days(RETENTION_DAYS);
                    match db::integrations::purge_settled_events(&self.pool, before).await {
                        Ok(0) => {}
                        Ok(n) => tracing::info!(events = n, "settled integration events purged"),
                        Err(e) => tracing::warn!(error = %e, "integration events purge failed"),
                    }
                }
                match self.run_once().await {
                    Ok(n) if n as i64 == BATCH => continue,
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, "webhook dispatch failed"),
                }
                tokio::time::sleep(self.interval).await;
            }
        })
    }

    /// One claim-and-deliver pass; returns how many events were claimed.
    pub async fn run_once(&self) -> Result<usize, sqlx::Error> {
        let events =
            db::integrations::claim_due_events(&self.pool, BATCH, LEASE.as_secs_f64()).await?;
        let n = events.len();
        let results = join_all(events.into_iter().map(|e| self.deliver(e))).await;
        for r in results {
            r?;
        }
        Ok(n)
    }

    /// Sends one leased event and records the attempt; returns it for the inline test delivery.
    pub async fn deliver(
        &self,
        e: db::integrations::DeliverableEvent,
    ) -> Result<Option<Attempt>, sqlx::Error> {
        let (Some(url), Some(secret_enc)) = (e.url.as_deref(), e.secret_enc.as_deref()) else {
            db::integrations::mark_failed(&self.pool, e.id, "Вебхук не настроен").await?;
            delivered("failed");
            return Ok(None);
        };
        let now = Utc::now();
        let prepared = (|| {
            let body = open_payload(&self.crypto, e.id, &e.payload_enc)?;
            let mut secrets = vec![open_secret(&self.crypto, e.company_id, secret_enc)?];
            if let (Some(prev), Some(until)) = (e.prev_secret_enc.as_deref(), e.prev_expires_at)
                && until > now
            {
                secrets.push(open_secret(&self.crypto, e.company_id, prev)?);
            }
            Ok::<_, crate::crypto::CryptoError>((body, secrets))
        })();
        let (body, secrets) = match prepared {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(event = %e.id, error = %err, "integration event payload unreadable");
                db::integrations::mark_failed(&self.pool, e.id, "Не удалось подготовить событие")
                    .await?;
                delivered("failed");
                return Ok(None);
            }
        };
        let keys: Vec<&[u8]> = secrets.iter().map(Vec::as_slice).collect();
        let header = signature::header_value(now.timestamp(), &keys, &body);
        let event_id = e.id.to_string();
        let attempt = self
            .sender
            .post(
                url,
                SignedRequest {
                    event_id: &event_id,
                    event_type: &e.event_type,
                    signature: &header,
                    body,
                },
            )
            .await;
        self.record(&e, url, &attempt).await?;
        Ok(Some(attempt))
    }

    async fn record(
        &self,
        e: &db::integrations::DeliverableEvent,
        url: &str,
        attempt: &Attempt,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        db::integrations::insert_attempt(
            &mut *tx,
            &db::integrations::NewAttempt {
                id: Uuid::now_v7(),
                event_id: e.id,
                url,
                status_code: attempt.status.map(|s| s as i16),
                latency_ms: attempt.latency_ms,
                error: attempt.error.as_deref(),
            },
        )
        .await?;
        let result = if attempt.ok() {
            db::integrations::mark_delivered(&mut *tx, e.id).await?;
            "delivered"
        } else {
            let error = attempt.error.as_deref().unwrap_or("Ошибка доставки");
            let failed = u16::try_from(e.attempts).unwrap_or(0).saturating_add(1);
            match next_step(failed, Utc::now(), e.deadline_at) {
                Next::RetryAt(at) => {
                    db::integrations::schedule_retry(&mut *tx, e.id, at, error).await?;
                    "retry"
                }
                Next::GiveUp => {
                    db::integrations::mark_failed(&mut *tx, e.id, error).await?;
                    "failed"
                }
            }
        };
        tx.commit().await?;
        delivered(result);
        Ok(())
    }
}

/// `webhook_deliveries_total{result}`: `delivered`, `retry` (failed, will be retried) or `failed`
/// (given up after the delivery window, or no webhook to deliver to).
fn delivered(result: &'static str) {
    metrics::counter!("webhook_deliveries_total", "result" => result).increment(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_schedule() {
        let mins: Vec<i64> = (1..=7).map(|n| backoff(n).num_minutes()).collect();
        assert_eq!(mins, [1, 5, 30, 120, 720, 720, 720]);
        assert_eq!(backoff(0), backoff(1));
    }

    #[test]
    fn gives_up_after_the_window() {
        let created = DateTime::from_timestamp(1_790_000_000, 0).unwrap();
        let deadline = created + Duration::hours(24);
        let mut now = created;
        let mut attempts = 1;
        let mut at = Vec::new();
        while let Next::RetryAt(t) = next_step(attempts, now, deadline) {
            at.push((t - created).num_minutes());
            now = t;
            attempts += 1;
        }
        assert_eq!(
            at,
            [1, 6, 36, 156, 876],
            "retries at 1 m, 6 m, 36 m, 2 h 36 m, 14 h 36 m"
        );
        assert_eq!(attempts, 6, "six attempts in total, then failed");
        assert!(matches!(next_step(1, deadline, deadline), Next::GiveUp));
    }
}
