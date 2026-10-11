use chrono::{DateTime, Utc};
use sqlx::PgExecutor;
use uuid::Uuid;

use crate::types::{AssessmentStatus, IntegrityEvent};

/// Inputs of the proctoring summary: the camera choice, delivered chunks, event counts by kind.
#[derive(Debug, Clone)]
pub struct Summary {
    pub session_id: Uuid,
    pub candidate_id: Uuid,
    pub camera: bool,
    pub mime: Option<String>,
    pub started_at: DateTime<Utc>,
    pub purged_at: Option<DateTime<Utc>>,
    pub chunks: i64,
    pub bytes: i64,
    pub tab_hidden: i64,
    pub window_blur: i64,
    pub fullscreen_exit: i64,
    pub paste: i64,
    pub camera_lost: i64,
    pub recording_failed: i64,
}

/// Proctoring of the test that assigned a candidate's current category: the employer badge.
#[derive(Debug, Clone)]
pub struct CardBadge {
    pub candidate_id: Uuid,
    pub camera: bool,
    pub chunks: i64,
    pub tab_hidden: i64,
    pub window_blur: i64,
    pub fullscreen_exit: i64,
    pub paste: i64,
    pub camera_lost: i64,
    pub recording_failed: i64,
}

/// Locked state an upload is admitted against.
#[derive(Debug, Clone)]
pub struct UploadTarget {
    pub camera: bool,
    pub purged: bool,
    pub consent_active: bool,
    pub status: AssessmentStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub chunks: i64,
    pub bytes: i64,
    pub events: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub seq: i32,
    pub size: i32,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: IntegrityEvent,
    pub occurred_at: DateTime<Utc>,
    pub item_seq: Option<i16>,
    pub duration_ms: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Recording {
    pub session_id: Uuid,
    pub candidate_id: Uuid,
}

pub async fn insert_proctoring(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    candidate_id: Uuid,
    camera: bool,
    consent_id: Option<Uuid>,
    started_at: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO proctoring_sessions (session_id, candidate_id, camera, consent_id, started_at)
         VALUES ($1, $2, $3, $4, $5)",
        session_id,
        candidate_id,
        camera,
        consent_id,
        started_at,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// `candidate_id = None` is the admin view of any session.
pub async fn summary(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    candidate_id: Option<Uuid>,
) -> Result<Option<Summary>, sqlx::Error> {
    sqlx::query_as!(
        Summary,
        r#"SELECT p.session_id, p.candidate_id, p.camera, p.mime, p.started_at, p.purged_at,
                  (SELECT count(*) FROM proctoring_chunks c WHERE c.session_id = p.session_id) AS "chunks!",
                  (SELECT coalesce(sum(c.size), 0)::bigint FROM proctoring_chunks c
                    WHERE c.session_id = p.session_id) AS "bytes!",
                  count(e.kind) FILTER (WHERE e.kind = 'tab_hidden') AS "tab_hidden!",
                  count(e.kind) FILTER (WHERE e.kind = 'window_blur') AS "window_blur!",
                  count(e.kind) FILTER (WHERE e.kind = 'fullscreen_exit') AS "fullscreen_exit!",
                  count(e.kind) FILTER (WHERE e.kind = 'paste') AS "paste!",
                  count(e.kind) FILTER (WHERE e.kind = 'camera_lost') AS "camera_lost!",
                  count(e.kind) FILTER (WHERE e.kind = 'recording_failed') AS "recording_failed!"
           FROM proctoring_sessions p
           LEFT JOIN proctoring_events e ON e.session_id = p.session_id
           WHERE p.session_id = $1 AND ($2::uuid IS NULL OR p.candidate_id = $2)
           GROUP BY p.session_id"#,
        session_id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

/// Row-locks the proctoring session so concurrent uploads see consistent counts.
pub async fn upload_target(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    candidate_id: Uuid,
) -> Result<Option<UploadTarget>, sqlx::Error> {
    sqlx::query_as!(
        UploadTarget,
        r#"SELECT p.camera,
                  p.purged_at IS NOT NULL AS "purged!",
                  EXISTS (SELECT 1 FROM user_consents uc
                           WHERE uc.id = p.consent_id AND uc.revoked_at IS NULL) AS "consent_active!",
                  s.status AS "status: AssessmentStatus", s.started_at, s.finished_at,
                  (SELECT count(*) FROM proctoring_chunks c WHERE c.session_id = p.session_id) AS "chunks!",
                  (SELECT coalesce(sum(c.size), 0)::bigint FROM proctoring_chunks c
                    WHERE c.session_id = p.session_id) AS "bytes!",
                  (SELECT count(*) FROM proctoring_events e WHERE e.session_id = p.session_id) AS "events!"
           FROM proctoring_sessions p
           JOIN assessment_sessions s ON s.id = p.session_id
           WHERE p.session_id = $1 AND p.candidate_id = $2
           FOR UPDATE OF p"#,
        session_id,
        candidate_id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert_chunk(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    seq: i32,
    size: i32,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO proctoring_chunks (session_id, seq, size) VALUES ($1, $2, $3)",
        session_id,
        seq,
        size,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn set_mime(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    mime: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE proctoring_sessions SET mime = $2 WHERE session_id = $1 AND mime IS NULL",
        session_id,
        mime,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn chunks(db: impl PgExecutor<'_>, session_id: Uuid) -> Result<Vec<Chunk>, sqlx::Error> {
    sqlx::query_as!(
        Chunk,
        "SELECT seq, size FROM proctoring_chunks WHERE session_id = $1 ORDER BY seq",
        session_id,
    )
    .fetch_all(db)
    .await
}

pub async fn insert_event(
    db: impl PgExecutor<'_>,
    session_id: Uuid,
    e: &Event,
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO proctoring_events (session_id, kind, occurred_at, item_seq, duration_ms)
         VALUES ($1, $2, $3, $4, $5)",
        session_id,
        e.kind as IntegrityEvent,
        e.occurred_at,
        e.item_seq,
        e.duration_ms,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

pub async fn events(db: impl PgExecutor<'_>, session_id: Uuid) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as!(
        Event,
        r#"SELECT kind AS "kind: IntegrityEvent", occurred_at, item_seq, duration_ms
           FROM proctoring_events
           WHERE session_id = $1
           ORDER BY occurred_at, received_at"#,
        session_id,
    )
    .fetch_all(db)
    .await
}

/// Recordings past retention whose files are still on disk, oldest first.
pub async fn expired_recordings(
    db: impl PgExecutor<'_>,
    started_before: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<Recording>, sqlx::Error> {
    sqlx::query_as!(
        Recording,
        "SELECT session_id, candidate_id FROM proctoring_sessions
         WHERE camera AND purged_at IS NULL AND started_at < $1
         ORDER BY started_at
         LIMIT $2",
        started_before,
        limit,
    )
    .fetch_all(db)
    .await
}

pub async fn candidate_recordings(
    db: impl PgExecutor<'_>,
    candidate_id: Uuid,
) -> Result<Vec<Recording>, sqlx::Error> {
    sqlx::query_as!(
        Recording,
        "SELECT session_id, candidate_id FROM proctoring_sessions
         WHERE candidate_id = $1 AND camera AND purged_at IS NULL",
        candidate_id,
    )
    .fetch_all(db)
    .await
}

/// Call after the files are gone. Chunk rows (position and size only) stay, so the summary
/// still tells a completed recording from an interrupted one.
pub async fn mark_purged(
    db: impl PgExecutor<'_>,
    session_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    sqlx::query!(
        "UPDATE proctoring_sessions SET purged_at = now()
         WHERE session_id = ANY ($1) AND purged_at IS NULL",
        session_ids,
    )
    .execute(db)
    .await
    .map(|r| r.rows_affected())
}

/// Of the given ids, those that still own proctoring sessions; the rest are orphaned directories.
pub async fn known_candidates(
    db: impl PgExecutor<'_>,
    candidate_ids: &[Uuid],
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT DISTINCT candidate_id FROM proctoring_sessions WHERE candidate_id = ANY ($1)",
        candidate_ids,
    )
    .fetch_all(db)
    .await
}
