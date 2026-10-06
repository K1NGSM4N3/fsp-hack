//! Company API keys for the inbound ATS API. A key is `fsp_live_` + 256 random bits; only its
//! SHA-256 is stored, next to a visible prefix. Keys authenticate only on `/api/v1/integrations`:
//! the extractor reads nothing but `Authorization: Bearer fsp_live_…`, so neither cookies nor
//! JWTs reach that tree, and a key is useless on cabinet routes.

use std::time::Duration;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64URL};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    crypto::sha256,
    error::{ApiError, ApiResult},
    state::AppState,
};

pub const KEY_PREFIX: &str = "fsp_live_";
/// Characters after `fsp_live_` shown in the cabinet: enough to tell keys apart.
const VISIBLE: usize = 8;
pub const RATE_PER_MINUTE: u64 = 120;
pub const MAX_ACTIVE_KEYS: i64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Scope {
    #[serde(rename = "invitations:read")]
    InvitationsRead,
    #[serde(rename = "candidates:read")]
    CandidatesRead,
    #[serde(rename = "outcomes:write")]
    OutcomesWrite,
    #[serde(rename = "needs:write")]
    NeedsWrite,
}

impl Scope {
    pub const ALL: [Scope; 4] = [
        Scope::InvitationsRead,
        Scope::CandidatesRead,
        Scope::OutcomesWrite,
        Scope::NeedsWrite,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Scope::InvitationsRead => "invitations:read",
            Scope::CandidatesRead => "candidates:read",
            Scope::OutcomesWrite => "outcomes:write",
            Scope::NeedsWrite => "needs:write",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|scope| scope.as_str() == s)
    }

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// Checked on every API call, so a bitset rather than the stored strings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Scopes(u8);

impl Scopes {
    pub fn from_strs<S: AsRef<str>>(items: &[S]) -> Self {
        Self(
            items
                .iter()
                .filter_map(|s| Scope::parse(s.as_ref()))
                .fold(0, |acc, s| acc | s.bit()),
        )
    }

    pub fn of(scopes: &[Scope]) -> Self {
        Self(scopes.iter().fold(0, |acc, s| acc | s.bit()))
    }

    pub const fn contains(self, scope: Scope) -> bool {
        self.0 & scope.bit() != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn to_vec(self) -> Vec<Scope> {
        Scope::ALL
            .into_iter()
            .filter(|s| self.contains(*s))
            .collect()
    }

    pub fn to_strings(self) -> Vec<String> {
        self.to_vec()
            .into_iter()
            .map(|s| s.as_str().to_owned())
            .collect()
    }
}

pub struct NewKey {
    pub key: String,
    pub prefix: String,
    pub hash: Vec<u8>,
}

pub fn generate() -> NewKey {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    from_secret(format!("{KEY_PREFIX}{}", B64URL.encode(bytes)))
}

/// For a key supplied by configuration (the demo ATS): same prefix and hash rules.
pub fn from_secret(key: String) -> NewKey {
    let visible: String = key.chars().take(KEY_PREFIX.len() + VISIBLE).collect();
    NewKey {
        prefix: visible,
        hash: sha256(key.as_bytes()),
        key,
    }
}

/// The caller of `/api/v1/integrations`: an active key and the company it belongs to. Requests
/// act on behalf of the company owner (for the contact access log).
#[derive(Debug, Clone, Copy)]
pub struct ApiClient {
    pub key_id: Uuid,
    pub company_id: Uuid,
    pub owner_user_id: Uuid,
    pub scopes: Scopes,
}

impl ApiClient {
    pub fn require(&self, scope: Scope) -> ApiResult<()> {
        if self.scopes.contains(scope) {
            Ok(())
        } else {
            Err(ApiError::forbidden(
                "insufficient_scope",
                format!("У ключа нет права {}", scope.as_str()),
            ))
        }
    }
}

fn unauthorized(code: &'static str, title: &str) -> ApiError {
    ApiError::new(StatusCode::UNAUTHORIZED, code, title)
}

fn bearer(parts: &Parts) -> Option<&str> {
    let value = parts.headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    scheme.eq_ignore_ascii_case("bearer").then(|| token.trim())
}

impl FromRequestParts<AppState> for ApiClient {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(client) = parts.extensions.get::<ApiClient>() {
            return Ok(*client);
        }
        let token = bearer(parts)
            .filter(|t| t.starts_with(KEY_PREFIX) && t.len() <= 128)
            .ok_or_else(|| {
                unauthorized(
                    "api_key_required",
                    "Нужен API-ключ компании: Authorization: Bearer fsp_live_…",
                )
            })?;
        let row = db::integrations::api_key_by_hash(&state.pool, &sha256(token.as_bytes()))
            .await?
            .ok_or_else(|| unauthorized("api_key_invalid", "Ключ не найден или отозван"))?;
        let hit = state
            .kv
            .hit(&format!("rl:apikey:{}", row.id), Duration::from_secs(60))
            .await?;
        if hit.over(RATE_PER_MINUTE) {
            return Err(ApiError::rate_limited("api_key", hit.retry_after));
        }
        db::integrations::touch_api_key(&state.pool, row.id).await?;
        let client = ApiClient {
            key_id: row.id,
            company_id: row.company_id,
            owner_user_id: row.owner_user_id,
            scopes: Scopes::from_strs(&row.scopes),
        };
        parts.extensions.insert(client);
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_random_prefixed_and_hashed() {
        let a = generate();
        let b = generate();
        assert!(a.key.starts_with(KEY_PREFIX));
        assert_eq!(a.key.len(), KEY_PREFIX.len() + 43);
        assert_ne!(a.key, b.key);
        assert_eq!(a.prefix.len(), KEY_PREFIX.len() + VISIBLE);
        assert!(a.key.starts_with(&a.prefix));
        assert_eq!(a.hash, sha256(a.key.as_bytes()));
        assert_eq!(from_secret(a.key.clone()).hash, a.hash);
    }

    #[test]
    fn scope_bitset() {
        let s = Scopes::from_strs(&["outcomes:write", "invitations:read", "admin:all"]);
        assert!(s.contains(Scope::OutcomesWrite) && s.contains(Scope::InvitationsRead));
        assert!(!s.contains(Scope::CandidatesRead) && !s.contains(Scope::NeedsWrite));
        assert_eq!(s.to_strings(), ["invitations:read", "outcomes:write"]);
        assert_eq!(Scopes::of(&Scope::ALL).to_vec(), Scope::ALL);
        assert!(Scopes::from_strs::<&str>(&[]).is_empty());
        for scope in Scope::ALL {
            assert_eq!(Scope::parse(scope.as_str()), Some(scope));
            assert_eq!(
                serde_json::to_value(scope).unwrap(),
                serde_json::Value::from(scope.as_str())
            );
        }
    }
}
