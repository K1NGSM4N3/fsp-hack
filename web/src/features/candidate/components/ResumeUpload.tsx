"use client";

import { FileText, FileUp, Info } from "lucide-react";
import { useId, useRef, useState, type DragEvent } from "react";
import type { Industry, ParsedResume, ResumeDraft, WorkFormat } from "@/api/types";
import { Alert, Button, Checkbox, ErrorAlert, Spinner } from "@/components/ui";
import { useUploadResume } from "@/features/candidate/hooks";
import { cn } from "@/lib/cn";
import { formatRub } from "@/lib/format";
import { GRADE_LABEL, INDUSTRY_LABEL, SPEC_LONG_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";

export interface ResumeApply {
  full_name?: string;
  email?: string;
  phone?: string;
  telegram?: string;
  city?: string;
  relocation?: boolean;
  skills?: string[];
  experience_years?: number;
  roles?: string[];
  soft_skills?: string[];
  industries?: Industry[];
  work_formats?: WorkFormat[];
  salary?: number;
}

type Key = keyof ResumeApply;

interface Row {
  key: Key;
  label: string;
  display: string | null;
}

function rows(p: ParsedResume): Row[] {
  const list = (xs: string[] | undefined | null) => (xs && xs.length ? xs.join(", ") : null);
  return [
    { key: "full_name", label: "ФИО", display: p.full_name ?? null },
    { key: "email", label: "Email", display: p.email ?? null },
    { key: "phone", label: "Телефон", display: p.phone ?? null },
    { key: "telegram", label: "Telegram", display: p.telegram ?? null },
    { key: "city", label: "Город", display: p.city ?? null },
    { key: "skills", label: "Навыки", display: list(p.skills.map(skillName)) },
    { key: "experience_years", label: "Стаж, лет", display: p.experience_years !== null && p.experience_years !== undefined ? String(p.experience_years) : null },
    { key: "roles", label: "Роли", display: list(p.roles) },
    { key: "soft_skills", label: "Софт-скиллы", display: list(p.soft_skills) },
    { key: "industries", label: "Отрасли", display: list(p.industries.map((i) => INDUSTRY_LABEL[i])) },
    { key: "work_formats", label: "Формат", display: list(p.work_formats.map((f) => WORK_FORMAT_LABEL[f])) },
    { key: "salary", label: "Ожидания", display: p.salary_expectation ? `от ${formatRub(p.salary_expectation)}` : null },
  ];
}

function toApply(p: ParsedResume, selected: Set<Key>): ResumeApply {
  const out: ResumeApply = {};
  const on = (k: Key) => selected.has(k);
  if (on("full_name") && p.full_name) out.full_name = p.full_name;
  if (on("email") && p.email) out.email = p.email;
  if (on("phone") && p.phone) out.phone = p.phone;
  if (on("telegram") && p.telegram) out.telegram = p.telegram;
  if (on("city") && p.city) out.city = p.city;
  if (on("skills")) out.skills = p.skills;
  if (on("experience_years") && typeof p.experience_years === "number") out.experience_years = p.experience_years;
  if (on("roles")) out.roles = p.roles;
  if (on("soft_skills")) out.soft_skills = p.soft_skills;
  if (on("industries")) out.industries = p.industries;
  if (on("work_formats") && p.work_formats.length) out.work_formats = p.work_formats;
  if (on("salary") && p.salary_expectation) out.salary = p.salary_expectation;
  if (typeof p.relocation === "boolean") out.relocation = p.relocation;
  return out;
}

export function ResumeUpload({ onApply }: { onApply: (values: ResumeApply) => void }) {
  const inputId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const upload = useUploadResume();
  const [drag, setDrag] = useState(false);
  const [selected, setSelected] = useState<Set<Key>>(new Set());
  const [fileName, setFileName] = useState<string | null>(null);

  const handle = (file: File | undefined) => {
    if (!file) return;
    setFileName(file.name);
    upload.mutate(file, {
      onSuccess: (r: ResumeDraft) => setSelected(new Set(rows(r.parsed).filter((x) => x.display).map((x) => x.key))),
    });
  };

  const onDrop = (e: DragEvent<HTMLLabelElement>) => {
    e.preventDefault();
    setDrag(false);
    handle(e.dataTransfer.files[0]);
  };

  const parsed = upload.data?.parsed;

  return (
    <div className="space-y-5">
      <label
        htmlFor={inputId}
        onDragOver={(e) => {
          e.preventDefault();
          setDrag(true);
        }}
        onDragLeave={() => setDrag(false)}
        onDrop={onDrop}
        className={cn(
          "flex cursor-pointer flex-col items-center justify-center rounded-card border border-dashed px-6 py-10 text-center transition-colors",
          drag ? "border-primary bg-primary-50" : "border-line-strong bg-white hover:border-primary",
        )}
      >
        {upload.isPending ? (
          <>
            <Spinner className="size-7 text-primary" />
            <p className="mt-3 font-medium text-ink">Распознаём резюме…</p>
            <p className="mt-1 text-sm text-ink-muted">Извлекаем ФИО, контакты, стек, роли и стаж</p>
          </>
        ) : (
          <>
            <span className="flex size-12 items-center justify-center rounded-full bg-primary-50 text-primary">
              <FileUp className="size-6" aria-hidden />
            </span>
            <p className="mt-3 font-medium text-ink">Перетащите PDF сюда или выберите файл</p>
            <p className="mt-1 text-sm text-ink-muted">До 5 МБ. Ничего не сохранится, пока вы не подтвердите поля.</p>
          </>
        )}
        <input
          ref={inputRef}
          id={inputId}
          type="file"
          accept="application/pdf,.pdf"
          className="sr-only"
          onChange={(e) => {
            handle(e.target.files?.[0]);
            e.target.value = "";
          }}
        />
      </label>

      {upload.isError && <ErrorAlert error={upload.error} />}

      {parsed && (
        <div className="rounded-card border border-line bg-white">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-5 py-4">
            <div className="flex items-center gap-2 text-sm">
              <FileText className="size-4 text-primary" aria-hidden />
              <span className="font-medium text-ink">{fileName}</span>
            </div>
            <span className="text-xs text-ink-subtle">Отметьте поля, которые перенести в профиль</span>
          </div>
          <ul className="divide-y divide-line">
            {rows(parsed).map((r) => (
              <li key={r.key} className="px-5 py-3">
                {r.display ? (
                  <Checkbox
                    checked={selected.has(r.key)}
                    onChange={(e) => {
                      const next = new Set(selected);
                      if (e.target.checked) next.add(r.key);
                      else next.delete(r.key);
                      setSelected(next);
                    }}
                    label={
                      <span className="flex flex-col sm:flex-row sm:gap-3">
                        <span className="w-28 shrink-0 text-ink-subtle">{r.label}</span>
                        <span className="text-ink">{r.display}</span>
                      </span>
                    }
                  />
                ) : (
                  <div className="flex flex-col text-sm sm:flex-row sm:gap-3">
                    <span className="w-28 shrink-0 pl-7 text-ink-subtle">{r.label}</span>
                    <span className="text-ink-faint">не найдено</span>
                  </div>
                )}
              </li>
            ))}
          </ul>
          {(parsed.specialization_hint || parsed.grade_hint) && (
            <div className="border-t border-line px-5 py-3">
              <Alert tone="info" icon={<Info className="size-5 text-primary" aria-hidden />}>
                По резюме похоже на {parsed.specialization_hint ? SPEC_LONG_LABEL[parsed.specialization_hint.specialization] : "вашу специализацию"}
                {parsed.grade_hint ? `, уровень ${GRADE_LABEL[parsed.grade_hint]}` : ""}. Категорию определит только тест — резюме помогает подбору как дополнительный сигнал.
              </Alert>
            </div>
          )}
          <div className="flex flex-wrap justify-end gap-2 border-t border-line bg-canvas/60 px-5 py-3">
            <Button variant="ghost" onClick={() => inputRef.current?.click()}>
              Другой файл
            </Button>
            <Button onClick={() => onApply(toApply(parsed, selected))} disabled={selected.size === 0}>
              Перенести в форму ({selected.size})
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
