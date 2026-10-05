"use client";

import { Search, SearchX, Send, Trophy } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useDeferredValue, useState } from "react";
import type { CandidateSearchParams, Grade, Specialization, WorkFormat } from "@/api/types";
import { CandidateCardView } from "@/components/domain/CandidateCard";
import { ReasonList, ScoreRing } from "@/components/domain/Explanation";
import { SkillInput } from "@/components/domain/Skills";
import { Button, Card, CardHeader, EmptyState, ErrorAlert, Field, Input, MoneyInput, PageHeader, PageSkeleton, Select, SkeletonCard, Switch } from "@/components/ui";
import { useCandidateSearch, useCategoryStats, useNeeds } from "@/features/employer/hooks";
import { InviteDialog, type InviteTarget } from "@/features/employer/components/InviteDialog";
import { ProctoringBadge } from "@/features/employer/components/ProctoringBadge";
import { cn } from "@/lib/cn";
import { formatRubShort, pluralize } from "@/lib/format";
import { GRADE_LABEL, GRADES, SPEC_LABEL, SPEC_LONG_LABEL, SPECIALIZATIONS, WORK_FORMAT_LABEL, WORK_FORMATS } from "@/lib/labels";

const PAGE = 10;

function CategoryMatrix({ spec, grade, onPick }: { spec: Specialization; grade: Grade | ""; onPick: (s: Specialization, g: Grade) => void }) {
  const q = useCategoryStats();
  if (q.isPending) return <SkeletonCard lines={4} />;
  if (q.isError) return null;
  const find = (s: Specialization, g: Grade) => q.data.find((c) => c.category.specialization === s && c.category.grade === g);
  return (
    <Card padding="sm" className="mb-6 overflow-x-auto">
      <CardHeader title="Категории" description="Кандидаты с подтверждённым тестом грейдом · из них с достижениями ФСП · медианные ожидания" className="mb-3 px-1" />
      <table className="w-full min-w-[34rem] border-separate border-spacing-1 text-sm">
        <thead>
          <tr>
            <th className="w-40" />
            {GRADES.map((g) => (
              <th key={g} scope="col" className="px-2 pb-1 text-left font-mono text-xs font-medium text-ink-subtle">
                {GRADE_LABEL[g]}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {SPECIALIZATIONS.map((s) => (
            <tr key={s}>
              <th scope="row" className="pr-2 text-left text-sm font-medium text-ink">
                {SPEC_LONG_LABEL[s]}
              </th>
              {GRADES.map((g) => {
                const c = find(s, g);
                const active = spec === s && grade === g;
                return (
                  <td key={g}>
                    <button
                      type="button"
                      onClick={() => onPick(s, g)}
                      aria-pressed={active}
                      aria-label={`${SPEC_LABEL[s]} ${GRADE_LABEL[g]}: ${c?.candidates ?? 0} кандидатов`}
                      className={cn(
                        "flex w-full items-center justify-between gap-2 rounded-control border px-2.5 py-2 text-left transition-colors",
                        active ? "border-primary bg-primary text-white" : "border-line bg-white hover:border-primary",
                      )}
                    >
                      <span className="tabular font-mono font-semibold">{c?.candidates ?? 0}</span>
                      <span className={cn("flex items-center gap-1 text-xs", active ? "text-primary-100" : "text-ink-subtle")}>
                        <Trophy className="size-3" aria-hidden />
                        {c?.with_fsp ?? 0}
                        <span className="hidden lg:inline">· {c?.median_expectation ? formatRubShort(c.median_expectation) : "—"}</span>
                      </span>
                    </button>
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </Card>
  );
}

function SearchView() {
  const params = useSearchParams();
  const [spec, setSpec] = useState<Specialization>((params.get("specialization") as Specialization | null) ?? "backend");
  const [grade, setGrade] = useState<Grade | "">((params.get("grade") as Grade | null) ?? "");
  const [skills, setSkills] = useState<string[]>([]);
  const [fspOnly, setFspOnly] = useState(false);
  const [adultsOnly, setAdultsOnly] = useState(false);
  const [format, setFormat] = useState<WorkFormat | "">("");
  const [city, setCity] = useState("");
  const [salaryMax, setSalaryMax] = useState<number | null>(null);
  const [q, setQ] = useState("");
  const [needId, setNeedId] = useState<string>("");
  const needs = useNeeds();
  const [limit, setLimit] = useState(PAGE);
  const [invite, setInvite] = useState<InviteTarget | null>(null);
  const dq = useDeferredValue(q);
  const dcity = useDeferredValue(city);

  const search: CandidateSearchParams = {
    specialization: spec,
    grade: grade || undefined,
    skills: skills.length ? skills.join(",") : undefined,
    fsp_only: fspOnly || undefined,
    adults_only: adultsOnly || undefined,
    work_format: format || undefined,
    city: dcity.trim() || undefined,
    max_expectation: salaryMax ?? undefined,
    q: dq.trim() || undefined,
    need_id: needId || undefined,
    limit,
    offset: 0,
  };
  const result = useCandidateSearch(search);

  const reset = () => {
    setGrade("");
    setNeedId("");
    setSkills([]);
    setFspOnly(false);
    setAdultsOnly(false);
    setFormat("");
    setCity("");
    setSalaryMax(null);
    setQ("");
    setLimit(PAGE);
  };

  return (
    <>
      <CategoryMatrix
        spec={spec}
        grade={grade}
        onPick={(s, g) => {
          const same = spec === s && grade === g;
          setSpec(s);
          setGrade(same ? "" : g);
          setLimit(PAGE);
        }}
      />
      <div className="grid gap-6 lg:grid-cols-[17rem_1fr]">
        <aside aria-label="Фильтры" className="h-fit space-y-4 rounded-card border border-line bg-white p-5 lg:sticky lg:top-20">
          <Field label="Поиск по навыкам и описанию">
            <Input type="search" leading={<Search className="size-4" aria-hidden />} value={q} onChange={(e) => setQ(e.target.value)} placeholder="kafka, платежи…" />
          </Field>
          <Field label="Специализация">
            <Select value={spec} onChange={(e) => setSpec(e.target.value as Specialization)}>
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
          <Field label="Стек" hint="Все выбранные навыки обязательны">
            <SkillInput value={skills} onChange={setSkills} placeholder="Go, Kafka…" />
          </Field>
          <Field label="Формат">
            <Select value={format} onChange={(e) => setFormat(e.target.value as WorkFormat | "")}>
              <option value="">Любой</option>
              {WORK_FORMATS.map((f) => (
                <option key={f} value={f}>
                  {WORK_FORMAT_LABEL[f]}
                </option>
              ))}
            </Select>
          </Field>
          <Field label="Город" hint="Удалённых кандидатов покажем всегда">
            <Input value={city} onChange={(e) => setCity(e.target.value)} placeholder="Москва" />
          </Field>
          <Field label="Ожидания не выше">
            <MoneyInput value={salaryMax} onChange={setSalaryMax} placeholder="300 000" />
          </Field>
          <Switch checked={fspOnly} onChange={setFspOnly} label="Есть достижения ФСП" />
          <Switch checked={adultsOnly} onChange={setAdultsOnly} label="Только 18+" description="Скрыть кандидатов 14–17 лет" />
          <Button variant="ghost" size="sm" className="w-full" onClick={reset}>
            Сбросить фильтры
          </Button>
        </aside>

        <section aria-label="Результаты поиска" aria-busy={result.isFetching} className="min-w-0 space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <p className="text-sm text-ink" aria-live="polite">
              {result.data ? `Найдено ${pluralize(result.data.total, ["кандидат", "кандидата", "кандидатов"])}` : "Ищем…"}
            </p>
            <div className="flex items-center gap-2">
              <label htmlFor="rank-by" className="shrink-0 text-sm text-ink-subtle">
                Ранжировать
              </label>
              <Select id="rank-by" value={needId} onChange={(e) => setNeedId(e.target.value)} className="w-72">
                <option value="">По силе профиля (тест + ФСП + актуальность)</option>
                {(needs.data ?? [])
                  .filter((n) => n.specialization === spec)
                  .map((n) => (
                    <option key={n.id} value={n.id}>
                      Под потребность «{n.title}»
                    </option>
                  ))}
              </Select>
            </div>
          </div>
          {result.isPending ? (
            <>
              <SkeletonCard lines={4} />
              <SkeletonCard lines={4} />
            </>
          ) : result.isError ? (
            <ErrorAlert error={result.error} onRetry={() => void result.refetch()} />
          ) : result.data.items.length === 0 ? (
            <EmptyState icon={<SearchX className="size-5" aria-hidden />} title="Никого не нашли" description="Ослабьте фильтры: уберите часть навыков, город или ограничение по зарплате." action={<Button variant="secondary" onClick={reset}>Сбросить фильтры</Button>} />
          ) : (
            <>
              <ul className="space-y-4">
                {result.data.items.map(({ candidate: c, strength, score, reasons, gaps }) => (
                  <li key={c.id}>
                    <CandidateCardView
                      candidate={c}
                      href={`/employer/candidates/${c.id}${needId ? `?need=${needId}` : ""}`}
                      highlightSkills={skills}
                      badges={<ProctoringBadge badge={c.proctoring} />}
                      aside={
                        typeof score === "number" ? (
                          <ScoreRing score={score} />
                        ) : (
                          <div className="text-right">
                            <p className="text-xs text-ink-subtle">Сила профиля</p>
                            <p className="tabular font-mono text-xl font-semibold text-primary">{Math.round(strength * 100)}</p>
                          </div>
                        )
                      }
                      footer={
                        <>
                          <Link href={`/employer/candidates/${c.id}${needId ? `?need=${needId}` : ""}`} className="rounded px-2 text-sm font-medium text-primary hover:underline">
                            Открыть профиль
                          </Link>
                          <span className="flex-1" />
                          <Button
                            size="sm"
                            icon={<Send className="size-4" aria-hidden />}
                            disabled={c.relation.invitation_status === "sent" || c.relation.invitation_status === "viewed"}
                            onClick={() => setInvite({ id: c.id, code: c.code, display_name: c.display_name, category: c.category, minor: c.minor })}
                          >
                            {c.relation.invitation_status === "sent" || c.relation.invitation_status === "viewed" ? "Приглашение отправлено" : "Пригласить"}
                          </Button>
                        </>
                      }
                    >
                      {(reasons.length > 0 || gaps.length > 0) && (
                        <div className="mt-3 border-t border-line pt-3">
                          <ReasonList reasons={reasons} gaps={gaps} />
                        </div>
                      )}
                    </CandidateCardView>
                  </li>
                ))}
              </ul>
              {result.data.total > result.data.items.length && (
                <div className="flex justify-center">
                  <Button variant="secondary" loading={result.isFetching} onClick={() => setLimit((l) => l + PAGE)}>
                    Показать ещё
                  </Button>
                </div>
              )}
            </>
          )}
        </section>
      </div>
      <InviteDialog open={invite !== null} onClose={() => setInvite(null)} target={invite} need={needs.data?.find((n) => n.id === needId) ?? null} />
    </>
  );
}

export default function CandidatesPage() {
  return (
    <div>
      <PageHeader title="Поиск по базе" description="Все кандидаты с подтверждённой категорией. Внутри категории выше — сильнее тест и достижения ФСП. Контакты скрыты до согласия кандидата." />
      <Suspense fallback={<PageSkeleton />}>
        <SearchView />
      </Suspense>
    </div>
  );
}
