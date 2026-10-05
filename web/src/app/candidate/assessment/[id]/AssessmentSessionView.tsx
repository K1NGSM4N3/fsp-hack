"use client";

import { useQueryClient } from "@tanstack/react-query";
import { ArrowRight, CheckCircle2, CircleSlash, Clock, LogOut, ShieldAlert, Timer, TrendingDown, TrendingUp } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useRef, useState, type FormEvent } from "react";
import { isApiError } from "@/api/client";
import type { Answer, AssessmentSession, CurrentItem, Offer } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { Alert, Badge, Button, ButtonLink, Card, CardHeader, CodeBlock, ErrorAlert, Input, Markdown, PageHeader, PageSkeleton, ProgressBar, Spinner, useToast } from "@/components/ui";
import { useAnswerItem, useAssessment, useStartAssessment } from "@/features/candidate/hooks";
import { qk } from "@/features/keys";
import { cn } from "@/lib/cn";
import { formatDate, formatDuration } from "@/lib/format";
import { ATTEMPT_KIND_HINT, ATTEMPT_KIND_LABEL, GRADE_LABEL, TOPIC_LABEL } from "@/lib/labels";

/** The API advances a timed-out item on the next read after this grace period. */
const GRACE_MS = 5500;


const SCALE_MIN = -2.5;
const SCALE_MAX = 2.5;
const scalePos = (v: number) => ((Math.max(SCALE_MIN, Math.min(SCALE_MAX, v)) - SCALE_MIN) / (SCALE_MAX - SCALE_MIN)) * 100;
const SCALE_ZONES = [
  { label: "ниже Junior", from: SCALE_MIN, to: -1 },
  { label: "Junior", from: -1, to: 0 },
  { label: "Middle", from: 0, to: 1 },
  { label: "Senior", from: 1, to: SCALE_MAX },
];

/** Grade bands use the same thresholds as the assessment engine: Junior from −1, Middle from 0, Senior from +1. */
function GradeScale({ level, spread }: { level: number; spread: number }) {
  return (
    <div className="relative mt-3 mb-1" role="img" aria-label="Положение результата на шкале грейдов">
      <div className="flex h-9 overflow-hidden rounded-control border border-line text-[11px]">
        {SCALE_ZONES.map((z) => (
          <span key={z.label} className="flex items-end justify-center border-r border-line bg-canvas pb-1 text-ink-subtle last:border-r-0" style={{ width: `${scalePos(z.to) - scalePos(z.from)}%` }}>
            {z.label}
          </span>
        ))}
      </div>
      <span className="absolute top-0 h-9 rounded-control bg-primary/15 ring-1 ring-primary/40" style={{ left: `${scalePos(level - spread)}%`, width: `${scalePos(level + spread) - scalePos(level - spread)}%` }} />
      <span className="absolute -top-1 h-11 w-0.5 -translate-x-1/2 rounded-full bg-ink" style={{ left: `${scalePos(level)}%` }} />
    </div>
  );
}

function useNow(intervalMs = 250) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), intervalMs);
    return () => clearInterval(t);
  }, [intervalMs]);
  return now;
}

function CountdownBadge({ seconds, total }: { seconds: number; total: number }) {
  const low = seconds <= 15;
  return (
    <div className="flex items-center gap-3" aria-live={low ? "assertive" : "off"}>
      <span
        className={cn("tabular inline-flex h-9 items-center gap-2 rounded-control border px-3 font-mono text-base font-semibold", low ? "border-accent-100 bg-accent-50 text-accent-strong" : "border-line bg-white text-ink")}
        aria-label={`Осталось ${Math.ceil(seconds)} секунд`}
      >
        <Timer className="size-4" aria-hidden />
        {formatDuration(seconds)}
      </span>
      <ProgressBar value={seconds} max={total} size="xs" tone={low ? "danger" : "primary"} className="hidden w-24 sm:block" label="Время на задание" />
    </div>
  );
}

function ItemForm({ current, sessionId }: { current: CurrentItem; sessionId: string }) {
  const qc = useQueryClient();
  const answer = useAnswerItem(sessionId);
  const toast = useToast();
  const now = useNow();
  const item = current.item;
  const deadline = new Date(current.deadline).getTime();
  const seconds = Math.max(0, (deadline - now) / 1000);
  const expired = now > deadline;
  const [choice, setChoice] = useState<number | null>(null);
  const [multi, setMulti] = useState<number[]>([]);
  const [text, setText] = useState("");
  const sent = useRef(false);
  const refreshed = useRef(false);

  useEffect(() => {
    if (!expired || sent.current || refreshed.current) return;
    if (now > deadline + GRACE_MS) {
      refreshed.current = true;
      toast.info("Время на задание вышло", "Ответ не засчитан, переходим к следующему");
      void qc.invalidateQueries({ queryKey: qk.assessment(sessionId) });
    }
  }, [expired, now, deadline, qc, sessionId, toast]);

  useEffect(() => {
    if (item.kind !== "single_choice") return;
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      const n = Number(e.key);
      if (n >= 1 && n <= item.options.length) setChoice(n - 1);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [item.kind, item.options.length]);

  const build = (): Answer | null => {
    if (item.kind === "single_choice") return choice === null ? null : { type: "choice", value: choice };
    if (item.kind === "multi_choice") return multi.length ? { type: "multi_choice", value: [...multi].sort((a, b) => a - b) } : null;
    if (item.kind === "number") {
      const n = Number(text.trim().replace(",", "."));
      return text.trim() && Number.isFinite(n) ? { type: "number", value: n } : null;
    }
    return text.trim() ? { type: "text", value: text.trim() } : null;
  };

  const value = build();
  const onSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!value || sent.current) return;
    sent.current = true;
    answer.mutate(
      { seq: current.seq, answer: value },
      {
        onError: (err) => {
          sent.current = false;
          if (isApiError(err) && (err.code === "stale_item" || err.code === "assessment_finished")) void qc.invalidateQueries({ queryKey: qk.assessment(sessionId) });
          else toast.error(isApiError(err) ? err.title : "Не удалось отправить ответ");
        },
      },
    );
  };

  return (
    <form noValidate onSubmit={onSubmit}>
      <Card padding="lg">
        <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone="primary">{TOPIC_LABEL[item.topic]}</Badge>
            <span className="text-sm text-ink-subtle">{item.title}</span>
          </div>
          <CountdownBadge seconds={seconds} total={item.time_limit_secs} />
        </div>

        <Markdown source={item.prompt} className="font-medium" />
        {item.code && <CodeBlock code={item.code.source} language={item.code.language} className="mt-4" />}

        <div className="mt-6">
          {item.kind === "single_choice" && (
            <fieldset>
              <legend className="sr-only">Варианты ответа</legend>
              <div className="grid gap-2">
                {item.options.map((opt, i) => (
                  <label
                    key={i}
                    className={cn(
                      "flex cursor-pointer items-center gap-3 rounded-control border px-4 py-3 transition-colors",
                      choice === i ? "border-primary bg-primary-50 ring-1 ring-primary" : "border-line-strong bg-white hover:border-primary",
                    )}
                  >
                    <input type="radio" name={`opt-${current.seq}`} checked={choice === i} onChange={() => setChoice(i)} className="peer sr-only" />
                    <span
                      className={cn(
                        "flex size-7 shrink-0 items-center justify-center rounded-md border font-mono text-sm peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-primary",
                        choice === i ? "border-primary bg-primary text-white" : "border-line-strong text-ink-subtle",
                      )}
                    >
                      {i + 1}
                    </span>
                    <span className="font-mono text-sm text-ink">{opt}</span>
                  </label>
                ))}
              </div>
              <p className="mt-2 hidden text-xs text-ink-subtle sm:block">Можно выбирать клавишами 1–{item.options.length}, Enter — ответить</p>
            </fieldset>
          )}
          {item.kind === "multi_choice" && (
            <fieldset>
              <legend className="mb-2 text-sm text-ink-muted">Выберите все верные варианты</legend>
              <div className="grid gap-2">
                {item.options.map((opt, i) => (
                  <label key={i} className={cn("flex cursor-pointer items-center gap-3 rounded-control border px-4 py-3", multi.includes(i) ? "border-primary bg-primary-50" : "border-line-strong")}>
                    <input type="checkbox" checked={multi.includes(i)} onChange={() => setMulti(multi.includes(i) ? multi.filter((x) => x !== i) : [...multi, i])} className="size-4 accent-primary" />
                    <span className="font-mono text-sm text-ink">{opt}</span>
                  </label>
                ))}
              </div>
            </fieldset>
          )}
          {(item.kind === "number" || item.kind === "text") && (
            <div className="max-w-sm">
              <label htmlFor={`ans-${current.seq}`} className="mb-1.5 block text-sm font-medium text-ink">
                Ваш ответ
              </label>
              <Input
                id={`ans-${current.seq}`}
                autoFocus
                autoComplete="off"
                inputMode={item.kind === "number" ? "decimal" : "text"}
                value={text}
                onChange={(e) => setText(e.target.value)}
                placeholder="Ответ"
                className="h-12 font-mono text-base"
              />
              {item.input_hint && <p className="mt-1.5 text-xs text-ink-subtle">{item.input_hint}</p>}
            </div>
          )}
        </div>

        <div className="mt-8 flex flex-wrap items-center justify-between gap-3 border-t border-line pt-5">
          <span className="text-sm text-ink-subtle">{expired ? "Время вышло — засчитываем пропуск…" : "Ответ нельзя изменить после отправки"}</span>
          <Button type="submit" size="lg" disabled={!value || expired} loading={answer.isPending}>
            Ответить
            <ArrowRight className="size-4" aria-hidden />
          </Button>
        </div>
      </Card>
    </form>
  );
}

function Runner({ session, refreshing }: { session: AssessmentSession; refreshing: boolean }) {
  return (
    <div className="mx-auto max-w-3xl">
      <div className="mb-6 flex flex-wrap items-center justify-between gap-4">
        <div>
          <p className="flex flex-wrap items-center gap-2 text-sm text-ink-muted">
            <CategoryBadge category={{ specialization: session.specialization, grade: session.claimed_grade }} size="sm" />
            {ATTEMPT_KIND_LABEL[session.attempt_kind].toLowerCase()} на грейд {GRADE_LABEL[session.claimed_grade]}
          </p>
          <h1 className="mt-2 text-[28px] leading-tight font-semibold tracking-[-0.03em] text-ink">Задание {session.answered + 1}</h1>
        </div>
        <Link href="/candidate/assessment" className="inline-flex items-center gap-1.5 rounded text-sm text-ink-muted hover:text-ink">
          <LogOut className="size-4" aria-hidden />
          Выйти
        </Link>
      </div>
      <div className="mb-6">
        <ProgressBar value={session.answered} max={session.max_items} label="Прогресс теста" caption={`Отвечено: ${session.answered}`} />
        <p className="mt-1.5 text-xs text-ink-subtle">
          Обычно {session.min_items}–{session.max_items} заданий: тест закончится, как только уровень станет ясен.
        </p>
      </div>
      {session.current && !refreshing ? (
        <ItemForm key={session.current.seq} current={session.current} sessionId={session.id} />
      ) : (
        <Card padding="lg" aria-busy>
          <div className="flex items-center gap-3 text-sm text-ink-muted" role="status">
            <Spinner className="size-4" />
            Подбираем задание под ваш уровень…
          </div>
        </Card>
      )}
      <p className="mt-6 flex items-center gap-2 text-xs text-ink-subtle">
        <Clock className="size-3.5" aria-hidden />
        Можно выйти и вернуться: незавершённый тест закрывается через час без активности.
      </p>
    </div>
  );
}

function OfferCard({ offer, session }: { offer: Offer; session: AssessmentSession }) {
  const router = useRouter();
  const start = useStartAssessment();
  const toast = useToast();
  const up = offer.kind === "placement_upgrade" || offer.kind === "upgrade";
  return (
    <Card className={up ? "border-primary-200 bg-primary-50/40" : ""}>
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-start gap-3">
          {up ? <TrendingUp className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden /> : <TrendingDown className="mt-0.5 size-5 shrink-0 text-ink-subtle" aria-hidden />}
          <div>
            <p className="font-medium text-ink">{offer.label}</p>
            <p className="mt-0.5 text-sm text-ink-muted">{ATTEMPT_KIND_HINT[offer.kind]}</p>
          </div>
        </div>
        <Button
          loading={start.isPending}
          variant={up ? "primary" : "secondary"}
          onClick={() =>
            start.mutate(
              { specialization: session.specialization, grade: offer.grade },
              {
                onSuccess: (s) => router.push(`/candidate/assessment/${s.id}`),
                onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось начать тест", isApiError(e) ? (e.detail ?? undefined) : undefined),
              },
            )
          }
        >
          Начать: {GRADE_LABEL[offer.grade]}
        </Button>
      </div>
    </Card>
  );
}

function Result({ session }: { session: AssessmentSession }) {
  const r = session.result;
  const category = { specialization: session.specialization, grade: session.claimed_grade };

  if (session.status === "expired" || !r) {
    return (
      <div className="mx-auto max-w-3xl">
        <PageHeader back={{ href: "/candidate/assessment", label: "Тестирование" }} title="Тест прерван" description="Сессия закрылась после часа без активности. Результат не засчитан." />
        <ButtonLink href="/candidate/assessment">Начать заново</ButtonLink>
      </div>
    );
  }

  const d = r.decision;
  const strength = d.passed && typeof r.test_strength === "number" ? Math.round(r.test_strength * 100) : null;
  const offers = r.offers.filter((o) => o.kind !== "placement");

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <PageHeader
        back={{ href: "/candidate/assessment", label: "Тестирование" }}
        title="Результат теста"
        description={`${ATTEMPT_KIND_LABEL[session.attempt_kind]} · завершён ${formatDate(session.finished_at)} · заданий: ${d.items_administered}`}
        className="mb-0"
      />

      <Card padding="lg" className={d.passed ? "border-success-200" : ""}>
        <div className="flex items-start gap-4">
          {d.passed ? <CheckCircle2 className="size-10 shrink-0 text-success" aria-hidden /> : <CircleSlash className="size-10 shrink-0 text-ink-faint" aria-hidden />}
          <div className="min-w-0">
            <h2 className="text-xl font-semibold tracking-tight text-ink">{d.passed ? "Грейд подтверждён" : `Грейд ${GRADE_LABEL[session.claimed_grade]} пока не подтверждён`}</h2>
            <div className="mt-2 flex flex-wrap items-center gap-3">
              <CategoryBadge category={category} long />
              {strength !== null && <span className="tabular font-mono text-sm text-ink">сила результата {strength}/100</span>}
            </div>
            <p className="mt-3 text-sm text-ink-muted">
              {r.assigned_grade
                ? "Категория присвоена — работодатели уже могут найти вас в поиске."
                : d.grade_measured
                  ? `По результату уровень соответствует ${GRADE_LABEL[d.grade_measured]}. Неудачная попытка не понижает уже присвоенный грейд.`
                  : "Попробуйте пройти тест позже — история попыток сохранена."}
            </p>
          </div>
        </div>
        <div className="mt-6 grid gap-5 sm:grid-cols-2">
          <div>
            <p className="text-xs text-ink-subtle">Где вы на шкале грейдов</p>
            <GradeScale level={d.theta} spread={1.96 * d.se} />
            <p className="mt-2 text-xs text-ink-subtle">Отметка показывает ваш результат, полоса — диапазон, в котором он находится почти наверняка.</p>
          </div>
          <div>
            <p className="text-xs text-ink-subtle">Уровень по тесту</p>
            <p className="font-mono text-2xl font-semibold text-ink">{d.grade_measured ? GRADE_LABEL[d.grade_measured] : "ниже Junior"}</p>
            {d.confident_above_next && <p className="mt-2 text-xs text-primary-strong">Уверенно выше порога следующего грейда</p>}
          </div>
        </div>
      </Card>

      {r.needs_review && (
        <Alert tone="warning" title="Результат отправлен на перепроверку" icon={<ShieldAlert className="size-5 text-warning" aria-hidden />}>
          Ответы выглядят нетипично: верные на сложных заданиях и неверные на простых. Мы можем попросить подтвердить результат.
        </Alert>
      )}

      {offers.map((o) => (
        <OfferCard key={`${o.kind}-${o.grade}`} offer={o} session={session} />
      ))}

      {r.topics.length > 0 && (
        <Card>
          <CardHeader title="По темам" description="Без разбора ответов: задания не раскрываются, чтобы тест оставался честным" />
          <ul className="space-y-3">
            {r.topics.map((t) => (
              <li key={t.topic} className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1">
                <span className="text-sm text-ink">{t.topic_ru || TOPIC_LABEL[t.topic]}</span>
                <span className="tabular font-mono text-sm text-ink-muted">
                  {t.correct}/{t.answered}
                </span>
                <ProgressBar value={t.correct} max={Math.max(1, t.answered)} size="xs" className="col-span-2" label={t.topic_ru || TOPIC_LABEL[t.topic]} />
              </li>
            ))}
          </ul>
        </Card>
      )}

      <div className="flex flex-wrap gap-3">
        <ButtonLink href="/candidate">В кабинет</ButtonLink>
        <ButtonLink href="/candidate/assessment" variant="secondary">
          История тестов
        </ButtonLink>
      </div>
    </div>
  );
}

export function AssessmentSessionView({ id }: { id: string }) {
  const session = useAssessment(id);
  if (session.isPending) return <PageSkeleton cards={1} />;
  if (session.isError) {
    return (
      <div>
        <PageHeader back={{ href: "/candidate/assessment", label: "Тестирование" }} title="Тест не найден" />
        <ErrorAlert error={session.error} onRetry={() => void session.refetch()} />
      </div>
    );
  }
  return session.data.status === "in_progress" ? <Runner session={session.data} refreshing={session.isFetching} /> : <Result session={session.data} />;
}
