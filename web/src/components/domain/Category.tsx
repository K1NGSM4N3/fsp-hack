import type { Category, SalaryRange as Salary } from "@/api/types";
import { cn } from "@/lib/cn";
import { formatNumber, formatSalaryRange } from "@/lib/format";
import { GRADE_LABEL, SPEC_LABEL, SPEC_LONG_LABEL } from "@/lib/labels";

export function CategoryBadge({ category, size = "md", long = false, className }: { category: Category; size?: "sm" | "md"; long?: boolean; className?: string }) {
  return (
    <span
      title={`${SPEC_LONG_LABEL[category.specialization]} · ${GRADE_LABEL[category.grade]}`}
      className={cn(
        "inline-flex max-w-full items-center overflow-hidden rounded-md border border-primary-200 bg-primary-50 font-mono font-medium text-primary-strong",
        size === "sm" ? "h-6 text-xs" : "h-7 text-[13px]",
        className,
      )}
    >
      <span className="truncate px-2">{long ? SPEC_LONG_LABEL[category.specialization] : SPEC_LABEL[category.specialization]}</span>
      <span className="h-full w-px bg-primary-200" aria-hidden />
      <span className="px-2">{GRADE_LABEL[category.grade]}</span>
    </span>
  );
}

export function SalaryRange({ salary, size = "md", className, label }: { salary: Salary; size?: "sm" | "md" | "lg" | "xl"; className?: string; label?: string }) {
  // `xl` is the headline amount of a card marked `@container`: it never wraps, so it scales with the card.
  const sizes = { sm: "text-sm", md: "text-base", lg: "text-xl", xl: "text-[clamp(1.25rem,8.5cqi,1.875rem)]" };
  return (
    <span className={cn("tabular font-mono font-semibold tracking-tight whitespace-nowrap text-ink", sizes[size], className)}>
      {label && <span className="sr-only">{label}: </span>}
      {formatSalaryRange(salary.min, salary.max)}
    </span>
  );
}

export function Percentile({ value, className }: { value: number; className?: string }) {
  return (
    <span className={cn("tabular font-mono", className)} title="Перцентиль внутри категории: доля соискателей категории с более слабым результатом теста">
      {formatNumber(value)}-й перцентиль
    </span>
  );
}
