//! Unsubscribe links. The token says whose email to turn off and which kind (or all), and is
//! signed so nobody can forge one for another account. It does not expire: a letter's
//! unsubscribe link keeps working, and using it twice changes nothing.
//!
//! Layout, base64url: `version(1) ‖ user id(16) ‖ scope(1) ‖ HMAC-SHA256(key, first 18 bytes)[..16]`.
//! Scope is the kind's index, `0xFF` — every email.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64URL};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use super::NotificationKind;

const VERSION: u8 = 1;
const ALL: u8 = 0xFF;
const SIGNED: usize = 18;
const TAG: usize = 16;
/// Purpose of the key derived from the blind-index secret (`Crypto::derive_key`).
pub const KEY_PURPOSE: &str = "notifications/unsubscribe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Kind(NotificationKind),
    All,
}

fn mac(key: &[u8; 32]) -> Hmac<Sha256> {
    <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC accepts any key size")
}

pub fn sign(key: &[u8; 32], user: Uuid, scope: Scope) -> String {
    let mut bytes = Vec::with_capacity(SIGNED + TAG);
    bytes.push(VERSION);
    bytes.extend_from_slice(user.as_bytes());
    bytes.push(match scope {
        Scope::Kind(k) => k.index(),
        Scope::All => ALL,
    });
    let mut m = mac(key);
    m.update(&bytes);
    bytes.extend_from_slice(&m.finalize().into_bytes()[..TAG]);
    B64URL.encode(bytes)
}

/// `None` for anything not signed by us, including a token cut short or padded.
pub fn verify(key: &[u8; 32], token: &str) -> Option<(Uuid, Scope)> {
    let bytes = B64URL.decode(token.trim()).ok()?;
    if bytes.len() != SIGNED + TAG || bytes[0] != VERSION {
        return None;
    }
    let (signed, tag) = bytes.split_at(SIGNED);
    let mut m = mac(key);
    m.update(signed);
    m.verify_truncated_left(tag).ok()?;
    let user = Uuid::from_slice(&signed[1..17]).ok()?;
    let scope = match signed[17] {
        ALL => Scope::All,
        i => Scope::Kind(NotificationKind::from_index(i)?),
    };
    Some((user, scope))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; 32] = [5; 32];

    #[test]
    fn round_trip() {
        let user = Uuid::now_v7();
        for scope in [
            Scope::All,
            Scope::Kind(NotificationKind::InvitationReceived),
            Scope::Kind(NotificationKind::TrustChanged),
        ] {
            let t = sign(&KEY, user, scope);
            assert!(
                t.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            );
            assert_eq!(verify(&KEY, &t), Some((user, scope)));
        }
    }

    #[test]
    fn tampering_is_rejected() {
        let user = Uuid::now_v7();
        let token = sign(
            &KEY,
            user,
            Scope::Kind(NotificationKind::InvitationExpiring),
        );
        let mut bytes = B64URL.decode(&token).unwrap();
        for i in 0..bytes.len() {
            bytes[i] ^= 1;
            assert_eq!(
                verify(&KEY, &B64URL.encode(&bytes)),
                None,
                "byte {i} flipped"
            );
            bytes[i] ^= 1;
        }
        let other = Uuid::now_v7();
        bytes[1..17].copy_from_slice(other.as_bytes());
        assert_eq!(
            verify(&KEY, &B64URL.encode(&bytes)),
            None,
            "another account"
        );
        assert_eq!(verify(&[6; 32], &token), None, "another key");
        assert_eq!(verify(&KEY, &token[..token.len() - 2]), None, "cut short");
        assert_eq!(verify(&KEY, &format!("{token}AA")), None, "padded");
        assert_eq!(verify(&KEY, "not a token"), None);
        assert_eq!(verify(&KEY, ""), None);
    }

    #[test]
    fn unknown_kind_index_is_rejected_even_if_signed() {
        let mut bytes = vec![VERSION];
        bytes.extend_from_slice(Uuid::nil().as_bytes());
        bytes.push(200);
        let mut m = mac(&KEY);
        m.update(&bytes);
        bytes.extend_from_slice(&m.finalize().into_bytes()[..TAG]);
        assert_eq!(verify(&KEY, &B64URL.encode(bytes)), None);
    }
}
