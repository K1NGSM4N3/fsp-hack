use std::sync::Arc;

use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;

use crate::{
    auth::jwt::JwtKeys, config::Config, crypto::Crypto, integrations::webhook::WebhookSender,
    kv::Kv, mail::Mailer, proctoring::RecordingStore,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub crypto: Arc<Crypto>,
    pub jwt: Arc<JwtKeys>,
    pub mailer: Arc<Mailer>,
    pub kv: Kv,
    pub http: reqwest::Client,
    pub metrics: Option<PrometheusHandle>,
    pub recordings: Arc<RecordingStore>,
    /// Outbound webhooks: SSRF-guarded client, separate from `http` (no redirects).
    pub webhooks: WebhookSender,
}
