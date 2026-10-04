use std::process::ExitCode;

use fsp_mock::{AppState, DEFAULT_BIND, DEFAULT_TOKEN, DataError, Registry, router};
use tracing_subscriber::EnvFilter;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error(transparent)]
    Data(#[from] DataError),
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
            tracing::error!(%error, "fsp-mock stopped");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), RunError> {
    let bind = env_or("FSP_MOCK_BIND", DEFAULT_BIND);
    let token = env_or("FSP_REGISTRY_TOKEN", DEFAULT_TOKEN);
    if token == DEFAULT_TOKEN {
        tracing::warn!("FSP_REGISTRY_TOKEN is the dev default; set a real one outside local demos");
    }

    let registry = Registry::embedded()?;
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .map_err(|source| RunError::Bind {
            addr: bind.clone(),
            source,
        })?;
    tracing::info!(%bind, participants = registry.len(), "fsp-mock listening");

    axum::serve(listener, router(AppState::new(registry, token)))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(RunError::Serve)
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
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
