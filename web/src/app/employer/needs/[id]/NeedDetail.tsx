"use client";

import { ArrowRight, ChevronDown, FlaskConical, Pencil, Search, Sparkles, Trophy, Users } from "lucide-react";
import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { isApiError } from "@/api/client";
import type { HiringNeed, NeedCategory, ShortlistFilters } from "@/api/types";
import { CategoryBadge, SalaryRange } from "@/components/domain/Category";
import { Alert, Badge, ButtonLink, Card, CardHeader, Chip, CodeBlock, EmptyState, ErrorAlert, Markdown, PageHeader, PageSkeleton, ProgressBar, SkeletonCard, useToast } from "@/components/ui";
import { useNeed, useNeedCategories, useNeedTestPlan, usePatchShortlist, useRefineShortlist, useShortlist } from "@/features/employer/hooks";
import { InviteDialog, type InviteTarget } from "@/features/employer/components/InviteDialog";
import { RefinePanel, ShortlistCard } from "@/features/employer/components/Shortlist";
import { cn } from "@/lib/cn";
import { formatDateTime, formatDuration, formatRub, pluralize } from "@/lib/format";
import { GRADE_LABEL, GRADES, TOPIC_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

function categoryReason(c: NeedCategory, need: HiringNeed): string {
  const idx = GRADES.indexOf(c.category.grade);
  const adjacent = need.grades.some((g) => Math.abs(GRADES.indexOf(g) - idx) === 1);
  let reason = c.requested ? "Совпадает с запрошенным грейдом" : adjacent ? "Соседний грейд — можно расширить поиск" : "Далеко от запроса";
  if (c.median_salary && c.median_salary > need.salary.max) reason += `; медианные ожидания выше вилки`;
  return reason;
}

function CategoryCards({ need }: { need: HiringNeed }) {
  const q = useNeedCategories(need.id);
  if (q.isPending) {
    return (
      <div className="grid gap-4 md:grid-cols-3">
        {[0, 1, 2].map((i) => (
          <SkeletonCard key={i} lines={3} />
        ))}
      </div>
    );
  }
  if (q.isError) return <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />;
  return (
    <div className="grid gap-4 md:grid-cols-3">
      {q.data.map((c) => {
        const overFork = Boolean(c.median_salary && c.median_salary > need.salary.max);
        return (
          <Link
            key={c.category.grade}
            href={`/employer/candidates?specialization=${c.category.specialization}&grade=${c.category.grade}`}
            className={cn("group rounded-card border bg-white p-5 shadow-card transition-colors hover:border-primary-300", c.requested ? "border-primary-200 ring-1 ring-primary-100" : "border-line")}
          >
            <div className="flex items-center justify-between gap-2">
              <CategoryBadge category={c.category} size="sm" />
              {c.requested ? <Badge tone="solid">Рекомендуем</Badge> : <Badge tone="muted">Соседняя</Badge>}
            </div>
            <div className="mt-4 grid grid-cols-2 gap-3">
              <div>
                <p className="text-xs text-ink-subtle">Кандидатов</p>
                <p className="tabular flex items-center gap-1.5 font-mono text-xl font-semibold text-ink">
                  <Users className="size-4 text-ink-subtle" aria-hidden />
                  {c.candidates}
                </p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">С достижениями ФСП</p>
                <p className="tabular flex items-center gap-1.5 font-mono text-xl font-semibold text-ink">
                  <Trophy className="size-4 text-primary" aria-hidden />
                  {c.with_fsp}
                </p>
              </div>
            </div>
            <div className="mt-4">
              <p className="text-xs text-ink-subtle">Медианные ожидания</p>
              <p className={cn("tabular font-mono font-semibold", overFork ? "text-warning" : "text-ink")}>{c.median_salary ? formatRub(c.median_salary) : "—"}</p>
            </div>
            <div className="mt-3">
              <ProgressBar value={c.stack_match * 100} size="xs" caption={`Совпадение со стеком: ${Math.round(c.stack_match * 100)} %`} label="Совпадение со стеком" />
            </div>
            <p className="mt-3 text-xs text-ink-muted">{categoryReason(c, need)}</p>
            <span className="mt-3 inline-flex items-center gap-1 text-sm font-medium text-primary group-hover:underline">
              Смотреть категорию <ArrowRight className="size-4" aria-hidden />
            </span>
          </Link>
        );
      })}
    </div>
  );
}

function ShortlistSection({ need }: { need: HiringNeed }) {
  const toast = useToast();
  const q = useShortlist(need.id);
  const refine = useRefineShortlist(need.id);
  const patch = usePatchShortlist(need.id);
  const [invite, setInvite] = useState<InviteTarget | null>(null);
  const [refined, setRefined] = useState(false);
  const autoBuilt = useRef(false);

  const generated = q.data?.need.shortlist_generated_at;
  useEffect(() => {
    if (q.data && !generated && !autoBuilt.current) {
      autoBuilt.current = true;
      refine.mutate(q.data.need.filters ?? {});
    }
  }, [q.data, generated, refine]);

  const apply = (filters: ShortlistFilters) =>
    refine.mutate(filters, {
      onSuccess: (s) => {
        setRefined(true);
        const fresh = s.items.filter((i) => i.is_new && !i.dismissed).length;
        toast.success("Подборка уточнена", fresh ? `Новых кандидатов: ${fresh}` : "Новых кандидатов нет — закреплённые на месте");
      },
      onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось уточнить подборку", isApiError(e) ? (e.fieldError("skills") ?? e.detail ?? undefined) : undefined),
    });

  const shortlist = q.data;
  const visible = shortlist?.items.filter((i) => !i.dismissed) ?? [];
  const dismissed = (shortlist?.items.length ?? 0) - visible.length;
  const fresh = visible.filter((i) => i.is_new).length;
  const building = refine.isPending && !generated;

  return (
    <div className="grid gap-6 lg:grid-cols-[18rem_1fr]">
      {shortlist ? <RefinePanel key={shortlist.need.shortlist_generated_at ?? "initial"} need={shortlist.need} pending={refine.isPending} onApply={apply} /> : <SkeletonCard lines={6} />}

      <section aria-label="Подборка кандидатов" aria-busy={q.isFetching || refine.isPending} className="min-w-0 space-y-4">
        {q.isPending || building ? (
          <>
            {building && <p className="text-sm text-ink-muted">Формируем подборку: ранжируем кандидатов категории и готовим объяснения…</p>}
            <SkeletonCard lines={4} />
            <SkeletonCard lines={4} />
          </>
        ) : q.isError ? (
          <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
        ) : (
          <>
            <div className="flex flex-wrap items-center justify-between gap-2 text-sm">
              <p className="text-ink">
                <span className="font-semibold">{pluralize(visible.length, ["кандидат", "кандидата", "кандидатов"])}</span>
                {fresh > 0 && <span className="text-primary"> · {fresh} новых</span>}
                {dismissed > 0 && <span className="text-ink-subtle"> · скрыто {dismissed}</span>}
              </p>
              <p className="text-xs text-ink-subtle">Сначала закреплённые, затем по баллу{generated ? ` · обновлено ${formatDateTime(generated)}` : ""}</p>
            </div>
            {refined && fresh > 0 && (
              <Alert tone="info" title={`После уточнения добавилось ${pluralize(fresh, ["кандидат", "кандидата", "кандидатов"])}`}>
                Они отмечены меткой «Новый». Закреплённые кандидаты сохранены, даже если не подходят под новые условия.
              </Alert>
            )}
            {visible.length === 0 ? (
              <EmptyState
                icon={<Search className="size-5" aria-hidden />}
                title="Под эти условия кандидатов нет"
                description="Ослабьте условия: добавьте соседний грейд, уберите обязательный навык или формат. Можно также посмотреть всю категорию в поиске."
                action={<ButtonLink href={`/employer/candidates?specialization=${need.specialization}`} variant="secondary">Поиск по базе</ButtonLink>}
              />
            ) : (
              <ul className="space-y-4">
                {visible.map((item) => (
                  <li key={item.candidate.id}>
                    <ShortlistCard
                      item={item}
                      need={shortlist?.need ?? need}
                      busy={patch.isPending && patch.variables?.candidateId === item.candidate.id}
                      onPin={(pinned) => patch.mutate({ candidateId: item.candidate.id, patch: { pinned } })}
                      onDismiss={() => patch.mutate({ candidateId: item.candidate.id, patch: { dismissed: true } }, { onSuccess: () => toast.info("Кандидат скрыт из подборки", "Он не появится снова при уточнении") })}
                      onInvite={() => setInvite({ id: item.candidate.id, code: item.candidate.code, display_name: item.candidate.display_name, category: item.candidate.category, minor: item.candidate.minor })}
                    />
                  </li>
                ))}
              </ul>
            )}
          </>
        )}
      </section>

      <InviteDialog open={invite !== null} onClose={() => setInvite(null)} target={invite} need={shortlist?.need ?? need} />
    </div>
  );
}

function TestPlanCard({ need }: { need: HiringNeed }) {
  const [open, setOpen] = useState(false);
  const q = useNeedTestPlan(need.id, open);
  return (
    <Card>
      <button type="button" aria-expanded={open} onClick={() => setOpen((v) => !v)} className="flex w-full items-start justify-between gap-3 text-left">
        <span className="flex items-start gap-3">
          <FlaskConical className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden />
          <span>
            <span className="block text-base font-semibold text-ink">Как проверяется категория под эту потребность</span>
            <span className="mt-1 block text-sm text-ink-muted">Темы теста и примеры заданий: двум кандидатам выпадают разные варианты одинаковой сложности</span>
          </span>
        </span>
        <ChevronDown className={cn("mt-1 size-5 shrink-0 text-ink-subtle transition-transform", open && "rotate-180")} aria-hidden />
      </button>
      {open && (
        <div className="mt-5 border-t border-line pt-5">
          {q.isPending ? (
            <SkeletonCard lines={4} className="border-0 p-0" />
          ) : q.isError ? (
            <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
          ) : (
            <div className="space-y-5">
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <CategoryBadge category={{ specialization: q.data.plan.specialization, grade: q.data.plan.grade }} size="sm" />
                <span className="text-ink-muted">
                  {q.data.plan.min_items}–{q.data.plan.max_items} заданий, тест подстраивается под ответы кандидата
                </span>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {q.data.plan.topics.map((t) => (
                  <Chip key={t.topic} tone="primary">
                    {t.topic_ru || TOPIC_LABEL[t.topic as keyof typeof TOPIC_LABEL] || t.topic}
                  </Chip>
                ))}
              </div>
              <div className="grid gap-4 lg:grid-cols-2">
                {q.data.samples.map((s) => (
                  <div key={s.label} className="min-w-0 rounded-control border border-line p-4">
                    <p className="mb-3 text-sm font-semibold text-ink">{s.label}</p>
                    <ol className="space-y-4">
                      {s.items.map((item, i) => (
                        <li key={`${item.template_id}-${i}`} className="text-sm">
                          <p className="flex flex-wrap items-center gap-2 text-xs text-ink-subtle">
                            <Badge tone="neutral">{TOPIC_LABEL[item.topic]}</Badge>
                            {item.title} · уровень {item.level + 1} · {formatDuration(item.time_limit_secs)}
                          </p>
                          <Markdown source={item.prompt} className="mt-1.5 text-sm" />
                          {item.code && <CodeBlock code={item.code.source} language={item.code.language} className="mt-2" />}
                        </li>
                      ))}
                    </ol>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </Card>
  );
}

export function NeedDetail({ id }: { id: string }) {
  const q = useNeed(id);
  if (q.isPending) return <PageSkeleton cards={3} />;
  if (q.isError) {
    return (
      <div>
        <PageHeader back={{ href: "/employer/needs", label: "Потребности" }} title="Потребность не найдена" />
        <ErrorAlert error={q.error} onRetry={() => void q.refetch()} />
      </div>
    );
  }
  const need = q.data;
  return (
    <div>
      <PageHeader
        back={{ href: "/employer/needs", label: "Потребности" }}
        eyebrow={
          <span className="flex flex-wrap gap-2">
            {need.grades.map((g) => (
              <CategoryBadge key={g} category={{ specialization: need.specialization, grade: g }} size="sm" />
            ))}
          </span>
        }
        title={need.title}
        description={
          <span className="flex flex-wrap items-center gap-x-4 gap-y-1">
            <SalaryRange salary={need.salary} size="sm" />
            <span>
              {need.work_formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ")}
              {need.city ? ` · ${need.city}` : ""}
            </span>
            {need.allow_adjacent_grades && <span>соседние грейды допустимы</span>}
          </span>
        }
        actions={
          <ButtonLink href={`/employer/needs/${need.id}/edit`} variant="secondary" icon={<Pencil className="size-4" aria-hidden />}>
            Изменить
          </ButtonLink>
        }
      />

      <div className="mb-8 flex flex-wrap items-center gap-1.5">
        <span className="mr-1 text-sm text-ink-subtle">Обязательно:</span>
        {need.must_skills.map((s) => (
          <Chip key={s} tone="primary">
            {skillName(s)}
          </Chip>
        ))}
        {need.nice_skills.length > 0 && <span className="mr-1 ml-3 text-sm text-ink-subtle">Плюсом:</span>}
        {need.nice_skills.map((s) => (
          <Chip key={s}>{skillName(s)}</Chip>
        ))}
      </div>

      <section className="mb-10">
        <CardHeader
          as="h2"
          title="Рекомендованные категории"
          description={`Кандидаты с подтверждённым тестом грейдом. Запрошено: ${need.grades.map((g) => GRADE_LABEL[g]).join(", ")}`}
          eyebrow={
            <span className="inline-flex items-center gap-1.5 font-mono text-xs tracking-wider text-primary uppercase">
              <Sparkles className="size-3.5" aria-hidden /> Шаг 1
            </span>
          }
        />
        <CategoryCards need={need} />
      </section>

      <section className="mb-10">
        <CardHeader
          as="h2"
          title="Подборка"
          description="Балл 0–100 с объяснением: что совпало и чего не хватает. Внутри категории выше — сильнее тест и достижения ФСП."
          eyebrow={
            <span className="inline-flex items-center gap-1.5 font-mono text-xs tracking-wider text-primary uppercase">
              <Sparkles className="size-3.5" aria-hidden /> Шаг 2
            </span>
          }
        />
        <ShortlistSection need={need} />
      </section>

      <TestPlanCard need={need} />
    </div>
  );
}
