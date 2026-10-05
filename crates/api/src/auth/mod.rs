pub mod jwt;
pub mod routes;

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request},
    http::{Method, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::ApiError, kv::KvError, state::AppState};

pub const ACCESS_COOKIE: &str = "fsp_at";
pub const REFRESH_COOKIE: &str = "fsp_rt";
/// Marker for the Next.js middleware: `fsp_rt` is path-scoped to /api and `fsp_at`
/// expires every 15 minutes, so page requests need a cookie that lives as long as the session.
pub const SESSION_COOKIE: &str = "fsp_session";
pub const REFRESH_TTL_DAYS: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Candidate,
    Employer,
    Admin,
}

impl From<db::types::UserRole> for Role {
    fn from(r: db::types::UserRole) -> Self {
        match r {
            db::types::UserRole::Candidate => Role::Candidate,
            db::types::UserRole::Employer => Role::Employer,
            db::types::UserRole::Admin => Role::Admin,
        }
    }
}

impl From<Role> for db::types::UserRole {
    fn from(r: Role) -> Self {
        match r {
            Role::Candidate => db::types::UserRole::Candidate,
            Role::Employer => db::types::UserRole::Employer,
            Role::Admin => db::types::UserRole::Admin,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: Role,
    pub jti: Uuid,
    pub exp: i64,
}

fn deny_key(jti: Uuid) -> String {
    format!("deny:jti:{jti}")
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(user) = parts.extensions.get::<AuthUser>() {
            return Ok(*user);
        }
        let bearer = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::to_owned);
        let token = match bearer {
            Some(t) => t,
            None => CookieJar::from_headers(&parts.headers)
                .get(ACCESS_COOKIE)
                .map(|c| c.value().to_owned())
                .ok_or_else(ApiError::unauthorized)?,
        };
        let claims = state
            .jwt
            .verify(&token)
            .ok_or_else(ApiError::unauthorized)?;
        if state.kv.has_flag(&deny_key(claims.jti)).await? {
            return Err(ApiError::unauthorized());
        }
        let user = AuthUser {
            id: claims.sub,
            role: claims.role,
            jti: claims.jti,
            exp: claims.exp,
        };
        parts.extensions.insert(user);
        Ok(user)
    }
}

impl OptionalFromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        match <AuthUser as FromRequestParts<AppState>>::from_request_parts(parts, state).await {
            Ok(user) => Ok(Some(user)),
            Err(ApiError::Status { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl AuthUser {
    pub async fn revoke(&self, state: &AppState) -> Result<(), KvError> {
        let ttl = (self.exp - chrono::Utc::now().timestamp()).max(1) as u64;
        state
            .kv
            .set_flag(&deny_key(self.jti), std::time::Duration::from_secs(ttl))
            .await
    }
}

macro_rules! role_extractor {
    ($name:ident, $role:path, $msg:literal) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name(pub AuthUser);

        impl FromRequestParts<AppState> for $name {
            type Rejection = ApiError;

            async fn from_request_parts(
                parts: &mut Parts,
                state: &AppState,
            ) -> Result<Self, Self::Rejection> {
                let user =
                    <AuthUser as FromRequestParts<AppState>>::from_request_parts(parts, state)
                        .await?;
                if user.role == $role {
                    Ok($name(user))
                } else {
                    Err(ApiError::forbidden("wrong_role", $msg))
                }
            }
        }

        impl std::ops::Deref for $name {
            type Target = AuthUser;
            fn deref(&self) -> &AuthUser {
                &self.0
            }
        }
    };
}

role_extractor!(CandidateUser, Role::Candidate, "Доступно только кандидатам");
role_extractor!(
    EmployerUser,
    Role::Employer,
    "Доступно только работодателям"
);
role_extractor!(AdminUser, Role::Admin, "Доступно только администраторам");

/// Cookies are sent cross-site on top-level navigations (SameSite=Lax), so state-changing
/// cookie-authenticated requests must also prove they come from our origin.
pub async fn csrf_guard(
    state: axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    let unsafe_method = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    let has_bearer = req.headers().contains_key(header::AUTHORIZATION);
    let has_cookie = req.headers().contains_key(header::COOKIE);
    if unsafe_method && has_cookie && !has_bearer {
        let origin = req
            .headers()
            .get(header::ORIGIN)
            .or_else(|| req.headers().get(header::REFERER))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !origin_matches(origin, &state.config.public_url) {
            return ApiError::forbidden("csrf", "Запрос отклонён: неверный источник")
                .into_response();
        }
    }
    next.run(req).await
}

fn origin_matches(origin: &str, public_url: &str) -> bool {
    origin == public_url || origin.starts_with(&format!("{public_url}/"))
}

pub fn session_cookies(jar: CookieJar, access: String, refresh: String, secure: bool) -> CookieJar {
    let access = Cookie::build((ACCESS_COOKIE, access))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time_dur(jwt::ACCESS_TTL_SECS));
    let refresh = Cookie::build((REFRESH_COOKIE, refresh))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict)
        .path("/api/v1/auth")
        .max_age(time_dur(REFRESH_TTL_DAYS * 86_400));
    let marker = Cookie::build((SESSION_COOKIE, "1"))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time_dur(REFRESH_TTL_DAYS * 86_400));
    jar.add(access).add(refresh).add(marker)
}

pub fn clear_session_cookies(jar: CookieJar) -> CookieJar {
    jar.remove(Cookie::build(ACCESS_COOKIE).path("/"))
        .remove(Cookie::build(REFRESH_COOKIE).path("/api/v1/auth"))
        .remove(Cookie::build(SESSION_COOKIE).path("/"))
}

fn time_dur(secs: i64) -> time::Duration {
    time::Duration::seconds(secs)
}

/// First hop of X-Forwarded-For, set by Caddy.
pub fn client_ip(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_owned())
}

pub fn user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.chars().take(300).collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn origin_check() {
        use super::origin_matches;
        assert!(origin_matches(
            "http://localhost:8080",
            "http://localhost:8080"
        ));
        assert!(origin_matches(
            "http://localhost:8080/candidate",
            "http://localhost:8080"
        ));
        assert!(!origin_matches(
            "http://localhost:8080.evil.com",
            "http://localhost:8080"
        ));
        assert!(!origin_matches("", "http://localhost:8080"));
    }
}
