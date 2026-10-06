"use client";

import { ShieldCheck } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { Company, CompanyProfile } from "@/api/types";
import { Button, Card, CardHeader, ErrorAlert, Field, Input, PageHeader, PageSkeleton, Textarea, useToast } from "@/components/ui";
import { DataExportCard } from "@/features/auth/components/DataExportCard";
import { useCompany, useUpdateCompany } from "@/features/employer/hooks";
import { NotificationSettingsCard } from "@/features/notifications/components/NotificationSettingsCard";
import { formatDateTime } from "@/lib/format";

type Draft = Record<keyof CompanyProfile, string>;

function toDraft(c: Company): Draft {
  return {
    name: c.name,
    description: c.description,
    industry: c.industry ?? "",
    website: c.website ?? "",
    city: c.city ?? "",
    size: c.size ?? "",
    contact_email: c.contact_email ?? "",
    contact_phone: c.contact_phone ?? "",
    contact_telegram: c.contact_telegram ?? "",
  };
}

function toProfile(d: Draft): CompanyProfile {
  const opt = (v: string) => v.trim() || null;
  return {
    name: d.name.trim(),
    description: d.description.trim(),
    industry: opt(d.industry),
    website: opt(d.website),
    city: opt(d.city),
    size: opt(d.size),
    contact_email: opt(d.contact_email),
    contact_phone: opt(d.contact_phone),
    contact_telegram: opt(d.contact_telegram),
  };
}

function CompanyForm({ company }: { company: Company }) {
  const toast = useToast();
  const save = useUpdateCompany();
  const [d, setD] = useState<Draft>(() => toDraft(company));
  const [nameError, setNameError] = useState<string | null>(null);
  const field = (k: keyof Draft) => ({ value: d[k], onChange: (e: { target: { value: string } }) => setD((x) => ({ ...x, [k]: e.target.value })) });
  const serverErr = (k: string) => (isApiError(save.error) ? save.error.fieldError(k) : undefined);
  return (
    <form
      noValidate
      className="max-w-3xl space-y-6"
      onSubmit={(e) => {
        e.preventDefault();
        if (!d.name.trim()) {
          setNameError("Укажите название");
          return;
        }
        setNameError(null);
        save.mutate(toProfile(d), { onSuccess: () => toast.success("Профиль компании сохранён"), onError: (err) => toast.error(isApiError(err) ? err.title : "Не удалось сохранить") });
      }}
    >
      <Card>
        <CardHeader title="О компании" description="Видно кандидату в каждом приглашении и вакансии" />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Название" required error={nameError ?? serverErr("name")} className="sm:col-span-2">
            <Input {...field("name")} autoComplete="organization" />
          </Field>
          <Field label="Направление деятельности" optional>
            <Input {...field("industry")} placeholder="Финтех" />
          </Field>
          <Field label="Размер" optional>
            <Input {...field("size")} placeholder="100–500 сотрудников" />
          </Field>
          <Field label="Город" optional>
            <Input {...field("city")} />
          </Field>
          <Field
            label="Сайт"
            optional
            error={serverErr("website")}
            hint={
              company.verified_domain ? (
                <span className="inline-flex items-center gap-1 text-success">
                  <ShieldCheck className="size-3.5" aria-hidden />
                  Домен {company.verified_domain} подтверждён
                </span>
              ) : (
                <Link href="/employer/trust" className="link-inline">
                  Сайт на домене вашей почты подтверждает компанию
                </Link>
              )
            }
          >
            <Input {...field("website")} type="url" placeholder="https://" />
          </Field>
          <Field label="Описание" optional className="sm:col-span-2" hint="Чем занимаетесь, продукт, команда. 2–4 предложения">
            <Textarea rows={5} {...field("description")} />
          </Field>
        </div>
      </Card>
      <Card>
        <CardHeader title="Контакты для кандидатов" description="Подставляются как способ связи в приглашение — их видно кандидату сразу" />
        <div className="grid gap-4 sm:grid-cols-3">
          <Field label="Email" optional error={serverErr("contact_email")}>
            <Input {...field("contact_email")} type="email" />
          </Field>
          <Field label="Телефон" optional>
            <Input {...field("contact_phone")} type="tel" />
          </Field>
          <Field label="Telegram" optional>
            <Input {...field("contact_telegram")} placeholder="@company_hr" />
          </Field>
        </div>
      </Card>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <span className="text-sm text-ink-subtle">Обновлено {formatDateTime(company.updated_at)}</span>
        <Button type="submit" loading={save.isPending}>
          Сохранить
        </Button>
      </div>
    </form>
  );
}

export default function CompanyPage() {
  const q = useCompany();
  return (
    <div>
      <PageHeader title="Профиль компании" description="Верификация компаний в MVP не требуется. Квота приглашений защищает кандидатов от спама." />
      {q.isPending ? <PageSkeleton cards={2} /> : q.isError ? <ErrorAlert error={q.error} onRetry={() => void q.refetch()} /> : <CompanyForm company={q.data} />}
      <div className="mt-10 max-w-3xl space-y-6">
        <NotificationSettingsCard />
        <DataExportCard description="JSON-файл со всем, что платформа хранит о вашем аккаунте и компании: email, согласия, профиль компании, потребности, вакансии, приглашения и задания." />
      </div>
    </div>
  );
}
