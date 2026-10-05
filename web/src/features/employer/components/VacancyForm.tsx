"use client";

import Link from "next/link";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { Grade, SimilarVacancy, Specialization, Vacancy, VacancyInput, VacancyStatus, WorkFormat } from "@/api/types";
import { SkillInput } from "@/components/domain/Skills";
import { Alert, Button, Card, CardHeader, Field, Input, MoneyInput, RadioCards, Select, Textarea, useToast } from "@/components/ui";
import { SalaryHint } from "@/features/employer/components/SalaryHint";
import { useSaveVacancy, useSimilarVacancies } from "@/features/employer/hooks";
import { GRADE_LABEL, GRADES, SPEC_LONG_LABEL, SPECIALIZATIONS, VACANCY_STATUS_LABEL, WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";

interface Draft {
  title: string;
  description: string;
  specialization: Specialization;
  grade: Grade;
  skills: string[];
  min: number | null;
  max: number | null;
  work_format: WorkFormat;
  city: string;
  status: VacancyStatus;
}

type Errors = Partial<Record<"title" | "description" | "min" | "max", string>>;

function toDraft(v?: Vacancy): Draft {
  return {
    title: v?.title ?? "",
    description: v?.description ?? "",
    specialization: v?.specialization ?? "backend",
    grade: v?.grade ?? "middle",
    skills: v?.skills ?? [],
    min: v?.salary.min ?? null,
    max: v?.salary.max ?? null,
    work_format: v?.work_format ?? "hybrid",
    city: v?.city ?? "",
    status: v?.status ?? "published",
  };
}

const STATUS_HINT: Record<VacancyStatus, string> = {
  draft: "Видна только вам",
  published: "В общем списке вакансий, кандидаты могут откликаться",
  closed: "Снята с публикации, отклики сохраняются",
};

export function VacancyForm({ vacancy, onSaved }: { vacancy?: Vacancy; onSaved: (v: Vacancy) => void }) {
  const toast = useToast();
  const save = useSaveVacancy(vacancy?.id);
  const similar = useSimilarVacancies();
  const [d, setD] = useState<Draft>(() => toDraft(vacancy));
  const [errors, setErrors] = useState<Errors>({});
  const [twins, setTwins] = useState<SimilarVacancy[] | null>(null);
  const set = (patch: Partial<Draft>) => {
    setD((x) => ({ ...x, ...patch }));
    if ("title" in patch || "description" in patch) setTwins(null);
  };

  const submit = () => {
    const e: Errors = {};
    if (d.title.trim().length < 3) e.title = "Укажите название";
    if (d.description.trim().length < 20) e.description = "Опишите задачи — хотя бы 20 символов";
    if (!d.min) e.min = "Зарплата обязательна";
    if (!d.max) e.max = "Зарплата обязательна";
    else if (d.min && d.max < d.min) e.max = "«До» меньше «от»";
    setErrors(e);
    if (Object.keys(e).length || !d.min || !d.max) return;
    const input: VacancyInput = {
      title: d.title.trim(),
      description: d.description.trim(),
      specialization: d.specialization,
      grade: d.grade,
      skills: d.skills,
      salary: { min: d.min, max: d.max },
      work_format: d.work_format,
      city: d.city.trim() || null,
      status: d.status,
    };
    if (twins === null && input.status !== "closed") {
      similar.mutate(
        { title: input.title, description: input.description, exclude_id: vacancy?.id ?? null },
        {
          onSuccess: (found) => {
            setTwins(found);
            if (found.length === 0) persist(input);
          },
          onError: () => persist(input),
        },
      );
      return;
    }
    persist(input);
  };

  const persist = (input: VacancyInput) => {
    save.mutate(input, {
      onSuccess: (v) => {
        toast.success(vacancy ? "Вакансия сохранена" : "Вакансия создана");
        onSaved(v);
      },
      onError: (err) => {
        if (isApiError(err) && err.errors) {
          setErrors({ title: err.fieldError("title"), description: err.fieldError("description"), min: err.fieldError("salary.min"), max: err.fieldError("salary.max") });
        }
        toast.error(isApiError(err) ? err.title : "Не удалось сохранить");
      },
    });
  };

  return (
    <form
      noValidate
      className="space-y-6"
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <Card>
        <CardHeader title="Вакансия" />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Название" required error={errors.title} className="sm:col-span-2">
            <Input value={d.title} onChange={(e) => set({ title: e.target.value })} />
          </Field>
          <Field label="Специализация" required>
            <Select value={d.specialization} onChange={(e) => set({ specialization: e.target.value as Specialization })}>
              {SPECIALIZATIONS.map((s) => (
                <option key={s} value={s}>
                  {SPEC_LONG_LABEL[s]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Грейд" required>
            <Select value={d.grade} onChange={(e) => set({ grade: e.target.value as Grade })}>
              {GRADES.map((g) => (
                <option key={g} value={g}>
                  {GRADE_LABEL[g]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Описание" required error={errors.description} hint="Задачи, команда, условия" className="sm:col-span-2">
            <Textarea rows={7} value={d.description} onChange={(e) => set({ description: e.target.value })} />
          </Field>
          <Field label="Стек" optional className="sm:col-span-2">
            <SkillInput value={d.skills} onChange={(skills) => set({ skills })} />
          </Field>
        </div>
      </Card>
      <Card>
        <CardHeader title="Условия" description="Вилка обязательна и видна кандидатам" />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Зарплата от, ₽" required error={errors.min}>
            <MoneyInput value={d.min} onChange={(min) => set({ min })} />
          </Field>
          <Field label="Зарплата до, ₽" required error={errors.max}>
            <MoneyInput value={d.max} onChange={(max) => set({ max })} />
          </Field>
          <SalaryHint category={{ specialization: d.specialization, grade: d.grade }} min={d.min} max={d.max} className="sm:col-span-2" />
          <Field label="Формат">
            <Select value={d.work_format} onChange={(e) => set({ work_format: e.target.value as WorkFormat })}>
              {WORK_FORMATS.map((f) => (
                <option key={f} value={f}>
                  {WORK_FORMAT_LABEL[f]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Город" optional>
            <Input value={d.city} onChange={(e) => set({ city: e.target.value })} />
          </Field>
        </div>
      </Card>
      <Card>
        <CardHeader title="Публикация" />
        <RadioCards<VacancyStatus>
          name="status"
          columns={3}
          value={d.status}
          onChange={(status) => set({ status })}
          options={(["draft", "published", "closed"] as VacancyStatus[]).map((s) => ({ value: s, title: VACANCY_STATUS_LABEL[s], description: STATUS_HINT[s] }))}
        />
      </Card>
      {twins && twins.length > 0 && (
        <Alert tone="warning" title="Похожая вакансия уже есть">
          <p>Текст почти совпадает с вашей вакансией. Возможно, удобнее обновить её — тогда отклики соберутся в одном месте, а кандидаты не увидят двух одинаковых предложений.</p>
          <ul className="mt-2 space-y-1">
            {twins.map((t) => (
              <li key={t.id}>
                <Link href={`/employer/vacancies/${t.id}`} className="font-medium text-primary hover:underline">
                  «{t.title}»
                </Link>
                <span className="text-ink-subtle">
                  {" "}
                  · {VACANCY_STATUS_LABEL[t.status].toLowerCase()} · совпадение {Math.round(t.similarity * 100)} %
                </span>
              </li>
            ))}
          </ul>
        </Alert>
      )}
      <div className="flex justify-end">
        <Button type="submit" size="lg" loading={save.isPending || similar.isPending}>
          {twins && twins.length > 0 ? (vacancy ? "Всё равно сохранить" : "Всё равно создать") : vacancy ? "Сохранить" : "Создать вакансию"}
        </Button>
      </div>
    </form>
  );
}
