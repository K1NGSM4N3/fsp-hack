"use client";

import { ArrowRight, CalendarClock, Clock, Lock, PlayCircle, RotateCcw, Shuffle, TrendingUp, Video } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { isApiError } from "@/api/client";
import type { AttemptKind, Grade } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { AssessmentStatusBadge } from "@/components/domain/StatusBadges";
import { Alert, Badge, Button, ButtonLink, Card, CardHeader, EmptyState, ErrorAlert, FieldSet, PageHeader, PageSkeleton, RadioCards, SkeletonCard } from "@/components/ui";
import { useAssessments, useCategory, useStartAssessment } from "@/features/candidate/hooks";
import { OnboardingStrip } from "@/features/candidate/components/Onboarding";
import { formatDate, formatDateTime } from "@/lib/format";
import { ATTEMPT_KIND_HINT, ATTEMPT_KIND_LABEL, GRADE_HINT, GRADE_LABEL, GRADES, LOCK_REASON_HINT, SPEC_LONG_LABEL } from "@/lib/labels";
import { ProctoringMark, useProctoringOptions } from "./_proctoring/ui";

function Rules() {
  const options = useProctoringOptions();
  const items = [
    { icon: <Shuffle className="size-4" aria-hidden />, text: "У каждого соискателя свой вариант заданий одинаковой сложности, слитые ответы не помогут" },
    { icon: <Clock className="size-4" aria-hidden />, text: "У каждого задания свой таймер; если время вышло — ответ не засчитывается" },
    { icon: <TrendingUp className="size-4" aria-hidden />, text: "Тест адаптивный: следующий вопрос подбирается под ваш уровень и заканчивается, как только уровень ясен" },
    { icon: <Lock className="size-4" aria-hidden />, text: "Правильность ответов во время теста не показывается" },
    ...(options.data?.mode === "optional"
      ? [{ icon: <Video className="size-4" aria-hidden />, text: "Видеозапись — по желанию: её видите вы, а компании — только отметку «с видеозаписью» или «без»" }]
      : []),
  ];
  return (
    <Card>
      <CardHeader title="Как устроен тест" />
      <ul className="space-y-3 text-sm">
        {items.map((i) => (
          <li key={i.text} className="flex gap-3">
            <span className="mt-0.5 text-primary">{i.icon}</span>
            <span className="text-ink-muted">{i.text}</span>
          </li>
        ))}
      </ul>
      <div className="mt-5 rounded-control bg-canvas p-4 text-sm">
        <p className="font-medium text-ink">Правила смены грейда</p>
        <ul className="mt-2 list-disc space-y-1 pl-5 text-ink-muted">
          <li>Не прошли при первом размещении — сразу можно пройти тест на грейд ниже.</li>
          <li>Тот же грейд после неудачной попытки — через 30 дней. Брошенный тест считается неудачной попыткой.</li>
          <li>Прошли уверенно — предложим дотест на грейд выше, один раз. Если этот грейд вы не прошли меньше 30 дней назад, дотест откроется, когда пройдут 30 дней.</li>
          <li>После присвоения грейд можно менять не чаще раза в 90 дней.</li>
          <li>Неудачная попытка повышения не понижает текущий грейд; следующая — через 30 дней, но не раньше конца 90-дневного окна.</li>
        </ul>
      </div>
    </Card>
  );
}

function StartCard() {
  const router = useRouter();
  const preselect = useSearchParams().get("grade") as Grade | null;
  const category = useCategory();
  const start = useStartAssessment();
  const options = useProctoringOptions();
  const [picked, setPicked] = useState<Grade | null>(null);
  const [error, setError] = useState<{ title: string; detail: string | null } | null>(null);

  if (category.isPending) return <SkeletonCard lines={5} />;
  if (category.isError) return <ErrorAlert error={category.error} onRetry={() => void category.refetch()} />;
  const c = category.data;

  if (!c.specialization) {
    return <EmptyState title="Сначала пройдите опрос" description="Опрос определяет специализацию, по которой будет тест." action={<ButtonLink href="/candidate/survey">Перейти к опросу</ButtonLink>} />;
  }

  const offerFor = (g: Grade) => c.available.find((o) => o.grade === g);
  const lockFor = (g: Grade) => c.locked.find((o) => o.grade === g);
  const nextLock = [...c.locked].sort((a, b) => a.available_at.localeCompare(b.available_at))[0];
  const allowed = GRADES.filter((g) => offerFor(g));
  const fallback = allowed.find((g) => g === preselect) ?? allowed.find((g) => g === c.claimed_grade_in_survey) ?? allowed[0];
  const grade = picked && offerFor(picked) ? picked : (fallback ?? null);
  const specialization = c.specialization;
  const note = (g: Grade) => {
    const offer = offerFor(g);
    if (offer?.kind === "placement" && g === "junior") return ATTEMPT_KIND_LABEL.placement;
    if (offer) return ATTEMPT_KIND_HINT[offer.kind];
    if (c.category?.grade === g) return "Ваш текущий грейд";
    const lock = lockFor(g);
    if (lock) return `${LOCK_REASON_HINT[lock.reason]}. Доступно с ${formatDate(lock.available_at)}`;
    return "Сейчас недоступно";
  };

  return (
    <Card padding="lg">
      <CardHeader
        title="Новый тест"
        description={
          <>
            Специализация: <span className="font-medium text-ink">{SPEC_LONG_LABEL[specialization]}</span> ·{" "}
            <Link href="/candidate/survey" className="link-inline">
              изменить в опросе
            </Link>
          </>
        }
      />
      {allowed.length === 0 ? (
        <Alert tone="info" title="Сейчас новый тест недоступен" icon={<CalendarClock className="size-5 text-primary" aria-hidden />}>
          {nextLock
            ? `${nextLock.label} — с ${formatDate(nextLock.available_at)}. ${LOCK_REASON_HINT[nextLock.reason]}.`
            : c.next_change_at
              ? `Смена грейда — не чаще раза в 90 дней. Следующая попытка — с ${formatDate(c.next_change_at)}.`
              : "Попробуйте позже."}
        </Alert>
      ) : (
        <FieldSet legend="Выберите грейд">
          <RadioCards<Grade>
            name="grade"
            columns={3}
            value={grade}
            onChange={(g) => {
              setPicked(g);
              setError(null);
            }}
            options={GRADES.map((g) => ({
              value: g,
              disabled: !offerFor(g),
              title: (
                <span className="flex items-center gap-2">
                  {GRADE_LABEL[g]}
                  {c.category?.grade === g && <span className="rounded bg-success-50 px-1.5 py-0.5 text-xs font-medium text-success">текущий</span>}
                </span>
              ),
              description: GRADE_HINT[g],
              aside: <span className={offerFor(g) ? "mt-2 block text-xs text-primary-strong" : "mt-2 block text-xs text-ink-subtle"}>{note(g)}</span>,
            }))}
          />
        </FieldSet>
      )}
      {error && (
        <Alert tone="danger" title={error.title} className="mt-5">
          {error.detail}
        </Alert>
      )}
      {allowed.length > 0 && (
        <div className="mt-6 flex flex-wrap items-center gap-4">
          <Button
            size="lg"
            icon={<PlayCircle className="size-5" aria-hidden />}
            disabled={!grade}
            loading={start.isPending}
            onClick={() => {
              if (!grade) return;
              if (options.data?.mode !== "off") {
                router.push(`/candidate/assessment/start?grade=${grade}`);
                return;
              }
              start.mutate(
                { specialization, grade },
                {
                  onSuccess: (s) => router.push(`/candidate/assessment/${s.id}`),
                  onError: (e) => setError(isApiError(e) ? { title: e.title, detail: e.detail } : { title: "Не удалось начать тест", detail: null }),
                },
              );
            }}
          >
            Начать тест{grade ? ` на ${GRADE_LABEL[grade]}` : ""}
          </Button>
          <span className="text-sm text-ink-subtle">
            Займёт 10–25 минут. Лучше пройти без перерывов.
            {options.data?.mode === "optional" ? " Перед началом предложим включить камеру — по желанию." : ""}
          </span>
        </div>
      )}
    </Card>
  );
}

function CurrentCategory() {
  const category = useCategory();
  const c = category.data;
  if (!c?.category) return null;
  const strength = typeof c.test_strength === "number" ? Math.round(c.test_strength * 100) : null;
  return (
    <Card className="mb-6">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p className="text-xs text-ink-subtle">Текущая категория</p>
          <div className="mt-1.5 flex flex-wrap items-center gap-3">
            <CategoryBadge category={c.category} long />
            {strength !== null && <span className="tabular font-mono text-sm text-ink">сила результата {strength}/100</span>}
          </div>
        </div>
        <p className="text-sm text-ink-muted">
          Присвоена {formatDate(c.assigned_at)}
          {c.next_change_at ? ` · смена грейда с ${formatDate(c.next_change_at)}` : ""}
        </p>
      </div>
    </Card>
  );
}

function HistoryCard() {
  const q = useAssessments();
  const category = useCategory();
  const assigned = category.data?.history ?? [];
  return (
    <Card padding="none">
      <div className="px-5 pt-5 sm:px-6 sm:pt-6">
        <CardHeader title="История тестов" description="Все попытки и решения сохраняются" />
      </div>
      {q.isPending ? (
        <div className="p-5">
          <SkeletonCard lines={3} className="border-0 p-0" />
        </div>
      ) : q.isError ? (
        <div className="p-5">
          <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
        </div>
      ) : q.data.length === 0 ? (
        <p className="px-6 pb-6 text-sm text-ink-subtle">Вы ещё не проходили тесты.</p>
      ) : (
        <ul className="divide-y divide-line border-t border-line">
          {q.data.map((s) => (
            <li key={s.id}>
              <Link href={`/candidate/assessment/${s.id}`} className="flex flex-col gap-2 px-5 py-4 hover:bg-canvas sm:flex-row sm:items-center sm:justify-between sm:px-6">
                <span className="flex flex-wrap items-center gap-3">
                  <CategoryBadge category={{ specialization: s.specialization, grade: s.claimed_grade }} size="sm" />
                  <AssessmentStatusBadge status={s.status} />
                  {s.proctoring && <ProctoringMark summary={s.proctoring} />}
                  {s.result?.needs_review && <Badge tone="warning">требует подтверждения</Badge>}
                  {s.review?.decision === "confirm" && <Badge tone="success">подтверждён модератором</Badge>}
                  {s.review?.decision === "annul" && <Badge tone="danger">аннулирован модератором</Badge>}
                  {s.review?.decision === "retest" && <Badge tone="warning">пересдача с видеозаписью</Badge>}
                  {s.recording_views.length > 0 && <Badge tone="muted">запись смотрел модератор</Badge>}
                  <span className="text-sm text-ink-muted">{ATTEMPT_KIND_LABEL[s.attempt_kind]}</span>
                </span>
                <span className="flex items-center gap-3 text-sm text-ink-subtle">
                  {formatDateTime(s.started_at)} · заданий: {s.answered}
                  {s.status === "in_progress" ? (
                    <span className="inline-flex items-center gap-1 font-medium text-primary">
                      Продолжить <ArrowRight className="size-4" aria-hidden />
                    </span>
                  ) : (
                    <ArrowRight className="size-4" aria-hidden />
                  )}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
      {assigned.length > 0 && (
        <div className="border-t border-line px-5 py-4 sm:px-6">
          <p className="mb-2 text-xs font-medium tracking-wide text-ink-subtle uppercase">Присвоения грейда</p>
          <ul className="space-y-1.5 text-sm">
            {assigned.map((h, i) => (
              <li key={i} className="flex flex-wrap items-center justify-between gap-2">
                <span className={h.annulled_at ? "text-ink-subtle line-through" : "text-ink"}>
                  {GRADE_LABEL[h.grade]} · {(ATTEMPT_KIND_LABEL[h.reason as AttemptKind] ?? h.reason).toLowerCase()}
                </span>
                {h.annulled_at && <Badge tone="danger">аннулировано {formatDate(h.annulled_at)}</Badge>}
                <span className="text-ink-subtle">{formatDate(h.assigned_at)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  );
}

function ActiveSessionBanner() {
  const q = useAssessments();
  const active = q.data?.find((s) => s.status === "in_progress");
  if (!active) return null;
  return (
    <Alert
      tone="warning"
      title="У вас есть незавершённый тест"
      className="mb-6"
      icon={<RotateCcw className="size-5 text-warning" aria-hidden />}
      action={<ButtonLink href={`/candidate/assessment/${active.id}`} size="sm">Продолжить тест на {GRADE_LABEL[active.claimed_grade]}</ButtonLink>}
    >
      Если не ответить в течение 30 минут после окончания времени на задание, тест закроется и засчитается как неудачная попытка.
    </Alert>
  );
}

export default function AssessmentPage() {
  return (
    <div>
      <PageHeader title="Тестирование" description="Категория присваивается только по результату теста." />
      <OnboardingStrip />
      <ActiveSessionBanner />
      <CurrentCategory />
      <div className="grid gap-6 lg:grid-cols-[1.4fr_1fr]">
        <div className="min-w-0 space-y-6">
          <Suspense fallback={<PageSkeleton cards={1} />}>
            <StartCard />
          </Suspense>
          <HistoryCard />
        </div>
        <Rules />
      </div>
    </div>
  );
}
