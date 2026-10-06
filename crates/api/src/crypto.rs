//! Algorithms sit behind traits and stored values are self-describing (PHC strings,
//! versioned ciphertexts), so a ГОСТ implementation or a certified СКЗИ can replace them
//! without a schema change (architecture.md §9.4).

use std::sync::LazyLock;

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{
        PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64URL};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("password hashing failed: {0}")]
    Hash(String),
    #[error("unknown ciphertext version")]
    UnknownVersion,
    #[error("ciphertext too short")]
    Truncated,
    #[error("ciphertext authentication failed")]
    Authentication,
    #[error("encryption failed")]
    Encrypt,
    #[error("decrypted value is not utf-8")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("decrypted value is not valid json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("key encoding: {0}")]
    KeyEncoding(String),
}

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, CryptoError>;
    fn verify(&self, password: &str, phc: &str) -> bool;
}

/// OWASP baseline: m = 19 MiB, t = 2, p = 1 (the crate's `Params::DEFAULT`).
pub struct Argon2id(Argon2<'static>);

impl Default for Argon2id {
    fn default() -> Self {
        Self(Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::DEFAULT,
        ))
    }
}

impl PasswordHasher for Argon2id {
    fn hash(&self, password: &str) -> Result<String, CryptoError> {
        let salt = SaltString::generate(&mut OsRng);
        self.0
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| CryptoError::Hash(e.to_string()))
    }

    fn verify(&self, password: &str, phc: &str) -> bool {
        PasswordHash::new(phc)
            .is_ok_and(|parsed| self.0.verify_password(password.as_bytes(), &parsed).is_ok())
    }
}

pub trait FieldCipher: Send + Sync {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.encrypt_bound(plaintext, &[])
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        self.decrypt_bound(ciphertext, &[])
    }

    /// `context` is authenticated but not stored: the ciphertext only decrypts with the same
    /// context, so a blob moved to another record or position fails instead of being accepted.
    fn encrypt_bound(&self, plaintext: &[u8], context: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt_bound(&self, ciphertext: &[u8], context: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

/// XChaCha20-Poly1305; 192-bit random nonces make collisions negligible without a counter.
/// Layout: `b"v1" || nonce(24) || ciphertext+tag`.
pub struct XChaChaV1(XChaCha20Poly1305);

const V1: &[u8; 2] = b"v1";
const NONCE_LEN: usize = 24;

impl XChaChaV1 {
    pub fn new(key: &[u8; 32]) -> Self {
        Self(XChaCha20Poly1305::new(key.into()))
    }
}

impl FieldCipher for XChaChaV1 {
    fn encrypt_bound(&self, plaintext: &[u8], context: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce = [0u8; NONCE_LEN];
        rand::rng().fill_bytes(&mut nonce);
        let ct = self
            .0
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: context,
                },
            )
            .map_err(|_| CryptoError::Encrypt)?;
        let mut out = Vec::with_capacity(V1.len() + NONCE_LEN + ct.len());
        out.extend_from_slice(V1);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn decrypt_bound(&self, data: &[u8], context: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let rest = data.strip_prefix(V1).ok_or(CryptoError::UnknownVersion)?;
        if rest.len() < NONCE_LEN {
            return Err(CryptoError::Truncated);
        }
        let (nonce, ct) = rest.split_at(NONCE_LEN);
        self.0
            .decrypt(
                XNonce::from_slice(nonce),
                Payload {
                    msg: ct,
                    aad: context,
                },
            )
            .map_err(|_| CryptoError::Authentication)
    }
}

pub struct Crypto {
    pub passwords: Box<dyn PasswordHasher>,
    pub fields: Box<dyn FieldCipher>,
    bidx: Hmac<Sha256>,
}

/// HMAC-SHA256 under any key, without the `Result` of `new_from_slice`, which never fails for
/// HMAC: as RFC 2104 prescribes, a key longer than the 64-byte block is hashed first and a shorter
/// one is zero-padded to the block, which is then a full-size key for `KeyInit::new`.
pub fn hmac_sha256(key: &[u8]) -> Hmac<Sha256> {
    let mut block = [0u8; 64];
    if key.len() > block.len() {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    <Hmac<Sha256> as KeyInit>::new(&block.into())
}

impl Crypto {
    pub fn new(field_key: &[u8; 32], bidx_key: &[u8; 32]) -> Self {
        Self {
            passwords: Box::new(Argon2id::default()),
            fields: Box::new(XChaChaV1::new(field_key)),
            bidx: hmac_sha256(bidx_key),
        }
    }

    /// Keyed hash for equality lookups on encrypted columns (blind index).
    pub fn blind_index(&self, value: &str) -> Vec<u8> {
        let mut mac = self.bidx.clone();
        mac.update(value.as_bytes());
        mac.finalize().into_bytes().to_vec()
    }

    /// A key for one purpose, derived from the blind-index secret: no new secret to configure,
    /// and the label keeps it apart from blind indexes (an email never starts with `derive:`).
    pub fn derive_key(&self, purpose: &str) -> [u8; 32] {
        let mut mac = self.bidx.clone();
        mac.update(b"derive:");
        mac.update(purpose.as_bytes());
        mac.finalize().into_bytes().into()
    }

    pub fn encrypt_str(&self, value: &str) -> Result<Vec<u8>, CryptoError> {
        self.fields.encrypt(value.as_bytes())
    }

    pub fn decrypt_str(&self, data: &[u8]) -> Result<String, CryptoError> {
        Ok(String::from_utf8(self.fields.decrypt(data)?)?)
    }

    pub fn encrypt_json<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, CryptoError> {
        self.fields.encrypt(&serde_json::to_vec(value)?)
    }

    pub fn decrypt_json<T: serde::de::DeserializeOwned>(
        &self,
        data: &[u8],
    ) -> Result<T, CryptoError> {
        Ok(serde_json::from_slice(&self.fields.decrypt(data)?)?)
    }
}

/// Verified when a login email is unknown, so both branches cost one Argon2 run. `None` only if
/// hashing a constant failed, which the default parameters rule out.
pub fn dummy_hash() -> Option<&'static str> {
    static HASH: LazyLock<Option<String>> =
        LazyLock::new(|| Argon2id::default().hash("dummy-password").ok());
    HASH.as_deref()
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// 256-bit URL-safe token and its SHA-256 (only the hash is stored).
pub fn new_token() -> (String, Vec<u8>) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let token = B64URL.encode(bytes);
    let hash = sha256(token.as_bytes());
    (token, hash)
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
    Sha256::digest(data).to_vec()
}

pub fn random_urlsafe(len_bytes: usize) -> String {
    let mut bytes = vec![0u8; len_bytes];
    rand::rng().fill_bytes(&mut bytes);
    B64URL.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_round_trip() {
        let h = Argon2id::default();
        let phc = h.hash("correct horse").unwrap();
        assert!(phc.starts_with("$argon2id$"));
        assert!(h.verify("correct horse", &phc));
        assert!(!h.verify("wrong", &phc));
        assert!(!h.verify("x", "garbage"));
        assert!(!h.verify("x", dummy_hash().unwrap()));
    }

    #[test]
    fn field_cipher_round_trip_and_tamper() {
        let c = XChaChaV1::new(&[7u8; 32]);
        let ct = c.encrypt(b"+7 999 123-45-67").unwrap();
        assert_eq!(c.decrypt(&ct).unwrap(), b"+7 999 123-45-67");
        assert_ne!(c.encrypt(b"same").unwrap(), c.encrypt(b"same").unwrap());
        let mut bad = ct.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert!(matches!(c.decrypt(&bad), Err(CryptoError::Authentication)));
        assert!(matches!(
            c.decrypt(b"v2xxx"),
            Err(CryptoError::UnknownVersion)
        ));
    }

    #[test]
    fn bound_ciphertext_needs_its_context() {
        let c = XChaChaV1::new(&[9u8; 32]);
        let ct = c.encrypt_bound(b"chunk", b"session-a:0").unwrap();
        assert_eq!(c.decrypt_bound(&ct, b"session-a:0").unwrap(), b"chunk");
        assert!(matches!(
            c.decrypt_bound(&ct, b"session-a:1"),
            Err(CryptoError::Authentication)
        ));
        assert!(matches!(c.decrypt(&ct), Err(CryptoError::Authentication)));
        assert_eq!(
            c.decrypt_bound(&c.encrypt(b"x").unwrap(), b"").unwrap(),
            b"x"
        );
    }

    #[test]
    fn hmac_key_of_any_length_matches_new_from_slice() {
        for len in [0, 1, 32, 63, 64, 65, 200] {
            let key = vec![0xA5u8; len];
            let mut ours = hmac_sha256(&key);
            let mut reference = <Hmac<Sha256> as Mac>::new_from_slice(&key).unwrap();
            ours.update(b"payload");
            reference.update(b"payload");
            assert_eq!(
                ours.finalize().into_bytes(),
                reference.finalize().into_bytes(),
                "{len}"
            );
        }
    }

    #[test]
    fn blind_index_is_deterministic_and_keyed() {
        let a = Crypto::new(&[1; 32], &[2; 32]);
        let b = Crypto::new(&[1; 32], &[3; 32]);
        let e = normalize_email("  Ivan@Example.RU ");
        assert_eq!(a.blind_index(&e), a.blind_index("ivan@example.ru"));
        assert_ne!(a.blind_index(&e), b.blind_index(&e));
    }

    #[test]
    fn derived_keys_differ_by_purpose_and_secret() {
        let a = Crypto::new(&[1; 32], &[2; 32]);
        let b = Crypto::new(&[1; 32], &[3; 32]);
        assert_eq!(a.derive_key("x"), a.derive_key("x"));
        assert_ne!(a.derive_key("x"), a.derive_key("y"));
        assert_ne!(a.derive_key("x"), b.derive_key("x"));
        assert_ne!(a.derive_key("x").to_vec(), a.blind_index("x"));
    }
}
