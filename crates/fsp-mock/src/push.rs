//! The registry's push to the platform: `POST <url>` with
//! `{event_id, type: "achievements.changed", fsp_id, occurred_at}` and
//! `FSP-Signature: t=<unix>,v1=hex(HMAC-SHA256(secret, "<t>." + body))` — the scheme the platform
//! uses for its own ATS webhooks, written here from that description rather than shared with the
//! platform's code, so a test checks the wire format and not one function against itself.

use std::time::Duration;

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

pub const EVENT_TYPE: &str = "achievements.changed";
const ATTEMPTS: u32 = 3;

/// Where pushes go: `FSP_REGISTRY_WEBHOOK_URL` and `FSP_REGISTRY_WEBHOOK_SECRET`.
#[derive(Debug, Clone)]
pub struct Push {
    pub url: String,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushReport {
    pub event_id: String,
    /// The platform's answer to the last attempt; `null` — it did not answer.
    pub status: Option<u16>,
    pub attempts: u32,
    pub error: Option<String>,
}

#[derive(Serialize)]
struct Body<'a> {
    event_id: &'a str,
    #[serde(rename = "type")]
    kind: &'a str,
    fsp_id: &'a str,
    occurred_at: String,
}

/// HMAC takes a key of any length, so this is `None` only in theory.
pub fn signature(secret: &str, timestamp: i64, body: &[u8]) -> Option<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(body);
    let digest = mac.finalize().into_bytes();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    Some(format!("t={timestamp},v1={hex}"))
}

/// One event, delivered with up to three attempts while the platform answers 5xx or 429 or does
/// not answer; every attempt carries the same `event_id`, so the platform applies it once.
pub async fn achievements_changed(
    http: &reqwest::Client,
    push: Option<&Push>,
    fsp_id: &str,
) -> Option<PushReport> {
    let push = push?;
    let event_id = format!("evt_{}", uuid::Uuid::now_v7().simple());
    let body = match serde_json::to_vec(&Body {
        event_id: &event_id,
        kind: EVENT_TYPE,
        fsp_id,
        occurred_at: Utc::now().to_rfc3339(),
    }) {
        Ok(b) => b,
        Err(e) => {
            return Some(PushReport {
                event_id,
                status: None,
                attempts: 0,
                error: Some(e.to_string()),
            });
        }
    };
    let mut report = PushReport {
        event_id,
        status: None,
        attempts: 0,
        error: None,
    };
    for attempt in 1..=ATTEMPTS {
        report.attempts = attempt;
        let sent = http
            .post(&push.url)
            .header("content-type", "application/json")
            .header(
                "fsp-signature",
                signature(&push.secret, Utc::now().timestamp(), &body).unwrap_or_default(),
            )
            .body(body.clone())
            .send()
            .await;
        match sent {
            Ok(r) => {
                let status = r.status();
                report.status = Some(status.as_u16());
                report.error = None;
                if !(status.is_server_error() || status.as_u16() == 429) {
                    break;
                }
            }
            Err(e) => {
                report.status = None;
                report.error = Some(e.to_string());
            }
        }
        if attempt < ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(500 * u64::from(attempt))).await;
        }
    }
    tracing::info!(
        event_id = %report.event_id,
        status = ?report.status,
        attempts = report.attempts,
        "achievements.changed pushed"
    );
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_like_the_platform_webhooks() {
        // The platform's fixed vector (crates/api/src/integrations/signature.rs):
        // printf '1700000000.{"id":"evt_1","type":"integration.test"}' \
        //   | openssl dgst -sha256 -hmac whsec_test_secret
        assert_eq!(
            signature(
                "whsec_test_secret",
                1_700_000_000,
                br#"{"id":"evt_1","type":"integration.test"}"#
            )
            .as_deref(),
            Some(
                "t=1700000000,v1=e4bdd70ee8ed6ee3d34e5ed2e8f1aeb793d4fe67ed8c7db4b7dcfb72d1615086"
            )
        );
    }
}
