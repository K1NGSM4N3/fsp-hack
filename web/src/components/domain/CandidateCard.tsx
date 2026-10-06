import { MapPin, Trophy, UserRound } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import type { CandidateCard as Card } from "@/api/types";
import { Badge } from "@/components/ui";
import { cn } from "@/lib/cn";
import { formatRelative, formatRub, pluralize } from "@/lib/format";
import { SEARCH_STATUS_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { CategoryBadge } from "./Category";
import { SkillList } from "./Skills";

export function candidateTitle(c: { code: string; display_name?: string | null }) {
  return c.display_name || `Кандидат ${c.code}`;
}

/** Neutral on purpose: an age bracket, not a warning. The birth date never reaches employers. */
export function MinorBadge() {
  return (
    <Badge tone="neutral" title="Кандидату от 14 до 18 лет; согласие законного представителя получено">
      14–17 лет
    </Badge>
  );
}

/** test_strength is θ's position inside the grade band, 0..1. */
export function strengthPoints(v: number | null | undefined): number | null {
  return v === null || v === undefined ? null : Math.round(v * 100);
}

export function CandidateFspLine({ fsp }: { fsp: Card["fsp"] }) {
  const top = fsp?.achievements[0];
  if (!fsp || !top) return <span className="text-sm text-ink-subtle">Нет данных ФСП</span>;
  return (
    <span className="inline-flex min-w-0 items-start gap-1.5 text-sm text-ink">
      <Trophy className="mt-0.5 size-4 shrink-0 text-primary" aria-hidden />
      <span className="min-w-0">
        {top.text}
        {fsp.achievements.length > 1 && <span className="text-ink-subtle"> · ещё {fsp.achievements.length - 1}</span>}
      </span>
    </span>
  );
}

interface CandidateCardViewProps {
  candidate: Card;
  href: string;
  highlightSkills?: string[];
  aside?: ReactNode;
  badges?: ReactNode;
  footer?: ReactNode;
  children?: ReactNode;
  className?: string;
}

/** Anonymous card: no name unless allowed, never contacts. */
export function CandidateCardView({ candidate: c, href, highlightSkills, aside, badges, footer, children, className }: CandidateCardViewProps) {
  const strength = strengthPoints(c.test_strength);
  return (
    <article className={cn("min-w-0 rounded-card border border-line bg-white p-4 shadow-card transition-colors hover:border-primary-200 sm:p-5", className)}>
      <div className="flex items-start gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className="flex size-8 items-center justify-center rounded-full bg-canvas text-ink-subtle" aria-hidden>
              <UserRound className="size-4" />
            </span>
            <Link href={href} className="tap rounded font-mono text-[15px] font-semibold text-ink hover:text-primary hover:underline">
              {candidateTitle(c)}
            </Link>
            {c.category && <CategoryBadge category={c.category} size="sm" />}
            {c.minor && <MinorBadge />}
            {badges}
          </div>
          <dl className="mt-3 grid grid-cols-2 gap-x-4 gap-y-2 text-sm sm:grid-cols-4">
            <div>
              <dt className="text-xs text-ink-subtle">Сила теста</dt>
              <dd className="tabular font-mono font-medium text-ink">{strength !== null ? `${strength}/100` : "—"}</dd>
            </div>
            <div>
              <dt className="text-xs text-ink-subtle">Опыт</dt>
              <dd className="text-ink">{c.experience_years !== null && c.experience_years !== undefined ? pluralize(Math.round(c.experience_years), ["год", "года", "лет"]) : "—"}</dd>
            </div>
            <div>
              <dt className="text-xs text-ink-subtle">Ожидания</dt>
              <dd className="tabular font-mono text-ink">{c.salary_expectation_min ? `от ${formatRub(c.salary_expectation_min)}` : "не указаны"}</dd>
            </div>
            <div>
              <dt className="text-xs text-ink-subtle">Активность</dt>
              <dd className="text-ink">{formatRelative(c.last_activity_at)}</dd>
            </div>
          </dl>
        </div>
        {aside}
      </div>
      <div className="mt-3">
        <SkillList skills={c.skills} highlight={highlightSkills} limit={8} />
      </div>
      <div className="mt-3 flex flex-col gap-2 text-sm sm:flex-row sm:items-center sm:justify-between">
        <CandidateFspLine fsp={c.fsp} />
        <span className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-muted">
          <span className="inline-flex items-center gap-1">
            <MapPin className="size-3.5" aria-hidden />
            {c.city ?? "Город не указан"}
            {c.relocation ? " · переезд" : ""}
          </span>
          <span>{c.work_formats.map((f) => WORK_FORMAT_LABEL[f]).join(", ")}</span>
          {c.search_status === "active" && <Badge tone="success">{SEARCH_STATUS_LABEL.active}</Badge>}
        </span>
      </div>
      {children}
      {footer && <div className="mt-4 flex flex-wrap items-center gap-2 border-t border-line pt-4">{footer}</div>}
    </article>
  );
}
