use ed25519_dalek::{SigningKey, pkcs8::EncodePrivateKey};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Role;
use crate::crypto::CryptoError;

pub const ACCESS_TTL_SECS: i64 = 15 * 60;
const ISSUER: &str = "fsp-talent";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: Role,
    pub jti: Uuid,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
}

pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
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
        let der = signing.to_pkcs8_der().map_err(|e| CryptoError::KeyEncoding(e.to_string()))?;
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[ISSUER]);
        validation.leeway = 5;
        Ok(Self {
            encoding: EncodingKey::from_ed_der(der.as_bytes()),
            decoding: DecodingKey::from_ed_der(signing.verifying_key().as_bytes()),
            validation,
        })
    }

    pub fn issue(&self, user_id: Uuid, role: Role, now: i64) -> Result<String, jsonwebtoken::errors::Error> {
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

    pub fn verify(&self, token: &str) -> Option<Claims> {
        jsonwebtoken::decode::<Claims>(token, &self.decoding, &self.validation).ok().map(|d| d.claims)
    }
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
        assert!(JwtKeys::new(Some([8; 32])).unwrap().verify(&token).is_none());
        assert!(keys.verify(&keys.issue(id, Role::Candidate, now - 3600).unwrap()).is_none());
    }
}
