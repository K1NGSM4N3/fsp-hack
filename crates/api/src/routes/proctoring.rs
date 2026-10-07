//! Proctoring endpoints (docs/architecture.md §5.6): the options for the proctoring step, video chunk
//! upload, integrity events and playback of the candidate's own recording. The moderator's view
//! of any session lives in `routes::moderation`; nothing here is routed to employers.

use assessment::IntegrityEvent;
use assessment::proctoring::{self as rules, ChunkAdmission, ChunkError};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::catalog::ConsentDocument;
use crate::{
    auth::CandidateUser,
    error::{ApiError, ApiResult, Problem},
    extract::{Json, Path, RawBody},
    proctoring::{self, CONSENT_KIND, LIMITS, MAX_BATCH, MAX_EVENTS, ProctoringMode, TAIL_SECS},
    state::AppState,
};

pub fn candidate_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(options))
        .routes(routes!(post_events))
        .routes(routes!(own_recording))
        .merge(
            OpenApiRouter::new()
                .routes(routes!(put_chunk))
                .layer(DefaultBodyLimit::max(
                    LIMITS.max_chunk_bytes as usize + 1024,
                )),
        )
}

fn chunk_too_large() -> ApiError {
    ApiError::new(
        StatusCode::PAYLOAD_TOO_LARGE,
        "chunk_too_large",
        "Фрагмент записи больше 1 МиБ",
    )
}

#[derive(Serialize, ToSchema)]
pub struct ProctoringOptions {
    pub mode: ProctoringMode,
    /// Consent text for the checkbox of the proctoring step; `None` when `mode = off`.
    pub consent: Option<ConsentDocument>,
    /// The candidate already holds an unrevoked grant of this version.
    pub consent_granted: bool,
    pub retention_days: i64,
    pub max_chunk_bytes: u32,
    pub max_events_per_batch: u16,
}

/// What the proctoring step shows before a test: mode, consent text, retention
#[utoipa::path(get, path = "/proctoring", tag = "proctoring",
    responses((status = 200, body = ProctoringOptions), (status = 401, body = Problem), (status = 403, body = Problem)),
    security(("cookie" = []), ("bearer" = [])))]
async fn options(
    State(state): State<AppState>,
    user: CandidateUser,
) -> ApiResult<Json<ProctoringOptions>> {
    let cfg = &state.config.proctoring;
    let doc = match cfg.mode {
        ProctoringMode::Optional => {
            db::consents::current_document(&state.pool, CONSENT_KIND).await?
        }
        ProctoringMode::Off => None,
    };
    let consent_granted = match &doc {
        Some(d) => db::consents::active_grant(&state.pool, user.id, d.id)
            .await?
            .is_some(),
        None => false,
    };
    Ok(Json(ProctoringOptions {
        mode: cfg.mode,
        consent: doc.map(ConsentDocument::from),
        consent_granted,
        retention_days: cfg.retention_days,
        max_chunk_bytes: LIMITS.max_chunk_bytes,
        max_events_per_batch: MAX_BATCH as u16,
    }))
}

fn closed() -> ApiError {
    ApiError::conflict("assessment_finished", "Тест уже завершён")
}

fn not_recording() -> ApiError {
    ApiError::conflict(
        "proctoring_not_recording",
        "Видеозапись для этого теста не ведётся",
    )
}

/// Only container types the browser recorder produces; anything else is served as WebM.
fn video_mime(headers: &HeaderMap) -> Option<&str> {
    let v = headers.get(header::CONTENT_TYPE)?.to_str().ok()?.trim();
    let clean = v.len() <= 64
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/;=,.+- ".contains(&b));
    (clean && (v.starts_with("video/webm") || v.starts_with("video/mp4"))).then_some(v)
}

/// Upload the next ~10-second chunk of the running test's video (raw MediaRecorder output)
#[utoipa::path(put, path = "/assessments/{id}/proctoring/chunks/{seq}", tag = "proctoring",
    params(
        ("id" = Uuid, Path, description = "assessment session"),
        ("seq" = u32, Path, description = "0-based, strictly in order; a repeated seq is acknowledged without rewriting"),
    ),
    request_body(content = Vec<u8>, content_type = "video/webm", description = "Chunk up to 1 MiB; the first chunk's Content-Type fixes the container"),
    responses(
        (status = 204, description = "Stored or already stored"),
        (status = 400, description = "`empty_chunk`", body = Problem),
        (status = 404, description = "Not the caller's session or proctoring was not set up", body = Problem),
        (status = 409, description = "`proctoring_off`, `proctoring_not_recording`, `assessment_finished`, `chunk_out_of_order`", body = Problem),
        (status = 413, description = "`chunk_too_large`, `recording_too_large`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn put_chunk(
    State(state): State<AppState>,
    user: CandidateUser,
    Path((id, seq)): Path<(Uuid, u32)>,
    headers: HeaderMap,
    body: Result<RawBody, ApiError>,
) -> ApiResult<StatusCode> {
    if state.config.proctoring.mode == ProctoringMode::Off {
        return Err(ApiError::conflict(
            "proctoring_off",
            "Видеозапись отключена",
        ));
    }
    let RawBody(body) = body.map_err(|e| match e {
        ApiError::Status {
            code: "payload_too_large",
            ..
        } => chunk_too_large(),
        other => other,
    })?;
    let size = u32::try_from(body.len()).unwrap_or(u32::MAX);
    let now = Utc::now();
    let mut tx = state.pool.begin().await?;
    let t = db::proctoring::upload_target(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !t.camera || t.purged || !t.consent_active {
        return Err(not_recording());
    }
    let in_progress = t.status == db::types::AssessmentStatus::InProgress;
    if !rules::accepts_uploads(
        in_progress,
        t.finished_at.map(|f| f.timestamp()),
        now.timestamp(),
        TAIL_SECS,
    ) {
        return Err(closed());
    }
    let stored = u32::try_from(t.chunks).unwrap_or(u32::MAX);
    match rules::admit_chunk(seq, size, stored, t.bytes.max(0) as u64, &LIMITS) {
        Ok(ChunkAdmission::Duplicate) => return Ok(StatusCode::NO_CONTENT),
        Ok(ChunkAdmission::Store) => {}
        Err(ChunkError::Empty) => {
            return Err(ApiError::bad_request(
                "empty_chunk",
                "Пустой фрагмент записи",
            ));
        }
        Err(ChunkError::TooLarge) => return Err(chunk_too_large()),
        Err(ChunkError::SessionFull) => {
            return Err(ApiError::new(
                StatusCode::PAYLOAD_TOO_LARGE,
                "recording_too_large",
                "Запись этого теста достигла предела",
            ));
        }
        Err(ChunkError::OutOfOrder { expected }) => {
            return Err(ApiError::conflict(
                "chunk_out_of_order",
                "Фрагменты записи пришли не по порядку",
            )
            .with_detail(format!("Ожидается фрагмент {expected}")));
        }
    }
    let sealed = proctoring::seal(&state.crypto, id, seq, &body)?;
    state
        .recordings
        .write_chunk(user.id, id, seq, &sealed)
        .await?;
    db::proctoring::insert_chunk(&mut *tx, id, seq as i32, size as i32).await?;
    if seq == 0
        && let Some(mime) = video_mime(&headers)
    {
        db::proctoring::set_mime(&mut *tx, id, mime).await?;
    }
    tx.commit().await?;
    metrics::counter!("proctoring_chunks_total").increment(1);
    metrics::counter!("proctoring_chunk_bytes_total").increment(u64::from(size));
    if !in_progress {
        // The test is graded; this tail upload changes the proctoring mark on the card.
        state.candidates.invalidate().await;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, ToSchema)]
pub struct EventInput {
    pub kind: IntegrityEvent,
    /// Browser time of the event; clamped to the session's lifetime.
    pub at: DateTime<Utc>,
    /// `current.seq` of the item on screen.
    pub item_seq: Option<u16>,
    /// How long the candidate was away (`tab_hidden`, `window_blur`).
    pub duration_ms: Option<u32>,
}

#[derive(Deserialize, ToSchema)]
pub struct EventBatch {
    /// Up to 50 events; the session keeps the first 500.
    pub events: Vec<EventInput>,
}

/// Log integrity events of the running test: tab hidden, window blur, fullscreen exit, paste, camera lost, recording failed
#[utoipa::path(post, path = "/assessments/{id}/proctoring/events", tag = "proctoring",
    params(("id" = Uuid, Path, description = "assessment session")),
    request_body = EventBatch,
    responses(
        (status = 204),
        (status = 404, body = Problem),
        (status = 409, description = "`proctoring_off`, `assessment_finished`", body = Problem),
        (status = 422, description = "`validation_failed`: more than 50 events", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn post_events(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
    Json(batch): Json<EventBatch>,
) -> ApiResult<StatusCode> {
    if state.config.proctoring.mode == ProctoringMode::Off {
        return Err(ApiError::conflict(
            "proctoring_off",
            "Видеозапись отключена",
        ));
    }
    if batch.events.len() > MAX_BATCH {
        return Err(ApiError::field("events", "Не больше 50 событий за раз"));
    }
    let now = Utc::now();
    let mut tx = state.pool.begin().await?;
    let t = db::proctoring::upload_target(&mut *tx, id, user.id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let in_progress = t.status == db::types::AssessmentStatus::InProgress;
    if !rules::accepts_uploads(
        in_progress,
        t.finished_at.map(|f| f.timestamp()),
        now.timestamp(),
        TAIL_SECS,
    ) {
        return Err(closed());
    }
    let room = usize::try_from((MAX_EVENTS - t.events).max(0)).unwrap_or(0);
    let latest = t.finished_at.unwrap_or(now).max(t.started_at);
    let stored = batch.events.len().min(room);
    for e in batch.events.iter().take(room) {
        db::proctoring::insert_event(
            &mut *tx,
            id,
            &db::proctoring::Event {
                kind: proctoring::to_db(e.kind),
                occurred_at: e.at.clamp(t.started_at, latest),
                item_seq: e.item_seq.and_then(|s| i16::try_from(s).ok()),
                duration_ms: e.duration_ms.map(|d| i32::try_from(d).unwrap_or(i32::MAX)),
            },
        )
        .await?;
    }
    tx.commit().await?;
    metrics::counter!("proctoring_events_total").increment(stored as u64);
    if !in_progress && stored > 0 {
        // The test is graded; these late events change the proctoring mark on the card.
        state.candidates.invalidate().await;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// The chunks of a recording that can be played, every one of them on disk; 404
/// `recording_unavailable` otherwise.
pub(crate) async fn playable_chunks(
    state: &AppState,
    s: &db::proctoring::Summary,
) -> ApiResult<Vec<db::proctoring::Chunk>> {
    let chunks = if s.purged_at.is_some() {
        Vec::new()
    } else {
        db::proctoring::chunks(&state.pool, s.session_id).await?
    };
    if chunks.is_empty()
        || !state
            .recordings
            .has_chunks(s.candidate_id, s.session_id, &chunks)
            .await?
    {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "recording_unavailable",
            "Видеозаписи нет: тест прошёл без камеры, запись не началась или уже удалена",
        ));
    }
    Ok(chunks)
}

pub(crate) fn stream_recording(
    state: &AppState,
    s: &db::proctoring::Summary,
    chunks: Vec<db::proctoring::Chunk>,
) -> Response {
    let length: i64 = chunks.iter().map(|c| i64::from(c.size)).sum();
    let body = proctoring::recording_body(
        state.recordings.clone(),
        state.crypto.clone(),
        s.candidate_id,
        s.session_id,
        chunks,
    );
    let mime = s
        .mime
        .as_deref()
        .and_then(|m| HeaderValue::from_str(m).ok())
        .unwrap_or(HeaderValue::from_static("video/webm"));
    (
        [
            (header::CONTENT_TYPE, mime),
            (header::CONTENT_LENGTH, HeaderValue::from(length)),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("private, no-store"),
            ),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("inline"),
            ),
        ],
        body,
    )
        .into_response()
}

/// Play the candidate's own recording: chunks decrypted and streamed in order
#[utoipa::path(get, path = "/assessments/{id}/proctoring/recording", tag = "proctoring",
    params(("id" = Uuid, Path, description = "assessment session")),
    responses(
        (status = 200, content_type = "video/webm", body = Vec<u8>),
        (status = 403, description = "`wrong_role`: employers never get the video", body = Problem),
        (status = 404, description = "`not_found`, `recording_unavailable`", body = Problem),
    ),
    security(("cookie" = []), ("bearer" = [])))]
async fn own_recording(
    State(state): State<AppState>,
    user: CandidateUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let s = db::proctoring::summary(&state.pool, id, Some(user.id))
        .await?
        .ok_or_else(ApiError::not_found)?;
    let chunks = playable_chunks(&state, &s).await?;
    Ok(stream_recording(&state, &s, chunks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_recorder_containers_are_trusted() {
        let mut h = HeaderMap::new();
        assert_eq!(video_mime(&h), None);
        h.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("video/webm;codecs=vp8"),
        );
        assert_eq!(video_mime(&h), Some("video/webm;codecs=vp8"));
        h.insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4"));
        assert_eq!(video_mime(&h), Some("video/mp4"));
        h.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
        assert_eq!(video_mime(&h), None);
        h.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("video/webm\"><script>"),
        );
        assert_eq!(video_mime(&h), None);
    }
}
