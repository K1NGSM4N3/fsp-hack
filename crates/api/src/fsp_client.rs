//! Outbound integrations with FSP: the OIDC provider (Keycloak realm `fsp`) and the registry.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as B64URL};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, jwk::JwkSet};
use serde::Deserialize;

use crate::{
    config::{FspConfig, OidcConfig},
    crypto::{random_urlsafe, sha256},
};

#[derive(Debug, thiserror::Error)]
pub enum FspError {
    #[error("FSP ID login is not configured")]
    OidcDisabled,
    #[error("FSP registry is not configured")]
    RegistryDisabled,
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("token exchange failed with status {0}")]
    TokenExchange(u16),
    #[error("id_token rejected: {0}")]
    IdToken(#[from] jsonwebtoken::errors::Error),
    #[error("id_token signed with unknown key")]
    UnknownKey,
    #[error("nonce mismatch")]
    Nonce,
    #[error("id_token has no fsp_id claim")]
    NoFspId,
    #[error("participant not found in the registry")]
    NotFound,
    #[error("registry answered {0}")]
    Registry(u16),
}

#[derive(Deserialize)]
struct Discovery {
    authorization_endpoint: String,
    token_endpoint: String,
    jwks_uri: String,
}

pub struct AuthorizationRequest {
    pub url: String,
    pub state: String,
    pub pkce_verifier: String,
    pub nonce: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: String,
}

#[derive(Deserialize)]
struct IdClaims {
    nonce: Option<String>,
    fsp_id: Option<String>,
}

fn oidc(cfg: &FspConfig) -> Result<&OidcConfig, FspError> {
    cfg.oidc.as_ref().ok_or(FspError::OidcDisabled)
}

async fn discovery(http: &reqwest::Client, cfg: &OidcConfig) -> Result<Discovery, FspError> {
    Ok(http
        .get(&cfg.discovery_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub async fn authorization_request(
    http: &reqwest::Client,
    cfg: &FspConfig,
) -> Result<AuthorizationRequest, FspError> {
    let cfg = oidc(cfg)?;
    let d = discovery(http, cfg).await?;
    let state = random_urlsafe(24);
    let nonce = random_urlsafe(24);
    let pkce_verifier = random_urlsafe(32);
    let challenge = B64URL.encode(sha256(pkce_verifier.as_bytes()));
    let mut url =
        reqwest::Url::parse(&d.authorization_endpoint).map_err(|_| FspError::OidcDisabled)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &cfg.client_id)
        .append_pair("redirect_uri", &cfg.redirect_url)
        .append_pair("scope", "openid")
        .append_pair("state", &state)
        .append_pair("nonce", &nonce)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");
    Ok(AuthorizationRequest {
        url: url.into(),
        state,
        pkce_verifier,
        nonce,
    })
}

/// Exchanges the code over the back-channel and returns the verified `fsp_id` claim.
pub async fn complete(
    http: &reqwest::Client,
    cfg: &FspConfig,
    code: &str,
    pkce_verifier: &str,
    nonce: &str,
) -> Result<String, FspError> {
    let cfg = oidc(cfg)?;
    let d = discovery(http, cfg).await?;
    let response = http
        .post(&d.token_endpoint)
        .basic_auth(&cfg.client_id, Some(&cfg.client_secret))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", cfg.redirect_url.as_str()),
            ("code_verifier", pkce_verifier),
        ])
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(FspError::TokenExchange(response.status().as_u16()));
    }
    let tokens: TokenResponse = response.json().await?;

    let header = jsonwebtoken::decode_header(&tokens.id_token)?;
    let jwks: JwkSet = http
        .get(&d.jwks_uri)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let jwk = match header.kid.as_deref() {
        Some(kid) => jwks.find(kid),
        None => jwks.keys.first(),
    }
    .ok_or(FspError::UnknownKey)?;
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[cfg.issuer.as_str()]);
    validation.set_audience(&[cfg.client_id.as_str()]);
    let claims = jsonwebtoken::decode::<IdClaims>(
        &tokens.id_token,
        &DecodingKey::from_jwk(jwk)?,
        &validation,
    )?
    .claims;
    if claims.nonce.as_deref() != Some(nonce) {
        return Err(FspError::Nonce);
    }
    claims
        .fsp_id
        .filter(|id| !id.trim().is_empty())
        .ok_or(FspError::NoFspId)
}

#[derive(Deserialize)]
struct RegistryAchievements {
    achievements: Vec<serde_json::Value>,
}

pub async fn fetch_achievements(
    http: &reqwest::Client,
    cfg: &FspConfig,
    fsp_id: &str,
) -> Result<Vec<serde_json::Value>, FspError> {
    let base = cfg
        .registry_url
        .as_deref()
        .ok_or(FspError::RegistryDisabled)?;
    let response = http
        .get(format!(
            "{}/participants/{fsp_id}/achievements",
            base.trim_end_matches('/')
        ))
        .bearer_auth(&cfg.registry_token)
        .send()
        .await?;
    match response.status().as_u16() {
        200 => Ok(response.json::<RegistryAchievements>().await?.achievements),
        404 => Err(FspError::NotFound),
        other => Err(FspError::Registry(other)),
    }
}
