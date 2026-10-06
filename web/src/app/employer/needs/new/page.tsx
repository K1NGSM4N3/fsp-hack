"use client";

import { Check, ScanText, Sparkles } from "lucide-react";
import { useRouter } from "next/navigation";
import { useRef, useState } from "react";
import { isApiError } from "@/api/client";
import type { ParsedNeed } from "@/api/types";
import { Alert, Badge, Button, Card, CardHeader, Chip, ErrorAlert, Field, PageHeader, Stepper, Textarea, useToast, type Step } from "@/components/ui";
import { useCreateNeed, useParseNeed } from "@/features/employer/hooks";
import { EMPTY_NEED, NeedForm, type NeedDraft } from "@/features/employer/components/NeedForm";
import { formatSalaryRange } from "@/lib/format";
import { GRADE_LABEL, SPEC_LONG_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

const MIN_TEXT = 15;

const EXAMPLE = `Backend-разработчик в команду платежей
Ищем middle Go-разработчика: платёжный шлюз, СБП и карты, сверки с банками. Обязательно: Go, PostgreSQL, Kafka, Docker. Будет плюсом Kubernetes и ClickHouse.
Гибрид в Москве или удалёнка. Вилка 260–340 тыс.`;

function fromParse(p: ParsedNeed, raw: string): NeedDraft {
  const firstLine = raw.split("\n").map((l) => l.trim()).find(Boolean) ?? "";
  return {
    ...EMPTY_NEED,
    title: firstLine.length <= 80 ? firstLine.replace(/[.:]$/, "") : "",
    specialization: p.specialization?.specialization ?? "",
    grades: p.grades.length ? p.grades : p.grade ? [p.grade] : [],
    must_skills: p.must_skills,
    nice_skills: p.nice_skills,
    min: p.salary_min ?? null,
    max: p.salary_max ?? null,
    work_formats: p.work_formats.length ? p.work_formats : EMPTY_NEED.work_formats,
    city: p.city ?? "",
    description: raw.trim(),
    industries: p.industries,
  };
}

function ParsedSummary({ p }: { p: ParsedNeed }) {
  const items: Array<[string, string | null]> = [
    ["Специализация", p.specialization ? SPEC_LONG_LABEL[p.specialization.specialization] : null],
    ["Грейд", p.grades.length ? p.grades.map((g) => GRADE_LABEL[g]).join(", ") : p.grade ? GRADE_LABEL[p.grade] : null],
    ["Вилка", p.salary_min && p.salary_max ? formatSalaryRange(p.salary_min, p.salary_max) : p.salary_min ? `от ${p.salary_min.toLocaleString("ru-RU")} ₽` : null],
    ["Формат", p.work_formats.length ? p.work_formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ") : null],
    ["Город", p.city ?? null],
    ["Опыт", p.experience_years_min ? `от ${p.experience_years_min} лет` : null],
  ];
  const found = items.filter(([, v]) => v).length + (p.must_skills.length ? 1 : 0);
  return (
    <Card className="border-primary-200 bg-primary-50/30">
      <CardHeader
        title="Распознано"
        description="Проверьте и поправьте поля ниже — потом сформируем подборку"
        action={<Badge tone={found >= 5 ? "success" : "warning"}>Найдено полей: {found} из 7</Badge>}
      />
      <dl className="grid gap-3 text-sm sm:grid-cols-2 lg:grid-cols-3">
        {items.map(([k, v]) => (
          <div key={k}>
            <dt className="text-ink-subtle">{k}</dt>
            <dd className={v ? "text-ink" : "text-ink-faint"}>{v ?? "не найдено"}</dd>
          </div>
        ))}
      </dl>
      <div className="mt-4 space-y-2">
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="mr-1 text-xs text-ink-subtle">Обязательные:</span>
          {p.must_skills.length ? p.must_skills.map((s) => <Chip key={s} tone="primary">{skillName(s)}</Chip>) : <span className="text-xs text-ink-faint">не найдено</span>}
        </div>
        {p.nice_skills.length > 0 && (
          <div className="flex flex-wrap items-center gap-1.5">
            <span className="mr-1 text-xs text-ink-subtle">Желательные:</span>
            {p.nice_skills.map((s) => (
              <Chip key={s}>{skillName(s)}</Chip>
            ))}
          </div>
        )}
      </div>
    </Card>
  );
}

const RECOGNISED = ["Специализация и грейд", "Обязательные и желательные навыки — с синонимами", "Вилка от и до", "Формат работы и город", "Отрасль и требования к опыту"];

export default function NewNeedPage() {
  const router = useRouter();
  const toast = useToast();
  const parse = useParseNeed();
  const create = useCreateNeed();
  const [text, setText] = useState("");
  const [textError, setTextError] = useState<string | null>(null);
  const [draft, setDraft] = useState<NeedDraft | null>(null);
  const [formKey, setFormKey] = useState(0);
  const formRef = useRef<HTMLDivElement>(null);

  const reveal = () => requestAnimationFrame(() => formRef.current?.scrollIntoView({ block: "start", behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" }));

  const runParse = () => {
    if (text.trim().length < MIN_TEXT) {
      setTextError("Опишите вакансию подробнее: хотя бы пару предложений о том, кого ищете");
      return;
    }
    parse.mutate(text, {
      onSuccess: (p) => {
        setDraft(fromParse(p, text));
        setFormKey((k) => k + 1);
        reveal();
      },
    });
  };

  const steps: Step[] = [
    { key: "text", label: "Текст вакансии", hint: "как есть, без шаблона", status: draft ? "done" : "current" },
    { key: "check", label: "Проверка полей", hint: "поправьте, если нужно", status: draft ? "current" : "upcoming" },
    { key: "shortlist", label: "Подборка", hint: "с объяснением балла", status: "upcoming" },
  ];

  return (
    <div className="max-w-6xl">
      <PageHeader
        back={{ href: "/employer/needs", label: "Потребности" }}
        eyebrow="Подбор"
        title="Новая потребность"
        description="Опишите, кого ищете, обычным текстом — система извлечёт специализацию, грейд, стек и вилку. Потом сразу покажем подборку."
      />

      <Stepper steps={steps} ariaLabel="Шаги создания потребности" className="mb-8 max-w-2xl" />

      <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_18.5rem] xl:grid-cols-[minmax(0,1fr)_20rem]">
        <Card padding="lg" className="min-w-0">
          <Field
            label="Описание"
            error={textError ?? undefined}
            hint="Можно вставить текст вакансии целиком"
            labelAction={
              <button
                type="button"
                className="tap text-sm font-medium text-primary hover:underline"
                onClick={() => {
                  setText(EXAMPLE);
                  setTextError(null);
                }}
              >
                Вставить пример
              </button>
            }
          >
            <Textarea
              rows={8}
              value={text}
              onChange={(e) => {
                setText(e.target.value);
                setTextError(null);
              }}
              placeholder="Ищем middle backend-разработчика на Go в финтех. Обязательно PostgreSQL и Kafka, будет плюсом Kubernetes. Удалённо, 250–320 тыс."
            />
          </Field>
          {parse.isError && !(isApiError(parse.error) && parse.error.status === 422) && <ErrorAlert error={parse.error} className="mt-4" />}
          {parse.isError && isApiError(parse.error) && parse.error.status === 422 && <Alert tone="warning" className="mt-4" title={parse.error.fieldError("text") ?? parse.error.title} />}
          <div className="mt-5 flex flex-wrap items-center gap-3">
            <Button icon={<Sparkles className="size-4" aria-hidden />} loading={parse.isPending} onClick={runParse}>
              {draft ? "Распознать заново" : "Распознать"}
            </Button>
            {!draft && (
              <Button
                variant="ghost"
                icon={<ScanText className="size-4" aria-hidden />}
                onClick={() => {
                  setDraft({ ...EMPTY_NEED, description: text });
                  setFormKey((k) => k + 1);
                  reveal();
                }}
              >
                Заполнить вручную
              </Button>
            )}
          </div>
        </Card>

        <aside aria-label="Что распознаётся" className="min-w-0">
          <Card padding="sm" className="bg-canvas/60 p-5">
            <p className="eyebrow text-ink-subtle">Что сервис поймёт из текста</p>
            <ul className="mt-4 space-y-2.5">
              {RECOGNISED.map((r) => (
                <li key={r} className="flex items-start gap-2.5 text-sm text-ink">
                  <Check className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
                  {r}
                </li>
              ))}
            </ul>
            <p className="mt-5 border-t border-line pt-4 text-xs leading-relaxed text-ink-subtle">Любое поле можно поправить вручную. Чем подробнее текст, тем точнее подборка.</p>
          </Card>
        </aside>
      </div>

      <div ref={formRef} className="scroll-mt-20">
        {parse.data && draft && (
          <div className="mt-6">
            <ParsedSummary p={parse.data} />
          </div>
        )}

        {draft && (
          <div className="mt-6">
            <NeedForm
              key={formKey}
              initial={draft}
              submitLabel="Сохранить и показать подборку"
              pending={create.isPending}
              serverError={isApiError(create.error) ? create.error : null}
              onCancel={() => router.push("/employer/needs")}
              onSubmit={(input) =>
                create.mutate(input, {
                  onSuccess: (need) => {
                    toast.success("Потребность сохранена", "Формируем подборку");
                    router.push(`/employer/needs/${need.id}`);
                  },
                  onError: (e) => toast.error(isApiError(e) ? e.title : "Не удалось сохранить"),
                })
              }
            />
          </div>
        )}
      </div>
    </div>
  );
}
