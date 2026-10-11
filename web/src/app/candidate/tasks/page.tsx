"use client";

import { Building2, CalendarClock, ListChecks, Star } from "lucide-react";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { MyTask } from "@/api/types";
import { TaskStatusBadge } from "@/components/domain/StatusBadges";
import { Badge, Button, ButtonLink, Card, CardHeader, Chip, EmptyState, ErrorAlert, Field, Markdown, PageHeader, SkeletonCard, Stat, Textarea, useToast } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useMyTasks, useSkipTask, useSubmitTask } from "@/features/candidate/hooks";
import { cn } from "@/lib/cn";
import { formatDate, formatRelative, plural } from "@/lib/format";
import { TASK_KIND_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

const MIN_ANSWER = 20;

function Rating({ value }: { value: number }) {
  return (
    <span className="inline-flex items-center gap-0.5" role="img" aria-label={`Оценка ${value} из 5`}>
      {[1, 2, 3, 4, 5].map((n) => (
        <Star key={n} className={cn("size-4", n <= value ? "fill-primary text-primary" : "text-line-strong")} aria-hidden />
      ))}
    </span>
  );
}

function OpenTask({ task }: { task: MyTask }) {
  const toast = useToast();
  const submit = useSubmitTask();
  const skip = useSkipTask();
  const [answer, setAnswer] = useState("");
  const [error, setError] = useState<string | null>(null);
  return (
    <Card padding="lg" className="border-primary-200">
      <div className="mb-4 flex flex-wrap items-center gap-2">
        <TaskStatusBadge status={task.status} />
        <Badge tone="neutral">{TASK_KIND_LABEL[task.kind]}</Badge>
        <span className="inline-flex items-center gap-1 text-sm text-ink-muted">
          <CalendarClock className="size-4" aria-hidden />
          до {formatDate(task.due_at)} ({formatRelative(task.due_at)})
        </span>
      </div>
      <h2 className="text-xl font-semibold tracking-tight text-ink">{task.title}</h2>
      {task.company_name && (
        <p className="mt-1 inline-flex items-center gap-1.5 text-sm text-ink-muted">
          <Building2 className="size-4" aria-hidden />
          {task.company_name}
        </p>
      )}
      <Markdown source={task.body} className="mt-4" />
      {task.skills.length > 0 && (
        <div className="mt-4 flex flex-wrap gap-1.5">
          {task.skills.map((s) => (
            <Chip key={s}>{skillName(s)}</Chip>
          ))}
        </div>
      )}
      <form
        noValidate
        className="mt-6 space-y-4 border-t border-line pt-6"
        onSubmit={(e) => {
          e.preventDefault();
          if (answer.trim().length < MIN_ANSWER) {
            setError(answer.trim() ? `Ответ слишком короткий: допишите хотя бы ${MIN_ANSWER} символов — идею и пару шагов` : "Напишите ответ — хотя бы пару предложений");
            return;
          }
          submit.mutate(
            { id: task.id, answer },
            {
              onSuccess: () => toast.success("Ответ отправлен", "Профиль отмечен как актуальный"),
              onError: (err) => setError(isApiError(err) ? (err.fieldError("answer") ?? err.title) : "Не удалось отправить"),
            },
          );
        }}
      >
        <Field
          label={task.kind === "approach" ? "Ваш подход" : "Ваше решение"}
          error={error ?? undefined}
          hint={`${task.kind === "approach" ? "Опишите идею, шаги и компромиссы — кода не нужно" : "Код или текст. Достаточно ключевой части"} · ${answer.trim().length} ${plural(answer.trim().length, ["символ", "символа", "символов"])}`}
        >
          <Textarea
            rows={8}
            value={answer}
            onChange={(e) => {
              setAnswer(e.target.value);
              setError(null);
            }}
            className="font-mono text-[13px]"
          />
        </Field>
        <div className="flex flex-wrap justify-end gap-2">
          <Button
            variant="ghost"
            loading={skip.isPending}
            onClick={() => skip.mutate(task.id, { onSuccess: () => toast.info("Задание пропущено", "Следующее придёт через неделю") })}
          >
            Пропустить
          </Button>
          <Button type="submit" loading={submit.isPending}>
            Отправить
          </Button>
        </div>
      </form>
    </Card>
  );
}

function PastTask({ task }: { task: MyTask }) {
  return (
    <li className="rounded-card border border-line bg-white p-5">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <p className="font-medium text-ink">{task.title}</p>
          <p className="mt-0.5 text-sm text-ink-subtle">
            {task.company_name ?? "Компания"} · {TASK_KIND_LABEL[task.kind].toLowerCase()} · {formatDate(task.submitted_at ?? task.assigned_at)}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          {typeof task.rating === "number" && <Rating value={task.rating} />}
          <TaskStatusBadge status={task.status} />
        </div>
      </div>
      {task.review_comment && <p className="mt-3 rounded-control bg-canvas p-3 text-sm text-ink">«{task.review_comment}»</p>}
    </li>
  );
}

const HOW = [
  { title: "Раз в неделю", text: "Компании вашей категории готовят короткие задачи на 15–30 минут. Новое задание приходит, только когда нет открытого." },
  { title: "Неделя на ответ", text: "Опишите подход или пришлите решение. Не до того — пропустите, на категорию это не влияет." },
  { title: "Оценка и отзыв", text: "Компания ставит оценку и пишет пару слов. Ответ отмечает профиль актуальным и поднимает его в подборках." },
];

function HowItWorks() {
  return (
    <Card padding="sm" className="bg-canvas/60 p-5">
      <h2 className="eyebrow text-ink-subtle">Как это устроено</h2>
      <ol className="mt-4 space-y-4">
        {HOW.map((h, i) => (
          <li key={h.title} className="flex gap-3">
            <span aria-hidden className="tabular mt-px flex size-5 shrink-0 items-center justify-center rounded-full bg-primary-50 font-mono text-xs font-semibold text-primary-strong">
              {i + 1}
            </span>
            <span className="min-w-0 text-sm">
              <span className="block font-medium text-ink">{h.title}</span>
              <span className="mt-0.5 block text-ink-muted">{h.text}</span>
            </span>
          </li>
        ))}
      </ol>
    </Card>
  );
}

function Progress({ past }: { past: MyTask[] }) {
  const done = past.filter((t) => t.status === "submitted" || t.status === "reviewed").length;
  const rated = past.filter((t) => typeof t.rating === "number");
  const avg = rated.length ? rated.reduce((s, t) => s + (t.rating ?? 0), 0) / rated.length : null;
  return (
    <Card padding="sm" className="p-5">
      <h2 className="eyebrow text-ink-subtle">Ваши ответы</h2>
      <div className="mt-4 grid grid-cols-2 gap-4">
        <Stat label="Отправлено" value={done} />
        <Stat label="Средняя оценка" value={avg === null ? "—" : avg.toFixed(1).replace(".", ",")} hint={rated.length ? `по ${rated.length} ${plural(rated.length, ["оценке", "оценкам", "оценкам"])}` : "оценок пока нет"} />
      </div>
    </Card>
  );
}

export default function TasksPage() {
  const q = useMyTasks();
  const hasCategory = Boolean(useMe().data?.candidate?.category);
  const open = q.data?.filter((t) => t.status === "assigned") ?? [];
  const past = q.data?.filter((t) => t.status !== "assigned") ?? [];
  return (
    <div className="max-w-6xl">
      <PageHeader eyebrow="Работа" title="Регулярные задания" description="Короткие задачи от компаний вашей категории — раз в неделю. Ответ поддерживает актуальность профиля и поднимает его в подборках." />
      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_19rem]">
        <div className="min-w-0">
          {q.isPending ? (
            <SkeletonCard lines={4} />
          ) : q.isError ? (
            <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
          ) : (
            <div className="space-y-8">
              {open.length > 0 ? (
                open.map((t) => <OpenTask key={t.id} task={t} />)
              ) : hasCategory ? (
                <EmptyState
                  icon={<ListChecks className="size-5" aria-hidden />}
                  title="Сейчас заданий нет"
                  description="Новое задание приходит не чаще раза в неделю. Как только компания вашей категории его выпустит, оно появится здесь, а на главной загорится отметка."
                  action={
                    <ButtonLink href="/candidate/profile" variant="secondary">
                      Обновить профиль
                    </ButtonLink>
                  }
                />
              ) : (
                <EmptyState
                  icon={<ListChecks className="size-5" aria-hidden />}
                  title="Задания приходят после теста"
                  description="Компании выдают задания по категориям. Подтвердите свою тестом — и первое задание придёт в течение недели."
                  action={<ButtonLink href="/candidate/assessment">Перейти к тесту</ButtonLink>}
                />
              )}
              {past.length > 0 && (
                <section>
                  <CardHeader title="История" description="Отправленные, пропущенные и просроченные задания" as="h2" />
                  <ul className="space-y-3">
                    {past.map((t) => (
                      <PastTask key={t.id} task={t} />
                    ))}
                  </ul>
                </section>
              )}
            </div>
          )}
        </div>
        <aside aria-label="О регулярных заданиях" className="min-w-0 space-y-4 lg:sticky lg:top-20 lg:self-start">
          {past.length > 0 && <Progress past={past} />}
          <HowItWorks />
        </aside>
      </div>
    </div>
  );
}
