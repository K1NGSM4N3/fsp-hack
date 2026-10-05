"use client";

import { ArrowRight, Plus, Target, Trash2 } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import type { HiringNeed } from "@/api/types";
import { CategoryBadge, SalaryRange } from "@/components/domain/Category";
import { Badge, Button, ButtonLink, Chip, Dialog, EmptyState, ErrorAlert, IconButton, PageHeader, SkeletonCard, useToast } from "@/components/ui";
import { useDeleteNeed, useNeeds } from "@/features/employer/hooks";
import { formatRelative } from "@/lib/format";
import { NEED_STATUS_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

function NeedRow({ need, onDelete }: { need: HiringNeed; onDelete: () => void }) {
  return (
    <li className="rounded-card border border-line bg-white p-5 shadow-card transition-colors hover:border-primary-200">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            {need.grades.map((g) => (
              <CategoryBadge key={g} category={{ specialization: need.specialization, grade: g }} size="sm" />
            ))}
            <Badge tone={need.status === "active" ? "success" : "muted"} dot>
              {NEED_STATUS_LABEL[need.status]}
            </Badge>
          </div>
          <Link href={`/employer/needs/${need.id}`} className="mt-2 block text-base font-semibold tracking-tight text-ink hover:text-primary hover:underline">
            {need.title}
          </Link>
          <p className="mt-1 text-sm text-ink-muted">
            {need.work_formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ")}
            {need.city ? ` · ${need.city}` : ""} · обновлена {formatRelative(need.updated_at)}
          </p>
        </div>
        <div className="flex shrink-0 items-start gap-2">
          <SalaryRange salary={need.salary} />
          <IconButton label={`Удалить «${need.title}»`} size="sm" onClick={onDelete}>
            <Trash2 className="size-4" aria-hidden />
          </IconButton>
        </div>
      </div>
      <div className="mt-3 flex flex-wrap gap-1.5">
        {need.must_skills.map((s) => (
          <Chip key={s} tone="primary">
            {skillName(s)}
          </Chip>
        ))}
      </div>
      <div className="mt-4 flex flex-wrap items-center justify-between gap-3 border-t border-line pt-4 text-sm">
        <span className="text-ink-muted">
          В подборке: <span className="tabular font-mono text-ink">{need.shortlisted}</span> · приглашено: <span className="tabular font-mono text-ink">{need.invited}</span>
        </span>
        <ButtonLink href={`/employer/needs/${need.id}`} size="sm" icon={<ArrowRight className="size-4" aria-hidden />}>
          Открыть подборку
        </ButtonLink>
      </div>
    </li>
  );
}

export default function NeedsPage() {
  const q = useNeeds();
  const del = useDeleteNeed();
  const toast = useToast();
  const [target, setTarget] = useState<HiringNeed | null>(null);
  return (
    <div>
      <PageHeader
        title="Потребности и подборки"
        description="Опишите, кого ищете, — система подберёт категории и кандидатов с объяснением."
        actions={
          <ButtonLink href="/employer/needs/new" icon={<Plus className="size-4" aria-hidden />}>
            Новая потребность
          </ButtonLink>
        }
      />
      {q.isPending ? (
        <div className="space-y-3">
          <SkeletonCard lines={3} />
          <SkeletonCard lines={3} />
        </div>
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState
          icon={<Target className="size-5" aria-hidden />}
          title="Пока нет потребностей"
          description="Вставьте текст вакансии или опишите задачу своими словами — мы распознаем стек, грейд и вилку и сразу покажем подборку."
          action={<ButtonLink href="/employer/needs/new">Описать потребность</ButtonLink>}
        />
      ) : (
        <ul className="space-y-4">
          {q.data.map((n) => (
            <NeedRow key={n.id} need={n} onDelete={() => setTarget(n)} />
          ))}
        </ul>
      )}
      <Dialog
        open={target !== null}
        onClose={() => setTarget(null)}
        size="sm"
        title="Удалить потребность?"
        footer={
          <>
            <Button variant="ghost" onClick={() => setTarget(null)}>
              Отмена
            </Button>
            <Button
              variant="danger"
              loading={del.isPending}
              onClick={() =>
                target &&
                del.mutate(target.id, {
                  onSuccess: () => {
                    setTarget(null);
                    toast.success("Потребность удалена");
                  },
                })
              }
            >
              Удалить
            </Button>
          </>
        }
      >
        <p className="text-sm text-ink-muted">Подборка и закреплённые кандидаты будут удалены. Отправленные приглашения сохранятся.</p>
      </Dialog>
    </div>
  );
}
