"use client";

import { Briefcase, Inbox, Plus } from "lucide-react";
import Link from "next/link";
import { CategoryBadge, SalaryRange } from "@/components/domain/Category";
import { VacancyStatusBadge } from "@/components/domain/StatusBadges";
import { ButtonLink, EmptyState, ErrorAlert, PageHeader, SkeletonCard } from "@/components/ui";
import { useOwnVacancies } from "@/features/employer/hooks";
import { formatRelative, pluralize } from "@/lib/format";
import { WORK_FORMAT_LABEL } from "@/lib/labels";

export default function EmployerVacanciesPage() {
  const q = useOwnVacancies();
  return (
    <div>
      <PageHeader
        title="Вакансии и отклики"
        description="Классический путь: кандидат откликается сам, и его контакты сразу доступны вам."
        actions={
          <ButtonLink href="/employer/vacancies/new" icon={<Plus className="size-4" aria-hidden />}>
            Новая вакансия
          </ButtonLink>
        }
      />
      {q.isPending ? (
        <div className="space-y-3">
          <SkeletonCard lines={2} />
          <SkeletonCard lines={2} />
        </div>
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState icon={<Briefcase className="size-5" aria-hidden />} title="Вакансий пока нет" description="Опубликуйте вакансию — она появится в общем списке, кандидаты смогут откликаться." action={<ButtonLink href="/employer/vacancies/new">Создать вакансию</ButtonLink>} />
      ) : (
        <ul className="space-y-3">
          {q.data.map((v) => (
            <li key={v.id}>
              <Link href={`/employer/vacancies/${v.id}`} className="block rounded-card border border-line bg-white p-5 shadow-card transition-colors hover:border-primary-300">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <VacancyStatusBadge status={v.status} />
                      <CategoryBadge category={{ specialization: v.specialization, grade: v.grade }} size="sm" />
                    </div>
                    <p className="mt-2 font-mono font-semibold text-ink">{v.title}</p>
                    <p className="mt-0.5 text-sm text-ink-muted">
                      {WORK_FORMAT_LABEL[v.work_format]}
                      {v.city ? `, ${v.city}` : ""} · {v.published_at ? `опубликована ${formatRelative(v.published_at)}` : `создана ${formatRelative(v.created_at)}`}
                    </p>
                  </div>
                  <div className="flex shrink-0 flex-col items-start gap-2 sm:items-end">
                    <SalaryRange salary={v.salary} size="sm" />
                    <span className="inline-flex items-center gap-1.5 text-sm text-ink-muted">
                      <Inbox className="size-4" aria-hidden />
                      {pluralize(v.applications, ["отклик", "отклика", "откликов"])}
                    </span>
                  </div>
                </div>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
