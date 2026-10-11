"use client";

import { Building2, Clock, Inbox, MapPin } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import type { InvitationStatus, ReceivedInvitation } from "@/api/types";
import { SalaryRange } from "@/components/domain/Category";
import { DomainBadge } from "@/components/domain/DomainBadge";
import { InvitationStatusBadge } from "@/components/domain/StatusBadges";
import { ButtonLink, EmptyState, ErrorAlert, LoadMore, PageHeader, SkeletonCard, Tabs, tabPanelProps } from "@/components/ui";
import { useReceivedInvitationPages } from "@/features/candidate/hooks";
import { cn } from "@/lib/cn";
import { formatRelative } from "@/lib/format";
import { WORK_FORMAT_LABEL } from "@/lib/labels";

type Filter = "active" | "accepted" | "archive";

const FILTERS: Record<Filter, InvitationStatus[]> = {
  active: ["sent", "viewed"],
  accepted: ["accepted"],
  archive: ["declined", "expired", "withdrawn"],
};

function InvitationRow({ inv }: { inv: ReceivedInvitation }) {
  const fresh = inv.status === "sent";
  const active = inv.status === "sent" || inv.status === "viewed";
  return (
    <li>
      <Link
        href={`/candidate/invitations/${inv.id}`}
        className={cn(
          "block rounded-card border bg-white p-5 shadow-card transition-colors hover:border-primary-300",
          fresh ? "border-primary-200 ring-1 ring-primary-100" : "border-line",
        )}
      >
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <InvitationStatusBadge status={inv.status} audience="candidate" />
              <span className="text-xs text-ink-subtle">{formatRelative(inv.created_at)}</span>
            </div>
            <p className={cn("mt-2 font-mono text-base tracking-tight text-ink", fresh ? "font-semibold" : "font-medium")}>{inv.title}</p>
            <p className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-ink-muted">
              <span className="inline-flex items-center gap-1.5">
                <Building2 className="size-4 text-ink-subtle" aria-hidden />
                {inv.company_name}
              </span>
              <DomainBadge domain={inv.company_verified_domain} />
              {inv.work_format && (
                <span className="inline-flex items-center gap-1.5">
                  <MapPin className="size-4 text-ink-subtle" aria-hidden />
                  {WORK_FORMAT_LABEL[inv.work_format]}
                  {inv.company_city && inv.work_format !== "remote" ? `, ${inv.company_city}` : ""}
                </span>
              )}
            </p>
          </div>
          <div className="shrink-0 sm:text-right">
            <SalaryRange salary={inv.salary} size="lg" className="block" label="Зарплата" />
            {active && (
              <p className="mt-1 inline-flex items-center gap-1 text-xs text-ink-subtle">
                <Clock className="size-3.5" aria-hidden />
                ответить {formatRelative(inv.expires_at)}
              </p>
            )}
          </div>
        </div>
      </Link>
    </li>
  );
}

export default function InvitationsPage() {
  const [filter, setFilter] = useState<Filter>("active");
  const q = useReceivedInvitationPages(FILTERS[filter]);
  const byStatus = q.data?.pages[0]?.by_status ?? [];
  const count = (f: Filter) => byStatus.filter((c) => FILTERS[f].includes(c.status)).reduce((n, c) => n + c.count, 0);
  const total = byStatus.reduce((n, c) => n + c.count, 0);
  const list = q.isPlaceholderData ? [] : (q.data?.pages.flatMap((p) => p.items) ?? []);
  return (
    <div>
      <PageHeader title="Приглашения" description="Компании приходят с вилкой и описанием задачи. Контакты откроются им, только если вы примете приглашение." />
      <Tabs<Filter>
        idPrefix="invitations"
        ariaLabel="Фильтр приглашений"
        value={filter}
        onChange={setFilter}
        className="mb-6"
        items={[
          { value: "active", label: "Ждут ответа", count: count("active") },
          { value: "accepted", label: "Принятые", count: count("accepted") },
          { value: "archive", label: "Архив", count: count("archive") },
        ]}
      />
      <div {...tabPanelProps("invitations", filter)}>
        {q.isPending || q.isPlaceholderData ? (
          <div className="space-y-3">
            <SkeletonCard lines={2} />
            <SkeletonCard lines={2} />
          </div>
        ) : q.isError ? (
          <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
        ) : list.length === 0 ? (
          <EmptyState
            icon={<Inbox className="size-5" aria-hidden />}
            title={filter === "active" ? "Новых приглашений нет" : "Здесь пока пусто"}
            description={
              total === 0
                ? "Компании находят соискателей по подтверждённой категории. Пройдите тест и держите статус «Активно ищу», а если есть разряды или призовые места — привяжите ФСП ID. Так вас увидят чаще."
                : "Загляните на другие вкладки."
            }
            action={total === 0 ? <ButtonLink href="/candidate/settings" variant="secondary">Настроить статус поиска</ButtonLink> : undefined}
          />
        ) : (
          <>
            <ul className="space-y-3">
              {list.map((inv) => (
                <InvitationRow key={inv.id} inv={inv} />
              ))}
            </ul>
            <LoadMore key={filter} pager={q} loaded={list.length} noun="Приглашения" />
          </>
        )}
      </div>
    </div>
  );
}
