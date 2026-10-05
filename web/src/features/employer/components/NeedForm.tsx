"use client";

import { useState } from "react";
import { isApiError, type ApiError } from "@/api/client";
import type { Grade, Industry, NeedInput, Specialization, WorkFormat } from "@/api/types";
import { SkillInput } from "@/components/domain/Skills";
import { Button, Card, CardHeader, Checkbox, ChipSelect, Field, Input, MoneyInput, Select, Textarea } from "@/components/ui";
import { GRADE_LABEL, GRADES, INDUSTRIES, INDUSTRY_LABEL, SPEC_LONG_LABEL, SPECIALIZATIONS, WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";

export interface NeedDraft {
  title: string;
  specialization: Specialization | "";
  grades: Grade[];
  allow_adjacent_grades: boolean;
  must_skills: string[];
  nice_skills: string[];
  min: number | null;
  max: number | null;
  work_formats: WorkFormat[];
  city: string;
  description: string;
  industries: Industry[];
}

export const EMPTY_NEED: NeedDraft = {
  title: "",
  specialization: "",
  grades: [],
  allow_adjacent_grades: false,
  must_skills: [],
  nice_skills: [],
  min: null,
  max: null,
  work_formats: ["remote", "hybrid"],
  city: "",
  description: "",
  industries: [],
};

type Errors = Partial<Record<"title" | "specialization" | "grades" | "must_skills" | "min" | "max", string>>;

function validate(d: NeedDraft): Errors {
  const e: Errors = {};
  if (!d.title.trim()) e.title = "Как назвать потребность";
  if (!d.specialization) e.specialization = "Выберите специализацию";
  if (!d.grades.length) e.grades = "Выберите хотя бы один грейд";
  if (!d.must_skills.length) e.must_skills = "Добавьте хотя бы один обязательный навык";
  if (!d.min) e.min = "Укажите «от»";
  if (!d.max) e.max = "Укажите «до»";
  else if (d.min && d.max < d.min) e.max = "«До» меньше «от»";
  return e;
}

export function toNeedInput(d: NeedDraft): NeedInput | null {
  if (!d.specialization || !d.min || !d.max) return null;
  return {
    title: d.title.trim(),
    specialization: d.specialization,
    grades: d.grades,
    allow_adjacent_grades: d.allow_adjacent_grades,
    must_skills: d.must_skills,
    nice_skills: d.nice_skills,
    salary: { min: d.min, max: d.max },
    work_formats: d.work_formats,
    city: d.city.trim() || null,
    description: d.description.trim(),
    industries: d.industries,
  };
}

interface NeedFormProps {
  initial: NeedDraft;
  submitLabel: string;
  pending: boolean;
  onSubmit: (input: NeedInput) => void;
  serverError?: ApiError | null;
  onCancel?: () => void;
}

export function NeedForm({ initial, submitLabel, pending, onSubmit, serverError, onCancel }: NeedFormProps) {
  const [d, setD] = useState<NeedDraft>(initial);
  const [errors, setErrors] = useState<Errors>({});
  const set = (patch: Partial<NeedDraft>) => setD((x) => ({ ...x, ...patch }));
  const fieldError = (key: string) => (serverError && isApiError(serverError) ? serverError.fieldError(key) : undefined);

  return (
    <form
      noValidate
      className="space-y-6"
      onSubmit={(e) => {
        e.preventDefault();
        const errs = validate(d);
        setErrors(errs);
        const input = toNeedInput(d);
        if (!Object.keys(errs).length && input) onSubmit(input);
      }}
    >
      <Card>
        <CardHeader title="Кого ищете" />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Название" error={errors.title ?? fieldError("title")} required className="sm:col-span-2">
            <Input value={d.title} onChange={(e) => set({ title: e.target.value })} placeholder="Backend-разработчик в команду платежей" />
          </Field>
          <Field label="Специализация" error={errors.specialization} required>
            <Select value={d.specialization} onChange={(e) => set({ specialization: e.target.value as Specialization | "" })}>
              <option value="">Выберите…</option>
              {SPECIALIZATIONS.map((s) => (
                <option key={s} value={s}>
                  {SPEC_LONG_LABEL[s]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Грейд" error={errors.grades ?? fieldError("grades")} required>
            <ChipSelect<Grade> ariaLabel="Грейд" options={GRADES.map((g) => ({ value: g, label: GRADE_LABEL[g] }))} value={d.grades} onChange={(grades) => set({ grades })} />
          </Field>
          <Checkbox
            className="sm:col-span-2"
            checked={d.allow_adjacent_grades}
            onChange={(e) => set({ allow_adjacent_grades: e.target.checked })}
            label="Рассматривать соседние грейды"
            description="Например, сильного Junior на позицию Middle — с пометкой в объяснении"
          />
        </div>
      </Card>

      <Card>
        <CardHeader title="Стек" description="Навыки распознаются с синонимами: «постгрес» → PostgreSQL" />
        <div className="space-y-4">
          <Field label="Обязательные навыки" error={errors.must_skills ?? fieldError("must_skills")} required hint="Главный признак подбора (30 % балла)">
            <SkillInput value={d.must_skills} onChange={(must_skills) => set({ must_skills })} tone="primary" />
          </Field>
          <Field label="Желательные навыки" optional>
            <SkillInput value={d.nice_skills} onChange={(nice_skills) => set({ nice_skills })} />
          </Field>
        </div>
      </Card>

      <Card>
        <CardHeader title="Условия" description="Вилка обязательна — кандидат увидит её в приглашении" />
        <div className="grid gap-4 sm:grid-cols-2">
          <Field label="Зарплата от, ₽" error={errors.min ?? fieldError("salary.min")} required>
            <MoneyInput value={d.min} onChange={(min) => set({ min })} placeholder="250 000" />
          </Field>
          <Field label="Зарплата до, ₽" error={errors.max ?? fieldError("salary.max")} required>
            <MoneyInput value={d.max} onChange={(max) => set({ max })} placeholder="320 000" />
          </Field>
          <Field label="Формат">
            <ChipSelect<WorkFormat> ariaLabel="Формат работы" options={WORK_FORMATS.map((f) => ({ value: f, label: WORK_FORMAT_LABEL[f] }))} value={d.work_formats} onChange={(work_formats) => set({ work_formats })} />
          </Field>
          <Field label="Город" optional hint="Для офиса и гибрида">
            <Input value={d.city} onChange={(e) => set({ city: e.target.value })} placeholder="Москва" />
          </Field>
        </div>
      </Card>

      <Card>
        <CardHeader title="Команда и задачи" />
        <div className="space-y-4">
          <Field label="Чем занимается команда" optional hint="Используется в подборе (близость текстов) и как черновик приглашения">
            <Textarea rows={5} value={d.description} onChange={(e) => set({ description: e.target.value })} />
          </Field>
          <Field label="Отрасль" optional>
            <ChipSelect<Industry> ariaLabel="Отрасль" options={INDUSTRIES.map((i) => ({ value: i, label: INDUSTRY_LABEL[i] }))} value={d.industries} onChange={(industries) => set({ industries })} />
          </Field>
        </div>
      </Card>

      <div className="flex flex-wrap justify-end gap-2">
        {onCancel && (
          <Button variant="ghost" onClick={onCancel}>
            Отмена
          </Button>
        )}
        <Button type="submit" size="lg" loading={pending}>
          {submitLabel}
        </Button>
      </div>
    </form>
  );
}
