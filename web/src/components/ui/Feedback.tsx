import { AlertTriangle, CheckCircle2, Info, XCircle } from "lucide-react";
import { AnimatedNumber } from "./AnimatedNumber";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

interface ProgressBarProps {
  value: number;
  max?: number;
  /** Accessible name only; use `caption` for visible text. */
  label?: string;
  caption?: ReactNode;
  tone?: "primary" | "success" | "warning" | "danger" | "ink";
  size?: "xs" | "sm" | "md";
  className?: string;
  showValue?: boolean;
}

const BAR_TONES = { primary: "bg-primary", success: "bg-success", warning: "bg-warning", danger: "bg-accent", ink: "bg-ink" };
const BAR_SIZES = { xs: "h-1", sm: "h-1.5", md: "h-2.5" };

export function ProgressBar({ value, max = 100, label, caption, tone = "primary", size = "sm", className, showValue = false }: ProgressBarProps) {
  const pct = Math.max(0, Math.min(100, max ? (value / max) * 100 : 0));
  return (
    <div className={cn("w-full", className)}>
      {(caption || showValue) && (
        <div className="mb-1 flex items-center justify-between gap-2 text-xs text-ink-muted">
          {caption && <span>{caption}</span>}
          {showValue && <span className="tabular font-mono">{Math.round(pct)} %</span>}
        </div>
      )}
      <div
        role="progressbar"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={max}
        aria-valuenow={Math.round(value)}
        className={cn("w-full overflow-hidden rounded-full bg-surface", BAR_SIZES[size])}
      >
        <div className={cn("h-full origin-left animate-grow-x rounded-full transition-[width] duration-500", BAR_TONES[tone])} style={{ width: `${pct}%` }} />
      </div>
    </div>
  );
}

interface EmptyStateProps {
  icon?: ReactNode;
  title: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  className?: string;
  compact?: boolean;
}

export function EmptyState({ icon, title, description, action, className, compact = false }: EmptyStateProps) {
  return (
    <div className={cn("relative flex flex-col items-center overflow-hidden rounded-card border border-dashed border-line-strong bg-white text-center", compact ? "px-4 py-6" : "px-6 py-14", className)}>
      {icon && <div className="mb-4 flex size-12 items-center justify-center rounded-2xl bg-primary-50 text-primary ring-8 ring-primary-50/50">{icon}</div>}
      <p className="text-base font-semibold tracking-[-0.01em] text-ink">{title}</p>
      {description && <p className="mt-1.5 max-w-md text-sm text-ink-muted">{description}</p>}
      {action && <div className="mt-4 flex flex-wrap justify-center gap-2">{action}</div>}
    </div>
  );
}

export function Skeleton({ className }: { className?: string }) {
  return <div aria-hidden className={cn("skeleton rounded-md", className)} />;
}

export function SkeletonCard({ lines = 3, className }: { lines?: number; className?: string }) {
  return (
    <div className={cn("rounded-card border border-line bg-white p-5", className)} aria-hidden>
      <Skeleton className="h-5 w-2/5" />
      <div className="mt-4 space-y-2.5">
        {Array.from({ length: lines }, (_, i) => (
          <Skeleton key={i} className={cn("h-3.5", i === lines - 1 ? "w-3/5" : "w-full")} />
        ))}
      </div>
    </div>
  );
}

export function PageSkeleton({ cards = 3 }: { cards?: number }) {
  return (
    <div className="space-y-6" role="status" aria-label="Загрузка">
      <Skeleton className="h-8 w-64" />
      <div className="grid gap-4 md:grid-cols-2">
        {Array.from({ length: cards }, (_, i) => (
          <SkeletonCard key={i} />
        ))}
      </div>
    </div>
  );
}

type AlertTone = "info" | "success" | "warning" | "danger";

const ALERT_TONES: Record<AlertTone, { box: string; icon: ReactNode }> = {
  info: { box: "border-primary-200 bg-primary-50 text-ink", icon: <Info className="size-5 text-primary" aria-hidden /> },
  success: { box: "border-success-200 bg-success-50 text-ink", icon: <CheckCircle2 className="size-5 text-success" aria-hidden /> },
  warning: { box: "border-warning-200 bg-warning-50 text-ink", icon: <AlertTriangle className="size-5 text-warning" aria-hidden /> },
  danger: { box: "border-accent-100 bg-accent-50 text-ink", icon: <XCircle className="size-5 text-accent-strong" aria-hidden /> },
};

interface AlertProps {
  tone?: AlertTone;
  title?: ReactNode;
  children?: ReactNode;
  action?: ReactNode;
  className?: string;
  icon?: ReactNode;
}

export function Alert({ tone = "info", title, children, action, className, icon }: AlertProps) {
  const t = ALERT_TONES[tone];
  return (
    <div role={tone === "danger" ? "alert" : "status"} className={cn("flex gap-3 rounded-card border p-4", t.box, className)}>
      <span className="mt-0.5 shrink-0">{icon ?? t.icon}</span>
      <div className="min-w-0 flex-1 text-sm">
        {title && <p className="font-semibold">{title}</p>}
        {children && <div className={cn("text-ink-muted", title && "mt-1")}>{children}</div>}
        {action && <div className="mt-3 flex flex-wrap gap-2">{action}</div>}
      </div>
    </div>
  );
}

interface StatProps {
  label: ReactNode;
  value: ReactNode;
  hint?: ReactNode;
  className?: string;
  tone?: "default" | "primary";
}

export function Stat({ label, value, hint, className, tone = "default" }: StatProps) {
  return (
    <div className={cn("flex min-w-0 flex-col", className)}>
      <div className="text-xs text-ink-subtle">{label}</div>
      <div className={cn("tabular mt-auto pt-1 font-display text-[26px] leading-tight font-semibold tracking-[-0.03em]", tone === "primary" ? "text-primary" : "text-ink")}>
        {typeof value === "number" ? <AnimatedNumber value={value} /> : value}
      </div>
      {hint && <div className="mt-0.5 text-xs text-ink-muted">{hint}</div>}
    </div>
  );
}
