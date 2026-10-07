//! FSP ID linking (architecture.md §7.1): OIDC Authorization Code + PKCE against the FSP
//! Keycloak realm, then a snapshot of achievements from the registry — pulled on link and on
//! request, pushed by the registry's signed webhook when a participant's achievements change.

use std::time::Duration as StdDuration;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use chrono::{DateTime, Duration, Utc};
use matching::{Achievement, FspScore};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgConnection, PgPool};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{CandidateUser, client_ip},
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Query, RawBody},
    fsp_client::{self, FspError},
    integrations::signature::{self, SIGNATURE_HEADER, SignatureError},
    notify::{Event, FspNote, Notifier},
    skills,
    state::AppState,
};

/// The one event type the registry pushes today; others are accepted and ignored.
pub const ACHIEVEMENTS_CHANGED: &str = "achievements.changed";
const PUSH_BODY_LIMIT: usize = 16 * 1024;
const PUSH_RATE_LIMIT: u64 = 300;
const PUSH_RATE_WINDOW: StdDuration = StdDuration::from_secs(60);

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(link_start))
        .routes(routes!(link_callback))
        .routes(routes!(link_dev))
        .routes(routes!(achievements))
        .routes(routes!(sync))
        .routes(routes!(unlink))
        .merge(
            OpenApiRouter::new()
                .routes(routes!(registry_event))
                .layer(DefaultBodyLimit::max(PUSH_BODY_LIMIT)),
        )
}

#[derive(Serialize, ToSchema)]
pub struct FspStatus {
    pub linked: bool,
    pub fsp_id: Option<String>,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    /// Snapshot from the registry (architecture.md §7.2). An empty list is a valid state.
    pub achievements: Vec<AchievementView>,
    /// Score for the candidate's specialization; "Нет данных ФСП" without history.
    pub score: Option<FspScore>,
}

#[derive(Serialize, ToSchema)]
pub struct AchievementView {
    #[serde(flatten)]
    pub achievement: Achievement,
    pub text: String,
}

impl FspStatus {
    fn unlinked() -> Self {
        Self {
            linked: false,
            fsp_id: None,
            linked_at: None,
            last_sync_at: None,
            achievements: Vec::new(),
            score: None,
        }
    }
}

fn settings_url(state: &AppState, outcome: &str) -> String {
    format!(
        "{}/candidate/settings?fsp={outcome}",
        state.config.public_url
    )
}

/// Start linking: redirects to the FSP ID login page
#[utoipa::path(get, path = "/link/start", tag = "fsp",
    responses((status = 303, description = "Redirect to FSP ID"), (status = 502, description = "`fsp_unavailable`: the FSP ID provider did not answer", body = Problem), (status = 503, description = "`fsp_oidc_disabled`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn link_start(State(state): State<AppState>, user: CandidateUser) -> ApiResult<Redirect> {
    let request = fsp_client::authorization_request(&state.http, &state.config.fsp)
        .await
        .map_err(fsp_error)?;
    db::fsp::purge_oidc_states(&state.pool).await?;
    db::fsp::insert_oidc_state(
        &state.pool,
        &request.state,
        user.id,
        &request.pkce_verifier,
        &request.nonce,
    )
    .await?;
    Ok(Redirect::to(&request.url))
}

#[derive(Deserialize, IntoParams)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// OIDC redirect target: verifies the id_token, links `fsp_id`, syncs achievements, returns to settings
#[utoipa::path(get, path = "/link/callback", tag = "fsp",
    params(CallbackQuery),
    responses((status = 303, description = "Redirect to /candidate/settings?fsp=linked|error")))]
async fn link_callback(State(state): State<AppState>, Query(q): Query<CallbackQuery>) -> Redirect {
    match finish_link(&state, q).await {
        Ok(()) => Redirect::to(&settings_url(&state, "linked")),
        Err(reason) => {
            tracing::warn!(%reason, "fsp link failed");
            Redirect::to(&format!(
                "{}&reason={reason}",
                settings_url(&state, "error")
            ))
        }
    }
}

async fn finish_link(state: &AppState, q: CallbackQuery) -> Result<(), &'static str> {
    if q.error.is_some() {
        return Err("cancelled");
    }
    let (Some(code), Some(oidc_state)) = (q.code, q.state) else {
        return Err("bad_request");
    };
    let pending = db::fsp::take_oidc_state(&state.pool, &oidc_state)
        .await
        .map_err(|_| "internal")?
        .ok_or("expired")?;
    if pending.created_at < Utc::now() - Duration::minutes(10) {
        return Err("expired");
    }
    let fsp_id = fsp_client::complete(
        &state.http,
        &state.config.fsp,
        &code,
        &pending.pkce_verifier,
        &pending.nonce,
    )
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "fsp id_token exchange failed");
        match e {
            FspError::NoFspId => "no_fsp_id",
            _ => "verification_failed",
        }
    })?;
    link(state, pending.user_id, &fsp_id, "oidc")
        .await
        .map_err(|e| match e {
            ApiError::Status {
                code: "fsp_already_linked",
                ..
            } => "already_linked",
            _ => "internal",
        })
}

async fn link(state: &AppState, candidate: uuid::Uuid, fsp_id: &str, via: &str) -> ApiResult<()> {
    if let Some(owner) = db::fsp::fsp_link_owner(&state.pool, fsp_id).await?
        && owner != candidate
    {
        return Err(ApiError::conflict(
            "fsp_already_linked",
            "Этот ФСП ID уже привязан к другому профилю",
        ));
    }
    let mut tx = state.pool.begin().await?;
    db::fsp::upsert_fsp_link(&mut *tx, candidate, fsp_id, via).await?;
    skills::replace_fsp_skills(&mut tx, candidate, None).await?;
    tx.commit().await?;
    if let Err(e) = sync_achievements(state, candidate, fsp_id).await {
        tracing::warn!(error = %e, "fsp registry sync failed, link kept");
    }
    state.candidates.invalidate().await;
    Ok(())
}

async fn sync_achievements(state: &AppState, candidate: uuid::Uuid, fsp_id: &str) -> ApiResult<()> {
    let snapshot = fetch_snapshot(state, fsp_id).await?;
    let mut tx = state.pool.begin().await?;
    apply_snapshot(&mut tx, candidate, &snapshot).await?;
    db::candidates::touch_candidate_activity(&mut *tx, candidate).await?;
    tx.commit().await?;
    Ok(())
}

/// The participant's achievements as the registry has them now; an unknown participant has none.
async fn fetch_snapshot(state: &AppState, fsp_id: &str) -> ApiResult<Value> {
    match fsp_client::fetch_achievements(&state.http, &state.config.fsp, fsp_id).await {
        Ok(a) => Ok(Value::Array(a)),
        Err(FspError::NotFound) => Ok(Value::Array(Vec::new())),
        Err(e) => Err(fsp_error(e)),
    }
}

/// What a sync changed, achievements matched by their registry `id`.
#[derive(Debug, Default, PartialEq)]
struct SnapshotDiff {
    added: Vec<Value>,
    removed: usize,
    changed: bool,
}

fn achievement_key(a: &Value) -> Value {
    a.get("id").cloned().unwrap_or_else(|| a.clone())
}

fn diff(old: &Value, new: &Value) -> SnapshotDiff {
    let empty = Vec::new();
    let old_list = old.as_array().unwrap_or(&empty);
    let new_list = new.as_array().unwrap_or(&empty);
    let old_keys: Vec<Value> = old_list.iter().map(achievement_key).collect();
    let new_keys: Vec<Value> = new_list.iter().map(achievement_key).collect();
    SnapshotDiff {
        added: new_list
            .iter()
            .zip(&new_keys)
            .filter(|(_, k)| !old_keys.contains(k))
            .map(|(a, _)| a.clone())
            .collect(),
        removed: old_keys.iter().filter(|k| !new_keys.contains(k)).count(),
        changed: old_list != new_list,
    }
}

/// Writes the snapshot and the skills it confirms (architecture.md §7.4) under the lock of the
/// stored one.
async fn apply_snapshot(
    tx: &mut PgConnection,
    candidate: uuid::Uuid,
    snapshot: &Value,
) -> ApiResult<SnapshotDiff> {
    let Some(old) = db::fsp::lock_fsp_snapshot(&mut *tx, candidate).await? else {
        return Ok(SnapshotDiff::default());
    };
    db::fsp::store_fsp_achievements(&mut *tx, snapshot, candidate).await?;
    skills::replace_fsp_skills(tx, candidate, Some(snapshot)).await?;
    Ok(diff(&old, snapshot))
}

fn fsp_note(d: &SnapshotDiff) -> FspNote {
    let latest = d
        .added
        .iter()
        .filter_map(|v| serde_json::from_value::<Achievement>(v.clone()).ok())
        .max_by(|a, b| a.date.cmp(&b.date))
        .map(|a| a.describe_ru());
    FspNote {
        added: u16::try_from(d.added.len()).unwrap_or(u16::MAX),
        removed: u16::try_from(d.removed).unwrap_or(u16::MAX),
        latest,
    }
}

fn fsp_error(e: FspError) -> ApiError {
    match e {
        FspError::OidcDisabled => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "fsp_oidc_disabled",
            "Вход через ФСП ID не настроен",
        ),
        FspError::RegistryDisabled => ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "fsp_registry_disabled",
            "Реестр ФСП не настроен",
        ),
        other => ApiError::new(
            StatusCode::BAD_GATEWAY,
            "fsp_unavailable",
            "Сервис ФСП недоступен",
        )
        .with_detail(other.to_string()),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct DevLinkInput {
    pub fsp_id: String,
}

/// Development only (`FSP_DEV_LINK=true`): link an FSP ID without Keycloak
#[utoipa::path(post, path = "/link/dev", tag = "fsp",
    request_body = DevLinkInput,
    responses((status = 200, body = FspStatus), (status = 404, description = "`not_found`: `FSP_DEV_LINK` is off", body = Problem), (status = 409, description = "`fsp_already_linked`", body = Problem), (status = 422, description = "`validation_failed`: `fsp_id`", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn link_dev(
    State(state): State<AppState>,
    user: CandidateUser,
    Json(input): Json<DevLinkInput>,
) -> ApiResult<Json<FspStatus>> {
    if !state.config.fsp.dev_link {
        return Err(ApiError::not_found());
    }
    let fsp_id = input.fsp_id.trim().to_uppercase();
    if fsp_id.is_empty() || fsp_id.len() > 40 {
        return Err(ApiError::field("fsp_id", "Укажите ФСП ID"));
    }
    link(&state, user.id, &fsp_id, "dev").await?;
    load_status(&state.pool, user.id).await.map(Json)
}

pub(crate) async fn load_status(pool: &PgPool, candidate: uuid::Uuid) -> ApiResult<FspStatus> {
    let Some(l) = db::fsp::fsp_link(pool, candidate).await? else {
        return Ok(FspStatus::unlinked());
    };
    let list = crate::pool::achievements(Some(&l.achievements));
    let spec = db::candidates::candidate_invitable(pool, candidate)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?
        .specialization
        .and_then(|s| s.parse().ok());
    let score = spec.map(|s| matching::fsp_score(&list, s, Utc::now().timestamp()));
    Ok(FspStatus {
        linked: true,
        fsp_id: Some(l.fsp_id),
        linked_at: Some(l.linked_at),
        last_sync_at: l.last_sync_at,
        achievements: list
            .into_iter()
            .map(|a| AchievementView {
                text: a.describe_ru(),
                achievement: a,
            })
            .collect(),
        score,
    })
}

/// Linked FSP ID and the achievements snapshot
#[utoipa::path(get, path = "/achievements", tag = "fsp",
    responses((status = 200, body = FspStatus)),
    security(("cookie" = []), ("bearer" = [])))]
async fn achievements(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<FspStatus>> {
    load_status(&state.pool, user.id).await.map(Json)
}

/// Re-sync achievements from the registry
#[utoipa::path(post, path = "/sync", tag = "fsp",
    responses((status = 200, body = FspStatus), (status = 404, body = Problem), (status = 502, description = "`fsp_unavailable`", body = Problem), (status = 503, description = "`fsp_registry_disabled`: the FSP registry is not configured", body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn sync(State(state): State<AppState>, user: CandidateUser) -> ApiResult<Json<FspStatus>> {
    let link = db::fsp::fsp_link(&state.pool, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    sync_achievements(&state, user.id, &link.fsp_id).await?;
    state.candidates.invalidate().await;
    load_status(&state.pool, user.id).await.map(Json)
}

#[derive(Deserialize, ToSchema)]
pub struct RegistryEvent {
    /// Unique per event; a redelivery carries the same id. Up to 128 characters.
    #[schema(example = "evt_01JAB3K9Q7T2")]
    pub event_id: String,
    /// `achievements.changed`; other types are accepted and ignored.
    #[serde(rename = "type")]
    #[schema(example = "achievements.changed")]
    pub kind: String,
    #[schema(example = "FSP-000011")]
    pub fsp_id: String,
    pub occurred_at: DateTime<Utc>,
}

fn push_example() -> Value {
    json!({
        "event_id": "evt_01JAB3K9Q7T2",
        "type": ACHIEVEMENTS_CHANGED,
        "fsp_id": "FSP-000011",
        "occurred_at": "2026-10-06T09:30:00Z"
    })
}

fn signature_problem(e: Option<SignatureError>) -> ApiError {
    match e {
        Some(SignatureError::Stale) => ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_expired",
            "Метка времени подписи старше 5 минут",
        ),
        _ => ApiError::new(
            StatusCode::UNAUTHORIZED,
            "signature_invalid",
            "Подпись реестра ФСП не прошла проверку",
        ),
    }
}

fn check_event(e: &RegistryEvent) -> ApiResult<()> {
    let printable = |s: &str| s.bytes().all(|b| b.is_ascii_graphic());
    if e.event_id.is_empty() || e.event_id.len() > 128 || !printable(&e.event_id) {
        return Err(ApiError::field(
            "event_id",
            "От 1 до 128 видимых ASCII-символов",
        ));
    }
    if e.fsp_id.trim().is_empty() || e.fsp_id.len() > 40 {
        return Err(ApiError::field("fsp_id", "Укажите ФСП ID"));
    }
    Ok(())
}

/// Everything the push checks before touching the database: the secret is configured, the
/// signature matches and is fresh (a captured request cannot be replayed later), the body parses.
fn verify_push(
    secret: Option<&str>,
    header: Option<&str>,
    body: &[u8],
    now: i64,
) -> ApiResult<RegistryEvent> {
    let Some(secret) = secret else {
        return Err(ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "fsp_webhook_disabled",
            "Приём событий реестра ФСП не настроен",
        ));
    };
    let header = header.ok_or_else(|| signature_problem(None))?;
    signature::verify(
        secret.as_bytes(),
        header,
        body,
        now,
        signature::TOLERANCE_SECS,
    )
    .map_err(|e| signature_problem(Some(e)))?;
    let Json(event) = Json::<RegistryEvent>::from_bytes(body)?;
    check_event(&event)?;
    Ok(event)
}

/// The ФСП registry reports that a participant's achievements changed
#[utoipa::path(post, path = "/registry/events", tag = "fsp",
    description = "Вебхук реестра ФСП: вместо ожидания ручной синхронизации реестр сам сообщает, что достижения участника изменились. \
        Подпись — как у вебхуков ATS: `FSP-Signature: t=<unix>,v1=hex(HMAC-SHA256(FSP_REGISTRY_WEBHOOK_SECRET, t + \".\" + тело))`; \
        метка `t` дальше 5 минут от часов сервера — 401 `signature_expired`, повтор перехваченного запроса не пройдёт. \
        Для привязанного участника платформа забирает достижения из реестра (как `POST /fsp/sync`), обновляет навыки, подтверждённые ФСП, \
        и, если что-то изменилось, присылает соискателю уведомление «Новые достижения ФСП в профиле». \
        Ответ одинаков — 202 без тела — для применённого, не изменившего ничего, повторного (`event_id` уже был) события и для `fsp_id`, \
        который на платформе не привязан: о таком событии ничего не сохраняется. Лимит — 300 запросов в минуту с одного адреса.",
    params(("FSP-Signature" = String, Header, description = "`t=<unix seconds>,v1=<hex HMAC-SHA256>`")),
    request_body(content = RegistryEvent, example = push_example),
    responses(
        (status = 202, description = "Принято (в том числе повтор и неизвестный `fsp_id`)"),
        (status = 400, description = "`invalid_json`", body = Problem),
        (status = 401, description = "`signature_invalid` — подписи нет, она повреждена или не совпала; `signature_expired` — метка старше 5 минут", body = Problem),
        (status = 413, description = "`payload_too_large` — тело больше 16 КиБ", body = Problem),
        (status = 422, description = "`invalid_json` — поля не по схеме; `validation_failed`: `event_id`, `fsp_id`", body = Problem),
        (status = 429, description = "`rate_limited`", body = Problem),
        (status = 502, description = "`fsp_unavailable` — реестр не отдал достижения; доставьте событие позже", body = Problem),
        (status = 503, description = "`fsp_webhook_disabled` — секрет вебхука не задан; `fsp_registry_disabled` — реестр не настроен", body = Problem),
    ))]
async fn registry_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<RawBody, ApiError>,
) -> ApiResult<StatusCode> {
    let ip = client_ip(&headers).unwrap_or_default();
    let hit = state
        .kv
        .hit(&format!("rl:fsp_push:{ip}"), PUSH_RATE_WINDOW)
        .await?;
    if hit.over(PUSH_RATE_LIMIT) {
        return Err(ApiError::rate_limited("fsp_push", hit.retry_after));
    }
    let RawBody(body) = body?;
    let header = headers.get(SIGNATURE_HEADER).and_then(|v| v.to_str().ok());
    let event = verify_push(
        state.config.fsp.webhook_secret.as_deref(),
        header,
        &body,
        Utc::now().timestamp(),
    )?;
    if event.kind != ACHIEVEMENTS_CHANGED {
        return Ok(StatusCode::ACCEPTED);
    }
    let Some(candidate) = db::fsp::fsp_link_owner(&state.pool, event.fsp_id.trim()).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    if db::fsp::registry_event_seen(&state.pool, &event.event_id).await? {
        return Ok(StatusCode::ACCEPTED);
    }
    let snapshot = fetch_snapshot(&state, event.fsp_id.trim()).await?;
    let mut tx = state.pool.begin().await?;
    if !db::fsp::insert_registry_event(&mut *tx, &event.event_id, candidate, event.occurred_at)
        .await?
    {
        return Ok(StatusCode::ACCEPTED);
    }
    let d = apply_snapshot(&mut tx, candidate, &snapshot).await?;
    if d.changed {
        db::fsp::mark_registry_event_changed(&mut *tx, &event.event_id).await?;
        Notifier::new(&state)
            .to_candidate(&mut *tx, candidate, &Event::FspAchievements(fsp_note(&d)))
            .await?;
    }
    db::fsp::purge_registry_events(&mut *tx).await?;
    tx.commit().await?;
    if d.changed {
        state.candidates.invalidate().await;
    }
    tracing::info!(
        event_id = %event.event_id,
        changed = d.changed,
        added = d.added.len(),
        removed = d.removed,
        "fsp registry event applied"
    );
    Ok(StatusCode::ACCEPTED)
}

/// Unlink FSP ID and drop the achievements snapshot and the skills it confirmed
#[utoipa::path(delete, path = "/link", tag = "fsp",
    responses((status = 204)),
    security(("cookie" = []), ("bearer" = [])))]
async fn unlink(State(state): State<AppState>, user: CandidateUser) -> ApiResult<StatusCode> {
    let mut tx = state.pool.begin().await?;
    db::fsp::delete_fsp_link(&mut *tx, user.id).await?;
    skills::replace_fsp_skills(&mut tx, user.id, None).await?;
    tx.commit().await?;
    state.candidates.invalidate().await;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rank(id: &str, date: &str) -> Value {
        json!({"id": id, "kind": "rank", "discipline": "algorithmic", "rank": "first", "date": date})
    }

    #[test]
    fn diff_matches_achievements_by_registry_id() {
        let old = json!([rank("A", "2024-01-01"), rank("B", "2025-01-01")]);
        let new = json!([
            rank("B", "2025-01-01"),
            rank("C", "2026-05-01"),
            rank("D", "2026-09-01")
        ]);
        let d = diff(&old, &new);
        assert_eq!(d.added.len(), 2);
        assert_eq!(d.removed, 1);
        assert!(d.changed);
        let note = fsp_note(&d);
        assert_eq!((note.added, note.removed), (2, 1));
        assert_eq!(
            note.latest.as_deref(),
            Some("I спортивный разряд по алгоритмическому программированию, 2026")
        );
        assert_eq!(diff(&new, &new), SnapshotDiff::default());
        let edited = json!([
            rank("B", "2025-02-01"),
            rank("C", "2026-05-01"),
            rank("D", "2026-09-01")
        ]);
        let d = diff(&new, &edited);
        assert!(
            d.changed && d.added.is_empty() && d.removed == 0,
            "an edit in place"
        );
    }

    const NOW: i64 = 1_700_000_000;
    const SECRET: &str = "whsec_registry";

    fn push(body: &str, t: i64, secret: &str) -> ApiResult<RegistryEvent> {
        let header = signature::header_value(t, &[secret.as_bytes()], body.as_bytes());
        verify_push(Some(SECRET), Some(&header), body.as_bytes(), NOW)
    }

    fn code(r: ApiResult<RegistryEvent>) -> (u16, &'static str, Vec<&'static str>) {
        r.err()
            .map(|e| e.summary())
            .unwrap_or((202, "", Vec::new()))
    }

    #[test]
    fn a_push_is_accepted_only_signed_fresh_and_well_formed() {
        let body = r#"{"event_id":"evt_1","type":"achievements.changed","fsp_id":"FSP-000011","occurred_at":"2026-10-06T09:30:00Z"}"#;
        let event = push(body, NOW, SECRET).expect("a signed, fresh event");
        assert_eq!(
            (event.event_id.as_str(), event.fsp_id.as_str()),
            ("evt_1", "FSP-000011")
        );
        assert_eq!(
            code(push(body, NOW - 300, SECRET)).0,
            202,
            "5 minutes is still fresh"
        );

        assert_eq!(code(push(body, NOW, "whsec_other")).1, "signature_invalid");
        assert_eq!(
            code(push(body, NOW - 301, SECRET)).1,
            "signature_expired",
            "a replay"
        );
        assert_eq!(
            code(push(body, NOW + 301, SECRET)).1,
            "signature_expired",
            "from the future"
        );
        let header = signature::header_value(NOW, &[SECRET.as_bytes()], body.as_bytes());
        let tampered = body.replace("evt_1", "evt_2");
        assert_eq!(
            code(verify_push(
                Some(SECRET),
                Some(&header),
                tampered.as_bytes(),
                NOW
            ))
            .1,
            "signature_invalid",
            "the signature covers the body"
        );
        assert_eq!(
            code(verify_push(Some(SECRET), None, body.as_bytes(), NOW)),
            (401, "signature_invalid", Vec::new())
        );
        assert_eq!(
            code(verify_push(None, Some(&header), body.as_bytes(), NOW)),
            (503, "fsp_webhook_disabled", Vec::new()),
            "without a secret the push is off"
        );
    }

    #[test]
    fn a_signed_body_must_still_be_a_valid_event() {
        let invalid = |body: &str| code(push(body, NOW, SECRET));
        assert_eq!(
            invalid(r#"{"event_id":1}"#),
            (422, "invalid_json", Vec::new())
        );
        assert_eq!(invalid("{"), (400, "invalid_json", Vec::new()));
        let event = |id: &str, fsp: &str| {
            json!({"event_id": id, "type": "achievements.changed", "fsp_id": fsp, "occurred_at": "2026-10-06T09:30:00Z"}).to_string()
        };
        for bad_id in [String::new(), "x".repeat(129), "evt 1".into()] {
            assert_eq!(
                invalid(&event(&bad_id, "FSP-000011")).2,
                ["event_id"],
                "{bad_id:?}"
            );
        }
        assert_eq!(invalid(&event("evt_1", " ")).2, ["fsp_id"]);
        assert_eq!(invalid(&event("evt_1", &"F".repeat(41))).2, ["fsp_id"]);
        assert_eq!(invalid(&event(&"x".repeat(128), "FSP-000011")).0, 202);
    }
}
