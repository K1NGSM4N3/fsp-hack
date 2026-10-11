//! Employer cabinet «Интеграции»: the outbound webhook (URL, one-time secret, rotation, test
//! event, delivery log with manual retry) and API keys for the company's ATS.

use std::time::Duration as StdDuration;

use axum::{extract::State, http::StatusCode};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::company::company_id;
use crate::{
    auth::EmployerUser,
    dto::DeliveryStatus,
    error::{ApiError, ApiResult, Problem, Validator},
    extract::{Json, Path, Query},
    integrations::{
        dispatcher::{self, Dispatcher},
        events::{DELIVERY_WINDOW_HOURS, EventType, Outbox},
        keys::{self, MAX_ACTIVE_KEYS, Scope, Scopes},
        seal_secret, signature,
    },
    notify::Notifier,
    page::{self, CursorPage},
    state::AppState,
};

const TEST_EVENTS_PER_MINUTE: u64 = 10;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(integrations_overview))
        .routes(routes!(put_webhook, delete_webhook))
        .routes(routes!(rotate_webhook_secret))
        .routes(routes!(send_test_event))
        .routes(routes!(list_deliveries))
        .routes(routes!(retry_delivery))
        .routes(routes!(list_api_keys, create_api_key))
        .routes(routes!(revoke_api_key))
}

#[derive(Serialize, ToSchema)]
pub struct WebhookView {
    pub url: String,
    /// The secret itself is shown once; afterwards only its beginning.
    pub secret_hint: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub secret_rotated_at: DateTime<Utc>,
    /// After a rotation the previous secret also signs deliveries until this moment.
    pub previous_secret_valid_until: Option<DateTime<Utc>>,
}

impl From<db::integrations::Webhook> for WebhookView {
    fn from(w: db::integrations::Webhook) -> Self {
        Self::at(w, Utc::now())
    }
}

impl WebhookView {
    /// The previous secret is shown only while it still signs deliveries.
    fn at(w: db::integrations::Webhook, now: DateTime<Utc>) -> Self {
        Self {
            url: w.url,
            secret_hint: w.secret_hint,
            created_at: w.created_at,
            updated_at: w.updated_at,
            secret_rotated_at: w.secret_rotated_at,
            previous_secret_valid_until: w
                .prev_secret_enc
                .and(w.prev_expires_at)
                .filter(|t| *t > now),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct DeliveryCounts {
    pub pending: i64,
    pub delivered: i64,
    pub failed: i64,
}

#[derive(Serialize, ToSchema)]
pub struct IntegrationsOverview {
    pub webhook: Option<WebhookView>,
    pub deliveries: DeliveryCounts,
    pub active_keys: i64,
    /// Base URL of the inbound API for the ATS, e.g. `https://…/api/v1/integrations`.
    pub api_base_url: String,
    pub event_types: Vec<EventType>,
    pub scopes: Vec<Scope>,
}

/// Webhook, delivery counters and API key count
#[utoipa::path(get, path = "/integrations", tag = "integrations",
    responses(
        (status = 200, body = IntegrationsOverview),
        (status = 401, body = Problem), (status = 403, body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn integrations_overview(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<IntegrationsOverview>> {
    let company = company_id(&state, user.id).await?;
    let counts = db::integrations::event_counts(&state.pool, company).await?;
    Ok(Json(IntegrationsOverview {
        webhook: db::integrations::webhook(&state.pool, company)
            .await?
            .map(WebhookView::from),
        deliveries: DeliveryCounts {
            pending: counts.pending,
            delivered: counts.delivered,
            failed: counts.failed,
        },
        active_keys: db::integrations::active_api_keys(&state.pool, company).await?,
        api_base_url: format!("{}/api/v1/integrations", state.config.public_url),
        event_types: EventType::ALL.to_vec(),
        scopes: Scope::ALL.to_vec(),
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct WebhookInput {
    /// `https://` address of the ATS endpoint.
    pub url: String,
}

#[derive(Serialize, ToSchema)]
pub struct WebhookSaved {
    pub webhook: WebhookView,
    /// Signing secret, present only right after creation or rotation. Store it in the ATS:
    /// it is not shown again.
    pub secret: Option<String>,
}

async fn checked_url(state: &AppState, raw: &str) -> ApiResult<String> {
    let url = state
        .webhooks
        .check_url(raw)
        .map_err(|e| ApiError::field("url", e.message_ru()))?;
    state
        .webhooks
        .check_resolution(&url)
        .await
        .map_err(|e| ApiError::field("url", e.message_ru()))?;
    Ok(url.to_string())
}

async fn load_webhook(state: &AppState, company: Uuid) -> ApiResult<WebhookView> {
    db::integrations::webhook(&state.pool, company)
        .await?
        .map(WebhookView::from)
        .ok_or_else(|| ApiError::not_found().with_detail("Вебхук не настроен"))
}

/// Set the webhook URL. The first save creates the signing secret and returns it once (201);
/// later saves only change the URL (200)
#[utoipa::path(put, path = "/integrations/webhook", tag = "integrations",
    request_body = WebhookInput,
    responses(
        (status = 201, description = "Создан, `secret` показан один раз", body = WebhookSaved),
        (status = 200, description = "Адрес изменён, секрет прежний", body = WebhookSaved),
        (status = 409, description = "`company_profile_required`; `webhook_changed`", body = Problem),
        (status = 422, description = "Адрес не https, ведёт во внутреннюю сеть или не находится в DNS", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn put_webhook(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(input): Json<WebhookInput>,
) -> ApiResult<(StatusCode, Json<WebhookSaved>)> {
    let company = company_id(&state, user.id).await?;
    let url = checked_url(&state, &input.url).await?;
    if db::integrations::update_webhook_url(&state.pool, company, &url).await? > 0 {
        return Ok((
            StatusCode::OK,
            Json(WebhookSaved {
                webhook: load_webhook(&state, company).await?,
                secret: None,
            }),
        ));
    }
    let secret = signature::new_secret();
    let created = db::integrations::insert_webhook(
        &state.pool,
        company,
        &url,
        &seal_secret(&state.crypto, company, &secret)?,
        &signature::secret_hint(&secret),
    )
    .await?;
    if created == 0 {
        return Err(ApiError::conflict(
            "webhook_changed",
            "Вебхук только что настроили в другой вкладке — обновите страницу",
        ));
    }
    Ok((
        StatusCode::CREATED,
        Json(WebhookSaved {
            webhook: load_webhook(&state, company).await?,
            secret: Some(secret),
        }),
    ))
}

/// Revoke the webhook: the URL and secret are erased, queued events are cancelled
#[utoipa::path(delete, path = "/integrations/webhook", tag = "integrations",
    responses((status = 204), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn delete_webhook(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    let mut tx = state.pool.begin().await?;
    if db::integrations::delete_webhook(&mut *tx, company).await? == 0 {
        return Err(ApiError::not_found());
    }
    db::integrations::cancel_pending_events(&mut *tx, company, "Вебхук отключён").await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// New signing secret, shown once. For 24 hours deliveries carry signatures with both the new
/// and the previous secret, so the ATS can switch without losing events
#[utoipa::path(post, path = "/integrations/webhook/rotate", tag = "integrations",
    responses((status = 200, body = WebhookSaved), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn rotate_webhook_secret(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<WebhookSaved>> {
    let company = company_id(&state, user.id).await?;
    let secret = signature::new_secret();
    let rotated = db::integrations::rotate_webhook_secret(
        &state.pool,
        company,
        &seal_secret(&state.crypto, company, &secret)?,
        &signature::secret_hint(&secret),
        Utc::now() + Duration::hours(dispatcher::ROTATION_OVERLAP_HOURS),
    )
    .await?;
    if rotated == 0 {
        return Err(ApiError::not_found().with_detail("Вебхук не настроен"));
    }
    Ok(Json(WebhookSaved {
        webhook: load_webhook(&state, company).await?,
        secret: Some(secret),
    }))
}

#[derive(Serialize, ToSchema)]
pub struct TestDelivery {
    pub event_id: Uuid,
    pub delivered: bool,
    pub status_code: Option<u16>,
    pub latency_ms: i32,
    pub error: Option<String>,
}

/// Send an `integration.test` event right now and report how the endpoint answered
#[utoipa::path(post, path = "/integrations/webhook/test", tag = "integrations",
    responses(
        (status = 200, body = TestDelivery),
        (status = 404, description = "Вебхук не настроен", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
        (status = 429, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn send_test_event(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<TestDelivery>> {
    let company = company_id(&state, user.id).await?;
    let hits = state
        .kv
        .hit(
            &format!("rl:webhook-test:{company}"),
            StdDuration::from_secs(60),
        )
        .await?;
    if hits.over(TEST_EVENTS_PER_MINUTE) {
        return Err(ApiError::rate_limited("webhook_test", hits.retry_after));
    }
    if !db::integrations::webhook_enabled(&state.pool, company).await? {
        return Err(ApiError::not_found().with_detail("Вебхук не настроен"));
    }
    let lease = Duration::from_std(dispatcher::LEASE).unwrap_or(Duration::minutes(1));
    let mut tx = state.pool.begin().await?;
    let event_id = Outbox::new(&state).test(&mut tx, company, lease).await?;
    tx.commit().await?;
    let event = db::integrations::deliverable_event(&state.pool, event_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    let attempt = Dispatcher::new(
        state.pool.clone(),
        state.crypto.clone(),
        state.webhooks.clone(),
        None,
    )
    .with_notifier(Notifier::new(&state))
    .deliver(event)
    .await?;
    Ok(Json(match attempt {
        Some(a) => TestDelivery {
            event_id,
            delivered: a.ok(),
            status_code: a.status,
            latency_ms: a.latency_ms,
            error: a.error,
        },
        None => TestDelivery {
            event_id,
            delivered: false,
            status_code: None,
            latency_ms: 0,
            error: Some("Вебхук не настроен".into()),
        },
    }))
}

#[derive(Serialize, ToSchema)]
pub struct AttemptView {
    pub at: DateTime<Utc>,
    pub url: String,
    pub status_code: Option<i16>,
    pub latency_ms: i32,
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DeliveryView {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub status: DeliveryStatus,
    pub created_at: DateTime<Utc>,
    /// For `pending`: when the next attempt is due.
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    /// Every HTTP attempt, oldest first.
    pub attempts: Vec<AttemptView>,
}

#[derive(Deserialize, IntoParams)]
pub struct DeliveriesQuery {
    /// Only events in this status.
    pub status: Option<DeliveryStatus>,
    /// `next_cursor` of the previous page.
    pub cursor: Option<String>,
    /// Default 50, at most 200.
    pub limit: Option<i64>,
}

/// Integration events with the per-attempt delivery log, newest first. Keyset pagination: pass
/// `next_cursor` as `cursor`
#[utoipa::path(get, path = "/integrations/deliveries", tag = "integrations",
    params(DeliveriesQuery),
    responses(
        (status = 200, body = CursorPage<DeliveryView>),
        (status = 400, description = "`invalid_query` — `cursor` не из ответа", body = Problem),
        (status = 409, description = "`company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_deliveries(
    State(state): State<AppState>,
    user: EmployerUser,
    Query(q): Query<DeliveriesQuery>,
) -> ApiResult<Json<CursorPage<DeliveryView>>> {
    let before = page::decode(q.cursor.as_deref())?;
    let limit = page::limit(q.limit, 50, 200);
    let company = company_id(&state, user.id).await?;
    let events = db::integrations::company_events(
        &state.pool,
        company,
        q.status.map(Into::into),
        before,
        limit + 1,
    )
    .await?;
    let (events, next_cursor) = page::split(events, limit, |e| (e.created_at, e.id));
    let ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();
    let attempts = db::integrations::event_attempts(&state.pool, &ids).await?;
    let mut out = Vec::with_capacity(events.len());
    for e in events {
        let Some(event_type) = EventType::parse(&e.event_type) else {
            continue;
        };
        let mine: Vec<AttemptView> = attempts
            .iter()
            .filter(|a| a.event_id == e.id)
            .map(|a| AttemptView {
                at: a.attempted_at,
                url: a.url.clone(),
                status_code: a.status_code,
                latency_ms: a.latency_ms,
                error: a.error.clone(),
            })
            .collect();
        let status: DeliveryStatus = e.status.into();
        out.push(DeliveryView {
            event_id: e.id,
            event_type,
            status,
            created_at: e.created_at,
            next_attempt_at: (status == DeliveryStatus::Pending).then_some(e.next_attempt_at),
            delivered_at: e.delivered_at,
            last_error: e.last_error,
            attempts: mine,
        });
    }
    Ok(Json(CursorPage {
        items: out,
        next_cursor,
    }))
}

/// «Отправить ещё раз»: queue the event again now, with a fresh 24-hour retry window
#[utoipa::path(post, path = "/integrations/deliveries/{id}/retry", tag = "integrations",
    params(("id" = Uuid, Path, description = "Event id (`FSP-Event-Id`)")),
    responses(
        (status = 202),
        (status = 404, body = Problem),
        (status = 409, description = "`webhook_missing`, `company_profile_required`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn retry_delivery(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    if !db::integrations::webhook_enabled(&state.pool, company).await? {
        return Err(ApiError::conflict(
            "webhook_missing",
            "Сначала укажите адрес вебхука",
        ));
    }
    let deadline = Utc::now() + Duration::hours(DELIVERY_WINDOW_HOURS);
    if db::integrations::retry_event(&state.pool, company, id, deadline).await? == 0 {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::ACCEPTED)
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyView {
    pub id: Uuid,
    pub name: String,
    /// Beginning of the key, e.g. `fsp_live_ab12cd34`.
    pub prefix: String,
    pub scopes: Vec<Scope>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<db::integrations::ApiKeyRow> for ApiKeyView {
    fn from(k: db::integrations::ApiKeyRow) -> Self {
        Self {
            id: k.id,
            name: k.name,
            prefix: k.prefix,
            scopes: Scopes::from_strs(&k.scopes).to_vec(),
            created_at: k.created_at,
            last_used_at: k.last_used_at,
            revoked_at: k.revoked_at,
        }
    }
}

/// Company API keys, active first
#[utoipa::path(get, path = "/integrations/keys", tag = "integrations",
    responses((status = 200, body = Vec<ApiKeyView>), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn list_api_keys(
    State(state): State<AppState>,
    user: EmployerUser,
) -> ApiResult<Json<Vec<ApiKeyView>>> {
    let company = company_id(&state, user.id).await?;
    Ok(Json(
        db::integrations::company_api_keys(&state.pool, company)
            .await?
            .into_iter()
            .map(ApiKeyView::from)
            .collect(),
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct ApiKeyInput {
    /// What the key is for, e.g. «Huntflow».
    pub name: String,
    pub scopes: Vec<Scope>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyCreated {
    pub key: ApiKeyView,
    /// The key itself, shown once: `Authorization: Bearer fsp_live_…`.
    pub secret: String,
}

/// Create an API key with the chosen scopes; the key is returned once
/// A key needs a name the company will recognize it by and at least one right.
fn check_key_input(name: &str, scopes: Scopes) -> ApiResult<&str> {
    let name = name.trim();
    let mut v = Validator::default();
    v.check(
        !name.is_empty(),
        "name",
        "Назовите ключ, например «Huntflow»",
    )
    .check(
        name.chars().count() <= 100,
        "name",
        "Не длиннее 100 символов",
    )
    .check(!scopes.is_empty(), "scopes", "Выберите хотя бы одно право");
    v.finish()?;
    Ok(name)
}

#[utoipa::path(post, path = "/integrations/keys", tag = "integrations",
    request_body = ApiKeyInput,
    responses(
        (status = 201, body = ApiKeyCreated),
        (status = 409, description = "`api_key_limit` — не больше 10 действующих ключей, `company_profile_required`", body = Problem),
        (status = 422, body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn create_api_key(
    State(state): State<AppState>,
    user: EmployerUser,
    Json(input): Json<ApiKeyInput>,
) -> ApiResult<(StatusCode, Json<ApiKeyCreated>)> {
    let scopes = Scopes::of(&input.scopes);
    let name = check_key_input(&input.name, scopes)?;
    let company = company_id(&state, user.id).await?;
    if db::integrations::active_api_keys(&state.pool, company).await? >= MAX_ACTIVE_KEYS {
        return Err(ApiError::conflict(
            "api_key_limit",
            format!("Не больше {MAX_ACTIVE_KEYS} действующих ключей — отзовите ненужный"),
        ));
    }
    let new = keys::generate();
    let id = Uuid::now_v7();
    db::integrations::insert_api_key(
        &state.pool,
        &db::integrations::NewApiKey {
            id,
            company_id: company,
            name,
            prefix: &new.prefix,
            key_hash: &new.hash,
            scopes: &scopes.to_strings(),
            created_by: Some(user.id),
        },
    )
    .await?;
    let key = db::integrations::company_api_keys(&state.pool, company)
        .await?
        .into_iter()
        .find(|k| k.id == id)
        .map(ApiKeyView::from)
        .ok_or(sqlx::Error::RowNotFound)?;
    Ok((
        StatusCode::CREATED,
        Json(ApiKeyCreated {
            key,
            secret: new.key,
        }),
    ))
}

/// Revoke an API key: it stops working immediately
#[utoipa::path(delete, path = "/integrations/keys/{id}", tag = "integrations",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 404, body = Problem), (status = 409, description = "`company_profile_required`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn revoke_api_key(
    State(state): State<AppState>,
    user: EmployerUser,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let company = company_id(&state, user.id).await?;
    if db::integrations::revoke_api_key(&state.pool, company, id).await? == 0 {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::keys::Scope;

    #[test]
    fn a_key_needs_a_name_and_a_right() {
        let one = Scopes::of(&[Scope::InvitationsRead]);
        assert_eq!(check_key_input("  Huntflow ", one).ok(), Some("Huntflow"));
        let both = check_key_input("  ", Scopes::of(&[]))
            .unwrap_err()
            .summary()
            .2;
        assert_eq!(both, ["name", "scopes"]);
        assert_eq!(
            check_key_input(&"я".repeat(101), one)
                .unwrap_err()
                .summary()
                .2,
            ["name"]
        );
        assert!(check_key_input(&"я".repeat(100), one).is_ok());
    }

    #[test]
    fn the_previous_secret_is_shown_while_it_still_signs() {
        let now = Utc::now();
        let webhook =
            |prev: Option<Vec<u8>>, until: Option<DateTime<Utc>>| db::integrations::Webhook {
                url: "https://ats.example/hook".into(),
                secret_enc: vec![1],
                secret_hint: "whsec_abcd…".into(),
                prev_secret_enc: prev,
                prev_expires_at: until,
                created_at: now,
                updated_at: now,
                secret_rotated_at: now,
            };
        let later = now + Duration::hours(24);
        let view = WebhookView::at(webhook(Some(vec![2]), Some(later)), now);
        assert_eq!(view.previous_secret_valid_until, Some(later));
        let view = WebhookView::at(webhook(Some(vec![2]), Some(later)), later);
        assert_eq!(view.previous_secret_valid_until, None, "expired");
        let view = WebhookView::at(webhook(None, Some(later)), now);
        assert_eq!(view.previous_secret_valid_until, None, "no previous secret");
        let shown = serde_json::to_string(&view).unwrap();
        assert!(
            !shown.contains("secret_enc") && shown.contains("whsec_abcd…"),
            "only the hint"
        );
    }
}
