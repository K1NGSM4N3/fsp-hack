"use client";

import { useQueryClient } from "@tanstack/react-query";
import { PlayCircle, Video, VideoOff } from "lucide-react";
import { useEffect, useState, type SyntheticEvent } from "react";
import { proctoringRecordingUrl } from "@/api/endpoints";
import type { AssessmentSession } from "@/api/types";
import { Button, Card, CardHeader } from "@/components/ui";
import { proctoringRemarks, proctoringStatusText } from "@/features/employer/components/ProctoringBadge";
import { qk } from "@/features/keys";
import { formatDate, formatDateTime } from "@/lib/format";
import { remarksForCandidate, useProctor } from "./ui";

/**
 * MediaRecorder WebM has no duration in its header; seeking far ahead once makes the browser scan
 * to the end and learn it, so the timeline works.
 */
function fixDuration(e: SyntheticEvent<HTMLVideoElement>) {
  const v = e.currentTarget;
  if (v.duration !== Infinity) return;
  const reset = () => {
    v.removeEventListener("timeupdate", reset);
    v.currentTime = 0;
  };
  v.addEventListener("timeupdate", reset);
  v.currentTime = 1e101;
}

type Availability = "checking" | "available" | "missing" | "failed";

/**
 * Asks for the headers first: the summary counts chunks from metadata, but the files can be gone
 * (demo data, a lost volume), and a bare <video> would only show a broken player.
 */
function Player({ sessionId }: { sessionId: string }) {
  const url = proctoringRecordingUrl(sessionId);
  const [state, setState] = useState<Availability>("checking");
  useEffect(() => {
    let live = true;
    fetch(url, { method: "HEAD", credentials: "include" })
      .then((r) => live && setState(r.status === 404 ? "missing" : "available"))
      .catch(() => live && setState("available"));
    return () => {
      live = false;
    };
  }, [url]);
  if (state === "checking") return <p className="mt-4 text-sm text-ink-subtle">Открываем запись…</p>;
  if (state === "missing")
    return (
      <p className="mt-4 rounded-control bg-canvas p-3 text-sm text-ink-muted">
        Видеофайл недоступен: на сервере его нет — запись не сохранилась или уже удалена. Отметка у результата и журнал событий от этого не меняются.
      </p>
    );
  if (state === "failed") return <p className="mt-4 text-sm text-ink-muted">Не удалось открыть запись. Обновите страницу и попробуйте ещё раз.</p>;
  return (
    <video
      controls
      playsInline
      preload="metadata"
      src={url}
      onLoadedMetadata={fixDuration}
      onError={() => setState("failed")}
      className="mt-4 aspect-[4/3] w-full max-w-md rounded-control border border-line bg-night"
    >
      Браузер не умеет показывать это видео.
    </video>
  );
}

export function ProctoringResultCard({ session }: { session: AssessmentSession }) {
  const s = session.proctoring;
  const qc = useQueryClient();
  const p = useProctor();
  const [open, setOpen] = useState(false);
  const delivered = p.sessionId === session.id && p.recording === "stopped";

  useEffect(() => {
    if (delivered) void qc.invalidateQueries({ queryKey: qk.assessment(session.id) });
  }, [delivered, qc, session.id]);

  if (!s) return null;
  const remarks = remarksForCandidate(s.events);
  const employerSees = [
    proctoringStatusText(s.status),
    session.review?.decision === "confirm" && session.result?.assigned_grade ? "Результат подтверждён" : null,
    proctoringRemarks({ left_test: s.events.tab_hidden + s.events.window_blur, pastes: s.events.paste, fullscreen_exits: s.events.fullscreen_exit }),
  ].filter(Boolean);
  const video = s.status === "recorded" || s.status === "recording" || (s.status === "interrupted" && s.playable);

  const title =
    s.status === "declined" ? "Тест без видеозаписи" : s.status === "interrupted" ? "Видеозапись прервалась" : s.status === "recording" ? "Сохраняем видеозапись…" : "Тест с видеозаписью";
  let text: string;
  if (s.purged) text = "Запись удалена: истёк срок хранения или вы отозвали согласие. Отметка у результата осталась.";
  else if (s.status === "declined") text = "Вы прошли тест без камеры. Результат засчитан так же, у него отметка «без видеозаписи».";
  else if (s.status === "interrupted")
    text = `Запись остановилась до конца теста. Результат засчитан, у него отметка «запись прервалась».${s.playable ? " Сохранённую часть можно посмотреть." : ""}`;
  else text = "Запись видите только вы. Работодатели видео не получают — только отметку.";

  return (
    <Card>
      <CardHeader
        title={
          <span className="flex items-center gap-2">
            {video ? <Video className="size-5 text-primary" aria-hidden /> : <VideoOff className="size-5 text-ink-faint" aria-hidden />}
            {title}
          </span>
        }
        description={text}
      />
      <div className="grid gap-4 text-sm sm:grid-cols-2">
        <div>
          <p className="text-xs text-ink-subtle">Во время теста</p>
          {remarks.length ? (
            <ul className="mt-1 list-disc space-y-0.5 pl-5 text-ink">
              {remarks.map((r) => (
                <li key={r}>{r}</li>
              ))}
            </ul>
          ) : (
            <p className="mt-1 text-ink">Замечаний нет</p>
          )}
        </div>
        <div>
          <p className="text-xs text-ink-subtle">Что видят работодатели</p>
          <p className="mt-1 text-ink">{employerSees.join(" · ")}</p>
          {s.delete_after && !s.purged && <p className="mt-2 text-xs text-ink-subtle">Запись удалим {formatDate(s.delete_after)}</p>}
        </div>
      </div>
      {session.recording_views.length > 0 && (
        <p className="mt-4 rounded-control bg-canvas p-3 text-xs text-ink-muted">
          Запись теста просмотрена модератором: {session.recording_views.map((at) => formatDateTime(at)).join(", ")}. Каждый просмотр записывается в журнал.
        </p>
      )}
      {s.playable &&
        (open ? (
          <Player sessionId={session.id} />
        ) : (
          <Button variant="secondary" size="sm" className="mt-5" icon={<PlayCircle className="size-4" aria-hidden />} onClick={() => setOpen(true)}>
            Посмотреть запись
          </Button>
        ))}
    </Card>
  );
}
