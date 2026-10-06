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
pub mod extract;
pub mod fsp_client;
pub mod guardian;
pub mod http_cache;
pub mod integrations;
pub mod kv;
pub mod mail;
pub mod metrics;
pub mod moderation;
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
        description = "Платформа подбора ИТ-специалистов: работодатель выходит на кандидата из подтверждённой тестом категории. Ошибки — RFC 9457 problem+json со стабильным полем `code`, в том числе ошибки разбора запроса: `invalid_json` (400 — не JSON, 422 — не по схеме), `unsupported_media_type` (415), `invalid_path`, `invalid_query` (400), `payload_too_large` (413). Методы с сессией отвечают 401 `unauthorized`; метод кабинета чужой роли — 403 `wrong_role`; изменяющий запрос с cookie без верного `Origin` — 403 `csrf`. 429 `rate_limited` приходит с `Retry-After` — сколько секунд осталось до сброса окна лимита.",
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
        (name = "Модерация", description = "Кабинет модератора (роль `admin`, в интерфейсе — «Модератор»): очередь разбора тестов с видеозаписью и решениями, очередь антифейка и жалоб с действиями, журнал аудита. Каждое действие и каждый просмотр записи теста пишутся в журнал. Списки — с курсором `cursor` / `next_cursor`"),
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
            routes::moderation::router().merge(routes::trust::admin_router()),
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
    let (_, mut api) = OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", api_v1())
        .split_for_parts();
    document_request_errors(&mut api);
    api
}

/// Problems raised before a handler runs, added to every operation they can reach: request
/// extractors (`extract`) by the operation's body and parameters, the session check by its
/// security. Codes a handler already lists are not repeated.
fn document_request_errors(api: &mut utoipa::openapi::OpenApi) {
    use utoipa::openapi::{
        Content, Ref, RefOr, ResponseBuilder,
        path::{Operation, ParameterIn},
    };

    fn add(op: &mut Operation, status: &str, code: &str, text: &str) {
        let line = format!("`{code}`: {text}");
        match op.responses.responses.get_mut(status) {
            Some(RefOr::T(r)) if r.description.contains(&format!("`{code}`")) => {}
            Some(RefOr::T(r)) if r.description.is_empty() => r.description = line,
            Some(RefOr::T(r)) => r.description = format!("{}; {line}", r.description),
            Some(RefOr::Ref(_)) => {}
            None => {
                let problem = Content::new(Some(Ref::from_schema_name("Problem")));
                op.responses.responses.insert(
                    status.to_owned(),
                    RefOr::T(
                        ResponseBuilder::new()
                            .description(line)
                            .content("application/json", problem)
                            .build(),
                    ),
                );
            }
        }
    }

    for item in api.paths.paths.values_mut() {
        let ops = [
            &mut item.get,
            &mut item.put,
            &mut item.post,
            &mut item.delete,
            &mut item.patch,
        ];
        for op in ops.into_iter().flatten() {
            let located = |at: ParameterIn| {
                op.parameters.iter().flatten().any(|p| match p {
                    RefOr::T(p) => p.parameter_in == at,
                    RefOr::Ref(_) => false,
                })
            };
            let (in_path, in_query) = (located(ParameterIn::Path), located(ParameterIn::Query));
            let body: Vec<String> = match &op.request_body {
                Some(RefOr::T(b)) => b.content.keys().cloned().collect(),
                _ => Vec::new(),
            };
            let session = op.security.iter().flatten().any(|s| {
                serde_json::to_value(s)
                    .is_ok_and(|v| v.get("cookie").is_some() || v.get("bearer").is_some())
            });
            if body.iter().any(|c| c == "application/json") {
                add(op, "400", "invalid_json", "the body is not valid JSON");
                add(
                    op,
                    "413",
                    "payload_too_large",
                    "the body is over the route's limit",
                );
                add(
                    op,
                    "415",
                    "unsupported_media_type",
                    "`Content-Type` is not `application/json`",
                );
                add(
                    op,
                    "422",
                    "invalid_json",
                    "the body does not match the schema (unknown enum value, wrong type, missing field)",
                );
            } else if body.iter().any(|c| c == "multipart/form-data") {
                add(
                    op,
                    "413",
                    "payload_too_large",
                    "the body is over the route's limit",
                );
                add(
                    op,
                    "415",
                    "unsupported_media_type",
                    "not `multipart/form-data`",
                );
            }
            if in_path {
                add(
                    op,
                    "400",
                    "invalid_path",
                    "a path parameter does not parse (e.g. not a UUID)",
                );
            }
            if in_query {
                add(
                    op,
                    "400",
                    "invalid_query",
                    "a query parameter is missing or does not parse",
                );
            }
            if session {
                add(op, "401", "unauthorized", "no session or it has expired");
            }
        }
    }
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
    let (api_router, mut api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", api_v1())
        .split_for_parts();
    document_request_errors(&mut api);

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
        .map_response(|res| error::problem_for_bare_413(boxed_response(res)))
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

    #[test]
    fn request_errors_are_documented_on_every_operation() {
        let api = serde_json::to_value(openapi()).unwrap_or_default();
        let described = |path: &str, method: &str, status: &str, code: &str| {
            api["paths"][path][method]["responses"][status]["description"]
                .as_str()
                .is_some_and(|d| d.contains(&format!("`{code}`")))
        };
        let start = "/api/v1/candidate/assessments";
        assert!(described(start, "post", "400", "invalid_json"));
        assert!(described(start, "post", "415", "unsupported_media_type"));
        assert!(described(start, "post", "422", "invalid_json"));
        assert!(described(start, "post", "422", "validation_failed"));
        assert!(described(start, "post", "401", "unauthorized"));
        assert!(described(
            "/api/v1/candidate/assessments/{id}",
            "get",
            "400",
            "invalid_path"
        ));
        assert!(described(
            "/api/v1/catalog/test-plan",
            "get",
            "400",
            "invalid_query"
        ));
        assert!(!described("/api/v1/catalog", "get", "401", "unauthorized"));
        assert!(described(
            "/api/v1/fsp/sync",
            "post",
            "503",
            "fsp_registry_disabled"
        ));
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
