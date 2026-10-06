"use client";

import { Briefcase } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { isApiError } from "@/api/client";
import { SalaryRange } from "@/components/domain/Category";
import { HiringOutcomes } from "@/components/domain/HiringOutcomes";
import { ApplicationStatusBadge } from "@/components/domain/StatusBadges";
import { VacancyBrowser } from "@/components/domain/VacancyBrowser";
import { Button, EmptyState, ErrorAlert, PageHeader, SkeletonCard, Tabs, tabPanelProps, useToast } from "@/components/ui";
import { useMyApplications, useWithdrawApplication } from "@/features/candidate/hooks";
import { formatDate } from "@/lib/format";

type View = "applications" | "browse";

function Applications({ onBrowse }: { onBrowse: () => void }) {
  const q = useMyApplications();
  const withdraw = useWithdrawApplication();
  const toast = useToast();
  if (q.isPending) return <SkeletonCard lines={2} />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  if (q.data.length === 0) {
    return (
      <EmptyState
        icon={<Briefcase className="size-5" aria-hidden />}
        title="Откликов пока нет"
        description="Отклик — дополнительный путь. Основной — приглашения от компаний по вашей категории."
        action={<Button onClick={onBrowse}>Смотреть вакансии</Button>}
      />
    );
  }
  return (
    <ul className="space-y-3">
      {q.data.map((a) => (
        <li key={a.id} className="rounded-card border border-line bg-white p-5 shadow-card">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div className="min-w-0">
              <Link href={`/candidate/vacancies/${a.vacancy_id}`} className="font-mono font-semibold text-ink hover:text-primary hover:underline">
                {a.vacancy_title}
              </Link>
              <p className="mt-1 text-sm text-ink-muted">
                {a.company_name} · отклик от {formatDate(a.created_at)}
              </p>
            </div>
            <div className="flex shrink-0 flex-col items-start gap-2 sm:items-end">
              <SalaryRange salary={a.salary} size="sm" />
              <ApplicationStatusBadge status={a.status} />
            </div>
          </div>
          <HiringOutcomes outcomes={a.outcomes} className="mt-4 border-t border-line pt-3" />
          {(a.status === "sent" || a.status === "viewed") && (
            <div className="mt-4 flex justify-end border-t border-line pt-3">
              <Button
                variant="ghost"
                size="sm"
                loading={withdraw.isPending && withdraw.variables === a.id}
                onClick={() =>
                  withdraw.mutate(a.id, {
                    onSuccess: () => toast.success("Отклик отозван"),
                    onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось отозвать"),
                  })
                }
              >
                Отозвать отклик
              </Button>
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

export default function CandidateVacanciesPage() {
  const [view, setView] = useState<View>("applications");
  return (
    <div>
      <PageHeader title="Вакансии и отклики" description="Отклик раскрывает компании ваши контакты так же, как принятое приглашение." />
      <Tabs<View>
        idPrefix="vacancies"
        ariaLabel="Раздел"
        value={view}
        onChange={setView}
        className="mb-6"
        items={[
          { value: "applications", label: "Мои отклики" },
          { value: "browse", label: "Все вакансии" },
        ]}
      />
      <div {...tabPanelProps("vacancies", view)}>
        {view === "applications" ? <Applications onBrowse={() => setView("browse")} /> : <VacancyBrowser hrefFor={(id) => `/candidate/vacancies/${id}`} />}
      </div>
    </div>
  );
}
