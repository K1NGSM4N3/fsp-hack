//! `POST /embed`, `GET /healthz`, `GET /metrics`. Inference runs on blocking threads behind a
//! semaphore: candle already spreads one forward pass over every core, so more passes at once only
//! add memory, and a caller that waits too long gets 503 instead of a growing queue.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use metrics_exporter_prometheus::{BuildError, Matcher, PrometheusBuilder, PrometheusHandle};
use serde::Serialize;
use tokio::sync::Semaphore;

use crate::{
    model::{Embedder, ModelError},
    wire::{EmbedRequest, EmbedResponse, MAX_BATCH, MAX_TEXT_CHARS, RequestError},
};

/// A full batch of the longest texts, four bytes per character, plus JSON escaping.
const BODY_LIMIT: usize = 2 * MAX_BATCH * MAX_TEXT_CHARS * 4;

pub struct Service {
    embedder: Embedder,
    permits: Semaphore,
    queue_wait: Duration,
    metrics: Option<PrometheusHandle>,
}

#[derive(Clone)]
pub struct AppState(Arc<Service>);

impl AppState {
    pub fn new(
        embedder: Embedder,
        concurrency: usize,
        queue_wait: Duration,
        metrics: Option<PrometheusHandle>,
    ) -> Self {
        Self(Arc::new(Service {
            embedder,
            permits: Semaphore::new(concurrency.max(1)),
            queue_wait,
            metrics,
        }))
    }
}

pub fn install_metrics() -> Result<PrometheusHandle, BuildError> {
    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("embedder_inference_seconds".into()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
        )?
        .install_recorder()?;
    for result in ["ok", "invalid", "busy", "error"] {
        metrics::counter!("embedder_requests_total", "result" => result).absolute(0);
    }
    metrics::counter!("embedder_texts_total").absolute(0);
    Ok(handle)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/embed", post(embed))
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(state)
}

#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error("{0}")]
    Body(String),
    #[error(transparent)]
    Invalid(#[from] RequestError),
    #[error("busy: no inference slot within {0:?}")]
    Busy(Duration),
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error("inference task failed")]
    Join,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, result) = match &self {
            ServiceError::Body(_) | ServiceError::Invalid(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "invalid")
            }
            ServiceError::Busy(_) => (StatusCode::SERVICE_UNAVAILABLE, "busy"),
            ServiceError::Model(_) | ServiceError::Join => {
                (StatusCode::INTERNAL_SERVER_ERROR, "error")
            }
        };
        metrics::counter!("embedder_requests_total", "result" => result).increment(1);
        if status.is_server_error() {
            tracing::warn!(error = %self, "embed failed");
        }
        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

async fn embed(
    State(state): State<AppState>,
    body: Result<Json<EmbedRequest>, JsonRejection>,
) -> Result<Json<EmbedResponse>, ServiceError> {
    let Json(req) = body.map_err(|e| ServiceError::Body(e.body_text()))?;
    req.validate()?;
    let service = Arc::clone(&state.0);
    let _permit = tokio::time::timeout(service.queue_wait, service.permits.acquire())
        .await
        .map_err(|_| ServiceError::Busy(service.queue_wait))?
        .map_err(|_| ServiceError::Join)?;
    let count = req.texts.len();
    let started = Instant::now();
    let worker = Arc::clone(&service);
    let vectors = tokio::task::spawn_blocking(move || {
        let texts: Vec<&str> = req.texts.iter().map(String::as_str).collect();
        worker.embedder.embed(&texts, req.kind)
    })
    .await
    .map_err(|_| ServiceError::Join)??;
    metrics::histogram!("embedder_inference_seconds").record(started.elapsed().as_secs_f64());
    metrics::counter!("embedder_texts_total").increment(count as u64);
    metrics::counter!("embedder_requests_total", "result" => "ok").increment(1);
    Ok(Json(EmbedResponse {
        model: service.embedder.id().to_owned(),
        dim: service.embedder.dim(),
        vectors,
    }))
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    model: String,
    dim: usize,
}

async fn healthz(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        model: state.0.embedder.id().to_owned(),
        dim: state.0.embedder.dim(),
    })
}

async fn metrics(State(state): State<AppState>) -> Response {
    match &state.0.metrics {
        Some(handle) => handle.render().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
