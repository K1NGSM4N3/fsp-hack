use std::time::{Duration, Instant};

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use metrics_exporter_prometheus::{BuildError, Matcher, PrometheusBuilder, PrometheusHandle};

pub fn install() -> Result<PrometheusHandle, BuildError> {
    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".into()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0],
        )?
        .install_recorder()?;
    // `install_recorder` leaves upkeep to the caller: without it every histogram sample stays
    // buffered until the next scrape, ~18 bytes a request, unbounded where nothing scrapes.
    let upkeep = handle.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(5));
        loop {
            tick.tick().await;
            upkeep.run_upkeep();
        }
    });
    zero_series();
    Ok(handle)
}

/// Exports the event counters at 0 before the first event: a series that appears only with its
/// first increment loses that increment in `increase()`, and its panel shows «No data».
fn zero_series() {
    for result in ["delivered", "retry", "failed"] {
        metrics::counter!("webhook_deliveries_total", "result" => result).absolute(0);
    }
    for scope in [
        "login",
        "register",
        "resend_verification",
        "guardian_letters",
        "api_key",
        "webhook_test",
        "reports",
        "eval_pairs",
    ] {
        metrics::counter!("rate_limit_rejections_total", "scope" => scope).absolute(0);
    }
    for name in [
        "proctoring_chunks_total",
        "proctoring_chunk_bytes_total",
        "proctoring_events_total",
    ] {
        metrics::counter!(name).absolute(0);
    }
}

/// Labels use the matched route template so cardinality stays bounded.
pub async fn track(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "unmatched".to_owned(), |p| p.as_str().to_owned());
    let method = req.method().to_string();
    let response = next.run(req).await;
    let labels = [
        ("method", method),
        ("path", path),
        ("status", response.status().as_u16().to_string()),
    ];
    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_request_duration_seconds", &labels)
        .record(start.elapsed().as_secs_f64());
    response
}
