pub mod age;
pub mod auth;
pub mod cards;
pub mod config;
pub mod contacts;
pub mod crypto;
pub mod db;
pub mod dto;
pub mod error;
pub mod fsp_client;
pub mod guardian;
pub mod integrations;
pub mod kv;
pub mod mail;
pub mod metrics;
pub mod pdf;
pub mod pool;
pub mod proctoring;
pub mod resume;
pub mod routes;
pub mod seed;
pub mod skills;
pub mod state;
pub mod trust;

use std::time::Duration;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderName, StatusCode},
    middleware,
    routing::get,
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
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

use crate::state::AppState;

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
        .merge(routes::vacancies::public_router())
        .merge(routes::guardian::public_router())
        .merge(routes::reports::router())
        .merge(routes::catalog::router())
        .merge(routes::assessment::catalog_router())
        .merge(routes::eval::router())
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, api) = OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", api_v1())
        .split_for_parts();
    api
}

pub fn app(state: AppState) -> Router {
    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api/v1", api_v1())
        .split_for_parts();

    let x_request_id = HeaderName::from_static("x-request-id");
    Router::new()
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
        .layer(DefaultBodyLimit::max(256 * 1024))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(SetSensitiveRequestHeadersLayer::new([
            axum::http::header::AUTHORIZATION,
            axum::http::header::COOKIE,
        ]))
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
        .layer(CatchPanicLayer::new())
        .with_state(state)
}
