"use client";

import { Building2, CheckCircle2, MapPin, Send } from "lucide-react";
import { useState } from "react";
import { isApiError } from "@/api/client";
import { Alert, Button, ButtonLink, Card, CardHeader, Chip, ErrorAlert, Field, PageHeader, PageSkeleton, Textarea, useToast } from "@/components/ui";
import { ReportButton } from "@/components/domain/ReportButton";
import { useMe } from "@/features/auth/hooks";
import { useMyApplications } from "@/features/candidate/hooks";
import { useSkillNamePreload } from "@/features/catalog/hooks";
import { useApply, useVacancy } from "@/features/vacancies/hooks";
import { formatDate } from "@/lib/format";
import { GRADE_LABEL, SPEC_LONG_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";
import { CategoryBadge, SalaryRange } from "./Category";
import { DomainBadge } from "./DomainBadge";
import { ApplicationStatusBadge } from "./StatusBadges";

function ApplyBox({ vacancyId, loginNext }: { vacancyId: string; loginNext: string }) {
  const toast = useToast();
  const me = useMe();
  const isCandidate = me.data?.role === "candidate";
  const apps = useMyApplications(isCandidate);
  const apply = useApply();
  const [cover, setCover] = useState("");
  const mine = isCandidate ? apps.data?.find((a) => a.vacancy_id === vacancyId && a.status !== "withdrawn") : undefined;

  if (me.isPending) return null;
  if (!me.data) {
    return (
      <Card>
        <CardHeader title="Откликнуться" description="Отклик раскрывает компании ваши контакты — так же, как принятое приглашение." />
        <ButtonLink href={`/login?next=${encodeURIComponent(loginNext)}`} className="w-full">
          Войти как кандидат
        </ButtonLink>
        <ButtonLink href="/register?role=candidate" variant="ghost" className="mt-2 w-full">
          Создать профиль
        </ButtonLink>
      </Card>
    );
  }
  if (!isCandidate) {
    return <Alert title="Вы вошли как работодатель">Откликаться на вакансии могут только кандидаты.</Alert>;
  }
  if (mine) {
    return (
      <Card>
        <div className="flex items-start gap-3">
          <CheckCircle2 className="mt-0.5 size-5 text-success" aria-hidden />
          <div>
            <p className="font-medium text-ink">Вы откликнулись {formatDate(mine.created_at)}</p>
            <div className="mt-2">
              <ApplicationStatusBadge status={mine.status} />
            </div>
            <p className="mt-2 text-sm text-ink-muted">Статус обновится, когда компания посмотрит отклик.</p>
          </div>
        </div>
      </Card>
    );
  }
  return (
    <Card>
      <CardHeader title="Откликнуться" description="Компания увидит вашу категорию и контакты — вы откликаетесь сами." />
      <form
        noValidate
        onSubmit={(e) => {
          e.preventDefault();
          apply.mutate(
            { id: vacancyId, coverLetter: cover },
            {
              onSuccess: () => toast.success("Отклик отправлен", "Следите за статусом в разделе «Вакансии и отклики»"),
              onError: (err) => toast.error(isApiError(err) ? err.title : "Не удалось отправить отклик", isApiError(err) ? (err.detail ?? undefined) : undefined),
            },
          );
        }}
        className="space-y-4"
      >
        <Field label="Сопроводительное письмо" optional hint="Пара предложений о том, почему вам интересна задача">
          <Textarea value={cover} onChange={(e) => setCover(e.target.value)} rows={4} maxLength={2000} />
        </Field>
        <Button type="submit" loading={apply.isPending} className="w-full" icon={<Send className="size-4" aria-hidden />}>
          Откликнуться
        </Button>
      </form>
    </Card>
  );
}

export function VacancyDetail({ id, backHref, backLabel }: { id: string; backHref: string; backLabel: string }) {
  useSkillNamePreload();
  const query = useVacancy(id);
  if (query.isPending) return <PageSkeleton cards={2} />;
  if (query.isError) {
    if (isApiError(query.error) && query.error.status === 404) {
      return (
        <div>
          <PageHeader title="Вакансия не найдена" description="Возможно, её сняли с публикации." back={{ href: backHref, label: backLabel }} />
        </div>
      );
    }
    return <ErrorAlert error={query.error} onRetry={() => void query.refetch()} />;
  }
  const v = query.data;
  return (
    <div>
      <PageHeader
        back={{ href: backHref, label: backLabel }}
        eyebrow={<CategoryBadge category={{ specialization: v.specialization, grade: v.grade }} />}
        title={v.title}
        description={
          <span className="flex flex-wrap items-center gap-x-4 gap-y-1">
            <span className="inline-flex items-center gap-1.5">
              <Building2 className="size-4" aria-hidden />
              {v.company_name}
            </span>
            <DomainBadge domain={v.company_verified_domain} />
            <span className="inline-flex items-center gap-1.5">
              <MapPin className="size-4" aria-hidden />
              {WORK_FORMAT_LABEL[v.work_format]}
              {v.city ? `, ${v.city}` : ""}
            </span>
          </span>
        }
      />
      <div className="grid gap-6 lg:grid-cols-[1fr_20rem]">
        <div className="min-w-0 space-y-6">
          <Card>
            <p className="text-xs text-ink-subtle">Зарплата</p>
            <SalaryRange salary={v.salary} size="xl" className="mt-1 block" label="Зарплата" />
            <dl className="mt-5 grid gap-4 text-sm sm:grid-cols-3">
              <div>
                <dt className="text-ink-subtle">Специализация</dt>
                <dd className="mt-0.5 text-ink">{SPEC_LONG_LABEL[v.specialization]}</dd>
              </div>
              <div>
                <dt className="text-ink-subtle">Грейд</dt>
                <dd className="mt-0.5 text-ink">{GRADE_LABEL[v.grade]}</dd>
              </div>
              <div>
                <dt className="text-ink-subtle">Опубликована</dt>
                <dd className="mt-0.5 text-ink">{formatDate(v.published_at)}</dd>
              </div>
            </dl>
          </Card>
          <Card>
            <CardHeader title="Описание" />
            <div className="space-y-3 text-sm leading-relaxed whitespace-pre-line text-ink sm:text-base">{v.description}</div>
            {v.skills.length > 0 && (
              <div className="mt-5">
                <p className="mb-2 text-sm font-medium text-ink">Стек</p>
                <div className="flex flex-wrap gap-1.5">
                  {v.skills.map((s) => (
                    <Chip key={s}>{skillName(s)}</Chip>
                  ))}
                </div>
              </div>
            )}
          </Card>
          {v.company_description && (
            <Card>
              <CardHeader title={`О компании «${v.company_name}»`} action={<DomainBadge domain={v.company_verified_domain} />} />
              <p className="text-sm leading-relaxed text-ink-muted">{v.company_description}</p>
            </Card>
          )}
        </div>
        <div className="space-y-4">
          <ApplyBox vacancyId={v.id} loginNext={`/vacancies/${v.id}`} />
          <div className="flex justify-end">
            <ReportButton targetKind="vacancy" targetId={v.id} />
          </div>
        </div>
      </div>
    </div>
  );
}
