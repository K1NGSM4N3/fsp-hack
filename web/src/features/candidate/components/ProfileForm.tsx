"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { Lock, Plus, Trash2 } from "lucide-react";
import { useEffect, useImperativeHandle, type Ref } from "react";
import { Controller, useFieldArray, useForm } from "react-hook-form";
import { z } from "zod";
import { isApiError } from "@/api/client";
import type { Industry, Profile, ProfileInput, WorkFormat } from "@/api/types";
import { SkillInput, SkillLegend, SkillList } from "@/components/domain/Skills";
import { Button, Card, CardHeader, ChipSelect, Checkbox, Field, IconButton, Input, MoneyInput, Textarea, useToast } from "@/components/ui";
import { useUpdateProfile } from "@/features/candidate/hooks";
import { INDUSTRIES, INDUSTRY_LABEL, WORK_FORMATS, WORK_FORMAT_LABEL } from "@/lib/labels";
import type { ResumeApply } from "./ResumeUpload";

const MONTH = /^\d{4}-(0[1-9]|1[0-2])$/;

const schema = z
  .object({
    full_name: z.string().trim().min(2, "Укажите имя и фамилию"),
    email: z.union([z.literal(""), z.string().trim().email("Некорректный email")]),
    phone: z.string().trim(),
    telegram: z.string().trim(),
    city: z.string().trim(),
    relocation: z.boolean(),
    work_formats: z.array(z.enum(["office", "hybrid", "remote"])).min(1, "Выберите хотя бы один формат"),
    salary: z.number().int().nonnegative().nullable(),
    about: z.string().max(2000, "Не больше 2000 символов"),
    experience_years: z.string().refine((v) => v === "" || (Number(v) >= 0 && Number(v) <= 60), "От 0 до 60 лет"),
    roles: z.string(),
    soft_skills: z.string(),
    industries: z.array(z.string()),
    skills: z.array(z.string()),
    experience: z.array(
      z.object({
        company: z.string().trim().min(1, "Укажите компанию"),
        position: z.string().trim().min(1, "Укажите должность"),
        start: z.string().regex(MONTH, "Формат ГГГГ-ММ"),
        end: z.string().refine((v) => v === "" || MONTH.test(v), "Формат ГГГГ-ММ"),
        description: z.string(),
      }),
    ),
  })
  .refine((v) => v.email || v.phone || v.telegram, { message: "Нужен хотя бы один контакт", path: ["email"] });

export type ProfileFormValues = z.infer<typeof schema>;

const splitList = (s: string) =>
  s
    .split(",")
    .map((x) => x.trim())
    .filter(Boolean);

function toValues(p: Profile | undefined, accountEmail?: string): ProfileFormValues {
  return {
    full_name: p?.full_name ?? "",
    email: p?.contacts.email || accountEmail || "",
    phone: p?.contacts.phone ?? "",
    telegram: p?.contacts.telegram ?? "",
    city: p?.city ?? "",
    relocation: p?.relocation ?? false,
    work_formats: p?.work_formats?.length ? p.work_formats : ["remote"],
    salary: p?.salary_expectation_min ?? null,
    about: p?.about ?? "",
    experience_years: p?.experience_years !== null && p?.experience_years !== undefined ? String(p.experience_years) : "",
    roles: p?.roles?.join(", ") ?? "",
    soft_skills: p?.soft_skills?.join(", ") ?? "",
    industries: p?.industries ?? [],
    skills: p?.skills ?? [],
    experience: (p?.experience ?? []).map((e) => ({ ...e, end: e.end ?? "", description: e.description ?? "" })),
  };
}

function toInput(v: ProfileFormValues): ProfileInput {
  return {
    full_name: v.full_name.trim(),
    contacts: { email: v.email.trim() || null, phone: v.phone.trim() || null, telegram: v.telegram.trim() || null },
    city: v.city.trim() || null,
    relocation: v.relocation,
    work_formats: v.work_formats,
    salary_expectation_min: v.salary,
    about: v.about.trim(),
    experience_years: v.experience_years === "" ? null : Number(v.experience_years),
    roles: splitList(v.roles),
    soft_skills: splitList(v.soft_skills),
    industries: v.industries as Industry[],
    experience: v.experience.map((e) => ({ ...e, end: e.end || null })),
    skills: v.skills,
  };
}

export interface ProfileFormHandle {
  apply: (values: ResumeApply) => void;
}

export function ProfileForm({ profile, accountEmail, onSaved, handleRef }: { profile: Profile | undefined; accountEmail?: string; onSaved?: () => void; handleRef?: Ref<ProfileFormHandle> }) {
  const toast = useToast();
  const save = useUpdateProfile();
  const form = useForm<ProfileFormValues>({ resolver: zodResolver(schema), defaultValues: toValues(profile, accountEmail) });
  const { register, control, handleSubmit, formState, reset, setError, getValues, setValue } = form;
  const experience = useFieldArray({ control, name: "experience" });
  const errors = formState.errors;

  useEffect(() => {
    if (profile) reset(toValues(profile, accountEmail));
  }, [profile, accountEmail, reset]);

  useImperativeHandle(handleRef, () => ({
    apply: (r) => {
      const opts = { shouldDirty: true, shouldValidate: false } as const;
      if (r.full_name) setValue("full_name", r.full_name, opts);
      if (r.email) setValue("email", r.email, opts);
      if (r.phone) setValue("phone", r.phone, opts);
      if (r.telegram) setValue("telegram", r.telegram, opts);
      if (r.city) setValue("city", r.city, opts);
      if (r.skills) setValue("skills", [...new Set([...getValues("skills"), ...r.skills])], opts);
      if (r.experience_years !== undefined) setValue("experience_years", String(r.experience_years), opts);
      if (r.roles) setValue("roles", r.roles.join(", "), opts);
      if (r.soft_skills) setValue("soft_skills", r.soft_skills.join(", "), opts);
      if (r.industries) setValue("industries", [...new Set([...getValues("industries"), ...r.industries])], opts);
      if (r.work_formats) setValue("work_formats", r.work_formats, opts);
      if (r.salary) setValue("salary", r.salary, opts);
      if (r.relocation !== undefined) setValue("relocation", r.relocation, opts);
    },
  }));

  const submit = (values: ProfileFormValues) =>
    save.mutate(toInput(values), {
      onSuccess: () => {
        toast.success("Профиль сохранён");
        onSaved?.();
      },
      onError: (err) => {
        if (isApiError(err) && err.errors) {
          for (const [key, msgs] of Object.entries(err.errors)) {
            const path = key.replace(/^contacts\./, "") as keyof ProfileFormValues;
            setError(path, { message: msgs[0] });
          }
        }
        toast.error(isApiError(err) ? err.title : "Не удалось сохранить профиль");
      },
    });

  return (
    <form noValidate onSubmit={handleSubmit(submit)} className="space-y-6">
      <Card>
        <CardHeader title="Основное" />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Имя и фамилия" error={errors.full_name?.message} required className="sm:col-span-2">
            <Input autoComplete="name" {...register("full_name")} />
          </Field>
          <Field label="Город" optional>
            <Input autoComplete="address-level2" placeholder="Москва" {...register("city")} />
          </Field>
          <Field label="Стаж в ИТ, лет" error={errors.experience_years?.message} optional>
            <Input inputMode="decimal" placeholder="3" {...register("experience_years")} />
          </Field>
          <Field label="Роли" optional hint="Через запятую: Backend-разработчик, Тимлид" className="sm:col-span-2">
            <Input {...register("roles")} />
          </Field>
        </div>
      </Card>

      <Card>
        <CardHeader
          title="Контакты"
          description={
            <span className="inline-flex items-center gap-1.5">
              <Lock className="size-3.5" aria-hidden />
              Работодатель увидит их только после вашего согласия
            </span>
          }
        />
        <div className="grid gap-4 sm:grid-cols-3">
          <Field label="Email" error={errors.email?.message}>
            <Input type="email" autoComplete="email" {...register("email")} />
          </Field>
          <Field label="Телефон" error={errors.phone?.message}>
            <Input type="tel" autoComplete="tel" placeholder="+7 900 000-00-00" {...register("phone")} />
          </Field>
          <Field label="Telegram" error={errors.telegram?.message}>
            <Input placeholder="@username" {...register("telegram")} />
          </Field>
        </div>
      </Card>

      <Card>
        <CardHeader title="Условия работы" />
        <div className="grid gap-5 sm:grid-cols-2">
          <Field label="Формат" error={errors.work_formats?.message} required className="sm:col-span-2">
            <Controller
              control={control}
              name="work_formats"
              render={({ field }) => (
                <ChipSelect<WorkFormat> ariaLabel="Формат работы" options={WORK_FORMATS.map((f) => ({ value: f, label: WORK_FORMAT_LABEL[f] }))} value={field.value} onChange={field.onChange} />
              )}
            />
          </Field>
          <Field label="Зарплата от" optional hint="На руки, в месяц. Помогает работодателям сразу предлагать подходящую вилку">
            <Controller control={control} name="salary" render={({ field }) => <MoneyInput value={field.value} onChange={field.onChange} placeholder="250 000" />} />
          </Field>
          <div className="flex items-end pb-2">
            <Checkbox label="Готов к переезду" {...register("relocation")} />
          </div>
        </div>
      </Card>

      <Card>
        <CardHeader title="Навыки и опыт" description="Подтверждённые тестом и ФСП навыки добавляются автоматически" />
        <div className="space-y-5">
          <Field label="Стек" hint="Ищите по названию или синониму: «постгрес», «k8s», «реакт»">
            <Controller control={control} name="skills" render={({ field }) => <SkillInput value={field.value} onChange={field.onChange} />} />
          </Field>
          {profile && profile.skill_details.length > 0 && (
            <div className="rounded-control bg-canvas p-4">
              <p className="mb-2 text-sm font-medium text-ink">Как видят ваши навыки работодатели</p>
              <SkillList skills={profile.skill_details} />
              <div className="mt-3">
                <SkillLegend />
              </div>
            </div>
          )}
          <Field label="Отрасли" optional>
            <Controller
              control={control}
              name="industries"
              render={({ field }) => (
                <ChipSelect<Industry>
                  ariaLabel="Отрасли"
                  options={INDUSTRIES.map((i) => ({ value: i, label: INDUSTRY_LABEL[i] }))}
                  value={field.value as Industry[]}
                  onChange={field.onChange}
                />
              )}
            />
          </Field>
          <Field label="Софт-скиллы" optional hint="Через запятую">
            <Input {...register("soft_skills")} placeholder="Коммуникация, Наставничество" />
          </Field>
          <Field label="О себе" optional error={errors.about?.message} hint="2–3 предложения о том, какие задачи вам интересны">
            <Textarea rows={4} {...register("about")} />
          </Field>
        </div>
      </Card>

      <Card>
        <CardHeader
          title="Опыт работы"
          description="Названия компаний можно скрыть в настройках приватности"
          action={
            <Button variant="soft" size="sm" icon={<Plus className="size-4" aria-hidden />} onClick={() => experience.append({ company: "", position: "", start: "", end: "", description: "" })}>
              Добавить место
            </Button>
          }
        />
        {experience.fields.length === 0 ? (
          <p className="text-sm text-ink-subtle">Пока пусто. Добавьте последние места работы — так подборка будет точнее.</p>
        ) : (
          <ol className="space-y-4">
            {experience.fields.map((f, i) => (
              <li key={f.id} className="rounded-control border border-line p-4">
                <div className="mb-3 flex items-center justify-between">
                  <span className="font-mono text-sm font-semibold text-ink-subtle">#{i + 1}</span>
                  <IconButton label={`Удалить место работы ${i + 1}`} size="sm" onClick={() => experience.remove(i)}>
                    <Trash2 className="size-4" aria-hidden />
                  </IconButton>
                </div>
                <div className="grid gap-3 sm:grid-cols-2">
                  <Field label="Компания" error={errors.experience?.[i]?.company?.message} required>
                    <Input {...register(`experience.${i}.company`)} />
                  </Field>
                  <Field label="Должность" error={errors.experience?.[i]?.position?.message} required>
                    <Input {...register(`experience.${i}.position`)} />
                  </Field>
                  <Field label="Начало" error={errors.experience?.[i]?.start?.message} required>
                    <Input type="month" {...register(`experience.${i}.start`)} />
                  </Field>
                  <Field label="Окончание" error={errors.experience?.[i]?.end?.message} hint="Пусто — работаю сейчас">
                    <Input type="month" {...register(`experience.${i}.end`)} />
                  </Field>
                  <Field label="Чем занимались" optional className="sm:col-span-2">
                    <Textarea rows={3} {...register(`experience.${i}.description`)} />
                  </Field>
                </div>
              </li>
            ))}
          </ol>
        )}
      </Card>

      <div className="sticky bottom-4 z-10 flex flex-wrap items-center justify-end gap-3 rounded-card border border-line bg-white/95 p-3 shadow-pop backdrop-blur">
        {formState.isDirty && <span className="mr-auto text-sm text-ink-muted">Есть несохранённые изменения</span>}
        <Button variant="ghost" onClick={() => reset(toValues(profile, accountEmail))} disabled={!formState.isDirty || save.isPending}>
          Отменить
        </Button>
        <Button type="submit" loading={save.isPending}>
          Сохранить профиль
        </Button>
      </div>
    </form>
  );
}
