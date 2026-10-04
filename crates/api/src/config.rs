use std::net::SocketAddr;

use base64::{Engine, engine::general_purpose::STANDARD as B64};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} is required")]
    Missing(&'static str),
    #[error("{name}: {reason}")]
    Invalid { name: &'static str, reason: String },
}

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub valkey_url: Option<String>,
    pub public_url: String,
    pub cookie_secure: bool,
    pub jwt_seed: Option<[u8; 32]>,
    pub field_key: [u8; 32],
    pub bidx_key: [u8; 32],
    pub mail: MailConfig,
    pub fsp: FspConfig,
    pub grade_change_cooldown_days: i64,
    pub seed_demo: bool,
    pub log_json: bool,
}

#[derive(Clone)]
pub enum MailConfig {
    Log,
    Smtp { url: String, from: String },
}

#[derive(Clone)]
pub struct FspConfig {
    pub oidc: Option<OidcConfig>,
    pub registry_url: Option<String>,
    pub registry_token: String,
    pub dev_link: bool,
}

#[derive(Clone)]
pub struct OidcConfig {
    /// Browser-facing issuer (`iss`); differs from the back-channel host inside Docker.
    pub issuer: String,
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn flag(name: &str, default: bool) -> bool {
    var(name).map_or(default, |v| {
        matches!(v.as_str(), "1" | "true" | "yes" | "on")
    })
}

fn parsed<T: std::str::FromStr>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T::Err: std::fmt::Display,
{
    match var(name) {
        Some(v) => v.parse().map_err(|e: T::Err| ConfigError::Invalid {
            name,
            reason: e.to_string(),
        }),
        None => Ok(default),
    }
}

fn key32(name: &'static str) -> Result<Option<[u8; 32]>, ConfigError> {
    let Some(raw) = var(name) else {
        return Ok(None);
    };
    let bytes = B64.decode(raw.trim()).map_err(|e| ConfigError::Invalid {
        name,
        reason: e.to_string(),
    })?;
    bytes
        .try_into()
        .map(Some)
        .map_err(|_| ConfigError::Invalid {
            name,
            reason: "must decode to exactly 32 bytes".into(),
        })
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let public_url = var("PUBLIC_URL")
            .unwrap_or_else(|| "http://localhost:3000".into())
            .trim_end_matches('/')
            .to_owned();

        let (field_key, bidx_key) = match (key32("FIELD_ENC_KEY")?, key32("BIDX_KEY")?) {
            (Some(f), Some(b)) => (f, b),
            (None, None) => {
                tracing::warn!("FIELD_ENC_KEY/BIDX_KEY not set: using fixed development keys");
                (
                    *b"dev-field-key-do-not-use-in-prd!",
                    *b"dev-bidx-key-do-not-use-in-prod!",
                )
            }
            _ => {
                return Err(ConfigError::Invalid {
                    name: "FIELD_ENC_KEY",
                    reason: "set both FIELD_ENC_KEY and BIDX_KEY or neither".into(),
                });
            }
        };

        let mail = match var("MAIL_MODE").as_deref() {
            Some("smtp") => MailConfig::Smtp {
                url: var("SMTP_URL").ok_or(ConfigError::Missing("SMTP_URL"))?,
                from: var("MAIL_FROM")
                    .unwrap_or_else(|| "ФСП Таланты <noreply@fsp-talent.local>".into()),
            },
            _ => MailConfig::Log,
        };

        let oidc = match (var("FSP_OIDC_ISSUER"), var("FSP_OIDC_CLIENT_SECRET")) {
            (Some(issuer), Some(client_secret)) => Some(OidcConfig {
                discovery_url: var("FSP_OIDC_DISCOVERY_URL")
                    .unwrap_or_else(|| format!("{issuer}/.well-known/openid-configuration")),
                issuer,
                client_id: var("FSP_OIDC_CLIENT_ID").unwrap_or_else(|| "fsp-talent".into()),
                client_secret,
                redirect_url: var("FSP_OIDC_REDIRECT_URL")
                    .unwrap_or_else(|| format!("{public_url}/api/v1/fsp/link/callback")),
            }),
            _ => None,
        };

        Ok(Self {
            bind: parsed("API_BIND", SocketAddr::from(([0, 0, 0, 0], 8000)))?,
            database_url: var("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?,
            valkey_url: var("VALKEY_URL"),
            cookie_secure: flag("COOKIE_SECURE", false),
            jwt_seed: key32("JWT_ED25519_SEED_B64")?,
            field_key,
            bidx_key,
            mail,
            fsp: FspConfig {
                oidc,
                registry_url: var("FSP_REGISTRY_URL"),
                registry_token: var("FSP_REGISTRY_TOKEN")
                    .unwrap_or_else(|| "dev-registry-token".into()),
                dev_link: flag("FSP_DEV_LINK", false),
            },
            grade_change_cooldown_days: parsed("GRADE_CHANGE_COOLDOWN_DAYS", 90)?,
            seed_demo: flag("SEED_DEMO", false),
            log_json: var("LOG_FORMAT").as_deref() == Some("json"),
            public_url,
        })
    }
}
