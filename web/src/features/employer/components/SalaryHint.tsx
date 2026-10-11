"use client";

import { TrendingUp } from "lucide-react";
import { useEffect, useState } from "react";
import type { Category, SalaryCheckQuery } from "@/api/types";
import { Alert } from "@/components/ui";
import { useSalaryCheck } from "@/features/employer/hooks";
import { cn } from "@/lib/cn";
import { formatRub } from "@/lib/format";
import { GRADE_LABEL, SPEC_LABEL } from "@/lib/labels";

const DEBOUNCE_MS = 450;

function useDebounced<T>(value: T): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setDebounced(value), DEBOUNCE_MS);
    return () => clearTimeout(t);
  }, [value]);
  return debounced;
}

/**
 * The category market next to the salary fields, and a soft warning when the range looks off.
 * Saving is never blocked: the warning only asks to double-check.
 */
export function SalaryHint({ category, min, max, className }: { category: Category | null | undefined; min: number | null; max: number | null; className?: string }) {
  const ready = category && min && max && max >= min;
  const key = ready ? `${category.specialization}:${category.grade}:${min}:${max}` : "";
  const debounced = useDebounced(key);
  const query: SalaryCheckQuery | null = (() => {
    if (!debounced) return null;
    const [specialization, grade, lo, hi] = debounced.split(":");
    return { specialization, grade, min: Number(lo), max: Number(hi) } as SalaryCheckQuery;
  })();
  const check = useSalaryCheck(query);
  if (!ready || !check.data) return null;
  const { market, warnings } = check.data;
  const name = `${SPEC_LABEL[check.data.category.specialization]} · ${GRADE_LABEL[check.data.category.grade]}`;
  return (
    <div className={cn("space-y-3", className)} aria-live="polite">
      {market && (
        <p className="flex items-start gap-2 text-xs text-ink-subtle">
          <TrendingUp className="mt-px size-3.5 shrink-0" aria-hidden />
          <span>
            Рынок {name}: медиана ожиданий {formatRub(market.median)}, 90 % кандидатов ожидают до {formatRub(market.p90)}
          </span>
        </p>
      )}
      {warnings.length > 0 && (
        <Alert tone="warning" title="Проверьте вилку">
          <ul className="space-y-1.5">
            {warnings.map((w) => (
              <li key={w.kind}>{w.text}</li>
            ))}
          </ul>
        </Alert>
      )}
    </div>
  );
}
