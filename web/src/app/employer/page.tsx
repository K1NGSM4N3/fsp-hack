"use client";

import { ArrowRight, CheckCheck, Lightbulb, MessageSquareReply, Plus, Search, Send, Target } from "lucide-react";
import type { ReactNode } from "react";
import Link from "next/link";
import type { InvitationStats, InvitationStatus } from "@/api/types";
import { SalaryRange } from "@/components/domain/Category";
import { AnimatedNumber, ButtonLink, Card, CardHeader, EmptyState, ErrorAlert, PageHeader, ProgressBar, SkeletonCard, Stat } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { useInvitationStats, useNeeds } from "@/features/employer/hooks";
import { cn } from "@/lib/cn";
import { formatPercent, plural } from "@/lib/format";
import { DECLINE_REASON_LABEL, INVITATION_STATUS_LABEL } from "@/lib/labels";

const FUNNEL: { status: InvitationStatus; tone: string }[] = [
  { status: "sent", tone: "bg-primary-300" },
  { status: "viewed", tone: "bg-warning" },
  { status: "accepted", tone: "bg-success" },
  { status: "declined", tone: "bg-accent" },
  { status: "expired", tone: "bg-ink-faint" },
  { status: "withdrawn", tone: "bg-line-strong" },
];

function Kpi({ icon, label, value, suffix, hint, accent = false }: { icon: ReactNode; label: string; value: number; suffix?: string; hint: string; accent?: boolean }) {
  return (
    <div className={cn("relative overflow-hidden rounded-card border p-5", accent ? "border-transparent bg-night text-white" : "border-line bg-white shadow-card")}>
      {accent && <div aria-hidden className="pointer-events-none absolute -top-16 -right-10 size-40 rounded-full bg-primary/50 blur-[60px]" />}
      <div className="relative flex items-center justify-between gap-3">
        <p className={cn("text-sm", accent ? "text-night-muted" : "text-ink-muted")}>{label}</p>
        <span className={cn("flex size-8 items-center justify-center rounded-[10px]", accent ? "bg-white/10 text-primary-300" : "bg-primary-50 text-primary")} aria-hidden>
          {icon}
        </span>
      </div>
      <p className="tabular relative mt-4 font-display text-[34px] leading-none font-semibold tracking-[-0.035em]">
        <AnimatedNumber value={value} />
        {suffix && <span className={cn("ml-1 text-lg font-medium", accent ? "text-night-muted" : "text-ink-subtle")}>{suffix}</span>}
      </p>
      <p className={cn("relative mt-2 text-xs", accent ? "text-night-faint" : "text-ink-subtle")}>{hint}</p>
    </div>
  );
}

function KpiRow({ s, needs }: { s: InvitationStats; needs: number | undefined }) {
  const accepted = s.by_status.find((x) => x.status === "accepted")?.count ?? 0;
  const total = s.by_status.reduce((a, x) => a + x.count, 0);
  return (
    <div className="stagger-rise mb-6 grid grid-cols-2 gap-4 lg:grid-cols-4">
      <Kpi accent icon={<Target className="size-4" />} label="Активные потребности" value={needs ?? 0} hint="подборки обновляются при уточнении" />
      <Kpi icon={<Send className="size-4" />} label="Приглашения за 7 дней" value={s.quota_used} suffix={`/ ${s.quota_limit}`} hint={`квота обновится через ${s.quota_resets_in_days} ${plural(s.quota_resets_in_days, ["день", "дня", "дней"])}`} />
      <Kpi icon={<MessageSquareReply className="size-4" />} label="Отвечают" value={Math.round(s.response_rate * 100)} suffix="%" hint="приняли или отклонили" />
      <Kpi icon={<CheckCheck className="size-4" />} label="Приняли" value={accepted} suffix={total ? `из ${total}` : undefined} hint="контакты открыты" />
    </div>
  );
}

function QuotaCard({ s }: { s: InvitationStats }) {
  const left = s.quota_limit - s.quota_used;
  return (
    <Card>
      <CardHeader
        title="Квота приглашений"
        description={`До ${s.quota_max} за 7 дней — лимит зависит от уровня доверия к компании, кандидаты получают только точечные предложения`}
        action={
          <Link href="/employer/trust" className="text-sm font-medium text-primary hover:underline">
            Как поднять
          </Link>
        }
      />
      <p className="tabular font-mono text-3xl font-semibold tracking-tight text-ink">
        {s.quota_used}
        <span className="text-xl text-ink-subtle"> / {s.quota_limit}</span>
      </p>
      <ProgressBar value={s.quota_used} max={s.quota_limit} size="md" tone={left <= 3 ? "danger" : "primary"} className="mt-3" label="Использовано приглашений" />
      <p className="mt-2 text-sm text-ink-muted">
        Осталось {left} · квота обновится через {s.quota_resets_in_days} {plural(s.quota_resets_in_days, ["день", "дня", "дней"])}
      </p>
    </Card>
  );
}

function FunnelCard({ s }: { s: InvitationStats }) {
  const counts = Object.fromEntries(s.by_status.map((x) => [x.status, x.count])) as Partial<Record<InvitationStatus, number>>;
  const total = s.by_status.reduce((a, x) => a + x.count, 0);
  const max = Math.max(1, ...s.by_status.map((x) => x.count));
  return (
    <Card>
      <CardHeader
        title="Воронка приглашений"
        description={`Всего ${total}`}
        action={
          <Link href="/employer/invitations" className="text-sm font-medium text-primary hover:underline">
            Все приглашения
          </Link>
        }
      />
      <div className="mb-5 grid grid-cols-2 gap-4">
        <Stat label="Отвечают" value={formatPercent(s.response_rate)} hint="приняли или отклонили" tone="primary" />
        <Stat label="Приняли" value={counts.accepted ?? 0} hint="контакты открыты" />
      </div>
      <ul className="space-y-2.5">
        {FUNNEL.map((f) => {
          const n = counts[f.status] ?? 0;
          return (
            <li key={f.status} className="grid grid-cols-[7rem_1fr_2.5rem] items-center gap-3 text-sm">
              <span className="text-ink-muted">{INVITATION_STATUS_LABEL[f.status]}</span>
              <span className="h-2.5 overflow-hidden rounded-full bg-surface" aria-hidden>
                <span className={cn("block h-full origin-left animate-grow-x rounded-full", f.tone)} style={{ width: `${(n / max) * 100}%` }} />
              </span>
              <span className="tabular text-right font-mono text-ink">{n}</span>
            </li>
          );
        })}
      </ul>
    </Card>
  );
}

function ReasonsCard({ s }: { s: InvitationStats }) {
  const total = s.decline_reasons.reduce((a, r) => a + r.count, 0);
  const top = s.decline_reasons[0];
  return (
    <Card>
      <CardHeader title="Почему отказывают" description="Агрегировано по всем отказам, анонимно" />
      {total === 0 ? (
        <p className="text-sm text-ink-subtle">Отказов пока нет.</p>
      ) : (
        <>
          <ul className="space-y-3">
            {s.decline_reasons.map((r) => {
              const share = r.count / total;
              return (
                <li key={r.reason ?? "none"}>
                  <div className="mb-1 flex items-center justify-between text-sm">
                    <span className="text-ink">{r.reason ? DECLINE_REASON_LABEL[r.reason] : "Без причины"}</span>
                    <span className="tabular font-mono text-ink-muted">
                      {Math.round(share * 100)} % · {r.count}
                    </span>
                  </div>
                  <ProgressBar value={share * 100} size="xs" tone={r.reason === "salary" ? "danger" : "primary"} label={r.reason ? DECLINE_REASON_LABEL[r.reason] : "Без причины"} />
                </li>
              );
            })}
          </ul>
          {top?.reason === "salary" && top.count / total >= 0.4 && (
            <p className="mt-4 flex items-start gap-2 rounded-control bg-warning-50 p-3 text-sm text-ink">
              <Lightbulb className="mt-0.5 size-4 shrink-0 text-warning" aria-hidden />
              {Math.round((top.count / total) * 100)} % отказов — из-за зарплаты. Сверьте вилку с медианными ожиданиями в карточках категорий.
            </p>
          )}
        </>
      )}
    </Card>
  );
}

function NeedsCard() {
  const q = useNeeds();
  return (
    <Card>
      <CardHeader
        title="Активные потребности"
        action={
          <ButtonLink href="/employer/needs/new" size="sm" variant="soft" icon={<Plus className="size-4" aria-hidden />}>
            Новая
          </ButtonLink>
        }
      />
      {q.isPending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState compact icon={<Target className="size-5" aria-hidden />} title="Нет потребностей" description="Опишите, кого ищете, — получите подборку с объяснением." action={<ButtonLink href="/employer/needs/new" size="sm">Описать</ButtonLink>} />
      ) : (
        <ul className="divide-y divide-line">
          {q.data.slice(0, 4).map((n) => (
            <li key={n.id}>
              <Link href={`/employer/needs/${n.id}`} className="group flex items-center justify-between gap-3 py-3 first:pt-0">
                <span className="min-w-0">
                  <span className="block truncate text-sm font-medium text-ink group-hover:text-primary">{n.title}</span>
                  <span className="text-xs text-ink-subtle">
                    в подборке {n.shortlisted} · приглашено {n.invited}
                  </span>
                </span>
                <span className="flex shrink-0 items-center gap-2">
                  <SalaryRange salary={n.salary} size="sm" className="hidden sm:inline" />
                  <ArrowRight className="size-4 text-ink-subtle group-hover:text-primary" aria-hidden />
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}

export default function EmployerDashboard() {
  const me = useMe();
  const stats = useInvitationStats();
  const needs = useNeeds();
  return (
    <div>
      <PageHeader
        eyebrow={<span className="eyebrow text-primary">Обзор найма</span>}
        title={me.data?.company?.name ?? "Главная"}
        description="Подборки, приглашения и обратная связь от кандидатов."
        actions={
          <>
            <ButtonLink href="/employer/candidates" variant="secondary" icon={<Search className="size-4" aria-hidden />}>
              Поиск по базе
            </ButtonLink>
            <ButtonLink href="/employer/needs/new" icon={<Plus className="size-4" aria-hidden />}>
              Новая потребность
            </ButtonLink>
          </>
        }
      />
      {stats.data && <KpiRow s={stats.data} needs={needs.data?.length} />}
      <div className="grid gap-6 lg:grid-cols-[1.3fr_1fr]">
        <div className="min-w-0 space-y-6">
          <NeedsCard />
          {stats.isPending ? <SkeletonCard lines={6} /> : stats.isError ? <ErrorAlert error={stats.error} onRetry={() => void stats.refetch()} /> : <FunnelCard s={stats.data} />}
        </div>
        <div className="min-w-0 space-y-6">
          {stats.data && <QuotaCard s={stats.data} />}
          {stats.data && <ReasonsCard s={stats.data} />}
        </div>
      </div>
    </div>
  );
}
