"use client";

import { CheckCircle2, Circle } from "lucide-react";
import { useRef, useState, type ReactNode } from "react";
import { isApiError, type ApiError } from "@/api/client";
import type { Grade, Industry, NeedInput, Specialization, WorkFormat } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { SkillInput } from "@/components/domain/Skills";
import { Button, Card, CardHeader, Checkbox, Chip, ChipSelect, Field, FormActions, FormStatus, Input, MoneyInput, Select, Textarea, focusFirstError } from "@/components/ui";
import { SalaryHint } from "@/features/employer/components/SalaryHint";
import { cn } from "@/lib/cn";
import { formatSalaryRange, plural } from "@/lib/format";
import { GRADE_LABEL, GRADES, INDUSTRIES, INDUSTRY_LABEL, SPEC_LONG_LABEL, SPECIALIZATIONS, WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";
import { skillName } from "@/lib/skills";

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

type Key = "title" | "specialization" | "grades" | "must_skills" | "min" | "max";
type Errors = Partial<Record<Key, string>>;

const FIELD_LABEL: Record<Key, string> = {
  title: "название",
  specialization: "специализация",
  grades: "грейд",
  must_skills: "обязательные навыки",
  min: "зарплата от",
  max: "зарплата до",
};

function validate(d: NeedDraft): Errors {
  const e: Errors = {};
  if (!d.title.trim()) e.title = "Назовите потребность — так её будет проще найти в списке";
  if (!d.specialization) e.specialization = "Выберите специализацию";
  if (!d.grades.length) e.grades = "Выберите хотя бы один грейд";
  if (!d.must_skills.length) e.must_skills = "Добавьте хотя бы один обязательный навык";
  if (!d.min) e.min = "Укажите нижнюю границу вилки";
  if (!d.max) e.max = "Укажите верхнюю границу вилки";
  return e;
}

function rangeError(d: NeedDraft): Errors {
  return d.min && d.max && d.max < d.min ? { max: "«До» меньше, чем «от»" } : {};
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

function Summary({ d }: { d: NeedDraft }) {
  const spec = d.specialization;
  const salary = d.min && d.max && d.max >= d.min ? formatSalaryRange(d.min, d.max) : null;
  const rows: { label: string; done: boolean; value: ReactNode }[] = [
    { label: "Название", done: Boolean(d.title.trim()), value: d.title.trim() || "как назвать потребность" },
    {
      label: "Категории",
      done: Boolean(spec && d.grades.length),
      value:
        spec && d.grades.length ? (
          <span className="flex flex-wrap gap-1.5">
            {d.grades.map((g) => (
              <CategoryBadge key={g} category={{ specialization: spec, grade: g }} size="sm" />
            ))}
          </span>
        ) : (
          "специализация и грейд"
        ),
    },
    {
      label: "Обязательные навыки",
      done: d.must_skills.length > 0,
      value: d.must_skills.length ? (
        <span className="flex flex-wrap gap-1.5">
          {d.must_skills.slice(0, 5).map((s) => (
            <Chip key={s} tone="primary">
              {skillName(s)}
            </Chip>
          ))}
          {d.must_skills.length > 5 && <span className="self-center text-xs text-ink-subtle">ещё {d.must_skills.length - 5}</span>}
        </span>
      ) : (
        "хотя бы один"
      ),
    },
    { label: "Вилка", done: Boolean(salary), value: salary ? <span className="tabular font-mono font-semibold text-ink">{salary}</span> : "от и до" },
    { label: "Формат", done: true, value: (d.work_formats.length ? d.work_formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ") : "Любой") + (d.city.trim() ? ` · ${d.city.trim()}` : "") },
  ];
  const left = rows.filter((r) => !r.done).length;
  return (
    <div>
      <p className="eyebrow text-ink-subtle">Кого ищем</p>
      <p className="mt-2 text-sm font-medium text-ink">{left ? `Для подборки осталось ${left} ${plural(left, ["пункт", "пункта", "пунктов"])}` : "Готово к подборке"}</p>
      <ul className="mt-4 space-y-3.5">
        {rows.map((r) => (
          <li key={r.label} className="flex gap-2.5">
            {r.done ? <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-success" aria-hidden /> : <Circle className="mt-0.5 size-4 shrink-0 text-line-strong" aria-hidden />}
            <div className="min-w-0 text-sm">
              <p className="text-xs text-ink-subtle">
                {r.label}
                <span className="sr-only">{r.done ? " — готово" : " — не заполнено"}</span>
              </p>
              <div className={cn("mt-1", r.done ? "text-ink" : "text-ink-faint")}>{r.value}</div>
            </div>
          </li>
        ))}
      </ul>
      <p className="mt-5 border-t border-line pt-4 text-xs leading-relaxed text-ink-subtle">
        В подборку попадут соискатели с подтверждённой тестом категорией. Обязательные навыки — главный признак сходства, желательные и отрасль поднимают выше.
      </p>
    </div>
  );
}

interface NeedFormProps {
  initial: NeedDraft;
  submitLabel: string;
  pending: boolean;
  onSubmit: (input: NeedInput) => void;
  serverError?: ApiError | null;
  onCancel?: () => void;
  /** Edit mode: the status line reports unsaved changes. */
  trackChanges?: boolean;
}

export function NeedForm({ initial, submitLabel, pending, onSubmit, serverError, onCancel, trackChanges = false }: NeedFormProps) {
  const form = useRef<HTMLFormElement>(null);
  const [d, setD] = useState<NeedDraft>(initial);
  const [attempted, setAttempted] = useState(false);
  const set = (patch: Partial<NeedDraft>) => setD((x) => ({ ...x, ...patch }));
  const fieldError = (key: string) => (serverError && isApiError(serverError) ? serverError.fieldError(key) : undefined);
  const errors: Errors = {
    ...(attempted ? validate(d) : {}),
    ...rangeError(d),
  };
  const shown = {
    title: errors.title ?? fieldError("title"),
    specialization: errors.specialization ?? fieldError("specialization"),
    grades: errors.grades ?? fieldError("grades"),
    must_skills: errors.must_skills ?? fieldError("must_skills"),
    min: errors.min ?? fieldError("salary.min"),
    max: errors.max ?? fieldError("salary.max"),
  };
  const missing = (Object.keys(shown) as Key[]).filter((k) => shown[k]).map((k) => FIELD_LABEL[k]);
  const dirty = trackChanges && JSON.stringify(d) !== JSON.stringify(initial);

  return (
    <form
      ref={form}
      noValidate
      className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_18.5rem] xl:grid-cols-[minmax(0,1fr)_20rem]"
      onSubmit={(e) => {
        e.preventDefault();
        setAttempted(true);
        const errs = { ...validate(d), ...rangeError(d) };
        const input = toNeedInput(d);
        if (Object.keys(errs).length || !input) {
          focusFirstError(form.current);
          return;
        }
        onSubmit(input);
      }}
    >
      <div className="min-w-0 space-y-6">
        <Card>
          <CardHeader step={1} title="Кого ищете" description="Категория — главный фильтр: в подборку попадают только те, кто подтвердил её тестом" />
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Название" error={shown.title} required className="sm:col-span-2">
              <Input value={d.title} onChange={(e) => set({ title: e.target.value })} placeholder="Например: Go-разработчик в команду платежей" maxLength={120} />
            </Field>
            <Field label="Специализация" error={shown.specialization} required>
              <Select value={d.specialization} onChange={(e) => set({ specialization: e.target.value as Specialization | "" })}>
                <option value="">Выберите…</option>
                {SPECIALIZATIONS.map((s) => (
                  <option key={s} value={s}>
                    {SPEC_LONG_LABEL[s]}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Грейд" error={shown.grades} required>
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
          <CardHeader step={2} title="Стек" description="Навыки распознаются с синонимами: «постгрес» → PostgreSQL" />
          <div className="space-y-4">
            <Field label="Обязательные навыки" error={shown.must_skills} required hint="Главный признак подбора: 30 % балла">
              <SkillInput value={d.must_skills} onChange={(must_skills) => set({ must_skills })} tone="primary" />
            </Field>
            <Field label="Желательные навыки" optional hint="Поднимают кандидата в подборке, но не отсеивают">
              <SkillInput value={d.nice_skills} onChange={(nice_skills) => set({ nice_skills })} />
            </Field>
          </div>
        </Card>

        <Card>
          <CardHeader step={3} title="Условия" description="Вилка обязательна — кандидат увидит её в приглашении" />
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Зарплата от, ₽" error={shown.min} required>
              <MoneyInput value={d.min} onChange={(min) => set({ min })} placeholder="Например: 250 000" />
            </Field>
            <Field label="Зарплата до, ₽" error={shown.max} required>
              <MoneyInput value={d.max} onChange={(max) => set({ max })} placeholder="Например: 320 000" />
            </Field>
            {d.specialization && d.grades[0] && <SalaryHint category={{ specialization: d.specialization, grade: d.grades[0] }} min={d.min} max={d.max} className="sm:col-span-2" />}
            <Field label="Формат" hint={d.work_formats.length ? undefined : "Ничего не выбрано — подойдёт любой"}>
              <ChipSelect<WorkFormat> ariaLabel="Формат работы" options={WORK_FORMATS.map((f) => ({ value: f, label: WORK_FORMAT_LABEL[f] }))} value={d.work_formats} onChange={(work_formats) => set({ work_formats })} />
            </Field>
            <Field label="Город" optional hint="Для офиса и гибрида">
              <Input value={d.city} onChange={(e) => set({ city: e.target.value })} placeholder="Например: Москва" />
            </Field>
          </div>
        </Card>

        <Card>
          <CardHeader step={4} title="Команда и задачи" description="Необязательно, но с описанием подборка точнее, а приглашение пишется быстрее" />
          <div className="space-y-4">
            <Field label="Чем занимается команда" optional hint="Используется в подборе (близость текстов) и как черновик приглашения">
              <Textarea rows={5} value={d.description} onChange={(e) => set({ description: e.target.value })} />
            </Field>
            <Field label="Отрасль" optional>
              <ChipSelect<Industry> ariaLabel="Отрасль" options={INDUSTRIES.map((i) => ({ value: i, label: INDUSTRY_LABEL[i] }))} value={d.industries} onChange={(industries) => set({ industries })} />
            </Field>
          </div>
        </Card>
      </div>

      <aside aria-label="Сводка потребности" className="min-w-0 lg:sticky lg:top-20 lg:self-start">
        <Card padding="sm" className="p-5">
          <Summary d={d} />
        </Card>
      </aside>

      <FormActions className="lg:col-span-2" status={<FormStatus missing={missing} dirty={dirty} onFix={() => focusFirstError(form.current)} />}>
        {onCancel && (
          <Button variant="ghost" onClick={onCancel}>
            Отмена
          </Button>
        )}
        <Button type="submit" size="lg" loading={pending}>
          {submitLabel}
        </Button>
      </FormActions>
    </form>
  );
}
