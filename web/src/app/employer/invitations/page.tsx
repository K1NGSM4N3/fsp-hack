"use client";

import { ChevronDown, Send, Undo2 } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { InvitationStatus, SentInvitation } from "@/api/types";
import { CategoryBadge, SalaryRange } from "@/components/domain/Category";
import { InvitationStatusBadge } from "@/components/domain/StatusBadges";
import { Button, ButtonLink, Dialog, EmptyState, ErrorAlert, LoadMore, PageHeader, SkeletonCard, Tabs, tabPanelProps, useToast } from "@/components/ui";
import { useSentInvitation, useSentInvitationPages, useWithdrawInvitation } from "@/features/employer/hooks";
import { cn } from "@/lib/cn";
import { formatDateTime, formatRelative } from "@/lib/format";
import { DECLINE_REASON_LABEL, INVITATION_STATUS_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";

type Filter = "all" | InvitationStatus;

function Events({ id }: { id: string }) {
  const q = useSentInvitation(id);
  if (q.isPending) return <p className="text-sm text-ink-subtle">Загружаем историю…</p>;
  if (q.isError || !q.data.events) return null;
  return (
    <ol className="space-y-1.5 text-sm">
      {q.data.events.map((e, i) => (
        <li key={i} className="flex items-center justify-between gap-3">
          <span className="text-ink">{INVITATION_STATUS_LABEL[e.status]}</span>
          <span className="tabular text-ink-subtle">{formatDateTime(e.at)}</span>
        </li>
      ))}
    </ol>
  );
}

function Row({ inv }: { inv: SentInvitation }) {
  const [open, setOpen] = useState(false);
  const toast = useToast();
  const withdraw = useWithdrawInvitation();
  const [confirm, setConfirm] = useState(false);
  const active = inv.status === "sent" || inv.status === "viewed";
  return (
    <li className="rounded-card border border-line bg-white shadow-card">
      <button type="button" aria-expanded={open} onClick={() => setOpen((v) => !v)} className="flex w-full flex-col gap-3 p-5 text-left sm:flex-row sm:items-center sm:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <InvitationStatusBadge status={inv.status} />
            <span className="font-mono text-sm font-semibold text-ink">Кандидат {inv.candidate_code}</span>
            {inv.candidate_category && <CategoryBadge category={inv.candidate_category} size="sm" />}
          </div>
          <p className="mt-1.5 truncate text-sm text-ink">{inv.title}</p>
          <p className="text-xs text-ink-subtle">отправлено {formatRelative(inv.created_at)}</p>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          <SalaryRange salary={inv.salary} size="sm" />
          <ChevronDown className={cn("size-5 text-ink-subtle transition-transform", open && "rotate-180")} aria-hidden />
        </div>
      </button>
      {open && (
        <div className="grid gap-6 border-t border-line p-5 md:grid-cols-[1fr_16rem]">
          <div className="space-y-3 text-sm">
            <p className="whitespace-pre-line text-ink">{inv.message}</p>
            <p className="text-ink-muted">
              Формат: {inv.work_format ? WORK_FORMAT_LABEL[inv.work_format] : "обсуждается"} · связь: {inv.contact}
            </p>
            {inv.decline_reason && (
              <p className="rounded-control bg-accent-50 p-3 text-ink">
                Причина отказа: <span className="font-medium">{DECLINE_REASON_LABEL[inv.decline_reason]}</span>
                {inv.decline_comment ? ` — «${inv.decline_comment}»` : ""}
              </p>
            )}
            <div className="flex flex-wrap gap-2 pt-1">
              <ButtonLink href={`/employer/candidates/${inv.candidate_id}${inv.need_id ? `?need=${inv.need_id}` : ""}`} size="sm" variant="secondary">
                {inv.status === "accepted" ? "Профиль и контакты" : "Профиль кандидата"}
              </ButtonLink>
              {inv.need_id && (
                <Link href={`/employer/needs/${inv.need_id}`} className="tap inline-flex items-center rounded px-2 text-sm font-medium text-primary hover:underline">
                  К подборке
                </Link>
              )}
              {active && (
                <Button
                  size="sm"
                  variant="ghost"
                  icon={<Undo2 className="size-4" aria-hidden />}
                  onClick={() => setConfirm(true)}
                >
                  Отозвать
                </Button>
              )}
              <Dialog
                open={confirm}
                onClose={() => setConfirm(false)}
                size="sm"
                title="Отозвать приглашение?"
                footer={
                  <>
                    <Button variant="ghost" onClick={() => setConfirm(false)}>
                      Отмена
                    </Button>
                    <Button
                      variant="danger"
                      loading={withdraw.isPending}
                      onClick={() =>
                        withdraw.mutate(inv.id, {
                          onSuccess: () => {
                            setConfirm(false);
                            toast.success("Приглашение отозвано");
                          },
                          onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось отозвать"),
                        })
                      }
                    >
                      Отозвать
                    </Button>
                  </>
                }
              >
                <p className="text-sm text-ink-muted">
                  Соискатель увидит, что приглашение больше не ждёт ответа. Место в квоте не вернётся, а частые отзывы снижают уровень доверия к компании.
                </p>
              </Dialog>
            </div>
          </div>
          <div>
            <p className="mb-2 text-xs font-medium tracking-wide text-ink-subtle uppercase">История</p>
            <Events id={inv.id} />
            {active && <p className="mt-3 text-xs text-ink-subtle">Истекает {formatRelative(inv.expires_at)}</p>}
          </div>
        </div>
      )}
    </li>
  );
}

const STATUSES: InvitationStatus[] = ["sent", "viewed", "accepted", "declined", "expired", "withdrawn"];

export default function EmployerInvitationsPage() {
  const [filter, setFilter] = useState<Filter>("all");
  const q = useSentInvitationPages(filter === "all" ? [] : [filter]);
  const byStatus = q.data?.pages[0]?.by_status ?? [];
  const count = (s: InvitationStatus) => byStatus.find((c) => c.status === s)?.count ?? 0;
  const total = byStatus.reduce((n, c) => n + c.count, 0);
  const list = q.isPlaceholderData ? [] : (q.data?.pages.flatMap((p) => p.items) ?? []);
  return (
    <div>
      <PageHeader title="Приглашения" description="Статусы обновляются, когда кандидат открывает приглашение и отвечает. Приглашение действует 14 дней." />
      <Tabs<Filter>
        idPrefix="sent"
        ariaLabel="Фильтр по статусу"
        value={filter}
        onChange={setFilter}
        className="mb-6"
        items={[{ value: "all" as Filter, label: "Все", count: total }, ...STATUSES.map((s) => ({ value: s as Filter, label: INVITATION_STATUS_LABEL[s], count: count(s) }))]}
      />
      <div {...tabPanelProps("sent", filter)}>
        {q.isPending || q.isPlaceholderData ? (
          <div className="space-y-3">
            <SkeletonCard lines={2} />
            <SkeletonCard lines={2} />
          </div>
        ) : q.isError ? (
          <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
        ) : list.length === 0 ? (
          <EmptyState
            icon={<Send className="size-5" aria-hidden />}
            title={total === 0 ? "Вы ещё никого не пригласили" : "Нет приглашений с таким статусом"}
            description={total === 0 ? "Откройте подборку по потребности или найдите кандидата в базе — и отправьте приглашение с вилкой." : undefined}
            action={total === 0 ? <ButtonLink href="/employer/needs">К подборкам</ButtonLink> : undefined}
          />
        ) : (
          <>
            <ul className="space-y-3">
              {list.map((inv) => (
                <Row key={inv.id} inv={inv} />
              ))}
            </ul>
            <LoadMore key={filter} pager={q} loaded={list.length} noun="Приглашения" />
          </>
        )}
      </div>
    </div>
  );
}
