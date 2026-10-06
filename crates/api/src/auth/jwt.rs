use std::{
    hash::BuildHasher,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use arc_swap::ArcSwapOption;
use ed25519_dalek::{SigningKey, pkcs8::EncodePrivateKey};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Role;
use crate::crypto::CryptoError;

pub const ACCESS_TTL_SECS: i64 = 15 * 60;
const ISSUER: &str = "fsp-talent";
const LEEWAY_SECS: i64 = 5;
/// Recently verified tokens kept; a power of two, direct-mapped.
const SEEN_SLOTS: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: Role,
    pub jti: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}

/// What a valid access token carries.
#[derive(Debug, Clone, Copy)]
pub struct Verified {
    pub sub: Uuid,
    pub role: Role,
    pub jti: Uuid,
    pub exp: i64,
}

struct Seen {
    token: Box<str>,
    claims: Verified,
}

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    /// The Ed25519 check is the dearest step of an authenticated request (about 50 µs, a quarter
    /// of the CPU of a cached route). A client sends the same token for its whole 15 minutes, so
    /// a token whose exact text passed the check before is only checked for expiry.
    seen: Box<[ArcSwapOption<Seen>]>,
    hasher: std::hash::RandomState,
}

impl JwtKeys {
    /// Without a configured seed the key is ephemeral and every restart logs users out.
    pub fn new(seed: Option<[u8; 32]>) -> Result<Self, CryptoError> {
        let seed = seed.unwrap_or_else(|| {
            tracing::warn!("JWT_ED25519_SEED_B64 not set: using an ephemeral signing key");
            let mut s = [0u8; 32];
            rand::rng().fill_bytes(&mut s);
            s
        });
        let signing = SigningKey::from_bytes(&seed);
        let der = signing
            .to_pkcs8_der()
            .map_err(|e| CryptoError::KeyEncoding(e.to_string()))?;
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[ISSUER]);
        validation.leeway = LEEWAY_SECS as u64;
        Ok(Self {
            encoding: EncodingKey::from_ed_der(der.as_bytes()),
            decoding: DecodingKey::from_ed_der(signing.verifying_key().as_bytes()),
            validation,
            seen: (0..SEEN_SLOTS).map(|_| ArcSwapOption::empty()).collect(),
            hasher: std::hash::RandomState::new(),
        })
    }

    pub fn issue(
        &self,
        user_id: Uuid,
        role: Role,
        now: i64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            sub: user_id,
            role,
            jti: Uuid::now_v7(),
            iat: now,
            exp: now + ACCESS_TTL_SECS,
            iss: ISSUER.into(),
        };
        jsonwebtoken::encode(&Header::new(Algorithm::EdDSA), &claims, &self.encoding)
    }

    pub fn verify(&self, token: &str) -> Option<Verified> {
        let slot = &self.seen[self.hasher.hash_one(token) as usize & (SEEN_SLOTS - 1)];
        if let Some(seen) = &*slot.load()
            && same_text(&seen.token, token)
        {
            return (seen.claims.exp >= unix_now() - LEEWAY_SECS).then_some(seen.claims);
        }
        let claims = jsonwebtoken::decode::<Claims>(token, &self.decoding, &self.validation)
            .ok()?
            .claims;
        let verified = Verified {
            sub: claims.sub,
            role: claims.role,
            jti: claims.jti,
            exp: claims.exp,
        };
        slot.store(Some(Arc::new(Seen {
            token: token.into(),
            claims: verified,
        })));
        Some(verified)
    }
}

/// Without an early exit: the cached text is someone's live token.
fn same_text(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |acc, (x, y)| acc | (x ^ y))
            == 0
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_verify() {
        let keys = JwtKeys::new(Some([9; 32])).unwrap();
        let now = chrono::Utc::now().timestamp();
        let id = Uuid::now_v7();
        let token = keys.issue(id, Role::Candidate, now).unwrap();
        let claims = keys.verify(&token).unwrap();
        assert_eq!(claims.sub, id);
        assert_eq!(claims.role, Role::Candidate);
        assert!(
            JwtKeys::new(Some([8; 32]))
                .unwrap()
                .verify(&token)
                .is_none()
        );
        assert!(
            keys.verify(&keys.issue(id, Role::Candidate, now - 3600).unwrap())
                .is_none()
        );
    }

    #[test]
    fn seen_tokens_still_expire_and_must_match_exactly() {
        let keys = JwtKeys::new(Some([9; 32])).unwrap();
        let id = Uuid::now_v7();
        let now = chrono::Utc::now().timestamp();
        let token = keys.issue(id, Role::Employer, now).unwrap();
        assert_eq!(keys.verify(&token).unwrap().sub, id);
        assert_eq!(keys.verify(&token).unwrap().role, Role::Employer);
        let mut forged = token.clone();
        forged.pop();
        forged.push(if token.ends_with('A') { 'B' } else { 'A' });
        assert!(keys.verify(&forged).is_none());
        assert!(keys.verify(&token[..token.len() - 1]).is_none());

        let stale = keys
            .issue(id, Role::Employer, now - ACCESS_TTL_SECS + 1)
            .unwrap();
        assert!(keys.verify(&stale).is_some());
        let slot = &keys.seen[keys.hasher.hash_one(stale.as_str()) as usize & (SEEN_SLOTS - 1)];
        let seen = slot.load_full().unwrap();
        slot.store(Some(Arc::new(Seen {
            token: seen.token.clone(),
            claims: Verified {
                exp: now - LEEWAY_SECS - 1,
                ..seen.claims
            },
        })));
        assert!(
            keys.verify(&stale).is_none(),
            "an expired cached token is rejected"
        );
    }
}
