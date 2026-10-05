"use client";

import { Search, SearchX } from "lucide-react";
import { useDeferredValue, useState } from "react";
import type { Grade, Specialization, VacancyQuery, WorkFormat } from "@/api/types";
import { Button, EmptyState, ErrorAlert, Field, Input, MoneyInput, Select, SkeletonCard } from "@/components/ui";
import { useVacancies } from "@/features/vacancies/hooks";
import { pluralize } from "@/lib/format";
import { GRADES, GRADE_LABEL, SPECIALIZATIONS, SPEC_LONG_LABEL, WORK_FORMATS, WORK_FORMAT_LABEL } from "@/lib/labels";
import { VacancyCard } from "./VacancyCard";

const PAGE = 10;

export function VacancyBrowser({ hrefFor }: { hrefFor: (id: string) => string }) {
  const [q, setQ] = useState("");
  const [specialization, setSpecialization] = useState<Specialization | "">("");
  const [grade, setGrade] = useState<Grade | "">("");
  const [workFormat, setWorkFormat] = useState<WorkFormat | "">("");
  const [salaryFrom, setSalaryFrom] = useState<number | null>(null);
  const [limit, setLimit] = useState(PAGE);
  const deferredQ = useDeferredValue(q);

  const params: VacancyQuery = {
    q: deferredQ.trim() || undefined,
    specialization: specialization || undefined,
    grade: grade || undefined,
    work_format: workFormat || undefined,
    salary_from: salaryFrom ?? undefined,
    limit,
    offset: 0,
  };
  const query = useVacancies(params);
  const reset = () => {
    setQ("");
    setSpecialization("");
    setGrade("");
    setWorkFormat("");
    setSalaryFrom(null);
    setLimit(PAGE);
  };
  const filtered = Boolean(q || specialization || grade || workFormat || salaryFrom);

  return (
    <div className="grid gap-6 lg:grid-cols-[17rem_1fr]">
      <aside aria-label="Фильтры вакансий" className="h-fit space-y-4 rounded-card border border-line bg-white p-5 lg:sticky lg:top-24">
        <Field label="Поиск">
          <Input leading={<Search className="size-4" aria-hidden />} placeholder="Go, React, финтех…" value={q} onChange={(e) => setQ(e.target.value)} type="search" />
        </Field>
        <Field label="Специализация">
          <Select value={specialization} onChange={(e) => setSpecialization(e.target.value as Specialization | "")}>
            <option value="">Все</option>
            {SPECIALIZATIONS.map((s) => (
              <option key={s} value={s}>
                {SPEC_LONG_LABEL[s]}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Грейд">
          <Select value={grade} onChange={(e) => setGrade(e.target.value as Grade | "")}>
            <option value="">Любой</option>
            {GRADES.map((g) => (
              <option key={g} value={g}>
                {GRADE_LABEL[g]}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Формат">
          <Select value={workFormat} onChange={(e) => setWorkFormat(e.target.value as WorkFormat | "")}>
            <option value="">Любой</option>
            {WORK_FORMATS.map((f) => (
              <option key={f} value={f}>
                {WORK_FORMAT_LABEL[f]}
              </option>
            ))}
          </Select>
        </Field>
        <Field label="Зарплата от" hint="Покажем вакансии, где верх вилки не ниже">
          <MoneyInput value={salaryFrom} onChange={setSalaryFrom} placeholder="200 000" />
        </Field>
        {filtered && (
          <Button variant="ghost" size="sm" onClick={reset} className="w-full">
            Сбросить фильтры
          </Button>
        )}
      </aside>

      <section aria-label="Список вакансий" aria-busy={query.isFetching} className="min-w-0 space-y-4">
        {query.isPending ? (
          Array.from({ length: 3 }, (_, i) => <SkeletonCard key={i} />)
        ) : query.isError ? (
          <ErrorAlert error={query.error} onRetry={() => void query.refetch()} />
        ) : query.data.items.length === 0 ? (
          <EmptyState
            icon={<SearchX className="size-5" aria-hidden />}
            title="Ничего не нашлось"
            description="Попробуйте убрать часть фильтров или изменить запрос."
            action={filtered ? <Button variant="secondary" onClick={reset}>Сбросить фильтры</Button> : undefined}
          />
        ) : (
          <>
            <p className="text-sm text-ink-muted" aria-live="polite">
              Найдено {pluralize(query.data.total, ["вакансия", "вакансии", "вакансий"])}
            </p>
            {query.data.items.map((v) => (
              <VacancyCard key={v.id} vacancy={v} href={hrefFor(v.id)} />
            ))}
            {query.data.total > query.data.items.length && (
              <div className="flex justify-center pt-2">
                <Button variant="secondary" loading={query.isFetching} onClick={() => setLimit((l) => l + PAGE)}>
                  Показать ещё
                </Button>
              </div>
            )}
          </>
        )}
      </section>
    </div>
  );
}
