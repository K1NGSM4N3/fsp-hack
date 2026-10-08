"use client";

import { ArrowRight, ScrollText, ShieldAlert, Video } from "lucide-react";
import type { ReactNode } from "react";
import { ButtonLink, Card, CardHeader, ErrorAlert, PageHeader, PageSkeleton, Stat } from "@/components/ui";
import { useOverview } from "@/features/admin/hooks";
import { DECISION_EFFECT, DECISION_LABEL } from "@/features/admin/labels";

function Section({ icon, title, description, href, action, children }: { icon: ReactNode; title: string; description: string; href: string; action: string; children: ReactNode }) {
  return (
    <Card padding="lg" className="flex flex-col">
      <CardHeader
        title={
          <span className="flex items-center gap-2">
            <span className="text-primary">{icon}</span>
            {title}
          </span>
        }
        description={description}
      />
      <div className="grid grid-cols-2 gap-x-4 gap-y-5">{children}</div>
      <ButtonLink href={href} variant="secondary" className="mt-6 self-start" icon={<ArrowRight className="size-4" aria-hidden />}>
        {action}
      </ButtonLink>
    </Card>
  );
}

export default function AdminOverviewPage() {
  const q = useOverview();
  if (q.isPending) return <PageSkeleton />;
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  const { proctoring: p, antifake: a, log: l } = q.data;
  return (
    <div>
      <PageHeader title="Обзор" description="Что ждёт решения и что модераторы сделали за последние 7 дней. Каждое действие и каждый просмотр записи теста попадают в журнал." />
      <div className="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
        <Section icon={<Video className="size-5" aria-hidden />} title="Прокторинг" description="Тесты, в которых ответы и поведение расходятся" href="/admin/proctoring" action="Очередь тестов">
          <Stat label="Требуют подтверждения" value={p.needs_review} tone={p.needs_review ? "primary" : "default"} />
          <Stat label="С замечаниями" value={p.remarks} />
          <Stat label="С видеозаписью" value={p.with_video} />
          <Stat label="Решений за неделю" value={p.decided_week} />
        </Section>
        <Section icon={<ShieldAlert className="size-5" aria-hidden />} title="Антифейк и жалобы" description="Отметки антифейка и жалобы кандидатов" href="/admin/antifake" action="Очередь антифейка">
          <Stat label="Открытые жалобы" value={a.complaints} tone={a.complaints ? "primary" : "default"} />
          <Stat label="Вакансии с отметками" value={a.vacancies} />
          <Stat label="Приглашения с отметками" value={a.invitations} />
          <Stat label="Компании с замечаниями" value={a.companies} />
          <Stat label="Снятые вакансии" value={a.unpublished_vacancies} hint="действующие меры" />
          <Stat label="Компании на паузе" value={a.frozen_companies} hint="действующие меры" />
        </Section>
        <Section icon={<ScrollText className="size-5" aria-hidden />} title="Журнал" description="Кто, когда и что сделал — с комментарием" href="/admin/log" action="Открыть журнал">
          <Stat label="Действий за неделю" value={l.actions_week} />
          <Stat label="Просмотров записей" value={l.recording_views_week} hint="за неделю" />
        </Section>
      </div>

      <Card className="mt-6">
        <CardHeader title="Как работают решения по тестам" description="Решение окончательное. Комментарий к нему кандидат увидит в истории тестов, работодатели — только отметку «Результат подтверждён»." />
        <dl className="grid gap-4 text-sm md:grid-cols-3">
          {(["confirm", "annul", "retest"] as const).map((d) => (
            <div key={d} className="rounded-control bg-canvas p-4">
              <dt className="font-medium text-ink">{DECISION_LABEL[d]}</dt>
              <dd className="mt-1 text-ink-muted">{DECISION_EFFECT[d]}</dd>
            </div>
          ))}
        </dl>
      </Card>
    </div>
  );
}
