use std::process::ExitCode;

use ats_mock::{AppState, Config, DEFAULT_BIND, router};
use tracing_subscriber::EnvFilter;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("ATS_DEMO_WEBHOOK_SECRET is required: the secret the platform signs webhooks with")]
    NoSecret,
    #[error("http client: {0}")]
    Http(#[from] reqwest::Error),
    #[error("cannot bind {addr}: {source}")]
    Bind {
        addr: String,
        source: std::io::Error,
    },
    #[error("server failed: {0}")]
    Serve(std::io::Error),
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "ats-mock stopped");
            ExitCode::FAILURE
        }
    }
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

async fn run() -> Result<(), RunError> {
    let bind = env("ATS_MOCK_BIND").unwrap_or_else(|| DEFAULT_BIND.to_owned());
    let config = Config {
        secret: env("ATS_DEMO_WEBHOOK_SECRET").ok_or(RunError::NoSecret)?,
        api_url: env("ATS_API_URL").unwrap_or_else(|| "http://localhost:8000".to_owned()),
        api_key: env("ATS_DEMO_API_KEY"),
    };
    if config.api_key.is_none() {
        tracing::warn!("ATS_DEMO_API_KEY not set: hiring stage buttons are disabled");
    }
    let state = AppState::new(config)?;
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .map_err(|source| RunError::Bind {
            addr: bind.clone(),
            source,
        })?;
    tracing::info!(%bind, "ats-mock listening");
    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(RunError::Serve)
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if std::env::var("LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json")) {
        builder.json().flatten_event(true).init();
    } else {
        builder.init();
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
