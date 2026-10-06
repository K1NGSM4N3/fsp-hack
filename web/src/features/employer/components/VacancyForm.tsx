"use client";

import { Building2, CheckCircle2, Circle, MapPin } from "lucide-react";
import Link from "next/link";
import { useRef, useState } from "react";
import { isApiError } from "@/api/client";
import type { Grade, SimilarVacancy, Specialization, Vacancy, VacancyInput, VacancyStatus, WorkFormat } from "@/api/types";
import { CategoryBadge } from "@/components/domain/Category";
import { SkillInput } from "@/components/domain/Skills";
import { Alert, Button, Card, CardHeader, Chip, Field, FormActions, FormStatus, Input, MoneyInput, RadioCards, Select, Textarea, focusFirstError, useToast } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { SalaryHint } from "@/features/employer/components/SalaryHint";
import { useSaveVacancy, useSimilarVacancies } from "@/features/employer/hooks";
import { newIdempotencyKey } from "@/lib/idempotency";
import { cn } from "@/lib/cn";
import { formatSalaryRange, plural } from "@/lib/format";
import { GRADE_LABEL, GRADES, SPEC_LONG_LABEL, SPECIALIZATIONS, VACANCY_STATUS_LABEL, WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";
import { skillName } from "@/lib/skills";

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

type Key = "title" | "description" | "min" | "max";
type Errors = Partial<Record<Key, string>>;

const FIELD_LABEL: Record<Key, string> = { title: "название", description: "описание", min: "зарплата от", max: "зарплата до" };
const MIN_DESCRIPTION = 20;

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

function validate(d: Draft): Errors {
  const e: Errors = {};
  if (d.title.trim().length < 3) e.title = "Назовите вакансию — хотя бы 3 символа";
  if (d.description.trim().length < MIN_DESCRIPTION) e.description = `Опишите задачи и команду — хотя бы ${MIN_DESCRIPTION} символов`;
  if (!d.min) e.min = "Укажите нижнюю границу вилки";
  if (!d.max) e.max = "Укажите верхнюю границу вилки";
  return e;
}

/** Shown as soon as both numbers are typed, not only after submit: an inverted range is never intended. */
function rangeError(d: Draft): Errors {
  return d.min && d.max && d.max < d.min ? { max: "«До» меньше, чем «от»" } : {};
}

const STATUS_HINT: Record<VacancyStatus, string> = {
  draft: "Видна только вам",
  published: "В общем списке вакансий, соискатели могут откликаться",
  closed: "Снята с публикации, отклики сохраняются",
};

function Preview({ d, company }: { d: Draft; company: string }) {
  const extra = d.skills.length - 4;
  return (
    <div className="rounded-control border border-line bg-white p-4">
      <p className={cn("text-[15px] leading-snug font-semibold", d.title.trim() ? "text-ink" : "text-ink-faint")}>{d.title.trim() || "Название вакансии"}</p>
      <p className="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-muted">
        <span className="inline-flex items-center gap-1">
          <Building2 className="size-3.5 text-ink-subtle" aria-hidden />
          {company}
        </span>
        <span className="inline-flex items-center gap-1">
          <MapPin className="size-3.5 text-ink-subtle" aria-hidden />
          {WORK_FORMAT_LABEL[d.work_format]}
          {d.city.trim() && d.work_format !== "remote" ? `, ${d.city.trim()}` : ""}
        </span>
      </p>
      <p className={cn("tabular mt-3 font-mono text-base font-semibold tracking-tight", d.min && d.max && d.max >= d.min ? "text-ink" : "text-ink-faint")}>
        {d.min && d.max && d.max >= d.min ? formatSalaryRange(d.min, d.max) : "Вилка от и до"}
      </p>
      <div className="mt-3 flex flex-wrap items-center gap-1.5">
        <CategoryBadge category={{ specialization: d.specialization, grade: d.grade }} size="sm" />
        {d.skills.slice(0, 4).map((s) => (
          <Chip key={s}>{skillName(s)}</Chip>
        ))}
        {extra > 0 && <span className="text-xs text-ink-subtle">ещё {extra}</span>}
      </div>
    </div>
  );
}

function Readiness({ items }: { items: { label: string; done: boolean; optional?: boolean }[] }) {
  const left = items.filter((i) => !i.done && !i.optional).length;
  return (
    <div>
      <p className="text-sm font-medium text-ink">{left ? `Осталось ${left} ${plural(left, ["пункт", "пункта", "пунктов"])}` : "Всё обязательное заполнено"}</p>
      <ul className="mt-3 space-y-2">
        {items.map((i) => (
          <li key={i.label} className="flex items-center gap-2 text-sm">
            {i.done ? <CheckCircle2 className="size-4 shrink-0 text-success" aria-hidden /> : <Circle className="size-4 shrink-0 text-line-strong" aria-hidden />}
            <span className={i.done ? "text-ink" : "text-ink-muted"}>{i.label}</span>
            {i.optional && !i.done && <span className="text-xs text-ink-subtle">желательно</span>}
            <span className="sr-only">{i.done ? " — готово" : " — не заполнено"}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function VacancyForm({ vacancy, onSaved, onCancel }: { vacancy?: Vacancy; onSaved: (v: Vacancy) => void; onCancel?: () => void }) {
  const toast = useToast();
  const me = useMe();
  const [key] = useState(newIdempotencyKey);
  const save = useSaveVacancy(vacancy?.id, key);
  const similar = useSimilarVacancies();
  const form = useRef<HTMLFormElement>(null);
  const [initial] = useState<Draft>(() => toDraft(vacancy));
  const [d, setD] = useState<Draft>(initial);
  const [attempted, setAttempted] = useState(false);
  const [server, setServer] = useState<Errors>({});
  const [twins, setTwins] = useState<SimilarVacancy[] | null>(null);

  const errors: Errors = { ...(attempted ? validate(d) : {}), ...rangeError(d), ...server };
  const missing = (Object.keys(errors) as Key[]).map((k) => FIELD_LABEL[k]);
  const dirty = JSON.stringify(d) !== JSON.stringify(initial);

  const set = (patch: Partial<Draft>) => {
    setD((x) => ({ ...x, ...patch }));
    setServer((s) => {
      const next = { ...s };
      for (const k of Object.keys(patch)) delete next[k as Key];
      return next;
    });
    if ("title" in patch || "description" in patch) setTwins(null);
  };

  const submit = () => {
    setAttempted(true);
    const e = { ...validate(d), ...rangeError(d) };
    if (Object.keys(e).length || !d.min || !d.max) {
      focusFirstError(form.current);
      return;
    }
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
          const found: Errors = { title: err.fieldError("title"), description: err.fieldError("description"), min: err.fieldError("salary.min"), max: err.fieldError("salary.max") };
          setServer(Object.fromEntries(Object.entries(found).filter(([, v]) => v)));
          focusFirstError(form.current);
        }
        toast.error(isApiError(err) ? err.title : "Не удалось сохранить");
      },
    });
  };

  const descriptionLeft = MIN_DESCRIPTION - d.description.trim().length;
  const salaryOk = Boolean(d.min && d.max && d.max >= d.min);

  return (
    <form
      ref={form}
      noValidate
      className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_18.5rem] xl:grid-cols-[minmax(0,1fr)_20rem]"
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <div className="min-w-0 space-y-6">
        <Card>
          <CardHeader step={1} title="О вакансии" description="Название и описание соискатели видят в общем списке и в карточке вакансии" />
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Название" required error={errors.title} className="sm:col-span-2">
              <Input value={d.title} onChange={(e) => set({ title: e.target.value })} placeholder="Go-разработчик в команду платежей" maxLength={120} />
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
            <Field
              label="Описание"
              required
              error={errors.description}
              hint={descriptionLeft > 0 ? `Задачи, команда, условия. Ещё ${descriptionLeft} ${plural(descriptionLeft, ["символ", "символа", "символов"])} до минимума` : "Задачи, команда, условия"}
              className="sm:col-span-2"
            >
              <Textarea rows={7} value={d.description} onChange={(e) => set({ description: e.target.value })} />
            </Field>
            <Field label="Стек" optional hint="По навыкам соискатели находят вакансию в поиске" className="sm:col-span-2">
              <SkillInput value={d.skills} onChange={(skills) => set({ skills })} />
            </Field>
          </div>
        </Card>

        <Card>
          <CardHeader step={2} title="Условия" description="Вилка обязательна: соискатель видит её до отклика" />
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Зарплата от, ₽" required error={errors.min}>
              <MoneyInput value={d.min} onChange={(min) => set({ min })} placeholder="250 000" />
            </Field>
            <Field label="Зарплата до, ₽" required error={errors.max}>
              <MoneyInput value={d.max} onChange={(max) => set({ max })} placeholder="320 000" />
            </Field>
            <SalaryHint category={{ specialization: d.specialization, grade: d.grade }} min={d.min} max={d.max} className="sm:col-span-2" />
            <Field label="Формат работы">
              <Select value={d.work_format} onChange={(e) => set({ work_format: e.target.value as WorkFormat })}>
                {WORK_FORMATS.map((f) => (
                  <option key={f} value={f}>
                    {WORK_FORMAT_LABEL[f]}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label="Город" optional hint={d.work_format === "remote" ? "Для удалённой работы можно не указывать" : "Где офис"}>
              <Input value={d.city} onChange={(e) => set({ city: e.target.value })} placeholder="Москва" />
            </Field>
          </div>
        </Card>

        <Card>
          <CardHeader step={3} title="Публикация" description="Статус можно сменить в любой момент" />
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
            <p>Текст почти совпадает с вашей вакансией. Возможно, удобнее обновить её — тогда отклики соберутся в одном месте, а соискатели не увидят двух одинаковых предложений.</p>
            <ul className="mt-2 space-y-1">
              {twins.map((t) => (
                <li key={t.id}>
                  <Link href={`/employer/vacancies/${t.id}`} className="link-inline font-medium">
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
      </div>

      <aside aria-label="Предпросмотр вакансии" className="min-w-0 space-y-4 lg:sticky lg:top-20 lg:self-start">
        <Card padding="sm" className="bg-canvas/60">
          <p className="eyebrow mb-3 text-ink-subtle">Так увидят соискатели</p>
          <Preview d={d} company={me.data?.company?.name ?? "Ваша компания"} />
        </Card>
        <Card padding="sm">
          <Readiness
            items={[
              { label: "Название", done: d.title.trim().length >= 3 },
              { label: "Описание задач", done: descriptionLeft <= 0 },
              { label: "Вилка от и до", done: salaryOk },
              { label: "Стек", done: d.skills.length > 0, optional: true },
            ]}
          />
        </Card>
      </aside>

      <FormActions className="lg:col-span-2" status={<FormStatus missing={missing} dirty={Boolean(vacancy) && dirty} onFix={() => focusFirstError(form.current)} />}>
        {onCancel && (
          <Button variant="ghost" onClick={onCancel}>
            Отмена
          </Button>
        )}
        <Button type="submit" size="lg" loading={save.isPending || similar.isPending}>
          {twins && twins.length > 0 ? (vacancy ? "Всё равно сохранить" : "Всё равно создать") : vacancy ? "Сохранить" : "Создать вакансию"}
        </Button>
      </FormActions>
    </form>
  );
}
