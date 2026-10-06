"use client";

import { ScanText, Sparkles } from "lucide-react";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { isApiError } from "@/api/client";
import type { ParsedNeed } from "@/api/types";
import { Alert, Badge, Button, Card, CardHeader, Chip, ErrorAlert, Field, PageHeader, Textarea, useToast } from "@/components/ui";
import { useCreateNeed, useParseNeed } from "@/features/employer/hooks";
import { EMPTY_NEED, NeedForm, type NeedDraft } from "@/features/employer/components/NeedForm";
import { formatSalaryRange } from "@/lib/format";
import { GRADE_LABEL, SPEC_LONG_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

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

export default function NewNeedPage() {
  const router = useRouter();
  const toast = useToast();
  const parse = useParseNeed();
  const create = useCreateNeed();
  const [text, setText] = useState("");
  const [draft, setDraft] = useState<NeedDraft | null>(null);
  const [formKey, setFormKey] = useState(0);

  const runParse = () =>
    parse.mutate(text, {
      onSuccess: (p) => {
        setDraft(fromParse(p, text));
        setFormKey((k) => k + 1);
      },
    });

  return (
    <div>
      <PageHeader
        back={{ href: "/employer/needs", label: "Потребности" }}
        title="Новая потребность"
        description="Опишите, кого ищете, обычным текстом — система извлечёт специализацию, грейд, стек и вилку. Потом сразу покажем подборку."
      />

      <Card padding="lg" className="mb-6">
        <Field label="Описание" hint="Можно вставить текст вакансии целиком" labelAction={<button type="button" className="tap text-sm font-medium text-primary hover:underline" onClick={() => setText(EXAMPLE)}>Вставить пример</button>}>
          <Textarea rows={7} value={text} onChange={(e) => setText(e.target.value)} placeholder="Ищем middle backend-разработчика на Go в финтех. Обязательно PostgreSQL и Kafka, будет плюсом Kubernetes. Удалённо, 250–320 тыс." />
        </Field>
        {parse.isError && !(isApiError(parse.error) && parse.error.status === 422) && <ErrorAlert error={parse.error} className="mt-4" />}
        {parse.isError && isApiError(parse.error) && parse.error.status === 422 && <Alert tone="warning" className="mt-4" title={parse.error.fieldError("text") ?? parse.error.title} />}
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <Button icon={<Sparkles className="size-4" aria-hidden />} loading={parse.isPending} disabled={text.trim().length < 15} onClick={runParse}>
            Распознать
          </Button>
          {!draft && (
            <Button
              variant="ghost"
              icon={<ScanText className="size-4" aria-hidden />}
              onClick={() => {
                setDraft({ ...EMPTY_NEED, description: text });
                setFormKey((k) => k + 1);
              }}
            >
              Заполнить вручную
            </Button>
          )}
        </div>
      </Card>

      {parse.data && draft && <div className="mb-6"><ParsedSummary p={parse.data} /></div>}

      {draft && (
        <NeedForm
          key={formKey}
          initial={draft}
          submitLabel="Сохранить и показать подборку"
          pending={create.isPending}
          serverError={isApiError(create.error) ? create.error : null}
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
      )}
    </div>
  );
}
