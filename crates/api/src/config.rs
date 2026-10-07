use std::{net::SocketAddr, path::PathBuf};

use base64::{Engine, engine::general_purpose::STANDARD as B64};

use crate::proctoring::ProctoringMode;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} is required")]
    Missing(&'static str),
    #[error("{name}: {reason}")]
    Invalid { name: &'static str, reason: String },
    #[error("{name}: {source}")]
    Parse {
        name: &'static str,
        source: ParseError,
    },
    #[error("{name}: {source}")]
    Base64 {
        name: &'static str,
        source: base64::DecodeError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Addr(#[from] std::net::AddrParseError),
}

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    /// Connections of this replica's pool (`DATABASE_POOL_SIZE`).
    pub database_pool_size: u32,
    pub valkey_url: Option<String>,
    pub public_url: String,
    pub cookie_secure: bool,
    pub jwt_seed: Option<[u8; 32]>,
    pub field_key: [u8; 32],
    pub bidx_key: [u8; 32],
    pub mail: MailConfig,
    pub fsp: FspConfig,
    pub grade_change_cooldown_days: i64,
    pub proctoring: ProctoringConfig,
    pub integrations: IntegrationsConfig,
    pub notifications: NotifyConfig,
    /// The embedder service; `None` with `EMBEDDINGS=off` or without `EMBEDDER_URL`.
    pub embeddings: Option<EmbeddingsConfig>,
    pub seed_demo: bool,
    /// Candidates per specialization in the demo seed; raised only for load tests.
    pub seed_per_spec: usize,
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
    /// Shared secret of the registry's signed pushes (`FSP_REGISTRY_WEBHOOK_SECRET`); unset —
    /// the push endpoint answers 503 and only pull works.
    pub webhook_secret: Option<String>,
    pub dev_link: bool,
}

#[derive(Clone)]
pub struct ProctoringConfig {
    pub mode: ProctoringMode,
    /// Encrypted video chunks; a persistent volume in production.
    pub dir: PathBuf,
    pub retention_days: i64,
}

#[derive(Clone)]
pub struct IntegrationsConfig {
    /// Hosts a webhook may reach over plain http and on private addresses (the ATS mock in
    /// compose). Empty in production: https to public addresses only.
    pub webhook_dev_hosts: Vec<String>,
    /// Outbox polling period; `None` uses the dispatcher default.
    pub dispatch_interval: Option<std::time::Duration>,
    /// The ATS invitations feed returns only changes older than this (`ATS_POLL_LAG_SECS`), so a
    /// transaction still committing is never jumped over.
    pub poll_lag: std::time::Duration,
    pub demo: Option<DemoIntegration>,
}

impl Default for IntegrationsConfig {
    fn default() -> Self {
        Self {
            webhook_dev_hosts: Vec::new(),
            dispatch_interval: None,
            poll_lag: std::time::Duration::from_secs(5),
            demo: None,
        }
    }
}

/// Email side of notifications (architecture.md §13); the defaults are the production rule.
#[derive(Clone, Debug)]
pub struct NotifyConfig {
    /// An item waits this long for others to join its letter (`NOTIFY_EMAIL_HOLD_SECS`).
    pub hold: std::time::Duration,
    /// At most one letter per recipient in this interval (`NOTIFY_EMAIL_GAP_SECS`).
    pub min_gap: std::time::Duration,
    /// At most this many letters per recipient in 24 hours (`NOTIFY_EMAIL_DAILY_CAP`).
    pub daily_cap: u32,
    /// How often the email dispatcher looks for work (`NOTIFY_INTERVAL_MS`).
    pub interval: std::time::Duration,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            hold: std::time::Duration::from_secs(120),
            min_gap: std::time::Duration::from_secs(600),
            daily_cap: 10,
            interval: std::time::Duration::from_secs(5),
        }
    }
}

/// Sentence embeddings (architecture.md §6.4): where the embedder listens and how the worker paces.
#[derive(Clone, Debug)]
pub struct EmbeddingsConfig {
    /// `EMBEDDER_URL`, e.g. `http://embedder:8090`.
    pub url: String,
    /// Per request to the embedder (`EMBEDDER_TIMEOUT_MS`).
    pub timeout: std::time::Duration,
    /// How often the idle worker looks at the queue (`EMBEDDINGS_INTERVAL_MS`).
    pub interval: std::time::Duration,
}

/// Connects the demo company to the ATS mock at startup: the key and the secret come from
/// `.env`, so both services agree on them without manual steps.
#[derive(Clone)]
pub struct DemoIntegration {
    pub owner_email: String,
    pub api_key: String,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
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
    T::Err: Into<ParseError>,
{
    match var(name) {
        Some(v) => v.parse().map_err(|e: T::Err| ConfigError::Parse {
            name,
            source: e.into(),
        }),
        None => Ok(default),
    }
}

fn key32(name: &'static str) -> Result<Option<[u8; 32]>, ConfigError> {
    let Some(raw) = var(name) else {
        return Ok(None);
    };
    let bytes = B64
        .decode(raw.trim())
        .map_err(|source| ConfigError::Base64 { name, source })?;
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

        let proctoring = ProctoringConfig {
            mode: match var("PROCTORING").as_deref() {
                None | Some("optional") => ProctoringMode::Optional,
                Some("off") => ProctoringMode::Off,
                Some(other) => {
                    return Err(ConfigError::Invalid {
                        name: "PROCTORING",
                        reason: format!("expected optional or off, got {other}"),
                    });
                }
            },
            dir: var("PROCTORING_DIR")
                .map_or_else(|| PathBuf::from("data/proctoring"), PathBuf::from),
            retention_days: parsed("PROCTORING_RETENTION_DAYS", 90)?,
        };

        let integrations = IntegrationsConfig {
            webhook_dev_hosts: var("WEBHOOK_DEV_HOSTS")
                .map(|v| {
                    v.split(',')
                        .map(|h| h.trim().to_ascii_lowercase())
                        .filter(|h| !h.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            dispatch_interval: var("WEBHOOK_DISPATCH_INTERVAL_MS")
                .map(|v| {
                    v.parse::<u64>()
                        .map(std::time::Duration::from_millis)
                        .map_err(|e| ConfigError::Parse {
                            name: "WEBHOOK_DISPATCH_INTERVAL_MS",
                            source: e.into(),
                        })
                })
                .transpose()?,
            poll_lag: std::time::Duration::from_secs(parsed(
                "ATS_POLL_LAG_SECS",
                IntegrationsConfig::default().poll_lag.as_secs(),
            )?),
            demo: var("ATS_DEMO_API_KEY").map(|api_key| DemoIntegration {
                owner_email: var("ATS_DEMO_OWNER_EMAIL")
                    .unwrap_or_else(|| "employer@demo.fsp".into()),
                api_key,
                webhook_url: var("ATS_DEMO_WEBHOOK_URL"),
                webhook_secret: var("ATS_DEMO_WEBHOOK_SECRET"),
            }),
        };

        let embeddings = match (flag("EMBEDDINGS", true), var("EMBEDDER_URL")) {
            (true, Some(url)) => Some(EmbeddingsConfig {
                url: url.trim_end_matches('/').to_owned(),
                timeout: std::time::Duration::from_millis(parsed("EMBEDDER_TIMEOUT_MS", 15_000)?),
                interval: std::time::Duration::from_millis(parsed(
                    "EMBEDDINGS_INTERVAL_MS",
                    1_000,
                )?),
            }),
            _ => None,
        };

        let defaults = NotifyConfig::default();
        let notifications = NotifyConfig {
            hold: std::time::Duration::from_secs(parsed(
                "NOTIFY_EMAIL_HOLD_SECS",
                defaults.hold.as_secs(),
            )?),
            min_gap: std::time::Duration::from_secs(parsed(
                "NOTIFY_EMAIL_GAP_SECS",
                defaults.min_gap.as_secs(),
            )?),
            daily_cap: parsed("NOTIFY_EMAIL_DAILY_CAP", defaults.daily_cap)?,
            interval: std::time::Duration::from_millis(parsed(
                "NOTIFY_INTERVAL_MS",
                defaults.interval.as_millis() as u64,
            )?),
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
                webhook_secret: var("FSP_REGISTRY_WEBHOOK_SECRET"),
                dev_link: flag("FSP_DEV_LINK", false),
            },
            grade_change_cooldown_days: parsed("GRADE_CHANGE_COOLDOWN_DAYS", 90)?,
            proctoring,
            integrations,
            notifications,
            embeddings,
            seed_demo: flag("SEED_DEMO", false),
            seed_per_spec: parsed("SEED_CANDIDATES_PER_SPEC", crate::seed::CANDIDATES_PER_SPEC)?,
            database_pool_size: parsed("DATABASE_POOL_SIZE", crate::db::POOL_SIZE)?,
            log_json: var("LOG_FORMAT").as_deref() == Some("json"),
            public_url,
        })
    }
}
