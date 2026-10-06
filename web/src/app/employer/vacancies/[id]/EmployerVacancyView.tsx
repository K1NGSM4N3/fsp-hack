"use client";

import { Check, Eye, Inbox, X } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import type { EmployerApplicationStatus, IncomingApplication } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { ApplicationStatusBadge, VacancyStatusBadge } from "@/components/domain/StatusBadges";
import { Alert, Button, EmptyState, ErrorAlert, PageHeader, PageSkeleton, SkeletonCard, Tabs, useToast } from "@/components/ui";
import { useOwnVacancy, useSetApplicationStatus, useVacancyApplications } from "@/features/employer/hooks";
import { VacancyForm } from "@/features/employer/components/VacancyForm";
import { formatDate, formatRelative } from "@/lib/format";

type View = "applications" | "edit";

function ApplicationRow({ a, vacancyId }: { a: IncomingApplication; vacancyId: string }) {
  const toast = useToast();
  const set = useSetApplicationStatus(vacancyId);
  const act = (status: EmployerApplicationStatus, msg: string) => set.mutate({ id: a.id, status }, { onSuccess: () => toast.success(msg) });
  const open = a.status === "sent" || a.status === "viewed";
  return (
    <li className="rounded-card border border-line bg-white p-5 shadow-card">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Link href={`/employer/candidates/${a.candidate_id}`} className="font-mono font-semibold text-ink hover:text-primary hover:underline">
              Кандидат {a.candidate_code}
            </Link>
            {a.candidate_category ? <CategoryBadge category={a.candidate_category} size="sm" /> : <span className="text-xs text-ink-subtle">категория не подтверждена</span>}
            <ApplicationStatusBadge status={a.status} />
          </div>
          <p className="mt-1 text-xs text-ink-subtle">отклик {formatRelative(a.created_at)} · контакты доступны в профиле</p>
          {a.cover_letter && <p className="mt-3 rounded-control bg-canvas p-3 text-sm text-ink">«{a.cover_letter}»</p>}
        </div>
      </div>
      {open && (
        <div className="mt-4 flex flex-wrap gap-2 border-t border-line pt-4">
          {a.status === "sent" && (
            <Button size="sm" variant="ghost" icon={<Eye className="size-4" aria-hidden />} onClick={() => act("viewed", "Отмечено как просмотренное")} disabled={set.isPending}>
              Просмотрено
            </Button>
          )}
          <Button size="sm" icon={<Check className="size-4" aria-hidden />} onClick={() => act("accepted", "Кандидат приглашён")} disabled={set.isPending}>
            Пригласить на интервью
          </Button>
          <Button size="sm" variant="secondary" icon={<X className="size-4" aria-hidden />} onClick={() => act("rejected", "Отказ отправлен")} disabled={set.isPending}>
            Отказать
          </Button>
        </div>
      )}
    </li>
  );
}

function Applications({ vacancyId }: { vacancyId: string }) {
  const q = useVacancyApplications(vacancyId);
  if (q.isPending) return <SkeletonCard lines={3} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  if (q.data.length === 0) return <EmptyState icon={<Inbox className="size-5" aria-hidden />} title="Откликов пока нет" description="Когда кандидат откликнется, его контакты сразу станут доступны — он сделал это сам." />;
  return (
    <ul className="space-y-3">
      {q.data.map((a) => (
        <ApplicationRow key={a.id} a={a} vacancyId={vacancyId} />
      ))}
    </ul>
  );
}

export function EmployerVacancyView({ id }: { id: string }) {
  const q = useOwnVacancy(id);
  const [view, setView] = useState<View>("applications");
  if (q.isPending) return <PageSkeleton cards={2} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const v = q.data;
  return (
    <div className="max-w-4xl">
      <PageHeader back={{ href: "/employer/vacancies", label: "Вакансии" }} eyebrow={<VacancyStatusBadge status={v.status} />} title={v.title} />
      {v.unpublished_by_moderator_at && (
        <Alert
          tone="danger"
          title={`Вакансию снял с публикации модератор ${formatDate(v.unpublished_by_moderator_at)}`}
          className="mb-6"
          action={
            <Link href="/employer/trust" className="text-sm font-medium text-primary hover:underline">
              Причина — в разделе «Доверие и квота»
            </Link>
          }
        >
          Её можно править, но не опубликовать снова, пока модератор не вернёт её. Отклики сохранились.
        </Alert>
      )}
      <Tabs<View>
        ariaLabel="Раздел вакансии"
        value={view}
        onChange={setView}
        className="mb-6"
        items={[
          { value: "applications", label: "Отклики", count: v.applications },
          { value: "edit", label: "Редактирование" },
        ]}
      />
      {view === "applications" ? <Applications vacancyId={v.id} /> : <VacancyForm vacancy={v} onSaved={() => setView("applications")} />}
    </div>
  );
}
