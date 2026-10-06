"use client";

import { Send } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { Category, HiringNeed, WorkFormat } from "@/api/types";
import { MinorBadge } from "@/components/domain/CandidateCard";
import { CategoryBadge } from "@/components/domain/Category";
import { Alert, Button, Dialog, Field, Input, MoneyInput, ProgressBar, Select, Textarea, useToast } from "@/components/ui";
import { SalaryHint } from "@/features/employer/components/SalaryHint";
import { useCompany, useInvitationStats, useNeeds, useOwnVacancies, useSendInvitation } from "@/features/employer/hooks";
import { WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";

export interface InviteTarget {
  id: string;
  code: string;
  display_name?: string | null;
  category?: Category | null;
  minor?: boolean;
}

interface Draft {
  title: string;
  message: string;
  min: number | null;
  max: number | null;
  work_format: WorkFormat | "";
  contact: string;
  link: string;
}

type Errors = Partial<Record<"title" | "message" | "min" | "max" | "contact", string>>;

function defaultContact(c: { contact_telegram?: string | null; contact_email?: string | null; contact_phone?: string | null } | undefined): string {
  if (!c) return "";
  if (c.contact_telegram) return `Telegram ${c.contact_telegram}`;
  return c.contact_email ?? c.contact_phone ?? "";
}

function validate(d: Draft): Errors {
  const e: Errors = {};
  if (d.title.trim().length < 3) e.title = "Укажите, на какую роль приглашаете";
  if (d.message.trim().length < 20) e.message = "Расскажите о задаче подробнее — хотя бы 20 символов";
  if (!d.min) e.min = "Укажите «от»";
  if (!d.max) e.max = "Укажите «до»";
  else if (d.min && d.max < d.min) e.max = "«До» не может быть меньше «от»";
  if (!d.contact.trim()) e.contact = "Как кандидату с вами связаться";
  return e;
}

function InviteForm({ target, need, onClose, onSent }: { target: InviteTarget; need?: HiringNeed | null; onClose: () => void; onSent?: () => void }) {
  const toast = useToast();
  const company = useCompany();
  const stats = useInvitationStats();
  const needs = useNeeds();
  const vacancies = useOwnVacancies();
  const send = useSendInvitation();
  const [key] = useState(() => (typeof crypto !== "undefined" && "randomUUID" in crypto ? crypto.randomUUID() : String(Date.now())));
  const [draft, setDraft] = useState<Draft>(() => ({
    title: need?.title ?? "",
    message: need ? `Здравствуйте! ${need.description || need.title}\n\nБудем рады рассказать подробнее.` : "",
    min: need?.salary.min ?? null,
    max: need?.salary.max ?? null,
    work_format: need?.work_formats[0] ?? "",
    contact: "",
    link: need ? `need:${need.id}` : "",
  }));
  const [errors, setErrors] = useState<Errors>({});
  const [conflict, setConflict] = useState<{ code: string; title: string; detail: string | null } | null>(null);

  const contact = draft.contact || defaultContact(company.data);
  const set = (patch: Partial<Draft>) => setDraft((d) => ({ ...d, ...patch }));
  const quotaLeft = stats.data ? stats.data.quota_limit - stats.data.quota_used : null;

  const submit = () => {
    const full = { ...draft, contact };
    const e = validate(full);
    setErrors(e);
    setConflict(null);
    if (Object.keys(e).length || !full.min || !full.max) return;
    const [kind, linkId] = full.link.split(":");
    send.mutate(
      {
        key,
        body: {
          candidate_id: target.id,
          need_id: kind === "need" ? linkId : null,
          vacancy_id: kind === "vacancy" ? linkId : null,
          title: full.title.trim(),
          message: full.message.trim(),
          salary: { min: full.min, max: full.max },
          work_format: full.work_format || null,
          contact: full.contact.trim(),
        },
      },
      {
        onSuccess: () => {
          toast.success("Приглашение отправлено", "Кандидат увидит вилку и описание до начала общения");
          onSent?.();
          onClose();
        },
        onError: (err) => {
          if (!isApiError(err)) return toast.error("Не удалось отправить приглашение");
          if (err.status === 409 || err.status === 404) setConflict({ code: err.code, title: err.title, detail: err.detail });
          else if (err.errors) {
            setErrors({
              title: err.fieldError("title"),
              message: err.fieldError("message"),
              min: err.fieldError("salary.min") ?? err.fieldError("salary"),
              max: err.fieldError("salary.max"),
              contact: err.fieldError("contact"),
            });
          } else toast.error(err.title, err.detail ?? undefined);
        },
      },
    );
  };

  return (
    <form
      id="invite-form"
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
      className="space-y-5"
    >
      <div className="flex flex-wrap items-center justify-between gap-3 rounded-control bg-canvas p-3">
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <span className="font-mono font-semibold text-ink">{target.display_name ?? `Кандидат ${target.code}`}</span>
          {target.category && <CategoryBadge category={target.category} size="sm" />}
          {target.minor && <MinorBadge />}
        </div>
        {quotaLeft !== null && stats.data && (
          <div className="w-40">
            <ProgressBar value={stats.data.quota_used} max={stats.data.quota_limit} size="xs" tone={quotaLeft <= 3 ? "danger" : "primary"} label="Квота приглашений" />
            <p className="mt-1 text-right text-xs text-ink-subtle">
              Осталось {quotaLeft} из {stats.data.quota_limit} на неделе
            </p>
          </div>
        )}
      </div>

      {target.minor && (
        <Alert tone="info" title="Кандидату нет 18">
          Трудовой кодекс бережёт работников до 18 лет: до 16 лет — только лёгкая работа в свободное от учёбы время и с согласия родителей, рабочая неделя и смена короче, вредные и опасные работы
          запрещены (ТК РФ, ст. 63, 92, 94, 265). Учтите это в описании задачи и графика.
        </Alert>
      )}

      {conflict && (
        <Alert
          tone={conflict.code === "invitation_quota_exceeded" ? "danger" : "warning"}
          title={conflict.title}
          action={
            conflict.code === "invitation_already_active" ? (
              <Link href="/employer/invitations" className="tap text-sm font-medium text-primary hover:underline">
                Открыть приглашения
              </Link>
            ) : conflict.code === "invitation_quota_exceeded" ? (
              <Link href="/employer/trust" className="tap text-sm font-medium text-primary hover:underline">
                Как поднять квоту
              </Link>
            ) : undefined
          }
        >
          {conflict.detail}
        </Alert>
      )}

      <Field label="Роль" error={errors.title} required>
        <Input value={draft.title} onChange={(e) => set({ title: e.target.value })} placeholder="Backend-разработчик в команду платежей" />
      </Field>

      <fieldset>
        <legend className="mb-1.5 text-sm font-medium text-ink">
          Зарплата, ₽ в месяц<span className="ml-0.5 text-accent-strong">*</span>
        </legend>
        <div className="grid grid-cols-2 gap-3">
          <Field label="от" error={errors.min}>
            <MoneyInput value={draft.min} onChange={(v) => set({ min: v })} placeholder="250 000" />
          </Field>
          <Field label="до" error={errors.max}>
            <MoneyInput value={draft.max} onChange={(v) => set({ max: v })} placeholder="320 000" />
          </Field>
        </div>
        <p className="mt-1.5 text-xs text-ink-subtle">Обязательно: кандидат видит вилку до начала общения</p>
        <SalaryHint category={target.category} min={draft.min} max={draft.max} className="mt-3" />
      </fieldset>

      <Field label="Сообщение кандидату" error={errors.message} required hint="Чем занимается команда, какая задача, почему именно этот кандидат">
        <Textarea rows={5} value={draft.message} onChange={(e) => set({ message: e.target.value })} maxLength={3000} />
      </Field>

      <div className="grid gap-4 sm:grid-cols-2">
        <Field label="Формат">
          <Select value={draft.work_format} onChange={(e) => set({ work_format: e.target.value as WorkFormat | "" })}>
            <option value="">Обсуждается</option>
            {WORK_FORMATS.map((f) => (
              <option key={f} value={f}>
                {WORK_FORMAT_LABEL[f]}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Способ связи" error={errors.contact} required hint="Виден кандидату сразу">
          <Input value={contact} onChange={(e) => set({ contact: e.target.value })} placeholder="Telegram @hr или email" />
        </Field>
      </div>

      <Field label="Связать с потребностью или вакансией" optional hint="Не обязательно: приглашать можно и без вакансии">
        <Select value={draft.link} onChange={(e) => set({ link: e.target.value })}>
          <option value="">Без привязки</option>
          {(needs.data ?? []).length > 0 && (
            <optgroup label="Потребности">
              {needs.data?.map((n) => (
                <option key={n.id} value={`need:${n.id}`}>
                  {n.title}
                </option>
              ))}
            </optgroup>
          )}
          {(vacancies.data ?? []).filter((v) => v.status === "published").length > 0 && (
            <optgroup label="Вакансии">
              {vacancies.data
                ?.filter((v) => v.status === "published")
                .map((v) => (
                  <option key={v.id} value={`vacancy:${v.id}`}>
                    {v.title}
                  </option>
                ))}
            </optgroup>
          )}
        </Select>
      </Field>

      <div className="sticky bottom-0 z-10 -mx-5 -mb-5 flex flex-wrap justify-end gap-2 border-t border-line bg-white px-5 py-3 sm:-mx-6 sm:px-6">
        <Button variant="ghost" onClick={onClose}>
          Отмена
        </Button>
        <Button type="submit" loading={send.isPending} icon={<Send className="size-4" aria-hidden />} disabled={quotaLeft === 0}>
          Отправить приглашение
        </Button>
      </div>
    </form>
  );
}

export function InviteDialog({ open, onClose, target, need, onSent }: { open: boolean; onClose: () => void; target: InviteTarget | null; need?: HiringNeed | null; onSent?: () => void }) {
  return (
    <Dialog open={open && target !== null} onClose={onClose} title="Приглашение кандидату" description="Контакты кандидата откроются, когда он примет приглашение." size="lg">
      {target && <InviteForm key={target.id} target={target} need={need} onClose={onClose} onSent={onSent} />}
    </Dialog>
  );
}
