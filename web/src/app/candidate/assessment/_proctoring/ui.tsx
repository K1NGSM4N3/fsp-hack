"use client";

import { useQuery } from "@tanstack/react-query";
import { Maximize2, Video, VideoOff } from "lucide-react";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { getProctoringOptions } from "@/api/endpoints";
import type { EventCounts, ProctoringSummary } from "@/api/types";
import { Badge } from "@/components/ui";
import { cn } from "@/lib/cn";
import { pluralize } from "@/lib/format";
import { proctor } from "./proctor";

export function useProctoringOptions() {
  return useQuery({ queryKey: ["candidate", "proctoring"], queryFn: getProctoringOptions, staleTime: 5 * 60_000 });
}

export function useProctor() {
  return useSyncExternalStore(proctor.subscribe, proctor.getSnapshot, proctor.getServerSnapshot);
}

export function CameraView({ stream, className, label }: { stream: MediaStream | null; className?: string; label: string }) {
  const ref = useRef<HTMLVideoElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.srcObject = stream;
    if (stream) void el.play().catch(() => undefined);
  }, [stream]);
  return <video ref={ref} muted playsInline autoPlay aria-label={label} className={cn("-scale-x-100 bg-night object-cover", className)} />;
}

/** What the candidate did during the test, in their words. */
export function remarksForCandidate(e: EventCounts): string[] {
  const left = e.tab_hidden + e.window_blur;
  const out: string[] = [];
  if (left) out.push(`уходили со вкладки или в другое окно — ${pluralize(left, ["раз", "раза", "раз"])}`);
  if (e.paste) out.push(`вставляли текст в ответ — ${pluralize(e.paste, ["раз", "раза", "раз"])}`);
  if (e.fullscreen_exit) out.push(`выходили из полноэкранного режима — ${pluralize(e.fullscreen_exit, ["раз", "раза", "раз"])}`);
  if (e.camera_lost) out.push(`камера отключалась — ${pluralize(e.camera_lost, ["раз", "раза", "раз"])}`);
  if (e.recording_failed) out.push("запись остановилась до конца теста");
  return out;
}

const MARK = {
  recording: { tone: "danger", icon: Video, text: "идёт запись" },
  recorded: { tone: "success", icon: Video, text: "с видеозаписью" },
  declined: { tone: "muted", icon: VideoOff, text: "без видеозаписи" },
  interrupted: { tone: "warning", icon: VideoOff, text: "запись прервалась" },
} as const;

export function ProctoringMark({ summary }: { summary: ProctoringSummary }) {
  const m = MARK[summary.status];
  const Icon = m.icon;
  return (
    <Badge tone={m.tone} icon={<Icon className="size-3" aria-hidden />}>
      {m.text}
    </Badge>
  );
}

/**
 * Always on screen while the camera records, but never over the answer: a slim strip that sticks
 * under the cabinet header, with an opaque background the item scrolls beneath.
 */
export function RecordingIndicator() {
  const p = useProctor();
  if (p.recording === "idle" || p.recording === "stopped") return null;
  const live = p.recording === "recording" && p.stream;
  return (
    <div className="sticky top-14 z-20 -mt-2 mb-4 bg-canvas/95 py-2 backdrop-blur-sm print:hidden">
      <div role="status" aria-live="polite" className="inline-flex max-w-full items-center gap-3 rounded-control border border-line bg-white p-1 pr-3 shadow-card">
        {live ? (
          <CameraView stream={p.stream} label="Ваша камера" className="h-9 w-12 shrink-0 rounded-md" />
        ) : (
          <span className="flex h-9 w-12 shrink-0 items-center justify-center rounded-md bg-canvas text-ink-faint">
            <VideoOff className="size-4" aria-hidden />
          </span>
        )}
        {live ? (
          <span className="flex items-center gap-2 text-sm font-medium text-ink">
            <span className="relative flex size-2.5">
              <span className="absolute inline-flex size-full animate-ping rounded-full bg-accent opacity-60 motion-reduce:hidden" />
              <span className="relative inline-flex size-2.5 rounded-full bg-accent" />
            </span>
            Идёт запись
          </span>
        ) : p.recording === "stopping" ? (
          <span className="text-sm font-medium text-ink">Сохраняем запись…</span>
        ) : (
          <span className="min-w-0 text-sm">
            <span className="font-medium text-warning">Запись прервалась</span>
            <span className="text-ink-subtle"> · тест продолжается, результат засчитается</span>
          </span>
        )}
      </div>
    </div>
  );
}

/** Offered, never forced: leaving full screen is logged only if the candidate entered it. */
export function FullscreenButton() {
  const [available, setAvailable] = useState(false);
  useEffect(() => {
    const sync = () => setAvailable(document.fullscreenEnabled && !document.fullscreenElement);
    sync();
    document.addEventListener("fullscreenchange", sync);
    return () => document.removeEventListener("fullscreenchange", sync);
  }, []);
  if (!available) return null;
  return (
    <button
      type="button"
      onClick={() => void document.documentElement.requestFullscreen().catch(() => undefined)}
      className="inline-flex items-center gap-1.5 rounded text-sm text-ink-muted hover:text-ink"
    >
      <Maximize2 className="size-4" aria-hidden />
      На весь экран
    </button>
  );
}
