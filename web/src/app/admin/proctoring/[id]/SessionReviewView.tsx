"use client";

import { CheckCircle2, CircleSlash, Clock, EyeOff, PlayCircle, ShieldAlert, Timer, TriangleAlert, Video, VideoOff } from "lucide-react";
import { useState, type ReactNode, type SyntheticEvent } from "react";
import { isApiError } from "@/api/client";
import { adminRecordingUrl, type ReviewDecision, type ReviewEvent, type SessionReview } from "@/api/admin";
import { CategoryBadge } from "@/components/domain/Category";
import { Alert, Badge, Button, Card, CardHeader, Dialog, ErrorAlert, Field, PageHeader, PageSkeleton, Textarea, useToast } from "@/components/ui";
import { useDecide, useSessionReview } from "@/features/admin/hooks";
import { DECISION_DONE, DECISION_EFFECT, DECISION_LABEL, EVENT_LABEL, VIDEO_LABEL, lz } from "@/features/admin/labels";
import { cn } from "@/lib/cn";
import { formatDate, formatDateTime } from "@/lib/format";
import { ATTEMPT_KIND_LABEL, GRADE_LABEL } from "@/lib/labels";
import { DifficultyPattern } from "./DifficultyPattern";

const COMMENT_MAX = 2000;

function clock(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

function seconds(ms: number | null | undefined): string | null {
  return typeof ms === "number" ? `${Math.max(1, Math.round(ms / 1000))} с` : null;
}

function Fact({ label, value, hint }: { label: string; value: ReactNode; hint?: ReactNode }) {
  return (
    <div className="min-w-0">
      <dt className="text-xs text-ink-subtle">{label}</dt>
      <dd className="mt-1 font-medium text-ink">{value}</dd>
      {hint && <dd className="mt-0.5 text-xs text-ink-muted">{hint}</dd>}
    </div>
  );
}

function assignmentText(r: SessionReview): string {
  const a = r.assignment;
  const now = a.current_category_name ? `Сейчас у кандидата: ${a.current_category_name}.` : "Сейчас категории у кандидата нет.";
  if (a.annulled_at) return `Присвоение по этому тесту аннулировано ${formatDate(a.annulled_at)}. ${now}`;
  if (a.current) return "Этот тест дал категорию, которую сейчас видят работодатели. Аннулирование её снимет.";
  if (a.assigned_at) return `Грейд из этого теста уже заменён более поздним тестом. ${now}`;
  return `Тест не присвоил грейд. ${now}`;
}

function Summary({ r }: { r: SessionReview }) {
  const passed = r.status === "passed";
  return (
    <Card>
      <div className="flex items-start gap-3">
        {passed ? <CheckCircle2 className="mt-0.5 size-6 shrink-0 text-success" aria-hidden /> : <CircleSlash className="mt-0.5 size-6 shrink-0 text-ink-faint" aria-hidden />}
        <div className="min-w-0">
          <h2 className="text-[17px] font-semibold text-ink">{passed ? `Сдан на ${GRADE_LABEL[r.category.grade]}` : r.status === "failed" ? "Не сдан" : "Тест ещё идёт"}</h2>
          <p className="mt-1 text-sm text-ink-muted">{assignmentText(r)}</p>
        </div>
      </div>
      <dl className="mt-5 grid grid-cols-2 gap-x-4 gap-y-4 sm:grid-cols-4">
        <Fact label="Оценка уровня θ̂" value={<span className="tabular font-mono">{lz(r.result.theta)}</span>} hint={typeof r.result.se === "number" ? `± ${lz(1.96 * r.result.se)} (95 %)` : undefined} />
        <Fact label={`Порог ${GRADE_LABEL[r.category.grade]}`} value={<span className="tabular font-mono">{lz(r.result.cut, 1)}</span>} />
        <Fact label="Уровень по тесту" value={r.result.measured_grade ? GRADE_LABEL[r.result.measured_grade] : "ниже Junior"} />
        <Fact label="Сила результата" value={typeof r.result.test_strength === "number" ? `${Math.round(r.result.test_strength * 100)}/100` : "—"} />
      </dl>
    </Card>
  );
}

function PersonFit({ r }: { r: SessionReview }) {
  const p = r.person_fit;
  return (
    <Card>
      <CardHeader
        title="Согласованность ответов"
        description="Person-fit l_z: насколько картина верных и неверных ответов похожа на ожидаемую при этом уровне. Ниже −2 — нетипично: так бывает примерно у 2 % честных кандидатов, поэтому сама по себе метрика не улика."
        action={
          <Badge tone={p.aberrant ? "danger" : "success"} title={`Порог ${lz(p.threshold)}`}>
            l_z {lz(p.lz)}
          </Badge>
        }
      />
      <p className="mb-4 rounded-control bg-canvas p-3 text-sm text-ink">{p.summary}</p>
      <DifficultyPattern answers={r.answers} theta={r.result.theta} cut={r.result.cut} grade={r.category.grade} />
    </Card>
  );
}

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

type Playback = "idle" | "checking" | "playing" | "missing" | "failed";

function Recording({ r }: { r: SessionReview }) {
  const [state, setState] = useState<Playback>("idle");
  const rec = r.recording;
  const url = adminRecordingUrl(r.session_id);
  const open = () => {
    setState("checking");
    fetch(url, { method: "HEAD", credentials: "include" })
      .then((res) => setState(res.ok ? "playing" : res.status === 404 ? "missing" : "failed"))
      .catch(() => setState("failed"));
  };
  let body: ReactNode;
  if (rec.video === "off") body = <p className="text-sm text-ink-muted">Тест шёл при выключенной видеозаписи: шага с камерой не было, событий браузера тоже. Решать можно только по ответам.</p>;
  else if (rec.video === "declined") body = <p className="text-sm text-ink-muted">Кандидат прошёл тест без камеры. Журнал событий браузера ведётся и без неё — он ниже, на ленте заданий.</p>;
  else if (rec.video === "purged") body = <p className="text-sm text-ink-muted">Запись удалена по сроку хранения (90 дней от теста) или после отзыва согласия. Сводка и журнал событий сохранились.</p>;
  else if (!rec.playable) body = <p className="text-sm text-ink-muted">Камера была включена, но ни одного фрагмента записи не пришло.</p>;
  else if (state === "playing")
    body = (
      <video controls autoPlay playsInline preload="metadata" src={url} onLoadedMetadata={fixDuration} onError={() => setState("failed")} className="aspect-[4/3] w-full max-w-lg rounded-control border border-line bg-night">
        Браузер не умеет показывать это видео.
      </video>
    );
  else if (state === "missing") body = <p className="text-sm text-ink-muted">Файлов записи на этом сервере нет: метаданные есть, видео — нет (так устроены демо-данные). Журнал событий ниже от этого не зависит.</p>;
  else
    body = (
      <div>
        <Button variant="secondary" icon={<PlayCircle className="size-4" aria-hidden />} loading={state === "checking"} onClick={open}>
          Открыть запись
        </Button>
        <p className="mt-2 text-xs text-ink-subtle">Просмотр попадёт в журнал аудита, а кандидат увидит дату в истории своих тестов.</p>
        {state === "failed" && <p className="mt-2 text-sm text-accent-strong">Не удалось открыть запись. Попробуйте ещё раз.</p>}
      </div>
    );
  const label = VIDEO_LABEL[rec.video];
  return (
    <Card>
      <CardHeader
        title={
          <span className="flex items-center gap-2">
            {rec.video === "kept" ? <Video className="size-5 text-primary" aria-hidden /> : <VideoOff className="size-5 text-ink-faint" aria-hidden />}
            Видеозапись
          </span>
        }
        action={<Badge tone={label.tone}>{label.text}</Badge>}
        description={rec.delete_after ? `Будет удалена ${formatDate(rec.delete_after)}` : undefined}
      />
      {body}
      {rec.views.length > 0 && (
        <div className="mt-4 border-t border-line pt-3 text-xs text-ink-muted">
          <p className="font-medium text-ink">Запись открывали</p>
          <p className="mt-1">{rec.views.map((v) => formatDateTime(v.at)).join(" · ")}</p>
        </div>
      )}
    </Card>
  );
}

function EventChip({ e }: { e: ReviewEvent }) {
  const behavioural = e.kind !== "camera_lost" && e.kind !== "recording_failed";
  const away = seconds(e.duration_ms);
  return (
    <span className={cn("inline-flex max-w-full items-center gap-1 rounded-md px-1.5 py-0.5 text-xs font-medium", behavioural ? "bg-warning-50 text-warning" : "bg-canvas text-ink-muted")}>
      <TriangleAlert className="size-3 shrink-0" aria-hidden />
      <span className="truncate">
        {EVENT_LABEL[e.kind] ?? e.kind}
        {away ? `, ${away}` : ""} · {clock(e.offset_ms)}
      </span>
    </span>
  );
}

function Timeline({ r }: { r: SessionReview }) {
  const byItem = new Map<number, ReviewEvent[]>();
  const loose: ReviewEvent[] = [];
  for (const e of r.events) {
    if (typeof e.item_seq === "number") byItem.set(e.item_seq, [...(byItem.get(e.item_seq) ?? []), e]);
    else loose.push(e);
  }
  const start = new Date(r.started_at).getTime();
  return (
    <Card padding="none">
      <div className="px-5 pt-5 sm:px-6 sm:pt-6">
        <CardHeader
          title="Лента: задания и события"
          description="Какое задание было на экране в момент ухода со вкладки или вставки текста. Время — от начала теста. Сам текст задания и ответ не показываются: экземпляр у каждого свой."
        />
      </div>
      {loose.length > 0 && (
        <div className="flex flex-wrap gap-1.5 border-t border-line px-5 py-3 sm:px-6">
          <span className="text-xs text-ink-subtle">Вне заданий:</span>
          {loose.map((e, i) => (
            <EventChip key={i} e={e} />
          ))}
        </div>
      )}
      <div className="hidden overflow-x-auto md:block">
        <table className="w-full text-left text-sm">
          <caption className="sr-only">Задания теста по порядку с событиями браузера</caption>
          <thead>
            <tr className="border-t border-line text-xs text-ink-subtle">
              <th scope="col" className="py-2.5 pr-3 pl-6 font-medium">
                №
              </th>
              <th scope="col" className="px-3 py-2.5 font-medium">
                Задание
              </th>
              <th scope="col" className="px-3 py-2.5 font-medium">
                Сложность
              </th>
              <th scope="col" className="px-3 py-2.5 font-medium">
                Ответ
              </th>
              <th scope="col" className="px-3 py-2.5 font-medium">
                На экране
              </th>
              <th scope="col" className="py-2.5 pr-6 pl-3 font-medium">
                События
              </th>
            </tr>
          </thead>
          <tbody>
            {r.answers.map((a) => {
              const events = byItem.get(a.seq) ?? [];
              return (
                <tr key={a.seq} className={cn("border-t border-line align-top", events.some((e) => e.kind !== "camera_lost" && e.kind !== "recording_failed") && "bg-warning-50/40")}>
                  <td className="tabular py-3 pr-3 pl-6 font-mono text-ink-subtle">{a.seq + 1}</td>
                  <td className="px-3 py-3">
                    <p className="text-ink">{a.title}</p>
                    <p className="text-xs text-ink-subtle">
                      {a.topic_ru ?? ""} · уровень {a.level}
                    </p>
                  </td>
                  <td className="tabular px-3 py-3 font-mono text-ink">{lz(a.difficulty)}</td>
                  <td className="px-3 py-3">{a.correct ? <Badge tone="success">верно</Badge> : <Badge tone="danger">{a.timed_out ? "время вышло" : "неверно"}</Badge>}</td>
                  <td className="tabular px-3 py-3 font-mono text-xs text-ink-muted">
                    {clock(new Date(a.shown_at).getTime() - start)}–{clock(new Date(a.answered_at).getTime() - start)}
                    <span className="block text-ink-subtle">{seconds(a.elapsed_ms)}</span>
                  </td>
                  <td className="py-3 pr-6 pl-3">
                    {events.length ? (
                      <div className="flex flex-wrap gap-1.5">
                        {events.map((e, i) => (
                          <EventChip key={i} e={e} />
                        ))}
                      </div>
                    ) : (
                      <span className="text-ink-faint">—</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <ol className="md:hidden">
        {r.answers.map((a) => {
          const events = byItem.get(a.seq) ?? [];
          return (
            <li key={a.seq} className="border-t border-line px-4 py-3">
              <div className="flex items-start justify-between gap-3">
                <p className="min-w-0 text-sm text-ink">
                  <span className="tabular mr-1.5 font-mono text-ink-subtle">{a.seq + 1}.</span>
                  {a.title}
                </p>
                {a.correct ? <Badge tone="success">верно</Badge> : <Badge tone="danger">{a.timed_out ? "время вышло" : "неверно"}</Badge>}
              </div>
              <p className="mt-1 text-xs text-ink-subtle">
                сложность {lz(a.difficulty)} · {clock(new Date(a.shown_at).getTime() - start)}–{clock(new Date(a.answered_at).getTime() - start)}
              </p>
              {events.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1.5">
                  {events.map((e, i) => (
                    <EventChip key={i} e={e} />
                  ))}
                </div>
              )}
            </li>
          );
        })}
      </ol>
      {!r.answers.length && <p className="border-t border-line px-6 py-4 text-sm text-ink-subtle">Ответов нет.</p>}
    </Card>
  );
}

function Decision({ r }: { r: SessionReview }) {
  const toast = useToast();
  const decide = useDecide(r.session_id);
  const [choice, setChoice] = useState<ReviewDecision | null>(null);
  const [comment, setComment] = useState("");
  const [confirm, setConfirm] = useState(false);
  const [touched, setTouched] = useState(false);

  if (r.decision) {
    const d = DECISION_DONE[r.decision.decision];
    return (
      <Card>
        <CardHeader title="Решение принято" description={formatDateTime(r.decision.decided_at)} action={<Badge tone={d.tone}>{d.text}</Badge>} />
        <p className="text-sm font-medium text-ink">{DECISION_LABEL[r.decision.decision]}</p>
        <blockquote className="mt-2 rounded-control border-l-2 border-line-strong bg-canvas px-3 py-2 text-sm whitespace-pre-line text-ink">{r.decision.comment}</blockquote>
        <p className="mt-3 text-xs text-ink-subtle">Кандидат видит решение и комментарий в истории тестов. Решение окончательное.</p>
      </Card>
    );
  }
  if (!r.allowed.length) {
    return (
      <Card>
        <CardHeader title="Решение" />
        <p className="text-sm text-ink-muted">Решение принимается по завершённому тесту.</p>
      </Card>
    );
  }
  const trimmed = comment.trim();
  const commentError = touched && !trimmed ? "Напишите, на чём основано решение" : undefined;
  const submit = () => {
    if (!choice || !trimmed) {
      setTouched(true);
      return;
    }
    decide.mutate(
      { decision: choice, comment: trimmed },
      {
        onSuccess: () => {
          setConfirm(false);
          toast.success("Решение сохранено", "Кандидат увидит его в истории тестов");
        },
        onError: () => setConfirm(false),
      },
    );
  };
  return (
    <Card>
      <CardHeader title="Решение" description="Посмотрите ответы, ленту событий и запись, затем выберите одно из решений." />
      <fieldset>
        <legend className="sr-only">Решение по тесту</legend>
        <div className="space-y-2">
          {r.allowed.map((d) => (
            <label
              key={d}
              className={cn(
                "flex cursor-pointer gap-3 rounded-control border p-3 transition-colors focus-within:ring-2 focus-within:ring-primary-200",
                choice === d ? "border-primary bg-primary-50/50" : "border-line hover:border-line-strong",
              )}
            >
              <input type="radio" name="decision" value={d} checked={choice === d} onChange={() => setChoice(d)} className="mt-1 size-4 shrink-0 accent-[var(--color-primary)]" />
              <span className="min-w-0">
                <span className="block text-sm font-medium text-ink">{DECISION_LABEL[d]}</span>
                <span className="mt-0.5 block text-xs leading-relaxed text-ink-muted">{DECISION_EFFECT[d]}</span>
              </span>
            </label>
          ))}
        </div>
      </fieldset>
      <Field label="Комментарий" hint="Его прочитает кандидат. Объясните, что вы увидели; персональные данные не пишите." error={commentError} required className="mt-4">
        <Textarea value={comment} maxLength={COMMENT_MAX} rows={5} onChange={(e) => setComment(e.target.value)} onBlur={() => setTouched(true)} />
      </Field>
      <p className="tabular mt-1 text-right text-xs text-ink-subtle">
        {comment.length}/{COMMENT_MAX}
      </p>
      {decide.isError && (
        <Alert tone="danger" title={isApiError(decide.error) ? decide.error.title : "Не удалось сохранить решение"} className="mt-3">
          {isApiError(decide.error) ? decide.error.detail : null}
        </Alert>
      )}
      <Button
        className="mt-4 w-full"
        variant={choice === "annul" ? "danger" : "primary"}
        disabled={!choice}
        loading={decide.isPending && !confirm}
        onClick={() => {
          if (!trimmed) {
            setTouched(true);
            return;
          }
          if (choice === "confirm") submit();
          else setConfirm(true);
        }}
      >
        {choice ? DECISION_LABEL[choice] : "Выберите решение"}
      </Button>
      <Dialog
        open={confirm}
        onClose={() => setConfirm(false)}
        size="sm"
        title={choice === "retest" ? "Назначить пересдачу?" : "Аннулировать результат?"}
        description={`Тест ${r.candidate_code}, ${r.category_name}. Решение окончательное.`}
        footer={
          <>
            <Button variant="secondary" onClick={() => setConfirm(false)}>
              Отмена
            </Button>
            <Button variant={choice === "annul" ? "danger" : "primary"} loading={decide.isPending} onClick={submit}>
              {choice ? DECISION_LABEL[choice] : ""}
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">{choice ? DECISION_EFFECT[choice] : ""}</p>
      </Dialog>
    </Card>
  );
}

export function SessionReviewView({ id }: { id: string }) {
  const q = useSessionReview(id);
  if (q.isPending) return <PageSkeleton cards={3} />;
  if (q.isError)
    return (
      <div>
        <PageHeader back={{ href: "/admin/proctoring", label: "Прокторинг" }} title="Тест не найден" />
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      </div>
    );
  const r = q.data;
  return (
    <div>
      <PageHeader
        back={{ href: "/admin/proctoring", label: "Прокторинг" }}
        eyebrow={<CategoryBadge category={r.category} size="sm" long />}
        title={`Тест ${r.candidate_code}`}
        description={`${ATTEMPT_KIND_LABEL[r.attempt_kind]} · начат ${formatDateTime(r.started_at)}${r.finished_at ? ` · завершён ${formatDateTime(r.finished_at)}` : ""} · заданий: ${r.answers.length}`}
        actions={
          r.decision ? (
            <Badge tone={DECISION_DONE[r.decision.decision].tone}>{DECISION_DONE[r.decision.decision].text}</Badge>
          ) : r.flagged ? (
            <Badge tone="warning" icon={<ShieldAlert className="size-3" aria-hidden />}>
              Требует подтверждения
            </Badge>
          ) : null
        }
      />
      <div className="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_22rem]">
        <div className="min-w-0 space-y-6">
          <Summary r={r} />
          <PersonFit r={r} />
          <Recording r={r} />
        </div>
        <div className="min-w-0 lg:col-span-2 lg:row-start-2">
          <Timeline r={r} />
        </div>
        {/* A sticky grid item is bounded by the whole grid, not its row, so it would slide over the full-width
            timeline; this cell stretches to row 1 and keeps the sticky aside inside it. */}
        <div className="lg:col-start-2 lg:row-start-1 lg:self-stretch">
          <aside className="space-y-4 lg:sticky lg:top-20">
            <Decision r={r} />
            <p className="flex gap-2 px-1 text-xs leading-relaxed text-ink-subtle">
              <EyeOff className="mt-0.5 size-3.5 shrink-0" aria-hidden />
              Имени и контактов кандидата здесь нет: чтобы решить, сам ли он проходил тест, хватает ответов, событий и записи. Это и есть минимум данных для разбора спора.
            </p>
            {r.proctoring && (
              <p className="flex gap-2 px-1 text-xs leading-relaxed text-ink-subtle">
                <Clock className="mt-0.5 size-3.5 shrink-0" aria-hidden />
                Событий за тест: {r.events.length}. Время событий — по часам браузера кандидата, сервер прижимает его к интервалу теста.
              </p>
            )}
            {r.answers.some((a) => a.timed_out) && (
              <p className="flex gap-2 px-1 text-xs leading-relaxed text-ink-subtle">
                <Timer className="mt-0.5 size-3.5 shrink-0" aria-hidden />
                «Время вышло» засчитывается как неверный ответ.
              </p>
            )}
          </aside>
        </div>
      </div>
    </div>
  );
}
