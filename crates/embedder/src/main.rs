use std::{path::PathBuf, process::ExitCode, time::Duration};

use embedder::{
    model::{self, Embedder, ModelError},
    server::{self, AppState},
};
use tracing_subscriber::EnvFilter;

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("EMBEDDER_MODEL: {0}")]
    Model(#[from] ModelError),
    #[error("{name}: {source}")]
    Config {
        name: &'static str,
        source: std::num::ParseIntError,
    },
    #[error("model loading task: {0}")]
    Load(#[from] tokio::task::JoinError),
    #[error("metrics: {0}")]
    Metrics(#[from] metrics_exporter_prometheus::BuildError),
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
            tracing::error!(%error, "embedder stopped");
            ExitCode::FAILURE
        }
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

fn parsed<T: std::str::FromStr<Err = std::num::ParseIntError>>(
    name: &'static str,
    default: T,
) -> Result<T, RunError> {
    match env(name) {
        Some(v) => v
            .trim()
            .parse()
            .map_err(|source| RunError::Config { name, source }),
        None => Ok(default),
    }
}

async fn run() -> Result<(), RunError> {
    let bind = env("EMBEDDER_BIND").unwrap_or_else(|| "0.0.0.0:8093".into());
    let name = env("EMBEDDER_MODEL").unwrap_or_else(|| model::MULTILINGUAL_E5_SMALL.name.into());
    let spec = model::spec(&name).ok_or_else(|| ModelError::Unknown(name.clone()))?;
    let dir = env("EMBEDDER_MODEL_DIR")
        .map_or_else(|| PathBuf::from("models").join(&name), PathBuf::from);
    let concurrency = parsed("EMBEDDER_CONCURRENCY", 1usize)?;
    let queue_wait = Duration::from_millis(parsed("EMBEDDER_QUEUE_WAIT_MS", 10_000u64)?);

    let started = std::time::Instant::now();
    let embedder = tokio::task::spawn_blocking(move || Embedder::load(&dir, spec)).await??;
    tracing::info!(
        model = embedder.id(),
        dim = embedder.dim(),
        elapsed_ms = started.elapsed().as_millis() as u64,
        "model loaded"
    );
    let metrics = server::install_metrics()?;
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .map_err(|source| RunError::Bind {
            addr: bind.clone(),
            source,
        })?;
    tracing::info!(%bind, concurrency, "embedder listening");
    axum::serve(
        listener,
        server::router(AppState::new(
            embedder,
            concurrency,
            queue_wait,
            Some(metrics),
        )),
    )
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
