"use client";

import Image from "next/image";
import { Printer } from "lucide-react";
import type { ReactNode } from "react";
import { Button, ErrorAlert, PageSkeleton } from "@/components/ui";
import { CategoryBadge } from "@/components/domain/Category";
import { FspAchievementList } from "@/components/domain/Fsp";
import { SkillLegend, SkillList } from "@/components/domain/Skills";
import { useCategory, useFspStatus, useProfile } from "@/features/candidate/hooks";
import { formatDate, formatRub, pluralize } from "@/lib/format";
import { INDUSTRY_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="break-inside-avoid border-t border-line pt-5">
      <h2 className="mb-3 font-mono text-sm font-semibold tracking-wider text-primary uppercase">{title}</h2>
      {children}
    </section>
  );
}

export default function ProfilePrintPage() {
  const profile = useProfile();
  const category = useCategory();
  const fsp = useFspStatus();

  if (profile.isPending) return <PageSkeleton cards={1} />;
  if (profile.isError) return <ErrorAlert error={profile.error} onRetry={() => void profile.refetch()} />;
  const p = profile.data;
  const roles = p.roles ?? [];
  const formats = p.work_formats ?? [];
  const experience = p.experience ?? [];
  const industries = p.industries ?? [];
  const strength = typeof category.data?.test_strength === "number" ? Math.round(category.data.test_strength * 100) : null;

  return (
    <div className="mx-auto max-w-3xl">
      <div className="mb-6 flex flex-wrap items-center justify-between gap-3 print:hidden">
        <p className="text-sm text-ink-muted">Стандартизированный профиль. Сохраните в PDF через печать браузера.</p>
        <Button icon={<Printer className="size-4" aria-hidden />} onClick={() => window.print()}>
          Печать / сохранить PDF
        </Button>
      </div>
      <article className="space-y-6 rounded-card border border-line bg-white p-8 print:border-0 print:p-0">
        <header className="flex items-start justify-between gap-6">
          <div>
            <h1 className="font-mono text-3xl font-semibold tracking-tight text-ink">{p.full_name || "Без имени"}</h1>
            <p className="mt-1 text-ink-muted">
              {[roles.join(", "), p.city, typeof p.experience_years === "number" ? `опыт ${pluralize(Math.round(p.experience_years), ["год", "года", "лет"])}` : null].filter(Boolean).join(" · ")}
            </p>
            <p className="mt-2 text-sm text-ink">{[p.contacts.email, p.contacts.phone, p.contacts.telegram].filter(Boolean).join(" · ")}</p>
          </div>
          <Image src="/brand/fsp-logo.svg" alt="ФСП" width={98} height={33} unoptimized className="h-8 w-auto shrink-0" />
        </header>

        <Section title="Категория">
          {p.category ? (
            <div className="flex flex-wrap items-center gap-3 text-sm">
              <CategoryBadge category={p.category} long />
              {strength !== null && <span className="text-ink">сила результата {strength}/100</span>}
              <span className="text-ink-subtle">тест от {formatDate(p.category_assigned_at)}</span>
            </div>
          ) : (
            <p className="text-sm text-ink-muted">Тест ещё не пройден</p>
          )}
        </Section>

        <Section title="Условия">
          <p className="text-sm text-ink">
            {formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ")}
            {p.relocation ? " · готов к переезду" : ""}
            {p.salary_expectation_min ? ` · ожидания от ${formatRub(p.salary_expectation_min)}` : ""}
          </p>
        </Section>

        {p.skill_details.length > 0 && (
          <Section title="Навыки">
            <SkillList skills={p.skill_details} />
            <div className="mt-2">
              <SkillLegend />
            </div>
          </Section>
        )}

        {p.about && (
          <Section title="О себе">
            <p className="text-sm leading-relaxed text-ink">{p.about}</p>
          </Section>
        )}

        {experience.length > 0 && (
          <Section title="Опыт">
            <ol className="space-y-4">
              {experience.map((e, i) => (
                <li key={i}>
                  <p className="font-medium text-ink">
                    {e.position} — {e.company}
                  </p>
                  <p className="text-xs text-ink-subtle">
                    {e.start} — {e.end ?? "по настоящее время"}
                  </p>
                  {e.description && <p className="mt-1 text-sm text-ink-muted">{e.description}</p>}
                </li>
              ))}
            </ol>
          </Section>
        )}

        <Section title="Достижения ФСП">
          {fsp.data?.achievements.length ? <FspAchievementList achievements={fsp.data.achievements} /> : <p className="text-sm text-ink-muted">Нет данных ФСП</p>}
        </Section>

        {industries.length > 0 && (
          <Section title="Отрасли">
            <p className="text-sm text-ink">{industries.map((i) => INDUSTRY_LABEL[i]).join(", ")}</p>
          </Section>
        )}

        <footer className="border-t border-line pt-4 text-xs text-ink-subtle">Профиль {p.public_code} · сформирован {formatDate(new Date().toISOString())} · ФСП Таланты</footer>
      </article>
    </div>
  );
}
