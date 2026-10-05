//! Proctoring of a test session (architecture §5.6): the summary shown to the candidate and as
//! a badge to employers, the "needs confirmation" rule, and admission of video chunks.
//!
//! Pure logic like the rest of the crate: time arrives as unix seconds, storage and HTTP live
//! in the API.

use serde::{Deserialize, Serialize};

/// What the browser reports while the test is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum IntegrityEvent {
    /// The test tab went to the background.
    TabHidden,
    /// Another window took focus while the tab stayed visible.
    WindowBlur,
    FullscreenExit,
    /// Paste into an answer field.
    Paste,
    /// The camera track ended: device unplugged, permission revoked.
    CameraLost,
    /// MediaRecorder failed, chunks could not be delivered, or the page was left mid-test.
    RecordingFailed,
}

impl IntegrityEvent {
    pub const ALL: [IntegrityEvent; 6] = [
        IntegrityEvent::TabHidden,
        IntegrityEvent::WindowBlur,
        IntegrityEvent::FullscreenExit,
        IntegrityEvent::Paste,
        IntegrityEvent::CameraLost,
        IntegrityEvent::RecordingFailed,
    ];

    /// Something the candidate did, as opposed to a technical failure of the recording.
    pub const fn is_behavioral(self) -> bool {
        matches!(
            self,
            IntegrityEvent::TabHidden
                | IntegrityEvent::WindowBlur
                | IntegrityEvent::FullscreenExit
                | IntegrityEvent::Paste
        )
    }
}

/// Per-kind counters indexed by `IntegrityEvent as usize`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventCounts([u16; IntegrityEvent::ALL.len()]);

impl EventCounts {
    pub fn add(&mut self, event: IntegrityEvent, n: u16) {
        let slot = &mut self.0[event as usize];
        *slot = slot.saturating_add(n);
    }

    pub fn get(&self, event: IntegrityEvent) -> u16 {
        self.0[event as usize]
    }

    /// Times the candidate left the test: tab switches and focus moving to another window.
    pub fn left_test(&self) -> u16 {
        self.get(IntegrityEvent::TabHidden)
            .saturating_add(self.get(IntegrityEvent::WindowBlur))
    }

    pub fn behavioral(&self) -> u32 {
        IntegrityEvent::ALL
            .iter()
            .filter(|e| e.is_behavioral())
            .map(|e| u32::from(self.get(*e)))
            .sum()
    }

    pub fn technical(&self) -> u32 {
        IntegrityEvent::ALL
            .iter()
            .filter(|e| !e.is_behavioral())
            .map(|e| u32::from(self.get(*e)))
            .sum()
    }
}

impl FromIterator<(IntegrityEvent, u16)> for EventCounts {
    fn from_iter<I: IntoIterator<Item = (IntegrityEvent, u16)>>(iter: I) -> Self {
        let mut counts = EventCounts::default();
        for (event, n) in iter {
            counts.add(event, n);
        }
        counts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ProctoringStatus {
    /// The candidate chose the camera and the test is still running.
    Recording,
    Recorded,
    /// The candidate continued without the camera.
    Declined,
    /// The camera was chosen but the recording stopped before the end of the test.
    Interrupted,
}

/// What is known about a session's proctoring at a given moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Facts {
    pub camera: bool,
    pub in_progress: bool,
    pub chunks: u32,
    pub counts: EventCounts,
}

/// Recomputed from stored facts on every read rather than frozen at the last answer: the final
/// chunk and the last event batch arrive a moment after the session ends (see
/// [`accepts_uploads`]), and the summary must include them.
pub fn status(f: &Facts) -> ProctoringStatus {
    if !f.camera {
        ProctoringStatus::Declined
    } else if f.counts.technical() > 0 || (!f.in_progress && f.chunks == 0) {
        ProctoringStatus::Interrupted
    } else if f.in_progress {
        ProctoringStatus::Recording
    } else {
        ProctoringStatus::Recorded
    }
}

/// «Требует подтверждения». Under the model `l_z` is roughly N(0, 1), so the −2 cut alone
/// flags about 2 % of honest candidates, and a single tab switch is routine. An aberrant
/// pattern ("hard items right, easy wrong") together with leaving the test or pasting is the
/// signature of outside help; either signal alone is not evidence.
pub fn needs_confirmation(aberrant: bool, counts: &EventCounts) -> bool {
    aberrant && counts.behavioral() > 0
}

/// Chunks are accepted while the test runs and for a short tail after the last answer: the
/// recorder emits its final chunk only after `stop()`, which happens once the result is known.
pub fn accepts_uploads(
    in_progress: bool,
    finished_at: Option<i64>,
    now: i64,
    tail_secs: i64,
) -> bool {
    in_progress || finished_at.is_some_and(|f| now <= f + tail_secs)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkLimits {
    pub max_chunk_bytes: u32,
    pub max_session_bytes: u64,
    pub max_chunks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkAdmission {
    Store,
    /// A retry of a chunk that is already stored: acknowledge without writing it again.
    Duplicate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChunkError {
    #[error("empty chunk")]
    Empty,
    #[error("chunk exceeds the per-chunk limit")]
    TooLarge,
    #[error("recording exceeds the per-session limit")]
    SessionFull,
    #[error("expected chunk {expected}")]
    OutOfOrder { expected: u32 },
}

/// MediaRecorder chunks are slices of one WebM stream: only the first carries the header, so
/// the stored prefix `0..n` must stay gap-free for the concatenation to play. Hence strict
/// order; the client uploads sequentially and gives up (status `interrupted`) on a hole.
pub fn admit_chunk(
    seq: u32,
    size: u32,
    stored_chunks: u32,
    stored_bytes: u64,
    limits: &ChunkLimits,
) -> Result<ChunkAdmission, ChunkError> {
    if seq < stored_chunks {
        return Ok(ChunkAdmission::Duplicate);
    }
    if size == 0 {
        return Err(ChunkError::Empty);
    }
    if size > limits.max_chunk_bytes {
        return Err(ChunkError::TooLarge);
    }
    if seq > stored_chunks {
        return Err(ChunkError::OutOfOrder {
            expected: stored_chunks,
        });
    }
    if seq >= limits.max_chunks || stored_bytes + u64::from(size) > limits.max_session_bytes {
        return Err(ChunkError::SessionFull);
    }
    Ok(ChunkAdmission::Store)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: ChunkLimits = ChunkLimits {
        max_chunk_bytes: 1000,
        max_session_bytes: 2500,
        max_chunks: 4,
    };

    fn counts(events: &[(IntegrityEvent, u16)]) -> EventCounts {
        events.iter().copied().collect()
    }

    fn facts(
        camera: bool,
        in_progress: bool,
        chunks: u32,
        events: &[(IntegrityEvent, u16)],
    ) -> Facts {
        Facts {
            camera,
            in_progress,
            chunks,
            counts: counts(events),
        }
    }

    #[test]
    fn counts_split_behavior_from_technical_failures() {
        let c = counts(&[
            (IntegrityEvent::TabHidden, 2),
            (IntegrityEvent::WindowBlur, 1),
            (IntegrityEvent::Paste, 1),
            (IntegrityEvent::CameraLost, 1),
            (IntegrityEvent::TabHidden, 1),
        ]);
        assert_eq!(c.get(IntegrityEvent::TabHidden), 3);
        assert_eq!(c.left_test(), 4);
        assert_eq!(c.behavioral(), 5);
        assert_eq!(c.technical(), 1);

        let mut full = EventCounts::default();
        full.add(IntegrityEvent::Paste, u16::MAX);
        full.add(IntegrityEvent::Paste, 5);
        assert_eq!(full.get(IntegrityEvent::Paste), u16::MAX);
    }

    #[test]
    fn status_follows_camera_choice_and_delivery() {
        use IntegrityEvent::*;
        use ProctoringStatus::*;
        assert_eq!(status(&facts(false, true, 0, &[])), Declined);
        assert_eq!(status(&facts(false, false, 0, &[(TabHidden, 3)])), Declined);
        assert_eq!(status(&facts(true, true, 0, &[])), Recording);
        assert_eq!(status(&facts(true, true, 4, &[(TabHidden, 2)])), Recording);
        assert_eq!(status(&facts(true, false, 9, &[(Paste, 1)])), Recorded);
        assert_eq!(
            status(&facts(true, true, 3, &[(CameraLost, 1)])),
            Interrupted
        );
        assert_eq!(
            status(&facts(true, false, 9, &[(RecordingFailed, 1)])),
            Interrupted
        );
        assert_eq!(
            status(&facts(true, false, 0, &[])),
            Interrupted,
            "camera chosen but nothing arrived"
        );
    }

    #[test]
    fn confirmation_needs_both_signals() {
        use IntegrityEvent::*;
        assert!(!needs_confirmation(false, &counts(&[(TabHidden, 5)])));
        assert!(!needs_confirmation(true, &counts(&[])));
        assert!(
            !needs_confirmation(true, &counts(&[(CameraLost, 1), (RecordingFailed, 1)])),
            "a broken camera is not the candidate's behavior"
        );
        assert!(needs_confirmation(true, &counts(&[(WindowBlur, 1)])));
        assert!(needs_confirmation(true, &counts(&[(Paste, 2)])));
        assert!(needs_confirmation(true, &counts(&[(FullscreenExit, 1)])));
    }

    #[test]
    fn uploads_close_after_the_tail() {
        assert!(accepts_uploads(true, None, 1_000, 60));
        assert!(accepts_uploads(false, Some(1_000), 1_060, 60));
        assert!(!accepts_uploads(false, Some(1_000), 1_061, 60));
        assert!(!accepts_uploads(false, None, 1_000, 60));
    }

    #[test]
    fn chunks_are_admitted_in_order_within_limits() {
        use ChunkAdmission::*;
        assert_eq!(admit_chunk(0, 900, 0, 0, &LIMITS), Ok(Store));
        assert_eq!(admit_chunk(1, 900, 1, 900, &LIMITS), Ok(Store));
        assert_eq!(admit_chunk(0, 900, 2, 1800, &LIMITS), Ok(Duplicate));
        assert_eq!(
            admit_chunk(3, 10, 2, 1800, &LIMITS),
            Err(ChunkError::OutOfOrder { expected: 2 })
        );
        assert_eq!(admit_chunk(2, 0, 2, 1800, &LIMITS), Err(ChunkError::Empty));
        assert_eq!(
            admit_chunk(2, 1001, 2, 1800, &LIMITS),
            Err(ChunkError::TooLarge)
        );
        assert_eq!(
            admit_chunk(2, 701, 2, 1800, &LIMITS),
            Err(ChunkError::SessionFull)
        );
        assert_eq!(admit_chunk(2, 700, 2, 1800, &LIMITS), Ok(Store));
        assert_eq!(
            admit_chunk(4, 1, 4, 2400, &LIMITS),
            Err(ChunkError::SessionFull),
            "chunk count cap"
        );
    }
}
