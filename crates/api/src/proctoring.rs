//! Proctoring storage, summaries and retention (docs/architecture.md §5.6, §9.2).
//!
//! Video chunks are files `PROCTORING_DIR/<candidate>/<session>/<seq>.bin`, each sealed with the
//! field cipher and bound to its session and position; Postgres keeps only metadata
//! (`db::proctoring`). Paths are built from database ids, never from client input.

use std::{io, path::PathBuf, sync::Arc, time::Duration};

use assessment::proctoring::{self as rules, ChunkLimits, EventCounts, Facts};
use assessment::{IntegrityEvent, ProctoringStatus};
use axum::body::{Body, Bytes};
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    crypto::{Crypto, CryptoError},
    error::InternalError,
};

/// 320×240 at ~100 kbit/s is ~125 KB per 10-second chunk; the caps leave room for bitrate
/// spikes and bound a session to two hours of video.
pub const LIMITS: ChunkLimits = ChunkLimits {
    max_chunk_bytes: 1 << 20,
    max_session_bytes: 64 << 20,
    max_chunks: 720,
};
pub const MAX_BATCH: usize = 50;
pub const MAX_EVENTS: i64 = 500;
pub const TAIL_SECS: i64 = 60;
pub const CONSENT_KIND: &str = "proctoring_video";

const SWEEP_EVERY: Duration = Duration::from_secs(6 * 3600);
const SWEEP_BATCH: i64 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProctoringMode {
    /// The candidate chooses: the camera on, or the test without video.
    Optional,
    /// No proctoring step, no recording, no events.
    Off,
}

pub fn to_db(e: IntegrityEvent) -> db::types::IntegrityEvent {
    use db::types::IntegrityEvent as D;
    match e {
        IntegrityEvent::TabHidden => D::TabHidden,
        IntegrityEvent::WindowBlur => D::WindowBlur,
        IntegrityEvent::FullscreenExit => D::FullscreenExit,
        IntegrityEvent::Paste => D::Paste,
        IntegrityEvent::CameraLost => D::CameraLost,
        IntegrityEvent::RecordingFailed => D::RecordingFailed,
    }
}

pub fn from_db(e: db::types::IntegrityEvent) -> IntegrityEvent {
    use db::types::IntegrityEvent as D;
    match e {
        D::TabHidden => IntegrityEvent::TabHidden,
        D::WindowBlur => IntegrityEvent::WindowBlur,
        D::FullscreenExit => IntegrityEvent::FullscreenExit,
        D::Paste => IntegrityEvent::Paste,
        D::CameraLost => IntegrityEvent::CameraLost,
        D::RecordingFailed => IntegrityEvent::RecordingFailed,
    }
}

fn narrow(n: i64) -> u16 {
    u16::try_from(n.max(0)).unwrap_or(u16::MAX)
}

fn counts(per_kind: [i64; 6]) -> EventCounts {
    IntegrityEvent::ALL
        .into_iter()
        .zip(per_kind)
        .map(|(e, n)| (e, narrow(n)))
        .collect()
}

pub fn facts(r: &db::proctoring::Summary, in_progress: bool) -> Facts {
    Facts {
        camera: r.camera,
        in_progress,
        chunks: u32::try_from(r.chunks.max(0)).unwrap_or(u32::MAX),
        counts: counts([
            r.tab_hidden,
            r.window_blur,
            r.fullscreen_exit,
            r.paste,
            r.camera_lost,
            r.recording_failed,
        ]),
    }
}

#[derive(Serialize, ToSchema)]
pub struct EventCountsView {
    pub tab_hidden: u16,
    pub window_blur: u16,
    pub fullscreen_exit: u16,
    pub paste: u16,
    pub camera_lost: u16,
    pub recording_failed: u16,
}

impl From<&EventCounts> for EventCountsView {
    fn from(c: &EventCounts) -> Self {
        Self {
            tab_hidden: c.get(IntegrityEvent::TabHidden),
            window_blur: c.get(IntegrityEvent::WindowBlur),
            fullscreen_exit: c.get(IntegrityEvent::FullscreenExit),
            paste: c.get(IntegrityEvent::Paste),
            camera_lost: c.get(IntegrityEvent::CameraLost),
            recording_failed: c.get(IntegrityEvent::RecordingFailed),
        }
    }
}

/// The candidate's own view of a session's proctoring.
#[derive(Serialize, ToSchema)]
pub struct ProctoringSummary {
    pub status: ProctoringStatus,
    pub events: EventCountsView,
    /// Video chunks received, about 10 seconds each.
    pub chunks: u32,
    pub bytes: u64,
    /// `GET /candidate/assessments/{id}/proctoring/recording` will play it.
    pub playable: bool,
    /// The video is deleted after this moment.
    pub delete_after: Option<DateTime<Utc>>,
    /// Already deleted: retention passed or the consent was revoked.
    pub purged: bool,
}

pub fn summary_view(
    r: &db::proctoring::Summary,
    f: &Facts,
    retention_days: i64,
) -> ProctoringSummary {
    let purged = r.purged_at.is_some();
    ProctoringSummary {
        status: rules::status(f),
        events: (&f.counts).into(),
        chunks: f.chunks,
        bytes: u64::try_from(r.bytes.max(0)).unwrap_or(0),
        playable: !purged && f.chunks > 0,
        delete_after: (r.camera && !purged)
            .then(|| r.started_at + chrono::Duration::days(retention_days)),
        purged,
    }
}

/// Employer-facing proctoring mark of the test behind the current category. No session id and
/// no link: the video stays out of reach until and after «Принять», like the contacts (V-2).
#[derive(Clone, Serialize, ToSchema)]
pub struct ProctoringBadge {
    pub status: ProctoringStatus,
    /// Times the candidate left the test tab or window.
    pub left_test: u16,
    pub pastes: u16,
    pub fullscreen_exits: u16,
}

pub fn badge(r: &db::proctoring::CardBadge) -> ProctoringBadge {
    let f = Facts {
        camera: r.camera,
        in_progress: false,
        chunks: u32::try_from(r.chunks.max(0)).unwrap_or(u32::MAX),
        counts: counts([
            r.tab_hidden,
            r.window_blur,
            r.fullscreen_exit,
            r.paste,
            r.camera_lost,
            r.recording_failed,
        ]),
    };
    ProctoringBadge {
        status: rules::status(&f),
        left_test: f.counts.left_test(),
        pastes: f.counts.get(IntegrityEvent::Paste),
        fullscreen_exits: f.counts.get(IntegrityEvent::FullscreenExit),
    }
}

/// Binds a chunk to its session and position: a file copied to another session or renamed to
/// another index fails authentication instead of playing someone else's video.
fn context(session: Uuid, seq: u32) -> [u8; 30] {
    let mut c = [0u8; 30];
    c[..10].copy_from_slice(b"proctoring");
    c[10..26].copy_from_slice(session.as_bytes());
    c[26..].copy_from_slice(&seq.to_be_bytes());
    c
}

pub fn seal(
    crypto: &Crypto,
    session: Uuid,
    seq: u32,
    chunk: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    crypto.fields.encrypt_bound(chunk, &context(session, seq))
}

pub fn open(
    crypto: &Crypto,
    session: Uuid,
    seq: u32,
    sealed: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    crypto.fields.decrypt_bound(sealed, &context(session, seq))
}

pub struct RecordingStore {
    root: PathBuf,
}

fn ignore_missing(r: io::Result<()>) -> io::Result<()> {
    match r {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

impl RecordingStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub async fn init(&self) -> io::Result<()> {
        tokio::fs::create_dir_all(&self.root).await
    }

    fn session_dir(&self, candidate: Uuid, session: Uuid) -> PathBuf {
        self.root
            .join(candidate.to_string())
            .join(session.to_string())
    }

    fn chunk_path(&self, candidate: Uuid, session: Uuid, seq: u32) -> PathBuf {
        self.session_dir(candidate, session)
            .join(format!("{seq:05}.bin"))
    }

    /// Write-then-rename, so a crash mid-write never leaves a truncated chunk under its final name.
    pub async fn write_chunk(
        &self,
        candidate: Uuid,
        session: Uuid,
        seq: u32,
        sealed: &[u8],
    ) -> io::Result<()> {
        tokio::fs::create_dir_all(self.session_dir(candidate, session)).await?;
        let path = self.chunk_path(candidate, session, seq);
        let tmp = path.with_extension("tmp");
        tokio::fs::write(&tmp, sealed).await?;
        tokio::fs::rename(&tmp, &path).await
    }

    pub async fn read_chunk(
        &self,
        candidate: Uuid,
        session: Uuid,
        seq: u32,
    ) -> io::Result<Vec<u8>> {
        tokio::fs::read(self.chunk_path(candidate, session, seq)).await
    }

    /// Every listed chunk is on disk. Checked before streaming: once the response headers are
    /// out, a missing file can only cut the video off mid-way.
    pub async fn has_chunks(
        &self,
        candidate: Uuid,
        session: Uuid,
        chunks: &[db::proctoring::Chunk],
    ) -> io::Result<bool> {
        for c in chunks {
            let seq = u32::try_from(c.seq).map_err(io::Error::other)?;
            if !tokio::fs::try_exists(self.chunk_path(candidate, session, seq)).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn remove_session(&self, candidate: Uuid, session: Uuid) -> io::Result<()> {
        ignore_missing(tokio::fs::remove_dir_all(self.session_dir(candidate, session)).await)
    }

    pub async fn remove_candidate(&self, candidate: Uuid) -> io::Result<()> {
        ignore_missing(tokio::fs::remove_dir_all(self.root.join(candidate.to_string())).await)
    }

    async fn candidate_dirs(&self) -> io::Result<Vec<Uuid>> {
        let mut out = Vec::new();
        let mut dir = match tokio::fs::read_dir(&self.root).await {
            Ok(d) => d,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e),
        };
        while let Some(entry) = dir.next_entry().await? {
            if let Some(id) = entry.file_name().to_str().and_then(|n| n.parse().ok()) {
                out.push(id);
            }
        }
        Ok(out)
    }
}

/// Plaintext chunks in order, decrypted one at a time: memory stays at one chunk per viewer.
pub fn recording_body(
    store: Arc<RecordingStore>,
    crypto: Arc<Crypto>,
    candidate: Uuid,
    session: Uuid,
    chunks: Vec<db::proctoring::Chunk>,
) -> Body {
    let stream = futures_util::stream::iter(chunks).then(move |c| {
        let store = store.clone();
        let crypto = crypto.clone();
        async move {
            let seq = u32::try_from(c.seq).map_err(io::Error::other)?;
            let sealed = store.read_chunk(candidate, session, seq).await?;
            let plain = open(&crypto, session, seq, &sealed).map_err(|e| {
                tracing::error!(%session, seq, error = %e, "proctoring chunk failed to decrypt");
                io::Error::other(e)
            })?;
            Ok::<_, io::Error>(Bytes::from(plain))
        }
    });
    Body::from_stream(stream)
}

/// Files first, metadata second: if marking fails the next sweep retries, while the reverse
/// order could leave video on disk that the database no longer lists.
pub async fn purge(
    pool: &PgPool,
    store: &RecordingStore,
    recordings: &[db::proctoring::Recording],
) -> Result<u64, InternalError> {
    for r in recordings {
        store.remove_session(r.candidate_id, r.session_id).await?;
    }
    let ids: Vec<Uuid> = recordings.iter().map(|r| r.session_id).collect();
    Ok(db::proctoring::mark_purged(pool, &ids).await?)
}

pub struct SweepReport {
    pub purged: u64,
    pub orphans: usize,
}

/// Recordings of sessions started before this are past retention: the mirror of
/// `delete_after` in [`summary_view`].
fn retention_cutoff(now: DateTime<Utc>, retention_days: i64) -> DateTime<Utc> {
    now - chrono::Duration::days(retention_days)
}

/// Retention plus a safety net for account deletion: a candidate directory without proctoring
/// rows belongs to a deleted account whose files were not removed right away.
pub async fn sweep(
    pool: &PgPool,
    store: &RecordingStore,
    retention_days: i64,
    now: DateTime<Utc>,
) -> Result<SweepReport, InternalError> {
    let cutoff = retention_cutoff(now, retention_days);
    let mut purged = 0;
    loop {
        let batch = db::proctoring::expired_recordings(pool, cutoff, SWEEP_BATCH).await?;
        purged += purge(pool, store, &batch).await?;
        if (batch.len() as i64) < SWEEP_BATCH {
            break;
        }
    }
    let dirs = store.candidate_dirs().await?;
    let mut known = db::proctoring::known_candidates(pool, &dirs).await?;
    known.sort_unstable();
    let mut orphans = 0;
    for dir in dirs.iter().filter(|d| known.binary_search(d).is_err()) {
        store.remove_candidate(*dir).await?;
        orphans += 1;
    }
    Ok(SweepReport { purged, orphans })
}

/// The first tick fires immediately: a sweep at startup, then every few hours.
pub fn spawn_sweeper(pool: PgPool, store: Arc<RecordingStore>, retention_days: i64) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(SWEEP_EVERY);
        loop {
            tick.tick().await;
            match sweep(&pool, &store, retention_days, Utc::now()).await {
                Ok(r) if r.purged > 0 || r.orphans > 0 => {
                    tracing::info!(
                        purged = r.purged,
                        orphans = r.orphans,
                        "proctoring retention sweep"
                    );
                }
                Ok(_) => {}
                Err(e) => tracing::warn!(error = %e, "proctoring retention sweep failed"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_is_bound_to_session_and_position() {
        let crypto = Crypto::new(&[3; 32], &[4; 32]);
        let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
        let sealed = seal(&crypto, a, 0, b"webm").unwrap();
        assert_eq!(open(&crypto, a, 0, &sealed).unwrap(), b"webm");
        assert!(open(&crypto, a, 1, &sealed).is_err());
        assert!(open(&crypto, b, 0, &sealed).is_err());
    }

    #[tokio::test]
    async fn store_round_trip_and_removal() {
        let root = std::env::temp_dir().join(format!("proctoring-test-{}", Uuid::now_v7()));
        let store = RecordingStore::new(root.clone());
        let (cand, sess) = (Uuid::now_v7(), Uuid::now_v7());
        store.write_chunk(cand, sess, 0, b"abc").await.unwrap();
        store.write_chunk(cand, sess, 1, b"def").await.unwrap();
        assert_eq!(store.read_chunk(cand, sess, 1).await.unwrap(), b"def");
        let listed = |n: i32| -> Vec<db::proctoring::Chunk> {
            (0..n)
                .map(|seq| db::proctoring::Chunk { seq, size: 3 })
                .collect()
        };
        assert!(store.has_chunks(cand, sess, &listed(2)).await.unwrap());
        assert!(
            !store.has_chunks(cand, sess, &listed(3)).await.unwrap(),
            "metadata without a file is not playable"
        );
        assert_eq!(store.candidate_dirs().await.unwrap(), [cand]);
        store.remove_session(cand, sess).await.unwrap();
        store.remove_session(cand, sess).await.unwrap();
        assert!(store.read_chunk(cand, sess, 0).await.is_err());
        store.remove_candidate(cand).await.unwrap();
        assert!(store.candidate_dirs().await.unwrap().is_empty());
        tokio::fs::remove_dir_all(root).await.unwrap();
    }

    #[test]
    fn badge_reports_behavior_not_the_video() {
        let row = db::proctoring::CardBadge {
            candidate_id: Uuid::now_v7(),
            camera: true,
            chunks: 12,
            tab_hidden: 1,
            window_blur: 1,
            fullscreen_exit: 0,
            paste: 3,
            camera_lost: 0,
            recording_failed: 0,
        };
        let b = badge(&row);
        assert_eq!(b.status, ProctoringStatus::Recorded);
        assert_eq!((b.left_test, b.pastes, b.fullscreen_exits), (2, 3, 0));
        let json = serde_json::to_value(&b).unwrap();
        let mut keys: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, ["fullscreen_exits", "left_test", "pastes", "status"]);
    }

    const RETENTION_DAYS: i64 = 30;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn session(camera: bool, chunks: i64) -> db::proctoring::Summary {
        db::proctoring::Summary {
            session_id: Uuid::from_u128(1),
            candidate_id: Uuid::from_u128(2),
            camera,
            mime: camera.then(|| "video/webm".to_owned()),
            started_at: at("2026-09-01T10:00:00Z"),
            purged_at: None,
            chunks,
            bytes: chunks * 125_000,
            tab_hidden: 0,
            window_blur: 1,
            fullscreen_exit: 0,
            paste: 2,
            camera_lost: 0,
            recording_failed: 0,
        }
    }

    fn view(r: &db::proctoring::Summary) -> ProctoringSummary {
        summary_view(r, &facts(r, false), RETENTION_DAYS)
    }

    #[test]
    fn recorded_session_plays_until_retention_ends() {
        let s = view(&session(true, 3));
        assert_eq!(s.status, ProctoringStatus::Recorded);
        assert!(s.playable && !s.purged);
        assert_eq!((s.chunks, s.bytes), (3, 375_000));
        assert_eq!(s.delete_after, Some(at("2026-10-01T10:00:00Z")));
        assert_eq!((s.events.window_blur, s.events.paste), (1, 2));
    }

    #[test]
    fn purged_recording_is_gone_for_good() {
        let purged = db::proctoring::Summary {
            purged_at: Some(at("2026-10-01T12:00:00Z")),
            ..session(true, 3)
        };
        let s = view(&purged);
        assert!(s.purged);
        assert!(!s.playable);
        assert_eq!(s.delete_after, None);
        assert_eq!(s.chunks, 3, "the summary outlives the video");
    }

    #[test]
    fn session_without_camera_has_nothing_to_delete() {
        let s = view(&session(false, 0));
        assert_eq!(s.status, ProctoringStatus::Declined);
        assert!(!s.playable && !s.purged);
        assert_eq!(s.delete_after, None);
    }

    #[test]
    fn session_without_chunks_is_not_playable() {
        let s = view(&session(true, 0));
        assert_eq!(s.status, ProctoringStatus::Interrupted);
        assert!(!s.playable);
        assert_eq!(s.delete_after, Some(at("2026-10-01T10:00:00Z")));
        let negative = db::proctoring::Summary {
            bytes: -1,
            ..session(true, 0)
        };
        assert_eq!(view(&negative).bytes, 0);
    }

    #[test]
    fn sweep_cutoff_matches_the_promised_deletion_time() {
        let now = at("2026-10-07T12:00:00Z");
        let cutoff = retention_cutoff(now, RETENTION_DAYS);
        assert_eq!(cutoff, at("2026-09-07T12:00:00Z"));
        let on_the_edge = db::proctoring::Summary {
            started_at: cutoff,
            ..session(true, 3)
        };
        assert_eq!(
            view(&on_the_edge).delete_after,
            Some(now),
            "the sweep takes sessions started strictly before the cutoff, so nothing goes early"
        );
    }
}
