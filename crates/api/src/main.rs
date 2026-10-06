use std::sync::Arc;

use api::{
    auth::jwt::JwtKeys,
    cache::CandidateCache,
    config::{Config, ConfigError},
    crypto::{Crypto, CryptoError},
    db::DbSetupError,
    integrations::{dispatcher::Dispatcher, webhook::WebhookSender},
    kv::{Kv, KvError},
    mail::{MailError, Mailer},
    proctoring::{ProctoringMode, RecordingStore},
    state::{AppState, Services},
};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[derive(Debug, thiserror::Error)]
enum StartupError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Db(#[from] DbSetupError),
    #[error(transparent)]
    Kv(#[from] KvError),
    #[error(transparent)]
    Mail(#[from] MailError),
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    #[error("metrics: {0}")]
    Metrics(#[from] metrics_exporter_prometheus::BuildError),
    #[error("http client: {0}")]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("PROCTORING_DIR: {0}")]
    Proctoring(std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[tokio::main]
async fn main() -> Result<(), StartupError> {
    let command = std::env::args().nth(1).unwrap_or_else(|| "serve".into());
    if command == "openapi" {
        println!("{}", api::openapi().to_pretty_json()?);
        return Ok(());
    }

    init_tracing(std::env::var("LOG_FORMAT").is_ok_and(|v| v == "json"));
    let config = Config::from_env()?;

    let pool = api::db::pool(&config.database_url, config.database_pool_size)?;
    api::db::migrate(&pool).await?;
    if command == "migrate" {
        return Ok(());
    }
    if command == "seed" || config.seed_demo {
        let crypto = Crypto::new(&config.field_key, &config.bidx_key);
        let seeded = api::seed::seed_sized(
            &pool,
            &crypto,
            config.proctoring.retention_days,
            config.seed_per_spec,
        )
        .await;
        match seeded {
            Ok(Some(r)) => tracing::info!(
                companies = r.companies,
                candidates = r.candidates,
                fsp_linked = r.fsp_linked,
                with_video = r.with_video,
                without_video = r.without_video,
                with_remarks = r.with_remarks,
                minors_confirmed = r.minors_confirmed,
                minors_pending = r.minors_pending,
                review_flagged = r.review_flagged,
                review_unproctored = r.review_unproctored,
                review_remarks = r.review_remarks,
                flagged_vacancies = r.flagged_vacancies,
                complaints = r.complaints,
                "demo data seeded"
            ),
            Ok(None) => tracing::info!("demo data already present"),
            Err(e) => tracing::error!(error = %e, "demo seed failed"),
        }
    }
    match api::trust::backfill_signatures(&pool).await {
        Ok(0) => {}
        Ok(n) => tracing::info!(vacancies = n, "vacancy text signatures backfilled"),
        Err(e) => tracing::warn!(error = %e, "vacancy text signature backfill failed"),
    }
    if command == "seed" {
        return Ok(());
    }

    api::guardian::spawn_sweeper(pool.clone());
    let recordings = Arc::new(RecordingStore::new(config.proctoring.dir.clone()));
    match recordings.init().await {
        Ok(()) => api::proctoring::spawn_sweeper(
            pool.clone(),
            recordings.clone(),
            config.proctoring.retention_days,
        ),
        Err(e) if config.proctoring.mode == ProctoringMode::Off => {
            tracing::warn!(error = %e, dir = %config.proctoring.dir.display(), "proctoring directory unavailable");
        }
        Err(e) => return Err(StartupError::Proctoring(e)),
    }

    let crypto = Arc::new(Crypto::new(&config.field_key, &config.bidx_key));
    let webhooks = WebhookSender::new(config.integrations.webhook_dev_hosts.clone())?;
    if let Some(demo) = &config.integrations.demo
        && let Err(e) = api::integrations::provision_demo(&pool, &crypto, &webhooks, demo).await
    {
        tracing::error!(error = %e, "demo ATS provisioning failed");
    }
    Dispatcher::new(
        pool.clone(),
        crypto.clone(),
        webhooks.clone(),
        config.integrations.dispatch_interval,
    )
    .spawn();

    let kv = Kv::connect(config.valkey_url.as_deref()).await?;
    let state = AppState::from(Services {
        crypto,
        jwt: Arc::new(JwtKeys::new(config.jwt_seed)?),
        mailer: Arc::new(Mailer::new(&config.mail)?),
        candidates: Arc::new(CandidateCache::new(kv.clone())),
        kv,
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?,
        metrics: Some(api::metrics::install()?),
        recordings,
        webhooks,
        pool,
        config: Arc::new(config),
    });

    match api::skills::backfill_fsp_skills(&state.pool).await {
        Ok(0) => {}
        Ok(n) => {
            tracing::info!(
                candidates = n,
                "fsp-confirmed skills brought in line with snapshots"
            );
            state.candidates.invalidate().await;
        }
        Err(e) => tracing::warn!(error = %e, "fsp skill backfill failed"),
    }

    let listener = tokio::net::TcpListener::bind(state.config.bind).await?;
    tracing::info!(addr = %state.config.bind, "listening");
    axum::serve(listener, api::app(state))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

fn init_tracing(json: bool) {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,api=debug"));
    let registry = tracing_subscriber::registry().with(filter);
    if json {
        registry.with(fmt::layer().json()).init();
    } else {
        registry.with(fmt::layer()).init();
    }
}

async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("shutting down");
}
