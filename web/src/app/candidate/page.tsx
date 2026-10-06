"use client";

import { ArrowRight, CalendarClock, Eye, ListChecks, Mail, TrendingUp, Trophy } from "lucide-react";
import Link from "next/link";
import { isApiError } from "@/api/client";
import { ButtonLink, Card, CardHeader, EmptyState, ErrorAlert, PageHeader, SkeletonCard, Stat } from "@/components/ui";
import { SalaryRange } from "@/components/domain/Category";
import { ScoreRing } from "@/components/domain/Explanation";
import { FspAchievementList, FspEmpty } from "@/components/domain/Fsp";
import { InvitationStatusBadge } from "@/components/domain/StatusBadges";
import { useMe } from "@/features/auth/hooks";
import { useCategory, useFspStatus, useMarket, useMyTasks, useReceivedInvitations } from "@/features/candidate/hooks";
import { LinkFspButton } from "@/features/candidate/components/FspCard";
import { OnboardingCard } from "@/features/candidate/components/Onboarding";
import { formatDate, formatRelative, pluralize } from "@/lib/format";
import { GRADE_LABEL, SPEC_LONG_LABEL, categoryLabel } from "@/lib/labels";

function CategoryCard() {
  const category = useCategory();
  if (category.isPending) return <SkeletonCard lines={4} />;
  if (category.isError) return <ErrorAlert error={category.error} onRetry={() => void category.refetch()} />;
  const c = category.data;
  if (!c.category) {
    return (
      <Card>
        <CardHeader title="Категория" />
        <p className="text-sm text-ink-muted">Категория появится после теста. До этого профиль не виден работодателям в поиске.</p>
        <ButtonLink href="/candidate/assessment" className="mt-4" variant="soft">
          К тестированию
        </ButtonLink>
      </Card>
    );
  }
  const strength = c.test_strength !== null && c.test_strength !== undefined ? Math.round(c.test_strength * 100) : null;
  return (
    <section className="relative isolate overflow-hidden rounded-card bg-night p-6 text-white shadow-pop sm:p-7">
      <div aria-hidden className="pointer-events-none absolute -top-24 -right-16 -z-10 size-72 rounded-full bg-primary/40 blur-[90px]" />
      <div aria-hidden className="bg-grid-night pointer-events-none absolute inset-0 -z-10 opacity-60 [mask-image:linear-gradient(to_left,black,transparent_70%)]" />
      <div className="flex flex-col gap-6 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-3">
            <h2 className="font-mono text-xs tracking-[0.14em] text-primary-300 uppercase">Ваша категория</h2>
            <span className="inline-flex items-center gap-1.5 rounded-full bg-success-300/15 px-2.5 py-0.5 text-xs font-medium text-success-300">
              <span className="size-1.5 rounded-full bg-success-300" aria-hidden />
              Подтверждена тестом
            </span>
          </div>
          <p className="mt-4 text-[28px] leading-tight font-semibold tracking-[-0.03em] sm:text-[32px]">{SPEC_LONG_LABEL[c.category.specialization]}</p>
          <p className="mt-1 font-mono text-lg text-primary-300">{GRADE_LABEL[c.category.grade]}</p>
          <p className="mt-4 max-w-sm text-sm leading-relaxed text-night-muted">Видна работодателям — по ней вас находят. Где вы внутри грейда: 0 — у нижней границы, 100 — у верхней.</p>
        </div>
        {strength !== null && <ScoreRing score={strength} size={128} tone="night" label="Сила результата в категории" caption="сила" />}
      </div>
      <div className="mt-6 grid gap-4 border-t border-night-line pt-5 text-sm sm:grid-cols-[1fr_auto] sm:items-center">
        <p className="flex items-start gap-2 text-night-muted">
          <CalendarClock className="mt-0.5 size-4 shrink-0 text-night-faint" aria-hidden />
          <span>
            Тест пройден {formatDate(c.assigned_at)}.{" "}
            {c.next_change_at ? (
              <>
                Сменить грейд можно с <span className="font-medium text-white">{formatDate(c.next_change_at)}</span>.
              </>
            ) : (
              <span className="text-white">Смена грейда доступна.</span>
            )}
          </span>
        </p>
        <Link href="/candidate/assessment" className="tap inline-flex items-center gap-1.5 font-medium text-white underline decoration-white/30 underline-offset-4 hover:decoration-white">
          {c.available.length ? "Пройти тест на другой грейд" : "История тестов"}
          <ArrowRight className="size-4" aria-hidden />
        </Link>
      </div>
    </section>
  );
}

function InvitationsCard() {
  const q = useReceivedInvitations();
  return (
    <Card>
      <CardHeader
        title="Приглашения"
        action={
          <Link href="/candidate/invitations" className="tap text-sm font-medium text-primary hover:underline">
            Все
          </Link>
        }
      />
      {q.isPending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : q.data.length === 0 ? (
        <EmptyState compact icon={<Mail className="size-5" aria-hidden />} title="Пока нет приглашений" description="Компании приглашают кандидатов из подтверждённых категорий. Держите статус «Активно ищу»." />
      ) : (
        <>
          <div className="mb-4 grid grid-cols-3 gap-3">
            <Stat label="Новые" value={q.data.filter((i) => i.status === "sent").length} tone="primary" />
            <Stat label="Ждут ответа" value={q.data.filter((i) => i.status === "sent" || i.status === "viewed").length} />
            <Stat label="Принято" value={q.data.filter((i) => i.status === "accepted").length} />
          </div>
          <ul className="divide-y divide-line">
            {q.data.slice(0, 3).map((inv) => (
              <li key={inv.id} className="py-3 first:pt-0 last:pb-0">
                <Link href={`/candidate/invitations/${inv.id}`} className="group flex items-start justify-between gap-3 rounded">
                  <span className="min-w-0">
                    <span className="block truncate text-sm font-medium text-ink group-hover:text-primary">{inv.title}</span>
                    <span className="block text-xs text-ink-subtle">
                      {inv.company_name} · {formatRelative(inv.created_at)}
                    </span>
                  </span>
                  <span className="flex shrink-0 flex-col items-end gap-1">
                    <SalaryRange salary={inv.salary} size="sm" />
                    <InvitationStatusBadge status={inv.status} audience="candidate" />
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        </>
      )}
    </Card>
  );
}

function MarketCard({ enabled }: { enabled: boolean }) {
  const q = useMarket(enabled);
  if (!enabled) return null;
  if (q.isPending) return <SkeletonCard lines={3} />;
  if (q.isError) return isApiError(q.error) && (q.error.status === 404 || q.error.status === 409) ? null : <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const m = q.data;
  return (
    <Card>
      <CardHeader title="Спрос в вашей категории" description={`${categoryLabel(m.category)} · за 30 дней, анонимно`} />
      <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
        <Stat label="Кандидатов в категории" value={m.peers_in_category} />
        <Stat label="Компаний смотрели вас" value={m.companies_viewed_you_30d} tone="primary" />
        <Stat label="Приглашений в категории" value={m.invitations_in_category_30d} />
        <Stat label="Открытых запросов" value={m.open_needs} />
      </div>
      {m.median_offer_min && m.median_offer_max && (
        <div className="mt-5 rounded-control bg-canvas p-4">
          <p className="text-xs text-ink-subtle">Медианная вилка предложений</p>
          <SalaryRange salary={{ min: m.median_offer_min, max: m.median_offer_max }} size="lg" />
        </div>
      )}
      <p className="mt-4 flex items-start gap-2 text-sm text-ink-muted">
        <TrendingUp className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
        <span>Чем выше сила результата и свежее активность, тем выше вы в подборках работодателей.</span>
      </p>
    </Card>
  );
}

function FspBlock() {
  const q = useFspStatus();
  return (
    <Card id="fsp">
      <CardHeader
        title="Достижения ФСП"
        description="Поднимают вас внутри категории"
        action={
          q.data?.linked ? (
            <Link href="/candidate/settings#fsp" className="tap text-sm font-medium text-primary hover:underline">
              Управлять
            </Link>
          ) : undefined
        }
      />
      {q.isPending ? (
        <SkeletonCard lines={2} className="border-0 p-0" />
      ) : q.isError ? (
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      ) : !q.data.linked ? (
        <FspEmpty text="ФСП ID не привязан" action={<LinkFspButton size="sm" variant="soft" />} />
      ) : q.data.achievements.length === 0 ? (
        <FspEmpty text={`${q.data.fsp_id}: достижений пока нет — это не влияет на позицию негативно`} />
      ) : (
        <>
          <p className="mb-3 flex items-center gap-2 text-sm text-ink-muted">
            <Trophy className="size-4 text-primary" aria-hidden />
            <span className="font-mono">{q.data.fsp_id}</span> · {pluralize(q.data.achievements.length, ["достижение", "достижения", "достижений"])}
          </p>
          <FspAchievementList achievements={q.data.achievements} limit={3} />
        </>
      )}
    </Card>
  );
}

function TaskTeaser() {
  const q = useMyTasks();
  const open = q.data?.find((t) => t.status === "assigned");
  if (!open) return null;
  return (
    <Card className="border-primary-200 bg-primary-50/40">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-start gap-3">
          <ListChecks className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden />
          <div>
            <p className="font-medium text-ink">Новое задание: {open.title}</p>
            <p className="mt-0.5 text-sm text-ink-muted">
              {open.company_name ?? "Компания"} · до {formatDate(open.due_at)}. Ответ обновляет актуальность профиля.
            </p>
          </div>
        </div>
        <ButtonLink href="/candidate/tasks" variant="secondary" size="sm">
          Открыть
        </ButtonLink>
      </div>
    </Card>
  );
}

export default function CandidateDashboard() {
  const me = useMe();
  const hasCategory = Boolean(me.data?.candidate?.category);
  return (
    <div>
      <PageHeader
        eyebrow={<span className="eyebrow text-primary">Обзор</span>}
        title="Главная"
        description="Ваша категория, приглашения и спрос — в одном месте."
        actions={
          hasCategory ? (
            <ButtonLink href="/candidate/invitations" variant="secondary" icon={<Eye className="size-4" aria-hidden />}>
              Приглашения
            </ButtonLink>
          ) : undefined
        }
      />
      <OnboardingCard />
      <div className="grid gap-6 lg:grid-cols-[1.4fr_1fr]">
        <div className="min-w-0 space-y-6">
          <CategoryCard />
          <TaskTeaser />
          <MarketCard enabled={hasCategory} />
        </div>
        <div className="min-w-0 space-y-6">
          <InvitationsCard />
          <FspBlock />
        </div>
      </div>
    </div>
  );
}
