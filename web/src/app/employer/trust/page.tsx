"use client";

import { ArrowRight, Globe, Lightbulb, Minus, Plus, ShieldCheck } from "lucide-react";
import Link from "next/link";
import type { TrustLevel, TrustReason, TrustStatus } from "@/api/types";
import { Badge, type BadgeTone, Card, CardHeader, ErrorAlert, PageHeader, PageSkeleton, ProgressBar } from "@/components/ui";
import { useTrustStatus } from "@/features/employer/hooks";
import { cn } from "@/lib/cn";
import { plural } from "@/lib/format";
import { TRUST_LEVEL_HINT, TRUST_LEVEL_LABEL } from "@/lib/labels";

const LEVEL_TONE: Record<TrustLevel, BadgeTone> = { high: "success", standard: "primary", new: "neutral", low: "warning" };

function signed(n: number): string {
  return n > 0 ? `+${n}` : n < 0 ? `−${Math.abs(n)}` : "0";
}

function QuotaCard({ t }: { t: TrustStatus }) {
  const left = Math.max(0, t.quota_limit - t.quota_used);
  return (
    <Card padding="lg">
      <div className="flex flex-wrap items-center gap-2">
        <Badge tone={LEVEL_TONE[t.level]} dot>
          {TRUST_LEVEL_LABEL[t.level]}
        </Badge>
      </div>
      <p className="mt-4 text-sm text-ink-subtle">Приглашений за 7 дней</p>
      <p className="tabular mt-1 font-display text-[40px] leading-none font-semibold tracking-[-0.035em] text-ink">
        {t.quota_limit}
        <span className="ml-2 text-lg font-medium text-ink-subtle">из {t.quota_max} возможных</span>
      </p>
      <ProgressBar value={t.quota_limit} max={t.quota_max} size="md" tone={t.level === "low" ? "warning" : "primary"} className="mt-4" label="Квота от максимальной" />
      <p className="mt-3 text-sm text-ink-muted">{TRUST_LEVEL_HINT[t.level]}</p>
      <dl className="mt-6 grid grid-cols-3 gap-3 border-t border-line pt-5 text-sm">
        <div>
          <dt className="text-ink-subtle">База</dt>
          <dd className="tabular mt-0.5 font-mono font-semibold text-ink">{t.quota_base}</dd>
        </div>
        <div>
          <dt className="text-ink-subtle">Бонусы</dt>
          <dd className="tabular mt-0.5 font-mono font-semibold text-success">{signed(t.bonus)}</dd>
        </div>
        <div>
          <dt className="text-ink-subtle">Замечания</dt>
          <dd className={cn("tabular mt-0.5 font-mono font-semibold", t.penalty < 0 ? "text-accent-strong" : "text-ink")}>{signed(t.penalty)}</dd>
        </div>
      </dl>
      <p className="mt-4 text-sm text-ink-muted">
        Сейчас использовано {t.quota_used}, осталось {left} {plural(left, ["приглашение", "приглашения", "приглашений"])}.
      </p>
    </Card>
  );
}

function ReasonRow({ r }: { r: TrustReason }) {
  const plus = r.points > 0;
  return (
    <li className="flex items-start gap-3 py-3 first:pt-0 last:pb-0">
      <span
        aria-hidden
        className={cn("mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-full", plus ? "bg-success-50 text-success" : "bg-accent-50 text-accent-strong")}
      >
        {plus ? <Plus className="size-3.5" /> : <Minus className="size-3.5" />}
      </span>
      <span className="min-w-0 flex-1 text-sm text-ink">{r.text}</span>
      <span className={cn("tabular shrink-0 font-mono text-sm font-semibold", plus ? "text-success" : "text-accent-strong")}>
        <span className="sr-only">{plus ? "Плюс" : "Минус"} к квоте: </span>
        {signed(r.points)}
      </span>
    </li>
  );
}

function ReasonsCard({ t }: { t: TrustStatus }) {
  return (
    <Card>
      <CardHeader title="Из чего складывается квота" description={`Базовая квота — ${t.quota_base}. Бонусы складываются до +${t.quota_max - t.quota_base}, квота не опускается ниже ${t.quota_min}.`} />
      {t.reasons.length === 0 ? (
        <p className="text-sm text-ink-muted">Пока ни бонусов, ни замечаний — так начинает любая компания.</p>
      ) : (
        <ul className="divide-y divide-line">
          {t.reasons.map((r) => (
            <ReasonRow key={r.kind} r={r} />
          ))}
        </ul>
      )}
    </Card>
  );
}

function AdviceCard({ t }: { t: TrustStatus }) {
  return (
    <Card>
      <CardHeader title="Как поднять квоту" />
      {t.advice.length === 0 ? (
        <p className="text-sm text-ink-muted">Квота максимальная. Точные вилки и ответы кандидатам её сохраняют.</p>
      ) : (
        <ol className="space-y-3">
          {t.advice.map((a, i) => (
            <li key={a} className="flex items-start gap-3 text-sm text-ink">
              <span className="tabular mt-px flex size-5 shrink-0 items-center justify-center rounded-full bg-primary-50 font-mono text-[11px] font-semibold text-primary">{i + 1}</span>
              <span>{a}</span>
            </li>
          ))}
        </ol>
      )}
    </Card>
  );
}

function DomainCard({ t }: { t: TrustStatus }) {
  const verified = t.domain.check.status === "verified";
  return (
    <Card>
      <CardHeader
        title="Корпоративный домен"
        action={
          verified ? (
            <Badge tone="success" icon={<ShieldCheck className="size-3.5" aria-hidden />}>
              Подтверждён
            </Badge>
          ) : (
            <Badge tone="muted">Не подтверждён</Badge>
          )
        }
      />
      <p className="text-sm leading-relaxed text-ink">{t.domain.hint}</p>
      {verified ? (
        <p className="mt-3 flex items-start gap-2 text-xs text-ink-subtle">
          <Globe className="mt-px size-3.5 shrink-0" aria-hidden />
          Отметку видят кандидаты в приглашениях и вакансиях.
        </p>
      ) : (
        t.domain.check.status !== "public_mail" && (
          <Link href="/employer/company" className="mt-4 inline-flex items-center gap-1.5 text-sm font-medium text-primary hover:underline">
            Указать сайт в профиле компании
            <ArrowRight className="size-4" aria-hidden />
          </Link>
        )
      )}
    </Card>
  );
}

function RuleCard({ t }: { t: TrustStatus }) {
  return (
    <Card>
      <CardHeader title="Правило целиком" description="Одинаковое для всех компаний" />
      <ul className="space-y-2 text-sm text-ink-muted">
        {t.rule.map((line) => (
          <li key={line} className="flex items-start gap-2">
            <Lightbulb className="mt-0.5 size-4 shrink-0 text-ink-faint" aria-hidden />
            <span>{line}</span>
          </li>
        ))}
      </ul>
    </Card>
  );
}

export default function TrustPage() {
  const q = useTrustStatus();
  return (
    <div>
      <PageHeader
        eyebrow={<span className="eyebrow text-primary">Аккаунт</span>}
        title="Доверие и квота"
        description="Квота приглашений защищает кандидатов от рассылок. Она растёт, когда компания подтверждает себя и кандидаты откликаются на её предложения. Здесь видно, из чего она складывается сейчас и что можно сделать."
      />
      {q.isPending ? (
        <PageSkeleton cards={4} />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : (
        <div className="grid gap-6 lg:grid-cols-[1fr_1.15fr]">
          <div className="min-w-0 space-y-6">
            <QuotaCard t={q.data} />
            <DomainCard t={q.data} />
          </div>
          <div className="min-w-0 space-y-6">
            <ReasonsCard t={q.data} />
            <AdviceCard t={q.data} />
            <RuleCard t={q.data} />
          </div>
        </div>
      )}
    </div>
  );
}
