/**
 * Proctoring of one running test: camera, MediaRecorder, chunk uploads and integrity events.
 *
 * A module singleton, so the camera opened on the proctoring step keeps recording after the
 * client-side navigation to the test. Everything is best-effort: a failure ends the recording
 * («запись прервалась») and never blocks answering.
 */
import { isApiError } from "@/api/client";
import { postProctoringEvents, uploadProctoringChunk } from "@/api/endpoints";
import type { IntegrityEvent, IntegrityEventInput } from "@/api/types";

const CHUNK_MS = 10_000;
const FLUSH_MS = 5_000;
const MAX_BATCH = 50;
const RETRY_MS = [1_000, 3_000, 8_000];
const BITRATE = 100_000;
const VIDEO: MediaTrackConstraints = {
  width: { ideal: 320 },
  height: { ideal: 240 },
  frameRate: { ideal: 15, max: 15 },
  facingMode: "user",
};
/** VP8 first: cheapest to encode, so recording does not compete with the test for CPU. */
const MIME_TYPES = ["video/webm;codecs=vp8", "video/webm", "video/mp4"];
/** The runner unmounts and remounts on StrictMode and fast refresh; only a real leave stops the camera. */
const LEAVE_GRACE_MS = 400;
/** Answers that arrive with the server's reply mean the session is closed: no point retrying. */
const FINAL_CODES = new Set(["chunk_out_of_order", "proctoring_not_recording", "assessment_finished", "proctoring_off", "chunk_too_large", "recording_too_large", "not_found"]);

export type CameraState = "off" | "starting" | "ready" | "denied" | "unavailable";
/** `stopping`: the test ended and the last chunk is on its way. */
export type RecordingState = "idle" | "recording" | "stopping" | "stopped" | "interrupted";

export interface ProctorSnapshot {
  camera: CameraState;
  recording: RecordingState;
  sessionId: string | null;
  stream: MediaStream | null;
  uploaded: number;
}

const INITIAL: ProctorSnapshot = { camera: "off", recording: "idle", sessionId: null, stream: null, uploaded: 0 };

function pickMime(): string | undefined {
  if (typeof MediaRecorder === "undefined") return undefined;
  return MIME_TYPES.find((m) => MediaRecorder.isTypeSupported(m));
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

class Proctor {
  private snap: ProctorSnapshot = INITIAL;
  private listeners = new Set<() => void>();
  private recorder: MediaRecorder | null = null;
  private nextSeq = 0;
  private queue: { seq: number; blob: Blob }[] = [];
  private pumping = false;
  private events: IntegrityEventInput[] = [];
  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private leaveTimer: ReturnType<typeof setTimeout> | null = null;
  private unmonitor: (() => void) | null = null;
  private itemSeq: number | null = null;

  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  getSnapshot = () => this.snap;

  getServerSnapshot = () => INITIAL;

  private set(patch: Partial<ProctorSnapshot>) {
    this.snap = { ...this.snap, ...patch };
    this.listeners.forEach((l) => l());
  }

  async openCamera(): Promise<CameraState> {
    if (this.snap.stream) return "ready";
    if (!navigator.mediaDevices?.getUserMedia || !pickMime()) {
      this.set({ camera: "unavailable" });
      return "unavailable";
    }
    this.set({ camera: "starting" });
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: VIDEO, audio: false });
      this.set({ camera: "ready", stream });
      return "ready";
    } catch (e) {
      const denied = e instanceof DOMException && (e.name === "NotAllowedError" || e.name === "SecurityError");
      const state: CameraState = denied ? "denied" : "unavailable";
      this.set({ camera: state });
      return state;
    }
  }

  /** Leaving the proctoring step without starting: the preview must not keep the camera on. */
  releasePreview() {
    if (this.snap.recording === "recording" || this.snap.recording === "stopping") return;
    this.releaseCamera();
  }

  private releaseCamera() {
    this.snap.stream?.getTracks().forEach((t) => t.stop());
    this.set({ stream: null, camera: "off" });
  }

  /** Called right after the session is created on the proctoring step. */
  begin(sessionId: string, camera: boolean) {
    this.reset(sessionId);
    this.monitor();
    if (camera) this.record();
  }

  /**
   * The page was reloaded or opened anew mid-test: the recorder from before is gone and a new one
   * cannot continue the same video file, so the recording is reported as stopped.
   */
  adopt(sessionId: string, cameraWasRecording: boolean) {
    if (this.snap.sessionId === sessionId) {
        if (!this.unmonitor) this.monitor();
        return;
    }
    this.reset(sessionId);
    this.monitor();
    if (cameraWasRecording) {
      this.log("recording_failed");
      this.set({ recording: "interrupted" });
      void this.flush();
    }
  }

  setItem(seq: number | null) {
    this.itemSeq = seq;
  }

  attach(sessionId: string) {
    if (this.snap.sessionId === sessionId && this.leaveTimer) {
      clearTimeout(this.leaveTimer);
      this.leaveTimer = null;
    }
  }

  /** The runner unmounted while the test is still running: the candidate left the test page. */
  detach(sessionId: string) {
    if (this.snap.sessionId !== sessionId) return;
    if (this.leaveTimer) clearTimeout(this.leaveTimer);
    this.leaveTimer = setTimeout(() => {
      this.leaveTimer = null;
      if (this.snap.recording === "recording") {
        this.log("recording_failed");
        this.stopRecorder("interrupted");
      }
      this.unmonitorAll();
      void this.flush(true);
    }, LEAVE_GRACE_MS);
  }

  /** The session has a result: deliver the last chunk and events, then free the camera. */
  finish(sessionId: string) {
    if (this.snap.sessionId !== sessionId) return;
    if (this.leaveTimer) {
      clearTimeout(this.leaveTimer);
      this.leaveTimer = null;
    }
    this.unmonitorAll();
    void this.flush();
    if (this.snap.recording === "recording") this.stopRecorder("stopping");
  }

  private reset(sessionId: string) {
    this.unmonitorAll();
    if (this.recorder) {
      this.recorder.ondataavailable = null;
      this.recorder.onstop = null;
      if (this.recorder.state !== "inactive") this.recorder.stop();
    }
    this.recorder = null;
    this.nextSeq = 0;
    this.queue = [];
    this.events = [];
    this.itemSeq = null;
    this.set({ sessionId, recording: "idle", uploaded: 0 });
  }

  private record() {
    const stream = this.snap.stream;
    const mimeType = pickMime();
    if (!stream || !mimeType) {
      this.log("recording_failed");
      this.set({ recording: "interrupted" });
      return;
    }
    let recorder: MediaRecorder;
    try {
      recorder = new MediaRecorder(stream, { mimeType, videoBitsPerSecond: BITRATE });
    } catch {
      this.log("recording_failed");
      this.set({ recording: "interrupted" });
      this.releaseCamera();
      return;
    }
    recorder.ondataavailable = (e) => {
      if (e.data.size > 0) {
        this.queue.push({ seq: this.nextSeq++, blob: e.data });
        void this.pump();
      }
    };
    recorder.onerror = () => {
      if (this.snap.recording !== "recording") return;
      this.log("recording_failed");
      this.stopRecorder("interrupted");
    };
    recorder.onstop = () => {
      this.releaseCamera();
      if (this.snap.recording === "stopping" && !this.pumping && this.queue.length === 0) this.set({ recording: "stopped" });
    };
    for (const track of stream.getVideoTracks()) {
      track.onended = () => {
        if (this.snap.recording !== "recording") return;
        this.log("camera_lost");
        this.stopRecorder("interrupted");
      };
    }
    this.recorder = recorder;
    recorder.start(CHUNK_MS);
    this.set({ recording: "recording" });
  }

  private stopRecorder(next: "stopping" | "interrupted") {
    this.set({ recording: next });
    if (this.recorder && this.recorder.state !== "inactive") this.recorder.stop();
    else this.releaseCamera();
    if (next === "interrupted") void this.flush();
  }

  /** Strictly sequential: the server stores a gap-free prefix of the WebM stream. */
  private async pump() {
    if (this.pumping) return;
    const sessionId = this.snap.sessionId;
    if (!sessionId) return;
    this.pumping = true;
    try {
      while (this.queue.length && this.snap.sessionId === sessionId) {
        const { seq, blob } = this.queue[0];
        if (!(await this.upload(sessionId, seq, blob))) {
          this.queue = [];
          if (this.snap.recording === "recording") {
            this.log("recording_failed");
            this.stopRecorder("interrupted");
          }
          break;
        }
        this.queue.shift();
        this.set({ uploaded: this.snap.uploaded + 1 });
      }
    } finally {
      this.pumping = false;
    }
    if (this.snap.recording === "stopping" && this.queue.length === 0 && this.recorder?.state === "inactive") {
      this.set({ recording: "stopped" });
    }
  }

  private async upload(sessionId: string, seq: number, blob: Blob): Promise<boolean> {
    for (let attempt = 0; ; attempt++) {
      try {
        await uploadProctoringChunk(sessionId, seq, blob);
        return true;
      } catch (e) {
        if (isApiError(e) && FINAL_CODES.has(e.code)) return false;
        if (attempt >= RETRY_MS.length) return false;
        await sleep(RETRY_MS[attempt]);
      }
    }
  }

  private log(kind: IntegrityEvent, at = Date.now(), durationMs?: number) {
    if (this.events.length >= MAX_BATCH * 4) return;
    this.events.push({
      kind,
      at: new Date(at).toISOString(),
      item_seq: this.itemSeq,
      duration_ms: durationMs === undefined ? null : Math.max(0, Math.round(durationMs)),
    });
  }

  private async flush(keepalive = false) {
    const sessionId = this.snap.sessionId;
    while (sessionId && this.events.length) {
      const batch = this.events.splice(0, MAX_BATCH);
      try {
        await postProctoringEvents(sessionId, batch, keepalive);
      } catch (e) {
        if (!isApiError(e) || e.status >= 500 || e.status === 0) this.events.unshift(...batch);
        return;
      }
    }
  }

  /** One leave of the test counts once: a tab switch fires both `blur` and `visibilitychange`. */
  private monitor() {
    let hiddenAt: number | null = null;
    let blurredAt: number | null = null;
    let fullscreen = Boolean(document.fullscreenElement);

    const onVisibility = () => {
      if (document.hidden) {
        hiddenAt = Date.now();
        blurredAt = null;
      } else if (hiddenAt !== null) {
        this.log("tab_hidden", hiddenAt, Date.now() - hiddenAt);
        hiddenAt = null;
      }
    };
    const onBlur = () => {
      const at = Date.now();
      setTimeout(() => {
        if (!document.hidden && hiddenAt === null && !document.hasFocus()) blurredAt = at;
      }, 80);
    };
    const onFocus = () => {
      if (blurredAt !== null) {
        this.log("window_blur", blurredAt, Date.now() - blurredAt);
        blurredAt = null;
      }
    };
    const onFullscreen = () => {
      if (document.fullscreenElement) fullscreen = true;
      else if (fullscreen) {
        fullscreen = false;
        this.log("fullscreen_exit");
      }
    };
    const onPaste = (e: ClipboardEvent) => {
      if (e.target instanceof Element && e.target.closest("[data-answer]")) this.log("paste");
    };
    const onPageHide = () => {
      if (hiddenAt !== null) this.log("tab_hidden", hiddenAt);
      void this.flush(true);
    };

    document.addEventListener("visibilitychange", onVisibility);
    window.addEventListener("blur", onBlur);
    window.addEventListener("focus", onFocus);
    document.addEventListener("fullscreenchange", onFullscreen);
    document.addEventListener("paste", onPaste, true);
    window.addEventListener("pagehide", onPageHide);
    this.flushTimer = setInterval(() => void this.flush(), FLUSH_MS);
    this.unmonitor = () => {
      document.removeEventListener("visibilitychange", onVisibility);
      window.removeEventListener("blur", onBlur);
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("fullscreenchange", onFullscreen);
      document.removeEventListener("paste", onPaste, true);
      window.removeEventListener("pagehide", onPageHide);
    };
  }

  private unmonitorAll() {
    this.unmonitor?.();
    this.unmonitor = null;
    if (this.flushTimer) clearInterval(this.flushTimer);
    this.flushTimer = null;
  }
}

export const proctor = new Proctor();
