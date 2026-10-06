use std::{ops::Deref, sync::Arc};

use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;

use crate::{
    auth::jwt::JwtKeys, cache::CandidateCache, config::Config, crypto::Crypto,
    integrations::webhook::WebhookSender, kv::Kv, mail::Mailer, proctoring::RecordingStore,
};

/// Handle to [`Services`]. axum clones the state several times per request (every `from_fn`
/// layer clones its inner stack, every handler its `State`); one reference count keeps that to a
/// single atomic instead of one per field, all contended across worker threads.
#[derive(Clone)]
pub struct AppState(Arc<Services>);

impl Deref for AppState {
    type Target = Services;

    fn deref(&self) -> &Services {
        &self.0
    }
}

impl From<Services> for AppState {
    fn from(services: Services) -> Self {
        Self(Arc::new(services))
    }
}

pub struct Services {
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
    /// Search pools and category stats; writes that change them call `invalidate`.
    pub candidates: Arc<CandidateCache>,
}
