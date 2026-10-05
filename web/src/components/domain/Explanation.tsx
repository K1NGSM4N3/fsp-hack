"use client";

import { Check, ChevronDown, X } from "lucide-react";
import { useId, useState } from "react";
import type { ScoreBreakdown } from "@/api/types";
import { cn } from "@/lib/cn";
import { SKILL_SOURCE_SHORT } from "@/lib/labels";

export function ScoreRing({ score, size = 56, label = "Соответствие" }: { score: number; size?: number; label?: string }) {
  const value = Math.round(score);
  const r = (size - 6) / 2;
  const c = 2 * Math.PI * r;
  const tone = value >= 75 ? "text-primary" : value >= 55 ? "text-primary-300" : "text-ink-faint";
  return (
    <div className="relative shrink-0" style={{ width: size, height: size }} role="img" aria-label={`${label}: ${value} из 100`}>
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} className="-rotate-90" aria-hidden>
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="currentColor" strokeWidth="4" className="text-surface" />
        <circle
          cx={size / 2}
          cy={size / 2}
          r={r}
          fill="none"
          stroke="currentColor"
          strokeWidth="4"
          strokeLinecap="round"
          strokeDasharray={c}
          strokeDashoffset={c * (1 - Math.max(0, Math.min(100, value)) / 100)}
          className={cn("transition-[stroke-dashoffset] duration-700", tone)}
        />
      </svg>
      <span className="tabular absolute inset-0 flex items-center justify-center font-mono text-base font-semibold text-ink">{value}</span>
    </div>
  );
}

export function ReasonList({ reasons, gaps, className }: { reasons: string[]; gaps: string[]; className?: string }) {
  return (
    <ul className={cn("space-y-1.5 text-sm", className)}>
      {reasons.map((r) => (
        <li key={`r-${r}`} className="flex items-start gap-2">
          <Check className="mt-0.5 size-4 shrink-0 text-success" aria-label="Плюс:" />
          <span className="text-ink">{r}</span>
        </li>
      ))}
      {gaps.map((g) => (
        <li key={`g-${g}`} className="flex items-start gap-2">
          <X className="mt-0.5 size-4 shrink-0 text-accent-strong" aria-label="Пробел:" />
          <span className="text-ink-muted">{g}</span>
        </li>
      ))}
    </ul>
  );
}

/** Why this candidate is in the list: top reasons, gaps and the per-feature breakdown of the 0–100 score. */
export function ExplanationPanel({ breakdown, defaultOpen = false, className }: { breakdown: ScoreBreakdown; defaultOpen?: boolean; className?: string }) {
  const [open, setOpen] = useState(defaultOpen);
  const id = useId();
  const features = breakdown.features.filter((f) => f.weight > 0);
  const max = Math.max(...features.map((f) => f.weight * 100), 1);
  return (
    <div className={className}>
      <ReasonList reasons={breakdown.reasons} gaps={breakdown.gaps} />
      <button
        type="button"
        aria-expanded={open}
        aria-controls={id}
        onClick={() => setOpen((v) => !v)}
        className="mt-3 inline-flex items-center gap-1 rounded text-sm font-medium text-primary hover:underline"
      >
        {open ? "Скрыть расчёт" : "Как посчитан балл"}
        <ChevronDown className={cn("size-4 transition-transform", open && "rotate-180")} aria-hidden />
      </button>
      {open && (
        <div id={id} className="mt-3 space-y-4 rounded-control border border-line bg-canvas/60 p-3">
          <div className="space-y-2.5">
            {features.map((f) => (
              <div key={f.key} className="grid grid-cols-[minmax(0,7.5rem)_1fr_auto] items-center gap-x-3 gap-y-0.5 text-xs sm:grid-cols-[10rem_1fr_auto]">
                <span className="truncate font-medium text-ink" title={f.label_ru}>
                  {f.label_ru}
                </span>
                <span className="relative h-2 overflow-hidden rounded-full bg-surface" aria-hidden>
                  <span className="absolute inset-y-0 left-0 rounded-full bg-primary-200" style={{ width: `${((f.weight * 100) / max) * 100}%` }} />
                  <span className="absolute inset-y-0 left-0 rounded-full bg-primary" style={{ width: `${((f.contribution / max) * 100).toFixed(1)}%` }} />
                </span>
                <span className="tabular w-16 text-right font-mono text-ink">
                  {f.contribution.toFixed(1)}
                  <span className="text-ink-subtle">/{Math.round(f.weight * 100)}</span>
                </span>
                <span className="col-span-3 text-ink-subtle">{f.explanation_ru}</span>
              </div>
            ))}
          </div>
          {breakdown.skills.length > 0 && (
            <div className="border-t border-line pt-3">
              <p className="mb-2 text-xs font-medium text-ink">Навыки из запроса</p>
              <ul className="grid gap-1.5 text-xs sm:grid-cols-2">
                {breakdown.skills.map((s) => (
                  <li key={`${s.skill}-${s.required}`} className="flex items-center justify-between gap-2 rounded bg-white px-2 py-1.5">
                    <span className="min-w-0 truncate">
                      <span className={cn("font-medium", s.credit >= 0.5 ? "text-ink" : "text-ink-subtle")}>{s.skill}</span>
                      {!s.required && <span className="text-ink-subtle"> · желательный</span>}
                      {s.matched_by && s.similarity < 0.99 && <span className="text-ink-subtle"> ≈ {s.matched_by}</span>}
                    </span>
                    <span className="shrink-0 text-ink-subtle">
                      {s.source ? SKILL_SOURCE_SHORT[s.source] : "нет"} · <span className="tabular font-mono text-ink">{s.credit.toFixed(2)}</span>
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          )}
          <p className="border-t border-line pt-2 text-xs text-ink-subtle">
            Итог — сумма баллов по признакам ({Math.round(breakdown.total)} из 100). Светлая полоса — максимум признака, тёмная — сколько набрал кандидат. Навык засчитывается с весом источника: тест и ФСП — 1,0, резюме — 0,7, заявлен — 0,5.
          </p>
        </div>
      )}
    </div>
  );
}
