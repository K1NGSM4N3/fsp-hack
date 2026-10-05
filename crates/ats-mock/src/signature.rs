//! Receiver side of `FSP-Signature: t=<unix>,v1=<hex>[,v1=<hex>]`, written from the published
//! description rather than shared with the platform's signer, so the end-to-end test checks the
//! wire format and not one function against itself.

use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

pub const TOLERANCE_SECS: i64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Valid,
    Missing,
    Malformed,
    Stale,
    Mismatch,
}

impl Verdict {
    pub const fn label_ru(self) -> &'static str {
        match self {
            Verdict::Valid => "подпись верна",
            Verdict::Missing => "нет подписи",
            Verdict::Malformed => "подпись повреждена",
            Verdict::Stale => "устаревшая метка времени",
            Verdict::Mismatch => "подпись не совпала",
        }
    }
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

pub fn verify(secret: &str, header: Option<&str>, body: &[u8], now: i64) -> Verdict {
    let Some(header) = header else {
        return Verdict::Missing;
    };
    let mut timestamp: Option<i64> = None;
    let mut signatures = Vec::new();
    for part in header.split(',') {
        match part.trim().split_once('=') {
            Some(("t", v)) => timestamp = v.parse().ok(),
            Some(("v1", v)) => match unhex(v) {
                Some(bytes) => signatures.push(bytes),
                None => return Verdict::Malformed,
            },
            _ => {}
        }
    }
    let Some(timestamp) = timestamp else {
        return Verdict::Malformed;
    };
    if signatures.is_empty() {
        return Verdict::Malformed;
    }
    if (now - timestamp).abs() > TOLERANCE_SECS {
        return Verdict::Stale;
    }
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return Verdict::Malformed;
    };
    mac.update(format!("{timestamp}.").as_bytes());
    mac.update(body);
    if signatures
        .iter()
        .any(|s| mac.clone().verify_slice(s).is_ok())
    {
        Verdict::Valid
    } else {
        Verdict::Mismatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &[u8] = br#"{"id":"evt_1","type":"integration.test"}"#;
    const HEADER: &str =
        "t=1700000000,v1=e4bdd70ee8ed6ee3d34e5ed2e8f1aeb793d4fe67ed8c7db4b7dcfb72d1615086";

    #[test]
    fn fixed_vector() {
        assert_eq!(
            verify("whsec_test_secret", Some(HEADER), BODY, 1_700_000_100),
            Verdict::Valid
        );
        assert_eq!(
            verify("whsec_other", Some(HEADER), BODY, 1_700_000_100),
            Verdict::Mismatch
        );
        assert_eq!(
            verify("whsec_test_secret", Some(HEADER), b"{}", 1_700_000_100),
            Verdict::Mismatch
        );
        assert_eq!(
            verify("whsec_test_secret", Some(HEADER), BODY, 1_700_000_301),
            Verdict::Stale
        );
        assert_eq!(verify("x", None, BODY, 0), Verdict::Missing);
        assert_eq!(verify("x", Some("v1=zz"), BODY, 0), Verdict::Malformed);
        assert_eq!(verify("x", Some("t=1"), BODY, 0), Verdict::Malformed);
    }

    #[test]
    fn any_of_several_signatures() {
        let rotated = format!("{HEADER},v1={}", "00".repeat(32));
        assert_eq!(
            verify("whsec_test_secret", Some(&rotated), BODY, 1_700_000_000),
            Verdict::Valid
        );
    }
}
