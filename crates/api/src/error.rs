use std::borrow::Cow;

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Serialize, ser::SerializeMap};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{crypto::CryptoError, kv::KvError, mail::MailError, pdf::PdfError};

/// RFC 9457 problem details. `code` is the stable identifier the UI switches on.
#[derive(Debug, Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: String,
    pub status: u16,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>, example = json!({"salary.min": ["Зарплата «от» обязательна"]}))]
    pub errors: Option<FieldErrors>,
    /// `consent_required`: the current version of the consent document to grant first
    /// (`POST /api/v1/me/consents`); `guardian_consent_required`: the document the legal
    /// representative confirms by the link from their letter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_id: Option<Uuid>,
    /// `consent_required`: kind of that document, e.g. `pd_publication`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_kind: Option<String>,
}

/// A handful of field errors on a cold path: a flat vec beats any map and keeps form order.
/// Serialized as `{"field": ["message", ...]}`.
#[derive(Debug, Default)]
pub struct FieldErrors(Vec<(&'static str, Cow<'static, str>)>);

impl FieldErrors {
    pub fn push(&mut self, field: &'static str, message: impl Into<Cow<'static, str>>) {
        self.0.push((field, message.into()));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Serialize for FieldErrors {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        for (i, (field, _)) in self.0.iter().enumerate() {
            if self.0[..i].iter().any(|(f, _)| f == field) {
                continue;
            }
            let messages: Vec<&str> = self.0[i..]
                .iter()
                .filter(|(f, _)| f == field)
                .map(|(_, m)| m.as_ref())
                .collect();
            map.serialize_entry(field, &messages)?;
        }
        map.end()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
    #[error("kv: {0}")]
    Kv(#[from] KvError),
    #[error("crypto: {0}")]
    Crypto(#[from] CryptoError),
    #[error("jwt: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("background task: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("mail: {0}")]
    Mail(#[from] MailError),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("pdf: {0}")]
    Pdf(#[from] PdfError),
    #[error("recording storage: {0}")]
    Io(#[from] std::io::Error),
    #[error("corrupt stored value: {0}")]
    Corrupt(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{title}")]
    Status {
        status: StatusCode,
        code: &'static str,
        title: String,
        detail: Option<String>,
    },
    #[error("validation failed")]
    Validation(FieldErrors),
    /// 409 `consent_required` naming the document to grant: the UI offers it right away.
    #[error("consent required: {kind}")]
    ConsentRequired {
        document_id: Uuid,
        kind: String,
        title: String,
    },
    /// 409 `guardian_consent_required`: a candidate under 18 waits for the legal representative;
    /// the candidate cannot grant this document, it only names it.
    #[error("guardian consent required")]
    GuardianConsentRequired { document_id: Uuid },
    #[error(transparent)]
    Internal(#[from] InternalError),
}

macro_rules! internal_from {
    ($($ty:ty),+ $(,)?) => {$(
        impl From<$ty> for ApiError {
            fn from(e: $ty) -> Self {
                ApiError::Internal(e.into())
            }
        }
    )+};
}

internal_from!(
    sqlx::Error,
    KvError,
    CryptoError,
    jsonwebtoken::errors::Error,
    serde_json::Error,
    tokio::task::JoinError,
    MailError,
    reqwest::Error,
    PdfError,
    std::io::Error,
);

/// Input of `POST /eval/pairs` over a limit or inconsistent: 422 naming the top-level field.
impl From<matching::pairs::PairsError> for ApiError {
    fn from(e: matching::pairs::PairsError) -> Self {
        ApiError::field(e.field(), e.to_string())
    }
}

impl From<catalog::UnknownSlug> for ApiError {
    fn from(e: catalog::UnknownSlug) -> Self {
        ApiError::Internal(InternalError::Corrupt(e.to_string()))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, title: impl Into<String>) -> Self {
        Self::Status {
            status,
            code,
            title: title.into(),
            detail: None,
        }
    }

    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        match self {
            Self::Status {
                status,
                code,
                title,
                ..
            } => Self::Status {
                status,
                code,
                title,
                detail: Some(detail.into()),
            },
            other => other,
        }
    }

    pub fn bad_request(code: &'static str, title: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, title)
    }

    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", "Требуется вход")
    }

    pub fn forbidden(code: &'static str, title: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, code, title)
    }

    /// Also returned for other users' resources so their existence is not revealed.
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", "Не найдено")
    }

    pub fn conflict(code: &'static str, title: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, title)
    }

    pub fn too_many_requests() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "Слишком много запросов, попробуйте позже",
        )
    }

    pub fn field(field: &'static str, message: impl Into<Cow<'static, str>>) -> Self {
        let mut errors = FieldErrors::default();
        errors.push(field, message);
        Self::Validation(errors)
    }
}

#[derive(Default)]
pub struct Validator(FieldErrors);

impl Validator {
    pub fn check(&mut self, ok: bool, field: &'static str, message: &'static str) -> &mut Self {
        if !ok {
            self.0.push(field, message);
        }
        self
    }

    /// For messages that need formatting: the closure runs only when the check fails.
    pub fn check_with(
        &mut self,
        ok: bool,
        field: &'static str,
        message: impl FnOnce() -> String,
    ) -> &mut Self {
        if !ok {
            self.0.push(field, message());
        }
        self
    }

    pub fn finish(self) -> ApiResult<()> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(ApiError::Validation(self.0))
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let problem = match self {
            ApiError::Status {
                status,
                code,
                title,
                detail,
            } => Problem {
                kind: "about:blank",
                title,
                status: status.as_u16(),
                code,
                detail,
                errors: None,
                document_id: None,
                document_kind: None,
            },
            ApiError::Validation(errors) => Problem {
                kind: "about:blank",
                title: "Проверьте введённые данные".into(),
                status: 422,
                code: "validation_failed",
                detail: None,
                errors: Some(errors),
                document_id: None,
                document_kind: None,
            },
            ApiError::ConsentRequired {
                document_id,
                kind,
                title,
            } => Problem {
                kind: "about:blank",
                title: "Нужно согласие".into(),
                status: 409,
                code: "consent_required",
                detail: Some(format!(
                    "«{title}»: без него профиль не показывается работодателям"
                )),
                errors: None,
                document_id: Some(document_id),
                document_kind: Some(kind),
            },
            ApiError::GuardianConsentRequired { document_id } => Problem {
                kind: "about:blank",
                title: "Нужно согласие законного представителя".into(),
                status: 409,
                code: "guardian_consent_required",
                detail: Some(
                    "Пока вам нет 18, компании увидят ваш профиль и контакты только после того, как законный представитель подтвердит согласие по ссылке из письма".into(),
                ),
                errors: None,
                document_id: Some(document_id),
                document_kind: Some(db::guardians::KIND.into()),
            },
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                Problem {
                    kind: "about:blank",
                    title: "Внутренняя ошибка сервера".into(),
                    status: 500,
                    code: "internal",
                    detail: None,
                    errors: None,
                    document_id: None,
                    document_kind: None,
                }
            }
        };
        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, Json(problem)).into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if status == StatusCode::TOO_MANY_REQUESTS {
            headers.insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_errors_group_by_field_in_insertion_order() {
        let mut v = Validator::default();
        v.check(false, "title", "a")
            .check(true, "x", "never")
            .check(false, "salary", "b")
            .check_with(false, "title", || "c".into());
        let Err(ApiError::Validation(errors)) = v.finish() else {
            panic!("expected validation error")
        };
        assert_eq!(
            serde_json::to_string(&errors).unwrap(),
            r#"{"title":["a","c"],"salary":["b"]}"#
        );
    }

    #[tokio::test]
    async fn consent_required_names_the_document() {
        let id = Uuid::now_v7();
        let response = ApiError::ConsentRequired {
            document_id: id,
            kind: "pd_publication".into(),
            title: "Согласие на публикацию".into(),
        }
        .into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], "consent_required");
        assert_eq!(body["status"], 409);
        assert_eq!(body["document_id"], id.to_string());
        assert_eq!(body["document_kind"], "pd_publication");
        assert!(body.get("errors").is_none());
    }

    #[tokio::test]
    async fn other_problems_carry_no_document() {
        let bytes = axum::body::to_bytes(ApiError::not_found().into_response().into_body(), 4096)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(body.get("document_id").is_none() && body.get("document_kind").is_none());
    }
}
