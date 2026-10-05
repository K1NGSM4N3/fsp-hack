//! Webhook signatures: `FSP-Signature: t=<unix seconds>,v1=<hex>[,v1=<hex>]`, where
//! `v1 = HMAC-SHA256(secret, "<t>." + body)`. The timestamp is inside the MAC, so a receiver that
//! rejects stale `t` also rejects a captured delivery replayed later. After a rotation the header
//! carries one `v1` per valid secret, so the ATS can switch keys without losing events.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64URL};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;

pub const SIGNATURE_HEADER: &str = "fsp-signature";
pub const EVENT_ID_HEADER: &str = "fsp-event-id";
pub const EVENT_TYPE_HEADER: &str = "fsp-event-type";
/// Receivers should refuse deliveries signed more than five minutes away from their clock.
pub const TOLERANCE_SECS: i64 = 300;
pub const SECRET_PREFIX: &str = "whsec_";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SignatureError {
    #[error("malformed signature header")]
    Malformed,
    #[error("timestamp outside the tolerance window")]
    Stale,
    #[error("no signature matches")]
    Mismatch,
}

pub fn new_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    format!("{SECRET_PREFIX}{}", B64URL.encode(bytes))
}

/// What the cabinet shows after the one-time reveal.
pub fn secret_hint(secret: &str) -> String {
    let visible: String = secret.chars().take(SECRET_PREFIX.len() + 4).collect();
    format!("{visible}…")
}

fn mac(secret: &[u8], timestamp: i64, body: &[u8]) -> Hmac<Sha256> {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret).expect("HMAC accepts any key size");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(body);
    mac
}

pub fn sign(secret: &[u8], timestamp: i64, body: &[u8]) -> [u8; 32] {
    mac(secret, timestamp, body).finalize().into_bytes().into()
}

pub fn header_value(timestamp: i64, secrets: &[&[u8]], body: &[u8]) -> String {
    let mut out = format!("t={timestamp}");
    for secret in secrets {
        out.push_str(",v1=");
        out.push_str(&hex(&sign(secret, timestamp, body)));
    }
    out
}

/// Reference check for receivers; the ATS mock implements the same steps independently.
pub fn verify(
    secret: &[u8],
    header: &str,
    body: &[u8],
    now: i64,
    tolerance_secs: i64,
) -> Result<(), SignatureError> {
    let mut timestamp = None;
    let mut candidates: Vec<[u8; 32]> = Vec::with_capacity(2);
    for part in header.split(',') {
        match part.trim().split_once('=') {
            Some(("t", v)) => timestamp = v.parse::<i64>().ok(),
            Some(("v1", v)) => candidates.push(unhex(v).ok_or(SignatureError::Malformed)?),
            _ => {}
        }
    }
    let timestamp = timestamp.ok_or(SignatureError::Malformed)?;
    if candidates.is_empty() {
        return Err(SignatureError::Malformed);
    }
    if (now - timestamp).abs() > tolerance_secs {
        return Err(SignatureError::Stale);
    }
    let expected = mac(secret, timestamp, body);
    if candidates
        .iter()
        .any(|c| expected.clone().verify_slice(c).is_ok())
    {
        Ok(())
    } else {
        Err(SignatureError::Mismatch)
    }
}

pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[usize::from(b >> 4)] as char);
        out.push(DIGITS[usize::from(b & 0x0f)] as char);
    }
    out
}

fn unhex(s: &str) -> Option<[u8; 32]> {
    let s = s.as_bytes();
    if s.len() != 64 {
        return None;
    }
    let nibble = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    };
    let mut out = [0u8; 32];
    for (i, pair) in s.chunks_exact(2).enumerate() {
        out[i] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &[u8] = br#"{"id":"evt_1","type":"integration.test"}"#;
    const T: i64 = 1_700_000_000;

    #[test]
    fn matches_a_fixed_vector() {
        // printf '1700000000.{"id":"evt_1","type":"integration.test"}' \
        //   | openssl dgst -sha256 -hmac whsec_test_secret
        assert_eq!(
            hex(&sign(b"whsec_test_secret", T, BODY)),
            "e4bdd70ee8ed6ee3d34e5ed2e8f1aeb793d4fe67ed8c7db4b7dcfb72d1615086"
        );
        assert_eq!(
            header_value(T, &[b"whsec_test_secret", b"whsec_old_secret"], BODY),
            "t=1700000000,\
             v1=e4bdd70ee8ed6ee3d34e5ed2e8f1aeb793d4fe67ed8c7db4b7dcfb72d1615086,\
             v1=e70575151c8e5d25889bd251524dbb2b8612ad7101d904b4c32d6dfaf9ceafe9"
        );
    }

    #[test]
    fn verifies_and_rejects() {
        let header = header_value(T, &[b"new", b"old"], BODY);
        assert_eq!(
            verify(b"new", &header, BODY, T + 10, TOLERANCE_SECS),
            Ok(())
        );
        assert_eq!(
            verify(b"old", &header, BODY, T, TOLERANCE_SECS),
            Ok(()),
            "either signature of a rotation passes"
        );
        assert_eq!(
            verify(b"other", &header, BODY, T, TOLERANCE_SECS),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            verify(b"new", &header, b"{\"id\":\"evt_2\"}", T, TOLERANCE_SECS),
            Err(SignatureError::Mismatch),
            "a changed body fails"
        );
        assert_eq!(
            verify(
                b"new",
                &header,
                BODY,
                T + TOLERANCE_SECS + 1,
                TOLERANCE_SECS
            ),
            Err(SignatureError::Stale),
            "a replay after the window fails"
        );
        let shifted = header.replacen("t=1700000000", "t=1700000100", 1);
        assert_eq!(
            verify(b"new", &shifted, BODY, T + 100, TOLERANCE_SECS),
            Err(SignatureError::Mismatch),
            "the timestamp is covered by the MAC"
        );
        for bad in ["", "t=1", "v1=abc", "t=x,v1=00"] {
            assert_eq!(
                verify(b"new", bad, BODY, T, TOLERANCE_SECS),
                Err(SignatureError::Malformed),
                "{bad}"
            );
        }
    }

    #[test]
    fn secrets_are_random_and_hinted() {
        let a = new_secret();
        assert!(a.starts_with(SECRET_PREFIX) && a.len() == SECRET_PREFIX.len() + 43);
        assert_ne!(a, new_secret());
        let hint = secret_hint(&a);
        assert!(hint.starts_with(&a[..10]) && hint.ends_with('…'));
    }
}
