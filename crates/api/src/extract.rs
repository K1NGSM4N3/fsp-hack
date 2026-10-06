//! Extractors for request input whose rejections are problem+json with a stable `code`
//! (`invalid_json`, `unsupported_media_type`, `invalid_path`, `invalid_query`,
//! `payload_too_large`, `invalid_body`) instead of axum's plain text. Handlers import these in place of axum's
//! own; `Json` also serves responses. Clippy (`crates/api/clippy.toml`) rejects the axum types.

#![allow(clippy::disallowed_types)]

use axum::{
    body::Bytes,
    extract::{FromRequest, FromRequestParts, Request, multipart::MultipartRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::error::ApiError;

/// JSON body in, JSON body out.
#[derive(Debug, Clone, Copy, Default, FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Debug, Clone, Copy, Default, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
pub struct Path<T>(pub T);

#[derive(Debug, Clone, Copy, Default, FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ApiError))]
pub struct Query<T>(pub T);

/// A raw body (a video chunk), buffered up to the route's `DefaultBodyLimit`.
pub struct RawBody(pub Bytes);

impl<S: Send + Sync> FromRequest<S> for RawBody {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Bytes::from_request(req, state).await {
            Ok(bytes) => Ok(RawBody(bytes)),
            Err(r) if r.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                Err(ApiError::payload_too_large())
            }
            Err(r) => Err(ApiError::bad_request(
                "invalid_body",
                "Не удалось прочитать тело запроса",
            )
            .with_detail(r.body_text())),
        }
    }
}

/// `multipart/form-data`; reading the fields stays with the handler.
pub struct Multipart(pub axum::extract::Multipart);

impl<S: Send + Sync> FromRequest<S> for Multipart {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        axum::extract::Multipart::from_request(req, state)
            .await
            .map(Multipart)
            .map_err(|r: MultipartRejection| {
                ApiError::new(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "unsupported_media_type",
                    "Нужен Content-Type: multipart/form-data",
                )
                .with_detail(r.body_text())
            })
    }
}
