use std::sync::Arc;

use deadpool_postgres::Pool;
use metrics_exporter_prometheus::PrometheusHandle;

use crate::{auth::jwt::JwtKeys, config::Config, crypto::Crypto, kv::Kv, mail::Mailer};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: Pool,
    pub crypto: Arc<Crypto>,
    pub jwt: Arc<JwtKeys>,
    pub mailer: Arc<Mailer>,
    pub kv: Kv,
    pub http: reqwest::Client,
    pub metrics: Option<PrometheusHandle>,
}
