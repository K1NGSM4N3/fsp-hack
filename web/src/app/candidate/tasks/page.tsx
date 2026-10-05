"use client";

import { Building2, CalendarClock, ListChecks, Star } from "lucide-react";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { MyTask } from "@/api/types";
import { TaskStatusBadge } from "@/components/domain/StatusBadges";
import { Badge, Button, Card, CardHeader, Chip, EmptyState, ErrorAlert, Field, Markdown, PageHeader, SkeletonCard, Textarea, useToast } from "@/components/ui";
import { useMyTasks, useSkipTask, useSubmitTask } from "@/features/candidate/hooks";
import { cn } from "@/lib/cn";
import { formatDate, formatRelative } from "@/lib/format";
import { TASK_KIND_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

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
      <h2 className="font-mono text-xl font-semibold tracking-tight text-ink">{task.title}</h2>
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
          if (!answer.trim()) {
            setError("Напишите ответ");
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
          hint={task.kind === "approach" ? "Опишите идею, шаги и компромиссы — кода не нужно" : "Код или текст. Достаточно ключевой части"}
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

export default function TasksPage() {
  const q = useMyTasks();
  const open = q.data?.filter((t) => t.status === "assigned") ?? [];
  const past = q.data?.filter((t) => t.status !== "assigned") ?? [];
  return (
    <div>
      <PageHeader title="Регулярные задания" description="Короткие задачи от компаний вашей категории — раз в неделю. Ответ поддерживает актуальность профиля и поднимает его в подборках." />
      {q.isPending ? (
        <SkeletonCard lines={4} />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <div className="space-y-8">
          {open.length > 0 ? (
            open.map((t) => <OpenTask key={t.id} task={t} />)
          ) : (
            <EmptyState
              icon={<ListChecks className="size-5" aria-hidden />}
              title="Сейчас заданий нет"
              description="Новое задание приходит не чаще раза в неделю, если у вас есть категория. Мы уведомим, когда оно появится."
            />
          )}
          {past.length > 0 && (
            <section>
              <CardHeader title="История" as="h2" />
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
  );
}
