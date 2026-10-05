"use client";

import { ArrowLeft, ArrowRight, Check, ChevronDown, Code2, Database, Plus, Server, ShieldCheck, TestTube2 } from "lucide-react";
import { useRouter } from "next/navigation";
import { useMemo, useState, type ReactNode } from "react";
import { isApiError } from "@/api/client";
import type { Direction, Grade, Industry, Specialization, SurveyResponse } from "@/api/types";
import { SkillInput } from "@/components/domain/Skills";
import { Alert, Button, Card, ChipSelect, Field, FieldSet, Input, PageHeader, PageSkeleton, ProgressBar, RadioCards, ToggleChip, useToast } from "@/components/ui";
import { useCatalog, useTypicalSkills } from "@/features/catalog/hooks";
import { useSubmitSurvey, useSurveyHistory } from "@/features/candidate/hooks";
import { OnboardingStrip } from "@/features/candidate/components/Onboarding";
import { cn } from "@/lib/cn";
import { formatDate } from "@/lib/format";
import {
  DIRECTION_LABEL,
  GRADE_HINT,
  GRADE_LABEL,
  GRADES,
  INDUSTRIES,
  INDUSTRY_LABEL,
  SELF_ASSESSMENT_LABEL,
  SPEC_DESCRIPTION,
  SPEC_DIRECTION,
  SPEC_LONG_LABEL,
  SPEC_TOPICS,
  SPECIALIZATIONS,
  TOPIC_LABEL,
} from "@/lib/labels";
import { skillName } from "@/lib/skills";

const STEPS = ["Направление", "Специализация", "Стек", "Отрасли", "Грейд"] as const;

const DIRECTION_ICON: Record<Direction, ReactNode> = {
  development: <Code2 className="size-5" aria-hidden />,
  infrastructure: <Server className="size-5" aria-hidden />,
  quality: <TestTube2 className="size-5" aria-hidden />,
  data_ai: <Database className="size-5" aria-hidden />,
  security: <ShieldCheck className="size-5" aria-hidden />,
};

const DIRECTIONS: Direction[] = ["development", "infrastructure", "quality", "data_ai", "security"];

interface Draft {
  direction: Direction | null;
  specialization: Specialization | null;
  skills: string[];
  experience_years: string;
  industries: Industry[];
  claimed_grade: Grade | null;
  self_assessment: Record<string, number>;
}

function fromHistory(s: SurveyResponse | undefined): Draft {
  return {
    direction: s?.direction ?? null,
    specialization: s?.specialization ?? null,
    skills: s?.skills ?? [],
    experience_years: s?.experience_years !== null && s?.experience_years !== undefined ? String(s.experience_years) : "",
    industries: s?.industries ?? [],
    claimed_grade: s?.claimed_grade ?? null,
    self_assessment: s?.self_assessment ?? {},
  };
}

function Scale({ topic, value, onChange }: { topic: string; value: number | undefined; onChange: (v: number) => void }) {
  return (
    <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
      <span className="text-sm text-ink">{topic}</span>
      <div role="radiogroup" aria-label={topic} className="flex gap-1">
        {[1, 2, 3, 4, 5].map((n) => (
          <button
            key={n}
            type="button"
            role="radio"
            aria-checked={value === n}
            title={SELF_ASSESSMENT_LABEL[n]}
            onClick={() => onChange(n)}
            className={cn(
              "flex h-9 w-11 items-center justify-center rounded-control border font-mono text-sm transition-colors",
              value === n ? "border-primary bg-primary text-white" : "border-line-strong bg-white text-ink-muted hover:border-primary hover:text-primary",
            )}
          >
            {n}
            <span className="sr-only"> — {SELF_ASSESSMENT_LABEL[n]}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

function History({ items }: { items: SurveyResponse[] }) {
  const [open, setOpen] = useState(false);
  if (!items.length) return null;
  return (
    <Card className="mt-8">
      <button type="button" aria-expanded={open} onClick={() => setOpen((v) => !v)} className="flex w-full items-center justify-between gap-3 text-left">
        <span className="text-base font-semibold text-ink">История опросов</span>
        <ChevronDown className={cn("size-5 text-ink-subtle transition-transform", open && "rotate-180")} aria-hidden />
      </button>
      {open && (
        <ul className="mt-4 divide-y divide-line">
          {items.map((s) => (
            <li key={s.id} className="flex flex-col gap-1 py-3 text-sm sm:flex-row sm:items-center sm:justify-between">
              <span className="text-ink">
                {SPEC_LONG_LABEL[s.specialization]} · заявлен {GRADE_LABEL[s.claimed_grade]}
              </span>
              <span className="text-ink-subtle">
                {formatDate(s.created_at)} · навыков: {s.skills.length}
              </span>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}

export default function SurveyPage() {
  const history = useSurveyHistory();
  if (history.isPending) return <PageSkeleton cards={1} />;
  return <SurveyWizard initial={fromHistory(history.data?.[0])} history={history.data ?? []} />;
}

function SurveyWizard({ initial, history }: { initial: Draft; history: SurveyResponse[] }) {
  const router = useRouter();
  const toast = useToast();
  const catalog = useCatalog();
  const submit = useSubmitSurvey();
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState<Draft>(initial);
  const [error, setError] = useState<string | null>(null);

  const update = (patch: Partial<Draft>) => {
    setError(null);
    setDraft((d) => ({ ...d, ...patch }));
  };

  const specsOf = (dir: Direction | null): Specialization[] => {
    if (!dir) return [];
    const fromCatalog = catalog.data?.directions.find((d) => d.slug === dir)?.specializations.map((s) => s.slug);
    return fromCatalog ?? SPECIALIZATIONS.filter((s) => SPEC_DIRECTION[s] === dir);
  };

  const typical = useTypicalSkills(draft.specialization);
  const suggestions = useMemo(() => (typical.data ?? []).filter((s) => !draft.skills.includes(s.slug)).slice(0, 12), [typical.data, draft.skills]);

  const canNext = [Boolean(draft.direction), Boolean(draft.specialization), draft.skills.length > 0, true, Boolean(draft.claimed_grade)][step];
  const stepHint = ["Выберите направление", "Выберите специализацию", "Добавьте хотя бы один навык", "", "Выберите грейд"][step];

  const finish = () => {
    if (!draft.direction || !draft.specialization || !draft.claimed_grade) return;
    const topics = SPEC_TOPICS[draft.specialization];
    submit.mutate(
      {
        direction: draft.direction,
        specialization: draft.specialization,
        claimed_grade: draft.claimed_grade,
        skills: draft.skills,
        industries: draft.industries,
        experience_years: draft.experience_years === "" ? null : Number(draft.experience_years),
        self_assessment: Object.fromEntries(Object.entries(draft.self_assessment).filter(([k]) => topics.includes(k as (typeof topics)[number]))),
      },
      {
        onSuccess: () => {
          toast.success("Опрос сохранён", "Теперь подтвердите грейд тестом");
          router.push(`/candidate/assessment?grade=${draft.claimed_grade}`);
        },
        onError: (e) => setError(isApiError(e) ? (Object.values(e.errors ?? {})[0]?.[0] ?? e.detail ?? e.title) : "Не удалось сохранить опрос"),
      },
    );
  };

  return (
    <div>
      <PageHeader title="Опрос" description="Специализация и стек определяют, какой тест вы пройдёте. Можно пройти заново — история сохранится." />
      <OnboardingStrip />

      <Card padding="lg">
        <div className="mb-6">
          <div className="mb-2 flex items-center justify-between text-sm">
            <span className="font-mono font-semibold text-ink">{STEPS[step]}</span>
            <span className="tabular text-ink-subtle">
              Шаг {step + 1} из {STEPS.length}
            </span>
          </div>
          <ProgressBar value={step + 1} max={STEPS.length} label="Прогресс опроса" />
        </div>

        <div className="min-h-64">
          {step === 0 && (
            <FieldSet legend="В каком направлении вы работаете?">
              <RadioCards<Direction>
                name="direction"
                columns={2}
                value={draft.direction}
                onChange={(d) => update({ direction: d, specialization: specsOf(d).length === 1 ? (specsOf(d)[0] ?? null) : null })}
                options={DIRECTIONS.map((d) => ({
                  value: d,
                  title: catalog.data?.directions.find((x) => x.slug === d)?.name ?? DIRECTION_LABEL[d],
                  description: specsOf(d)
                    .map((s) => SPEC_LONG_LABEL[s])
                    .join(", "),
                  icon: DIRECTION_ICON[d],
                }))}
              />
            </FieldSet>
          )}

          {step === 1 && (
            <FieldSet legend="Ваша специализация">
              <RadioCards<Specialization>
                name="specialization"
                columns={2}
                value={draft.specialization}
                onChange={(s) => update({ specialization: s })}
                options={specsOf(draft.direction).map((s) => ({ value: s, title: SPEC_LONG_LABEL[s], description: SPEC_DESCRIPTION[s] }))}
              />
            </FieldSet>
          )}

          {step === 2 && (
            <div className="space-y-5">
              <Field label="С какими технологиями работаете?" hint="Начните вводить или выберите из популярных ниже">
                <SkillInput value={draft.skills} onChange={(skills) => update({ skills })} />
              </Field>
              {suggestions.length > 0 && (
                <div>
                  <p className="mb-2 text-xs text-ink-subtle">Популярное в {SPEC_LONG_LABEL[draft.specialization as Specialization].toLowerCase()}</p>
                  <div className="flex flex-wrap gap-2">
                    {suggestions.map((s) => (
                      <ToggleChip key={s.slug} selected={false} onToggle={() => setDraft((d) => ({ ...d, skills: d.skills.includes(s.slug) ? d.skills : [...d.skills, s.slug] }))} icon={<Plus className="size-3.5" aria-hidden />}>
                        {s.name}
                      </ToggleChip>
                    ))}
                  </div>
                </div>
              )}
              <Field label="Опыт в специализации, лет" optional className="max-w-48">
                <Input inputMode="decimal" value={draft.experience_years} onChange={(e) => update({ experience_years: e.target.value.replace(/[^\d.,]/g, "").replace(",", ".") })} />
              </Field>
            </div>
          )}

          {step === 3 && (
            <FieldSet legend="В каких отраслях был опыт?" hint="Необязательно. Отрасль не влияет на категорию, но помогает подбору">
              <ChipSelect<Industry> ariaLabel="Отрасли" options={INDUSTRIES.map((i) => ({ value: i, label: INDUSTRY_LABEL[i] }))} value={draft.industries} onChange={(industries) => update({ industries })} />
            </FieldSet>
          )}

          {step === 4 && draft.specialization && (
            <div className="space-y-8">
              <FieldSet legend="Какой грейд хотите подтвердить?" hint="Тест начнётся с этого уровня. Не прошли — сразу можно пройти на грейд ниже.">
                <RadioCards<Grade>
                  name="grade"
                  columns={3}
                  value={draft.claimed_grade}
                  onChange={(g) => update({ claimed_grade: g })}
                  options={GRADES.map((g) => ({ value: g, title: GRADE_LABEL[g], description: GRADE_HINT[g] }))}
                />
              </FieldSet>
              <FieldSet legend="Оцените себя по темам теста" hint="1 — не знаком, 5 — эксперт. На категорию не влияет, помогает калибровке.">
                <div className="space-y-3">
                  {SPEC_TOPICS[draft.specialization].map((t) => (
                    <Scale key={t} topic={TOPIC_LABEL[t]} value={draft.self_assessment[t]} onChange={(v) => setDraft((d) => ({ ...d, self_assessment: { ...d.self_assessment, [t]: v } }))} />
                  ))}
                </div>
              </FieldSet>
            </div>
          )}
        </div>

        {error && <Alert tone="danger" title={error} className="mt-6" />}

        <div className="mt-8 flex flex-wrap items-center justify-between gap-3 border-t border-line pt-5">
          <Button variant="ghost" icon={<ArrowLeft className="size-4" aria-hidden />} onClick={() => setStep((s) => Math.max(0, s - 1))} disabled={step === 0}>
            Назад
          </Button>
          <div className="flex items-center gap-3">
            {!canNext && stepHint && <span className="hidden text-sm text-ink-subtle sm:inline">{stepHint}</span>}
            {step < STEPS.length - 1 ? (
              <Button onClick={() => setStep((s) => s + 1)} disabled={!canNext}>
                Далее
                <ArrowRight className="size-4" aria-hidden />
              </Button>
            ) : (
              <Button onClick={finish} disabled={!canNext} loading={submit.isPending} icon={<Check className="size-4" aria-hidden />}>
                Сохранить и перейти к тесту
              </Button>
            )}
          </div>
        </div>
      </Card>

      {draft.skills.length > 0 && step !== 2 && (
        <p className="mt-4 text-sm text-ink-muted">
          Стек: {draft.skills.map(skillName).join(", ")}
        </p>
      )}

      <History items={history} />
    </div>
  );
}
