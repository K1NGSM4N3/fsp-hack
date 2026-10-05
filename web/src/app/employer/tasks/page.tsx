"use client";

import { ChevronDown, ListChecks, Plus, Star } from "lucide-react";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { EmployerTask, Grade, Specialization, TaskKind, TaskSubmission } from "@/api/types";
import { SkillInput } from "@/components/domain/Skills";
import { TaskStatusBadge } from "@/components/domain/StatusBadges";
import { Badge, Button, Card, CardHeader, ChipSelect, Dialog, EmptyState, ErrorAlert, Field, Input, Markdown, PageHeader, RadioCards, Select, SkeletonCard, Switch, Textarea, useToast } from "@/components/ui";
import { CategoryBadge } from "@/components/domain/Category";
import { useCreateTask, useEmployerTasks, useReviewSubmission, useSetTaskActive, useTaskSubmissions } from "@/features/employer/hooks";
import { cn } from "@/lib/cn";
import { formatDate, formatRelative } from "@/lib/format";
import { GRADE_LABEL, GRADES, SPEC_LONG_LABEL, SPECIALIZATIONS, TASK_KIND_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

function ReviewForm({ taskId, sub }: { taskId: string; sub: TaskSubmission }) {
  const toast = useToast();
  const review = useReviewSubmission(taskId);
  const [rating, setRating] = useState(sub.rating ?? 0);
  const [comment, setComment] = useState(sub.review_comment ?? "");
  return (
    <div className="mt-3 space-y-3 border-t border-line pt-3">
      <div role="radiogroup" aria-label="Оценка" className="flex items-center gap-1">
        {[1, 2, 3, 4, 5].map((n) => (
          <button key={n} type="button" role="radio" aria-checked={rating === n} aria-label={`${n} из 5`} onClick={() => setRating(n)} className="rounded p-0.5">
            <Star className={cn("size-5", n <= rating ? "fill-primary text-primary" : "text-line-strong")} aria-hidden />
          </button>
        ))}
      </div>
      <Textarea rows={2} value={comment} onChange={(e) => setComment(e.target.value)} placeholder="Короткий отзыв кандидату" aria-label="Отзыв" />
      <Button
        size="sm"
        disabled={rating === 0}
        loading={review.isPending}
        onClick={() => review.mutate({ submissionId: sub.id, body: { rating, comment: comment.trim() || null } }, { onSuccess: () => toast.success("Оценка сохранена"), onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось сохранить") })}
      >
        {sub.status === "reviewed" ? "Обновить оценку" : "Оценить"}
      </Button>
    </div>
  );
}

function Submissions({ task }: { task: EmployerTask }) {
  const q = useTaskSubmissions(task.id);
  if (q.isPending) return <SkeletonCard lines={2} className="border-0 p-0" />;
  if (q.isError) return <ErrorAlert error={q.error} />;
  if (q.data.length === 0) return <p className="text-sm text-ink-subtle">Ответов пока нет. Задание выдаётся кандидатам категории раз в неделю.</p>;
  return (
    <ul className="space-y-3">
      {q.data.map((s) => (
        <li key={s.id} className="rounded-control border border-line p-4">
          <div className="flex flex-wrap items-center gap-2">
            <span className="font-mono text-sm font-semibold text-ink">Кандидат {s.candidate_code}</span>
            {s.candidate_category && <CategoryBadge category={s.candidate_category} size="sm" />}
            <TaskStatusBadge status={s.status} />
            {s.submitted_at && <span className="text-xs text-ink-subtle">{formatRelative(s.submitted_at)}</span>}
          </div>
          {s.answer && <p className="mt-2 font-mono text-[13px] whitespace-pre-line text-ink">{s.answer}</p>}
          <ReviewForm taskId={task.id} sub={s} />
        </li>
      ))}
    </ul>
  );
}

function TaskRow({ task }: { task: EmployerTask }) {
  const [open, setOpen] = useState(false);
  const setActive = useSetTaskActive();
  return (
    <Card padding="none">
      <button type="button" aria-expanded={open} onClick={() => setOpen((v) => !v)} className="flex w-full flex-col gap-2 p-5 text-left sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone={task.active ? "success" : "muted"} dot>
              {task.active ? "Выдаётся" : "Пауза"}
            </Badge>
            <Badge tone="neutral">{TASK_KIND_LABEL[task.kind]}</Badge>
            {task.grades.map((g) => (
              <CategoryBadge key={g} category={{ specialization: task.specialization, grade: g }} size="sm" />
            ))}
          </div>
          <p className="mt-2 font-mono font-semibold text-ink">{task.title}</p>
          <p className="text-xs text-ink-subtle">
            создано {formatDate(task.created_at)} · ответов {task.submissions} · оценено {task.reviewed}
          </p>
        </div>
        <ChevronDown className={cn("size-5 shrink-0 text-ink-subtle transition-transform", open && "rotate-180")} aria-hidden />
      </button>
      {open && (
        <div className="space-y-5 border-t border-line p-5">
          <Markdown source={task.body} />
          {task.skills.length > 0 && <p className="text-sm text-ink-muted">Навыки: {task.skills.map(skillName).join(", ")}</p>}
          <Switch checked={task.active} onChange={(active) => setActive.mutate({ id: task.id, active })} disabled={setActive.isPending} label="Выдавать кандидатам" description="Задание получат кандидаты выбранных категорий, у которых нет открытого задания" />
          <div>
            <p className="mb-3 text-sm font-semibold text-ink">Ответы</p>
            <Submissions task={task} />
          </div>
        </div>
      )}
    </Card>
  );
}

function CreateDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const toast = useToast();
  const create = useCreateTask();
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [kind, setKind] = useState<TaskKind>("approach");
  const [spec, setSpec] = useState<Specialization>("backend");
  const [grades, setGrades] = useState<Grade[]>(["middle"]);
  const [skills, setSkills] = useState<string[]>([]);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const submit = () => {
    const e: Record<string, string> = {};
    if (title.trim().length < 3) e.title = "Укажите название";
    if (body.trim().length < 30) e.body = "Опишите задание — хотя бы 30 символов";
    if (!grades.length) e.grades = "Выберите грейды";
    setErrors(e);
    if (Object.keys(e).length) return;
    create.mutate(
      { title: title.trim(), body: body.trim(), kind, specialization: spec, grades, skills },
      {
        onSuccess: () => {
          toast.success("Задание создано", "Его получат кандидаты выбранных категорий");
          onClose();
        },
        onError: (err) => toast.error(isApiError(err) ? err.title : "Не удалось создать"),
      },
    );
  };
  return (
    <Dialog
      open={open}
      onClose={onClose}
      size="lg"
      title="Новое регулярное задание"
      description="Короткая задача на 15–30 минут. Ответ поднимает актуальность профиля кандидата."
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            Отмена
          </Button>
          <Button loading={create.isPending} onClick={submit}>
            Создать
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <Field label="Название" required error={errors.title}>
          <Input value={title} onChange={(e) => setTitle(e.target.value)} />
        </Field>
        <Field label="Что нужно сделать" required error={errors.body} hint="Поддерживается `код` и блоки ```">
          <Textarea rows={6} value={body} onChange={(e) => setBody(e.target.value)} />
        </Field>
        <RadioCards<TaskKind>
          name="kind"
          value={kind}
          onChange={setKind}
          options={[
            { value: "approach", title: TASK_KIND_LABEL.approach, description: "Кандидат описывает идею и компромиссы" },
            { value: "solution", title: TASK_KIND_LABEL.solution, description: "Кандидат присылает решение или код" },
          ]}
        />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Специализация">
            <Select value={spec} onChange={(e) => setSpec(e.target.value as Specialization)}>
              {SPECIALIZATIONS.map((s) => (
                <option key={s} value={s}>
                  {SPEC_LONG_LABEL[s]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Грейды" error={errors.grades}>
            <ChipSelect<Grade> ariaLabel="Грейды" options={GRADES.map((g) => ({ value: g, label: GRADE_LABEL[g] }))} value={grades} onChange={setGrades} />
          </Field>
        </div>
        <Field label="Навыки" optional>
          <SkillInput value={skills} onChange={setSkills} />
        </Field>
      </div>
    </Dialog>
  );
}

export default function EmployerTasksPage() {
  const q = useEmployerTasks();
  const [creating, setCreating] = useState(false);
  return (
    <div>
      <PageHeader
        title="Регулярные задания"
        description="Короткие задачи для кандидатов нужных категорий: помогают увидеть ход мысли и поддерживают актуальность профилей."
        actions={
          <Button icon={<Plus className="size-4" aria-hidden />} onClick={() => setCreating(true)}>
            Новое задание
          </Button>
        }
      />
      {q.isPending ? (
        <SkeletonCard lines={3} />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState icon={<ListChecks className="size-5" aria-hidden />} title="Заданий пока нет" description="Создайте задание — его получат кандидаты выбранной категории." action={<Button onClick={() => setCreating(true)}>Создать задание</Button>} />
      ) : (
        <div className="space-y-4">
          <CardHeader title={`Заданий: ${q.data.length}`} as="h2" className="mb-0" />
          {q.data.map((t) => (
            <TaskRow key={t.id} task={t} />
          ))}
        </div>
      )}
      <CreateDialog key={String(creating)} open={creating} onClose={() => setCreating(false)} />
    </div>
  );
}
