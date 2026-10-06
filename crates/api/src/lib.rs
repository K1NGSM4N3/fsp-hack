pub mod age;
pub mod auth;
pub mod cache;
pub mod cards;
pub mod config;
pub mod contacts;
pub mod crypto;
pub mod db;
pub mod dto;
pub mod error;
pub mod fsp_client;
pub mod guardian;
pub mod http_cache;
pub mod integrations;
pub mod kv;
pub mod mail;
pub mod metrics;
pub mod pdf;
pub mod pool;
pub mod proctoring;
pub mod public_code;
pub mod resume;
pub mod routes;
pub mod seed;
pub mod skills;
pub mod state;
pub mod trust;

use std::{convert::Infallible, time::Duration};

use axum::{
    BoxError, Router, ServiceExt,
    body::{Body, Bytes, HttpBody},
    extract::{DefaultBodyLimit, Request},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware,
    response::Response,
    routing::{IntoMakeService, get},
};
use tower::{Service, ServiceBuilder};
use tower_http::{
    catch_panic::CatchPanicLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestId, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{http_cache::Policy, state::AppState};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "ФСП Таланты API",
        description = "Платформа подбора ИТ-специалистов: работодатель выходит на кандидата из подтверждённой тестом категории. Ошибки — RFC 9457 problem+json со стабильным полем `code`.",
        version = "1.0.0"
    ),
    modifiers(&SecuritySchemes),
    tags(
        (name = "auth", description = "Регистрация, вход, сессии"),
        (name = "account", description = "Текущий пользователь, согласия, удаление аккаунта"),
        (name = "catalog", description = "Справочники"),
        (name = "candidate", description = "Кабинет кандидата: профиль, приватность"),
        (name = "employer", description = "Кабинет работодателя: компания"),
        (name = "fsp", description = "Привязка ФСП ID (OIDC + PKCE) и достижения из реестра ФСП"),
        (name = "assessment", description = "Адаптивный тест на грейд, категория, план теста под вакансию"),
        (name = "proctoring", description = "Видеозапись теста и журнал событий: согласие, фрагменты записи, просмотр. Работодателю видео недоступно"),
        (name = "matching", description = "Категории, подборки, поиск по банку кандидатов"),
        (name = "tasks", description = "Регулярные короткие задания от работодателей"),
        (name = "vacancies", description = "Вакансии и отклики — дополнительный путь, инициатива кандидата"),
        (name = "invitations", description = "Приглашения — выход работодателя на кандидата, раскрытие контактов"),
        (name = "guardian", description = "Согласие законного представителя соискателя 14–17 лет: публичная страница по ссылке из письма"),
        (name = "Валидация", description = "Проверка подбора на своих данных: ваш набор «вакансия – кандидат» через рабочий подбор и базовую линию, метрики по разметке"),
        (name = "trust", description = "Антифейк: уровень доверия и квота компании, проверка вилки и дублей вакансий, очередь модерации"),
        (name = "integrations", description = "Кабинет работодателя: вебхук в ATS компании, журнал доставок, API-ключи"),
        (name = "Интеграции ATS", description = "API для ATS работодателя. Доступ — только API-ключом компании: `Authorization: Bearer fsp_live_…`, cookie и JWT здесь не принимаются. Лимит — 120 запросов в минуту на ключ. События приходят вебхуком (тело — схема `WebhookEvent`, подпись — заголовок `FSP-Signature: t=…,v1=HMAC-SHA256(secret, t + \".\" + body)`), здесь — опрос, профиль кандидата после согласия, этапы найма и импорт вакансий"),
    ),
    components(schemas(integrations::events::WebhookEvent))
)]
struct ApiDoc;

struct SecuritySchemes;

impl Modify for SecuritySchemes {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
        components.add_security_scheme(
            "api_key",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("fsp_live_…")
                    .description(Some("API-ключ компании из кабинета «Интеграции»"))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "cookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(auth::ACCESS_COOKIE))),
        );
    }
}

pub fn api_v1() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .nest("/auth", auth::routes::router())
        .nest("/me", routes::me::router().merge(routes::export::router()))
        .nest(
            "/candidate",
            routes::candidate_profile::router()
                .merge(routes::survey::router())
                .merge(routes::invitations::candidate_router())
                .merge(routes::vacancies::candidate_router())
                .merge(routes::tasks::candidate_router())
                .merge(routes::market::candidate_router())
                .merge(routes::assessment::candidate_router())
                .merge(routes::proctoring::candidate_router())
                .merge(routes::guardian::candidate_router()),
        )
        .nest(
            "/employer",
            routes::company::router()
                .merge(routes::invitations::employer_router())
                .merge(routes::vacancies::employer_router())
                .merge(routes::tasks::employer_router())
                .merge(routes::market::employer_router())
                .merge(routes::needs::router())
                .merge(routes::candidates::router())
                .merge(routes::assessment::employer_router())
                .merge(routes::trust::employer_router())
                .merge(routes::integrations::router()),
        )
        .nest("/integrations", routes::ats::router())
        .nest("/fsp", routes::fsp::router())
        .nest(
            "/admin",
            routes::proctoring::admin_router().merge(routes::trust::admin_router()),
        )
        .merge(listing_routes().layer(middleware::from_fn_with_state(
            Policy::Listing,
            http_cache::public,
        )))
        .merge(routes::vacancies::apply_router())
        .merge(
            routes::guardian::public_router().layer(middleware::from_fn_with_state(
                Policy::NoStore,
                http_cache::public,
            )),
        )
        .merge(routes::reports::router())
        .merge(static_routes().layer(middleware::from_fn_with_state(
            Policy::Static,
            http_cache::public,
        )))
        .merge(
            routes::eval::router()
                .merge(routes::assessment::catalog_router())
                .layer(middleware::from_fn_with_state(
                    Policy::NoStore,
                    http_cache::public,
                )),
        )
}

/// Reference data, cached publicly for 5 minutes. Only handlers without credentials belong here.
fn static_routes() -> OpenApiRouter<AppState> {
    routes::catalog::router()
}

/// Published vacancies, cached publicly for 30 seconds. Only handlers without credentials.
fn listing_routes() -> OpenApiRouter<AppState> {
    routes::vacancies::public_router()
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, api) = OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", api_v1())
        .split_for_parts();
    api
}

/// UUIDv4 from the thread-local generator: `MakeRequestUuid` asks the OS for entropy on every
/// request, a syscall that showed up in the profile of the cheapest routes.
#[derive(Clone, Copy)]
struct RandomRequestId;

impl MakeRequestId for RandomRequestId {
    fn make_request_id<B>(&mut self, _: &axum::http::Request<B>) -> Option<RequestId> {
        let id = uuid::Builder::from_random_bytes(rand::random()).into_uuid();
        let mut buf = uuid::Uuid::encode_buffer();
        HeaderValue::from_str(id.hyphenated().encode_lower(&mut buf))
            .ok()
            .map(RequestId::new)
    }
}

/// The server. Middleware that does not need the matched route wraps the router once, as a
/// plain tower stack: added with `Router::layer` it would wrap every route, and axum clones a
/// route's whole stack for each request and again inside every `from_fn` of it. Metrics read the
/// route template and count CSRF rejections, so these two stay per route.
pub fn app(
    state: AppState,
) -> IntoMakeService<
    impl Service<Request, Response = Response, Error = Infallible, Future: Send>
    + Clone
    + Send
    + 'static,
> {
    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", api_v1())
        .split_for_parts();

    let router: Router = Router::new()
        .merge(api_router)
        .merge(SwaggerUi::new("/api/docs").url("/api/v1/openapi.json", api))
        .route("/healthz", get(routes::health::healthz))
        .route("/readyz", get(routes::health::readyz))
        .route("/metrics", get(routes::health::metrics))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::csrf_guard,
        ))
        .layer(middleware::from_fn(metrics::track))
        .with_state(state);

    let x_request_id = HeaderName::from_static("x-request-id");
    let service = ServiceBuilder::new()
        .map_response(boxed_response)
        .layer(CatchPanicLayer::new())
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            RandomRequestId,
        ))
        .layer(PropagateRequestIdLayer::new(x_request_id))
        .layer(SetSensitiveRequestHeadersLayer::new([
            axum::http::header::AUTHORIZATION,
            axum::http::header::COOKIE,
        ]))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .map_response(boxed_response)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .map_request(boxed_request)
        .layer(middleware::from_fn::<_, (Request,)>(
            http_cache::private_by_default,
        ))
        .service(router);
    ServiceExt::<Request>::into_make_service(service)
}

/// `from_fn` takes an axum `Body` and `TimeoutLayer` a body with `Default`: the conversions that
/// `Router::layer` makes between layers.
fn boxed_request<B>(req: axum::http::Request<B>) -> Request
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    req.map(Body::new)
}

fn boxed_response<B>(res: axum::http::Response<B>) -> Response
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    res.map(Body::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operations(router: OpenApiRouter<AppState>) -> Vec<(String, serde_json::Value)> {
        let (_, api) = router.split_for_parts();
        let json = serde_json::to_value(&api).unwrap_or_default();
        let mut out = Vec::new();
        if let Some(paths) = json["paths"].as_object() {
            for (path, item) in paths {
                for (method, op) in item.as_object().into_iter().flatten() {
                    out.push((format!("{method} {path}"), op.clone()));
                }
            }
        }
        out
    }

    /// A handler that reads credentials must never sit behind a public `Cache-Control`.
    #[test]
    fn publicly_cached_routes_take_no_credentials() {
        let ops: Vec<_> = operations(static_routes())
            .into_iter()
            .chain(operations(listing_routes()))
            .collect();
        assert!(ops.len() >= 5, "{ops:?}");
        for (name, op) in ops {
            assert!(name.starts_with("get "), "{name}: only reads are cached");
            assert!(op.get("security").is_none(), "{name} requires credentials");
        }
    }
}
