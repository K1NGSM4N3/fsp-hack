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
    dto::EmailStatus,
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Query},
    notify::{
        Audience, EmailMask, Event, NotificationKind,
        render::{self, Channel},
        token::{self, Scope},
    },
    page,
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

/// `rows` were fetched with `limit + 1`: keeps `limit` of them and, when there was one more, the
/// id to continue after.
fn feed_page(
    mut rows: Vec<db::notifications::FeedRow>,
    limit: i64,
) -> (Vec<db::notifications::FeedRow>, Option<Uuid>) {
    let keep = usize::try_from(limit).unwrap_or(usize::MAX);
    if rows.len() <= keep {
        return (rows, None);
    }
    rows.truncate(keep);
    let next = rows.last().map(|r| r.id);
    (rows, next)
}

/// Your notifications, newest first. Keyset pagination: pass `next_cursor` as `cursor`
#[utoipa::path(get, path = "/notifications", tag = TAG, operation_id = "list_notifications",
    params(FeedQuery),
    responses((status = 200, body = NotificationPage), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn feed(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<FeedQuery>,
) -> ApiResult<Json<NotificationPage>> {
    let limit = page::limit(q.limit, PAGE, MAX_PAGE);
    let rows = db::notifications::feed(
        &state.pool,
        user.id,
        q.unread.unwrap_or(false),
        q.cursor,
        limit + 1,
    )
    .await?;
    let (rows, next_cursor) = feed_page(rows, limit);
    Ok(Json(NotificationPage {
        items: rows.into_iter().filter_map(view).collect(),
        next_cursor,
    }))
}

#[derive(Debug, PartialEq, Eq)]
enum ReadTarget<'a> {
    All,
    Ids(&'a [Uuid]),
}

fn read_target(req: &MarkRead) -> ApiResult<ReadTarget<'_>> {
    if req.ids.len() > MAX_IDS {
        return Err(ApiError::field("ids", "Не больше 100 за раз"));
    }
    if req.all {
        Ok(ReadTarget::All)
    } else if req.ids.is_empty() {
        Err(ApiError::field("ids", "Укажите уведомления или all: true"))
    } else {
        Ok(ReadTarget::Ids(&req.ids))
    }
}

/// Mark items read: the listed `ids` or `all`. Returns the new unread count
#[utoipa::path(post, path = "/notifications/read", tag = TAG, operation_id = "mark_notifications_read",
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
    match read_target(&req)? {
        ReadTarget::All => {
            db::notifications::mark_all_read(&state.pool, user.id).await?;
        }
        ReadTarget::Ids(ids) => {
            db::notifications::mark_read(&state.pool, user.id, ids).await?;
        }
    }
    let count = db::notifications::unread_count(&state.pool, user.id, UNREAD_CAP).await?;
    Ok(Json(UnreadCount { count }))
}

/// Unread items for the badge: one indexed count, made to be polled
#[utoipa::path(get, path = "/notifications/unread-count", tag = TAG, operation_id = "unread_notifications",
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
#[utoipa::path(get, path = "/notification-settings", tag = TAG, operation_id = "notification_settings",
    responses((status = 200, body = NotificationSettings), (status = 401, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn settings(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<NotificationSettings>> {
    let off = db::notifications::email_off(&state.pool, user.id).await?;
    Ok(Json(settings_view(user.role, off)))
}

/// The bits to turn off and on: kinds of the caller's role only, each at most once.
fn email_toggles(audience: Audience, toggles: &[EmailToggle]) -> ApiResult<(i32, i32)> {
    let (mut off, mut on, mut seen) = (0i32, 0i32, 0i32);
    for t in toggles {
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
    Ok((off, on))
}

/// Turn email on or off per kind; kinds not listed keep their setting. A letter already queued
/// leaves out the kinds turned off before it is sent
#[utoipa::path(put, path = "/notification-settings", tag = TAG, operation_id = "update_notification_settings",
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
    let (off, on) = email_toggles(audience, &req.email)?;
    let mask = db::notifications::update_email_off(&state.pool, user.id, off, on).await?;
    Ok(Json(settings_view(user.role, mask)))
}

/// What a token turns off for an account of `audience` (for `Scope::Kind`, the kind's own
/// audience): the bits and the answer.
fn unsubscribe_effect(scope: Scope, audience: Audience) -> (i32, Unsubscribed) {
    let (bits, kind, audience) = match scope {
        Scope::Kind(k) => (k.bit(), Some(k), k.audience()),
        Scope::All => (EmailMask::all_of(audience), None, audience),
    };
    let view = Unsubscribed {
        kind,
        title: scope.label().to_owned(),
        settings_path: render::settings_path(audience),
    };
    (bits, view)
}

/// The letter's unsubscribe link opened in a browser: a GET never unsubscribes (RFC 8058 — mail
/// scanners follow links), it leads to the page with the button
#[utoipa::path(get, path = "/notifications/unsubscribe", tag = TAG, operation_id = "unsubscribe_link",
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
#[utoipa::path(post, path = "/notifications/unsubscribe", tag = TAG, operation_id = "unsubscribe",
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
    let account = match scope {
        Scope::Kind(k) => k.audience(),
        Scope::All => db::users::user_by_id(&state.pool, user)
            .await?
            .map(|u| Role::from(u.role))
            .and_then(audience)
            .unwrap_or(Audience::Candidate),
    };
    let (bits, view) = unsubscribe_effect(scope, account);
    db::notifications::turn_email_off(&state.pool, user, bits).await?;
    Ok(Json(view))
}

#[derive(Serialize, ToSchema)]
pub struct EmailRecord {
    pub id: Uuid,
    /// `skipped`: every kind in it was turned off before it went out.
    pub status: EmailStatus,
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
                status: e.status.into(),
                items: e.items,
                created_at: e.created_at,
                sent_at: e.sent_at,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn messages(e: &ApiError) -> serde_json::Value {
        match e {
            ApiError::Validation(errors) => serde_json::to_value(errors).unwrap(),
            other => panic!("expected a validation error, got {other:?}"),
        }
    }

    fn toggle(kind: NotificationKind, enabled: bool) -> EmailToggle {
        EmailToggle { kind, enabled }
    }

    #[test]
    fn settings_list_the_kinds_of_the_role_with_everything_on_by_default() {
        let candidate = settings_view(Role::Candidate, 0);
        assert_eq!(candidate.items.len(), 11);
        assert!(candidate.items.iter().all(|i| i.email && i.in_app));
        assert!(
            candidate
                .items
                .iter()
                .all(|i| i.kind.audience() == Audience::Candidate)
        );
        assert_eq!(
            candidate.items[0].kind,
            NotificationKind::InvitationReceived
        );
        assert_eq!(candidate.items[0].title, "Новые приглашения");
        let employer = settings_view(Role::Employer, 0);
        assert_eq!(employer.items.len(), 9);
        assert!(
            employer
                .items
                .iter()
                .all(|i| i.kind.audience() == Audience::Employer && i.email)
        );
        assert!(settings_view(Role::Admin, 0).items.is_empty());
    }

    #[test]
    fn a_kind_turned_off_loses_only_its_email() {
        let view = settings_view(Role::Candidate, NotificationKind::InvitationReceived.bit());
        let off: Vec<NotificationKind> = view
            .items
            .iter()
            .filter(|i| !i.email)
            .map(|i| i.kind)
            .collect();
        assert_eq!(off, [NotificationKind::InvitationReceived]);
        assert!(view.items.iter().all(|i| i.in_app));
    }

    #[test]
    fn email_toggles_become_bits_to_turn_off_and_on() {
        let (off, on) = email_toggles(
            Audience::Candidate,
            &[
                toggle(NotificationKind::InvitationExpiring, false),
                toggle(NotificationKind::InvitationReceived, true),
                toggle(NotificationKind::HiringStage, false),
            ],
        )
        .unwrap();
        assert_eq!(
            off,
            NotificationKind::InvitationExpiring.bit() | NotificationKind::HiringStage.bit()
        );
        assert_eq!(on, NotificationKind::InvitationReceived.bit());
        assert_eq!(email_toggles(Audience::Employer, &[]).unwrap(), (0, 0));
    }

    #[test]
    fn a_kind_of_the_other_role_is_rejected() {
        let e = email_toggles(
            Audience::Candidate,
            &[toggle(NotificationKind::InvitationAccepted, false)],
        )
        .unwrap_err();
        assert_eq!(e.summary(), (422, "validation_failed", vec!["email"]));
        assert_eq!(
            messages(&e),
            json!({ "email": ["Это уведомление другой роли"] })
        );
    }

    #[test]
    fn the_same_kind_twice_is_rejected() {
        let e = email_toggles(
            Audience::Employer,
            &[
                toggle(NotificationKind::TrustChanged, false),
                toggle(NotificationKind::TrustChanged, true),
            ],
        )
        .unwrap_err();
        assert_eq!(e.summary(), (422, "validation_failed", vec!["email"]));
        assert_eq!(messages(&e), json!({ "email": ["Каждый вид — один раз"] }));
    }

    #[test]
    fn unsubscribing_from_a_kind_turns_off_only_its_bit() {
        let (bits, view) = unsubscribe_effect(
            Scope::Kind(NotificationKind::InvitationExpiring),
            Audience::Candidate,
        );
        assert_eq!(bits, NotificationKind::InvitationExpiring.bit());
        assert_eq!(view.kind, Some(NotificationKind::InvitationExpiring));
        assert_eq!(view.title, "Напоминание о сроке ответа");
        assert_eq!(view.settings_path, "/candidate/settings#notifications");

        let (bits, view) = unsubscribe_effect(
            Scope::Kind(NotificationKind::TrustChanged),
            Audience::Employer,
        );
        assert_eq!(bits, NotificationKind::TrustChanged.bit());
        assert_eq!(view.title, "Уровень доверия и квота");
        assert_eq!(view.settings_path, "/employer/company#notifications");
    }

    #[test]
    fn unsubscribing_from_everything_turns_off_every_kind_of_the_account() {
        let (bits, view) = unsubscribe_effect(Scope::All, Audience::Employer);
        assert_eq!(bits, EmailMask::all_of(Audience::Employer));
        assert_eq!(view.kind, None);
        assert_eq!(view.title, "все письма");
        assert_eq!(view.settings_path, "/employer/company#notifications");

        let (bits, view) = unsubscribe_effect(Scope::All, Audience::Candidate);
        assert_eq!(bits, EmailMask::all_of(Audience::Candidate));
        assert_eq!(view.settings_path, "/candidate/settings#notifications");
    }

    fn mark(ids: u128, all: bool) -> MarkRead {
        MarkRead {
            ids: (0..ids).map(Uuid::from_u128).collect(),
            all,
        }
    }

    #[test]
    fn mark_read_takes_the_listed_ids_or_everything() {
        let some = mark(2, false);
        assert_eq!(read_target(&some).unwrap(), ReadTarget::Ids(&some.ids));
        assert_eq!(read_target(&mark(0, true)).unwrap(), ReadTarget::All);
        assert_eq!(read_target(&mark(3, true)).unwrap(), ReadTarget::All);
        let full = mark(100, false);
        assert_eq!(read_target(&full).unwrap(), ReadTarget::Ids(&full.ids));
    }

    #[test]
    fn mark_read_needs_ids_or_all() {
        let e = read_target(&mark(0, false)).unwrap_err();
        assert_eq!(e.summary(), (422, "validation_failed", vec!["ids"]));
        assert_eq!(
            messages(&e),
            json!({ "ids": ["Укажите уведомления или all: true"] })
        );
    }

    #[test]
    fn mark_read_takes_at_most_100_ids_even_with_all() {
        for all in [false, true] {
            let e = read_target(&mark(101, all)).unwrap_err();
            assert_eq!(messages(&e), json!({ "ids": ["Не больше 100 за раз"] }));
        }
    }

    fn rows(n: u128) -> Vec<db::notifications::FeedRow> {
        let at = DateTime::from_timestamp(1_790_000_000, 0).unwrap();
        (1..=n)
            .map(|i| db::notifications::FeedRow {
                id: Uuid::from_u128(i),
                kind: "invitation_received".into(),
                payload: "{}".into(),
                created_at: at,
                read_at: None,
            })
            .collect()
    }

    #[test]
    fn feed_page_keeps_the_limit_and_continues_after_its_last_item() {
        let (page, next) = feed_page(rows(3), 2);
        assert_eq!(page.len(), 2);
        assert_eq!(next, Some(Uuid::from_u128(2)));
        let (page, next) = feed_page(rows(2), 2);
        assert_eq!((page.len(), next), (2, None));
        let (page, next) = feed_page(Vec::new(), 20);
        assert_eq!((page.len(), next), (0, None));
    }

    #[test]
    fn feed_limit_is_20_by_default_and_kept_within_1_to_50() {
        assert_eq!(page::limit(None, PAGE, MAX_PAGE), 20);
        assert_eq!(page::limit(Some(0), PAGE, MAX_PAGE), 1);
        assert_eq!(page::limit(Some(-5), PAGE, MAX_PAGE), 1);
        assert_eq!(page::limit(Some(500), PAGE, MAX_PAGE), 50);
    }
}
