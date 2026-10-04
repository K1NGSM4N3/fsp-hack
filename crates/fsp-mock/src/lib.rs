//! Mock of the FSP participant registry. The API calls it after an FSP ID is linked
//! (architecture.md §7.1) to fetch the participant card and the achievements snapshot.

mod model;

use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;

pub use model::{
    Achievement, AchievementError, AchievementKind, DataError, Discipline, Entry, EventLevel,
    Participant, ParticipantCard, Place, Rank, RankSummary, Registry,
};

pub const DEFAULT_BIND: &str = "0.0.0.0:8090";
pub const DEFAULT_TOKEN: &str = "dev-registry-token";

#[derive(Clone)]
pub struct AppState {
    registry: &'static Registry,
    token: Arc<str>,
}

impl AppState {
    pub fn new(registry: &'static Registry, token: impl Into<Arc<str>>) -> Self {
        Self {
            registry,
            token: token.into(),
        }
    }
}

pub fn router(state: AppState) -> Router {
    let participants = Router::new()
        .route("/participants/{fsp_id}", get(participant_card))
        .route(
            "/participants/{fsp_id}/achievements",
            get(participant_achievements),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), require_token));

    Router::new()
        .route("/healthz", get(healthz))
        .merge(participants)
        .fallback(not_found)
        .layer(middleware::from_fn(log_requests))
        .with_state(state)
}

async fn healthz() -> Response {
    json_response(StatusCode::OK, r#"{"status":"ok"}"#)
}

async fn participant_card(State(state): State<AppState>, Path(fsp_id): Path<String>) -> Response {
    match state.registry.get(&fsp_id) {
        Some(entry) => json_response(StatusCode::OK, entry.card_json()),
        None => participant_not_found(),
    }
}

async fn participant_achievements(
    State(state): State<AppState>,
    Path(fsp_id): Path<String>,
) -> Response {
    match state.registry.get(&fsp_id) {
        Some(entry) => json_response(StatusCode::OK, entry.achievements_json()),
        None => participant_not_found(),
    }
}

async fn not_found() -> Response {
    problem(StatusCode::NOT_FOUND, "not_found", "Ресурс не найден")
}

fn participant_not_found() -> Response {
    problem(
        StatusCode::NOT_FOUND,
        "participant_not_found",
        "Участник с таким ФСП ID не найден",
    )
}

async fn require_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if bearer_matches(req.headers(), &state.token) {
        return next.run(req).await;
    }
    let mut response = problem(
        StatusCode::UNAUTHORIZED,
        "unauthorized",
        "Нужен токен доступа к реестру ФСП",
    );
    response.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(r#"Bearer realm="fsp-registry""#),
    );
    response
}

fn bearer_matches(headers: &HeaderMap, expected: &str) -> bool {
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some((scheme, token)) = value.split_once(' ') else {
        return false;
    };
    scheme.eq_ignore_ascii_case("bearer")
        && constant_time_eq(token.trim().as_bytes(), expected.as_bytes())
}

/// No early exit on the first mismatching byte, so timing does not leak a token prefix.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[derive(Serialize)]
struct Problem {
    #[serde(rename = "type")]
    kind: &'static str,
    title: &'static str,
    status: u16,
    code: &'static str,
}

fn problem(status: StatusCode, code: &'static str, title: &'static str) -> Response {
    let body = Problem {
        kind: "about:blank",
        title,
        status: status.as_u16(),
        code,
    };
    let json = serde_json::to_string(&body).unwrap_or_default();
    (
        status,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        )],
        json,
    )
        .into_response()
}

fn json_response(status: StatusCode, body: &'static str) -> Response {
    (
        status,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )],
        body,
    )
        .into_response()
}

async fn log_requests(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let started = Instant::now();
    let response = next.run(req).await;
    tracing::info!(
        %method,
        %path,
        status = response.status().as_u16(),
        elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
        "request"
    );
    response
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use serde_json::Value;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

    const TOKEN: &str = "test-token";

    struct Reply {
        status: u16,
        head: String,
        body: Value,
    }

    async fn spawn() -> SocketAddr {
        let registry = Registry::embedded().expect("embedded data");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router(AppState::new(registry, TOKEN)))
                .await
                .unwrap();
        });
        addr
    }

    async fn get(addr: SocketAddr, path: &str, auth: Option<&str>) -> Reply {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let mut request =
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n");
        if let Some(auth) = auth {
            request.push_str(&format!("Authorization: {auth}\r\n"));
        }
        request.push_str("\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).await.unwrap();
        let text = String::from_utf8(raw).unwrap();
        let (head, body) = text.split_once("\r\n\r\n").unwrap();
        Reply {
            status: head[9..12].parse().unwrap(),
            head: head.to_ascii_lowercase(),
            body: serde_json::from_str(body).unwrap(),
        }
    }

    #[tokio::test]
    async fn healthz_needs_no_token() {
        let addr = spawn().await;
        let r = get(addr, "/healthz", None).await;
        assert_eq!(r.status, 200);
        assert_eq!(r.body["status"], "ok");
    }

    #[tokio::test]
    async fn participants_require_the_registry_token() {
        let addr = spawn().await;
        for auth in [
            None,
            Some("Bearer wrong-token"),
            Some("Basic dGVzdC10b2tlbg=="),
        ] {
            let r = get(addr, "/participants/FSP-000001", auth).await;
            assert_eq!(r.status, 401, "auth = {auth:?}");
            assert!(r.head.contains("www-authenticate: bearer"));
            assert!(r.head.contains("content-type: application/problem+json"));
            assert_eq!(r.body["code"], "unauthorized");
        }
        let unknown = get(addr, "/participants/FSP-999999", None).await;
        assert_eq!(
            unknown.status, 401,
            "existence is not revealed without a token"
        );
    }

    #[tokio::test]
    async fn serves_card_and_achievements() {
        let addr = spawn().await;
        let auth = Some("Bearer test-token");

        let card = get(addr, "/participants/FSP-000003", auth).await;
        assert_eq!(card.status, 200);
        assert_eq!(card.body["fsp_id"], "FSP-000003");
        assert_eq!(card.body["full_name"], "Иванов Дмитрий Сергеевич");
        assert_eq!(card.body["region"], "Новосибирская область");
        assert_eq!(card.body["ranks"][0]["discipline"], "security");
        assert_eq!(card.body["ranks"][0]["rank"], "cms");

        let list = get(
            addr,
            "/participants/fsp-000003/achievements",
            Some("bearer test-token"),
        )
        .await;
        assert_eq!(list.status, 200);
        assert_eq!(list.body["fsp_id"], "FSP-000003");
        let achievements = list.body["achievements"].as_array().unwrap();
        assert_eq!(achievements.len(), 8);
        assert_eq!(achievements[0]["date"], "2025-08-30");
        assert_eq!(achievements[0]["event_level"], "international");

        let empty = get(addr, "/participants/FSP-000010/achievements", auth).await;
        assert_eq!(empty.status, 200);
        assert_eq!(empty.body["achievements"], Value::Array(vec![]));
    }

    #[tokio::test]
    async fn unknown_participant_is_404() {
        let addr = spawn().await;
        let auth = Some("Bearer test-token");
        for path in [
            "/participants/FSP-999999",
            "/participants/FSP-999999/achievements",
            "/participants/not-an-id",
        ] {
            let r = get(addr, path, auth).await;
            assert_eq!(r.status, 404, "{path}");
            assert_eq!(r.body["code"], "participant_not_found");
        }
        let other = get(addr, "/nope", None).await;
        assert_eq!(other.status, 404);
        assert_eq!(other.body["code"], "not_found");
    }

    #[test]
    fn constant_time_eq_compares_whole_input() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"a"));
    }
}
