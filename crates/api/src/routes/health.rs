use axum::{
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};

use crate::state::AppState;

pub async fn healthz() -> &'static str {
    "ok"
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = db::ping(&state.pool).await.is_ok();
    let kv_ok = state.kv.ping().await.is_ok();
    let status = if db_ok && kv_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, format!("db={db_ok} kv={kv_ok}"))
}

pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state
        .metrics
        .as_ref()
        .map(|h| h.render())
        .unwrap_or_default();
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], body)
}
