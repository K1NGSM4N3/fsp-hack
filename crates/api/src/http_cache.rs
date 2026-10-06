//! `Cache-Control` of API responses (architecture.md §12.1). The default is `private, no-store`:
//! every response that is not explicitly public carries it, which covers everything behind a
//! cookie, a bearer token or an API key, and anything with personal data (152-ФЗ: no shared or
//! intermediary cache may keep it). A router group opts into public caching with
//! `from_fn_with_state(policy, public)`; its handlers take no credentials, which `lib.rs` tests
//! against the OpenAPI.

use axum::{
    body::{Body, Bytes, HttpBody},
    extract::{Request, State},
    http::{
        HeaderMap, HeaderValue, Method, StatusCode,
        header::{CACHE_CONTROL, CONTENT_LENGTH, ETAG, IF_NONE_MATCH, SET_COOKIE},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};

use std::hash::Hasher;

use crate::error::ApiError;

pub const PRIVATE: &str = "private, no-store";
pub const UNAUTHORIZED: &str = "private, no-cache";
pub const NO_STORE: &str = "no-store";
pub const STATIC: &str = "public, max-age=300, stale-while-revalidate=86400";
pub const LISTING: &str = "public, max-age=30, stale-while-revalidate=120";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// Reference data that changes only with a deploy: catalog, consent documents, test plans.
    Static,
    /// Public listings without personal data: published vacancies.
    Listing,
    /// Anonymous but personal or one-off: guardian links, evaluation of uploaded data.
    NoStore,
}

impl Policy {
    const fn header(self) -> &'static str {
        match self {
            Policy::Static => STATIC,
            Policy::Listing => LISTING,
            Policy::NoStore => NO_STORE,
        }
    }
}

/// Outermost: a response nobody marked gets `private, no-store`. A 401 carries no data and gets
/// `private, no-cache` instead: Chrome never completes a `no-store` response whose body the page
/// does not read, and the session probe of every public page leaves its 401s unread, so each would
/// pin an HTTP/1.1 connection until garbage collection.
pub async fn private_by_default(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let policy = if res.status() == StatusCode::UNAUTHORIZED {
        UNAUTHORIZED
    } else {
        PRIVATE
    };
    res.headers_mut()
        .entry(CACHE_CONTROL)
        .or_insert(HeaderValue::from_static(policy));
    res
}

pub async fn public(State(policy): State<Policy>, req: Request, next: Next) -> Response {
    let cacheable_method = matches!(*req.method(), Method::GET | Method::HEAD);
    let if_none_match = req.headers().get(IF_NONE_MATCH).cloned();
    let mut res = next.run(req).await;
    let cacheable = policy != Policy::NoStore
        && cacheable_method
        && res.status() == StatusCode::OK
        && !res.headers().contains_key(SET_COOKIE);
    if !cacheable {
        res.headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(NO_STORE));
        return res;
    }
    let (mut parts, body) = res.into_parts();
    let (etag, body) = match parts.headers.get(ETAG).cloned() {
        Some(etag) => (etag, body),
        None => match axum::body::to_bytes(body, usize::MAX).await {
            Ok(bytes) => (etag_of(&bytes), Body::from(bytes)),
            Err(e) => return ApiError::from(e).into_response(),
        },
    };
    parts
        .headers
        .insert(CACHE_CONTROL, HeaderValue::from_static(policy.header()));
    parts.headers.insert(ETAG, etag.clone());
    if if_none_match.is_some_and(|v| matches_etag(&v, &etag)) {
        // Without an explicit length the router would add `Content-Length: 0`, which a 304 may
        // carry only if the full response were empty (RFC 9110 §8.6).
        let length = HttpBody::size_hint(&body).exact();
        let mut headers = HeaderMap::with_capacity(3);
        for name in [CACHE_CONTROL, ETAG] {
            if let Some(v) = parts.headers.get(&name) {
                headers.insert(name, v.clone());
            }
        }
        if let Some(n) = length {
            headers.insert(CONTENT_LENGTH, HeaderValue::from(n));
        }
        parts.status = StatusCode::NOT_MODIFIED;
        parts.headers = headers;
        return Response::from_parts(parts, Body::empty());
    }
    Response::from_parts(parts, body)
}

/// Strong validator of the exact bytes sent: 64-bit SipHash with fixed keys, so every replica
/// derives the same tag. It only has to notice a change, and SHA-256 without the `asm` feature
/// cost a tenth of the throughput of the vacancy list.
pub fn etag_of(body: &Bytes) -> HeaderValue {
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write(body);
    HeaderValue::from_str(&format!("\"{:016x}\"", hasher.finish()))
        .unwrap_or(HeaderValue::from_static("\"\""))
}

/// `If-None-Match` uses the weak comparison (RFC 9110 §13.1.2): `W/` is ignored on both sides,
/// so a tag a compressing proxy weakened still matches.
fn matches_etag(if_none_match: &HeaderValue, etag: &HeaderValue) -> bool {
    let Ok(list) = if_none_match.to_str() else {
        return false;
    };
    let ours = opaque(etag.to_str().unwrap_or_default());
    list.split(',')
        .map(str::trim)
        .any(|t| t == "*" || (!ours.is_empty() && opaque(t) == ours))
}

fn opaque(tag: &str) -> &str {
    tag.strip_prefix("W/").unwrap_or(tag)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        middleware::{from_fn, from_fn_with_state},
        routing::get,
    };
    use tower::ServiceExt;

    use super::*;

    fn app(policy: Option<Policy>) -> Router {
        let routes = Router::new()
            .route(
                "/data",
                get(|| async { "payload" }).post(|| async { "posted" }),
            )
            .route(
                "/missing",
                get(|| async { (StatusCode::NOT_FOUND, "no such thing") }),
            )
            .route(
                "/me",
                get(|| async { (StatusCode::UNAUTHORIZED, "sign in") }),
            )
            .route(
                "/login",
                get(|| async { ([(SET_COOKIE, "fsp_at=x; HttpOnly")], "token") }),
            )
            .route(
                "/tagged",
                get(|| async { ([(ETAG, "\"fixed\"")], "precomputed") }),
            );
        let routes = match policy {
            Some(p) => routes.layer(from_fn_with_state(p, public)),
            None => routes,
        };
        routes.layer(from_fn(private_by_default))
    }

    async fn call(app: Router, method: Method, uri: &str, inm: Option<&str>) -> Response {
        let mut req = Request::builder().method(method).uri(uri);
        if let Some(v) = inm {
            req = req.header(IF_NONE_MATCH, v);
        }
        app.oneshot(req.body(Body::empty()).unwrap()).await.unwrap()
    }

    fn cache_control(res: &Response) -> &str {
        res.headers()[CACHE_CONTROL].to_str().unwrap()
    }

    #[tokio::test]
    async fn unmarked_routes_are_private() {
        for uri in ["/data", "/missing", "/login"] {
            let res = call(app(None), Method::GET, uri, None).await;
            assert_eq!(cache_control(&res), PRIVATE, "{uri}");
            assert!(res.headers().get(ETAG).is_none());
        }
        let res = call(app(None), Method::GET, "/me", None).await;
        assert_eq!(cache_control(&res), UNAUTHORIZED);
    }

    #[tokio::test]
    async fn public_groups_get_their_policy_and_a_strong_etag() {
        for (policy, header) in [(Policy::Static, STATIC), (Policy::Listing, LISTING)] {
            let res = call(app(Some(policy)), Method::GET, "/data", None).await;
            assert_eq!(res.status(), StatusCode::OK);
            assert_eq!(cache_control(&res), header);
            let etag = res.headers()[ETAG].to_str().unwrap().to_owned();
            assert_eq!(etag, etag_of(&Bytes::from_static(b"payload")));
            assert!(etag.starts_with('"') && etag.len() == 18);
            let body = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
            assert_eq!(&body[..], b"payload");
        }
    }

    #[tokio::test]
    async fn matching_validator_gives_304_without_body() {
        let etag = etag_of(&Bytes::from_static(b"payload"));
        let tag = etag.to_str().unwrap();
        for inm in [
            tag.to_owned(),
            format!("W/{tag}"),
            format!("\"other\", {tag}"),
            "*".to_owned(),
        ] {
            let res = call(app(Some(Policy::Static)), Method::GET, "/data", Some(&inm)).await;
            assert_eq!(res.status(), StatusCode::NOT_MODIFIED, "{inm}");
            assert_eq!(cache_control(&res), STATIC);
            assert_eq!(res.headers()[ETAG], etag);
            assert_eq!(res.headers()[CONTENT_LENGTH], "7", "length of the 200 body");
            let body = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
            assert!(body.is_empty());
        }
        let res = call(
            app(Some(Policy::Static)),
            Method::GET,
            "/data",
            Some("\"stale\""),
        )
        .await;
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn precomputed_etag_is_kept() {
        let res = call(
            app(Some(Policy::Static)),
            Method::GET,
            "/tagged",
            Some("\"fixed\""),
        )
        .await;
        assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(res.headers()[ETAG], "\"fixed\"");
    }

    #[tokio::test]
    async fn errors_writes_cookies_and_no_store_groups_are_never_stored() {
        let cases = [
            (Policy::Static, Method::GET, "/missing"),
            (Policy::Listing, Method::POST, "/data"),
            (Policy::Static, Method::GET, "/login"),
            (Policy::NoStore, Method::GET, "/data"),
        ];
        for (policy, method, uri) in cases {
            let res = call(app(Some(policy)), method.clone(), uri, None).await;
            assert_eq!(cache_control(&res), NO_STORE, "{policy:?} {method} {uri}");
            assert!(res.headers().get(ETAG).is_none());
        }
    }
}
