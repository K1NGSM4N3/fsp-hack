//! The notification feed, the unread badge, email settings and the public unsubscribe link
//! (architecture.md §13).

use axum::{
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, Role},
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Query},
    notify::{
        Audience, EmailMask, Event, NotificationKind,
        render::{self, Channel},
        token::{self, Scope},
    },
    state::AppState,
};

const TAG: &str = "Уведомления";
const PAGE: i64 = 20;
const MAX_PAGE: i64 = 50;
/// The badge says «99+» past this.
const UNREAD_CAP: i64 = 100;
const MAX_IDS: usize = 100;

pub fn me_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(feed))
        .routes(routes!(mark_read))
        .routes(routes!(unread_count))
        .routes(routes!(settings, update_settings))
}

pub fn public_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(unsubscribe_page, unsubscribe))
}

#[derive(Serialize, ToSchema)]
pub struct NotificationView {
    pub id: Uuid,
    pub kind: NotificationKind,
    /// One line: who did what.
    pub title: String,
    pub body: String,
    /// Where it leads in the cabinet, e.g. `/candidate/invitations/{id}`.
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct NotificationPage {
    pub items: Vec<NotificationView>,
    /// Pass as `cursor` for the next page; `null` on the last one.
    pub next_cursor: Option<Uuid>,
}

#[derive(Deserialize, IntoParams)]
pub struct FeedQuery {
    /// Only unread items.
    pub unread: Option<bool>,
    /// `next_cursor` of the previous page.
    pub cursor: Option<Uuid>,
    /// 1–50, 20 by default.
    pub limit: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct UnreadCount {
    /// Up to 100: the badge shows «99+» beyond that.
    pub count: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct MarkRead {
    /// Up to 100 ids of your own items.
    #[serde(default)]
    pub ids: Vec<Uuid>,
    /// Everything unread.
    #[serde(default)]
    pub all: bool,
}

#[derive(Serialize, ToSchema)]
pub struct KindSetting {
    pub kind: NotificationKind,
    pub title: &'static str,
    pub description: &'static str,
    /// Always on: the feed in the cabinet keeps every item.
    pub in_app: bool,
    pub email: bool,
}

#[derive(Serialize, ToSchema)]
pub struct NotificationSettings {
    /// The kinds of your role, in the order the settings page shows them.
    pub items: Vec<KindSetting>,
}

#[derive(Deserialize, ToSchema)]
pub struct EmailToggle {
    pub kind: NotificationKind,
    pub enabled: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct SettingsInput {
    /// Only the kinds to change; up to one entry per kind of your role.
    pub email: Vec<EmailToggle>,
}

#[derive(Deserialize, IntoParams)]
pub struct UnsubscribeQuery {
    /// Token from the letter's unsubscribe link.
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct Unsubscribed {
    /// The kind turned off; `null` — every email.
    pub kind: Option<NotificationKind>,
    /// What stopped, in words: «Новые приглашения» or «все письма».
    pub title: String,
    /// Where to turn it back on after signing in.
    pub settings_path: &'static str,
}

fn audience(role: Role) -> Option<Audience> {
    match role {
        Role::Candidate => Some(Audience::Candidate),
        Role::Employer => Some(Audience::Employer),
        Role::Admin => None,
    }
}

pub(crate) fn view(row: db::notifications::FeedRow) -> Option<NotificationView> {
    let Some(event) = Event::parse(&row.payload) else {
        tracing::warn!(id = %row.id, kind = %row.kind, "notification payload unreadable");
        return None;
    };
    let text = render::text(&event, Channel::InApp);
    Some(NotificationView {
        id: row.id,
        kind: event.kind(),
        title: text.title,
        body: text.body,
        link: text.link,
        created_at: row.created_at,
        read_at: row.read_at,
    })
}

pub(crate) fn settings_view(role: Role, email_off: i32) -> NotificationSettings {
    let mask = EmailMask(email_off);
    NotificationSettings {
        items: audience(role)
            .into_iter()
            .flat_map(NotificationKind::of)
            .map(|kind| KindSetting {
                kind,
                title: kind.title(),
                description: kind.description(),
                in_app: true,
                email: mask.allows(kind),
            })
            .collect(),
    }
}

/// Your notifications, newest first. Keyset pagination: pass `next_cursor` as `cursor`
#[utoipa::path(get, path = "/notifications", tag = TAG,
    params(FeedQuery),
    responses((status = 200, body = NotificationPage), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn feed(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<FeedQuery>,
) -> ApiResult<Json<NotificationPage>> {
    let limit = q.limit.unwrap_or(PAGE).clamp(1, MAX_PAGE);
    let mut rows = db::notifications::feed(
        &state.pool,
        user.id,
        q.unread.unwrap_or(false),
        q.cursor,
        limit + 1,
    )
    .await?;
    let more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let next_cursor = more.then(|| rows.last().map(|r| r.id)).flatten();
    Ok(Json(NotificationPage {
        items: rows.into_iter().filter_map(view).collect(),
        next_cursor,
    }))
}

/// Mark items read: the listed `ids` or `all`. Returns the new unread count
#[utoipa::path(post, path = "/notifications/read", tag = TAG,
    request_body = MarkRead,
    responses(
        (status = 200, body = UnreadCount),
        (status = 401, body = Problem),
        (status = 422, description = "`validation_failed`: `ids` — neither ids nor `all`, or more than 100 ids", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn mark_read(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<MarkRead>,
) -> ApiResult<Json<UnreadCount>> {
    if req.ids.len() > MAX_IDS {
        return Err(ApiError::field("ids", "Не больше 100 за раз"));
    }
    if req.all {
        db::notifications::mark_all_read(&state.pool, user.id).await?;
    } else if req.ids.is_empty() {
        return Err(ApiError::field("ids", "Укажите уведомления или all: true"));
    } else {
        db::notifications::mark_read(&state.pool, user.id, &req.ids).await?;
    }
    let count = db::notifications::unread_count(&state.pool, user.id, UNREAD_CAP).await?;
    Ok(Json(UnreadCount { count }))
}

/// Unread items for the badge: one indexed count, made to be polled
#[utoipa::path(get, path = "/notifications/unread-count", tag = TAG,
    responses((status = 200, body = UnreadCount), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn unread_count(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<UnreadCount>> {
    let count = db::notifications::unread_count(&state.pool, user.id, UNREAD_CAP).await?;
    Ok(Json(UnreadCount { count }))
}

/// Which notifications also come by email. In-app is always on
#[utoipa::path(get, path = "/notification-settings", tag = TAG,
    responses((status = 200, body = NotificationSettings), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn settings(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<NotificationSettings>> {
    let off = db::notifications::email_off(&state.pool, user.id).await?;
    Ok(Json(settings_view(user.role, off)))
}

/// Turn email on or off per kind; kinds not listed keep their setting. A letter already queued
/// leaves out the kinds turned off before it is sent
#[utoipa::path(put, path = "/notification-settings", tag = TAG,
    request_body = SettingsInput,
    responses(
        (status = 200, body = NotificationSettings),
        (status = 401, body = Problem),
        (status = 422, description = "`validation_failed`: `email` — a kind of the other role, or the same kind twice", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn update_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SettingsInput>,
) -> ApiResult<Json<NotificationSettings>> {
    let Some(audience) = audience(user.role) else {
        return Err(ApiError::field("email", "Для этой роли писем нет"));
    };
    let (mut off, mut on, mut seen) = (0i32, 0i32, 0i32);
    for t in &req.email {
        if t.kind.audience() != audience {
            return Err(ApiError::field("email", "Это уведомление другой роли"));
        }
        if seen & t.kind.bit() != 0 {
            return Err(ApiError::field("email", "Каждый вид — один раз"));
        }
        seen |= t.kind.bit();
        if t.enabled {
            on |= t.kind.bit();
        } else {
            off |= t.kind.bit();
        }
    }
    let mask = db::notifications::update_email_off(&state.pool, user.id, off, on).await?;
    Ok(Json(settings_view(user.role, mask)))
}

fn scope_label(scope: Scope) -> (Option<NotificationKind>, String) {
    match scope {
        Scope::Kind(k) => (Some(k), k.title().to_owned()),
        Scope::All => (None, "все письма".to_owned()),
    }
}

/// The letter's unsubscribe link opened in a browser: a GET never unsubscribes (RFC 8058 — mail
/// scanners follow links), it leads to the page with the button
#[utoipa::path(get, path = "/notifications/unsubscribe", tag = TAG,
    params(UnsubscribeQuery),
    responses(
        (status = 303, description = "To `/notifications/unsubscribe?token=…` on the site", headers(("Location" = String))),
        (status = 400, description = "`invalid_token`", body = Problem),
    ))]
async fn unsubscribe_page(
    State(state): State<AppState>,
    Query(q): Query<UnsubscribeQuery>,
) -> ApiResult<Response> {
    let key = state.crypto.derive_key(token::KEY_PURPOSE);
    if token::verify(&key, &q.token).is_none() {
        return Err(invalid_token());
    }
    let location = format!(
        "{}/notifications/unsubscribe?token={}",
        state.config.public_url,
        q.token.trim()
    );
    let location = HeaderValue::from_str(&location).map_err(|_| invalid_token())?;
    Ok((StatusCode::SEE_OTHER, [(header::LOCATION, location)]).into_response())
}

fn invalid_token() -> ApiError {
    ApiError::bad_request("invalid_token", "Ссылка недействительна")
        .with_detail("Откройте ссылку из письма целиком")
}

/// One-click unsubscribe (RFC 8058): the mail client POSTs `List-Unsubscribe=One-Click` here,
/// the site's button POSTs the same. No session needed — the signed token names the account and
/// the kind. Repeating it changes nothing
#[utoipa::path(post, path = "/notifications/unsubscribe", tag = TAG,
    params(UnsubscribeQuery),
    responses(
        (status = 200, body = Unsubscribed),
        (status = 400, description = "`invalid_token`: not a link from our letter", body = Problem),
    ))]
async fn unsubscribe(
    State(state): State<AppState>,
    Query(q): Query<UnsubscribeQuery>,
) -> ApiResult<Json<Unsubscribed>> {
    let key = state.crypto.derive_key(token::KEY_PURPOSE);
    let (user, scope) = token::verify(&key, &q.token).ok_or_else(invalid_token)?;
    let (bits, settings_path) = match scope {
        Scope::Kind(k) => (k.bit(), render::settings_path(k.audience())),
        Scope::All => {
            let role = db::users::user_by_id(&state.pool, user)
                .await?
                .map(|u| Role::from(u.role));
            let audience = role.and_then(audience).unwrap_or(Audience::Candidate);
            (EmailMask::all_of(audience), render::settings_path(audience))
        }
    };
    db::notifications::turn_email_off(&state.pool, user, bits).await?;
    let (kind, title) = scope_label(scope);
    Ok(Json(Unsubscribed {
        kind,
        title,
        settings_path,
    }))
}

#[derive(Serialize, ToSchema)]
pub struct EmailRecord {
    pub id: Uuid,
    /// `pending`, `sent`, `failed` or `skipped` (every kind in it was turned off meanwhile).
    pub status: String,
    pub items: i64,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct NotificationExport {
    pub settings: NotificationSettings,
    pub items: Vec<NotificationView>,
    /// Letters the platform sent or tried to send you; their text is the items above.
    pub emails: Vec<EmailRecord>,
}

/// Everything about notifications for the data export (ст. 14 152-ФЗ).
pub(crate) async fn export(state: &AppState, user: AuthUser) -> ApiResult<NotificationExport> {
    let off = db::notifications::email_off(&state.pool, user.id).await?;
    let rows = db::notifications::feed(&state.pool, user.id, false, None, i64::MAX).await?;
    let emails = db::notifications::user_emails(&state.pool, user.id).await?;
    Ok(NotificationExport {
        settings: settings_view(user.role, off),
        items: rows.into_iter().filter_map(view).collect(),
        emails: emails
            .into_iter()
            .map(|e| EmailRecord {
                id: e.id,
                status: match e.status {
                    db::types::NotificationEmailStatus::Pending => "pending",
                    db::types::NotificationEmailStatus::Sent => "sent",
                    db::types::NotificationEmailStatus::Failed => "failed",
                    db::types::NotificationEmailStatus::Skipped => "skipped",
                }
                .to_owned(),
                items: e.items,
                created_at: e.created_at,
                sent_at: e.sent_at,
            })
            .collect(),
    })
}
