import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

export type BadgeTone = "neutral" | "primary" | "success" | "warning" | "danger" | "muted" | "solid";

const TONES: Record<BadgeTone, string> = {
  neutral: "bg-surface text-ink",
  muted: "bg-canvas text-ink-subtle border border-line",
  primary: "bg-primary-50 text-primary-strong",
  success: "bg-success-50 text-success",
  warning: "bg-warning-50 text-warning",
  danger: "bg-accent-50 text-accent-strong",
  solid: "bg-primary text-white",
};

const DOTS: Record<BadgeTone, string> = {
  neutral: "bg-ink-subtle",
  muted: "bg-ink-faint",
  primary: "bg-primary",
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-accent",
  solid: "bg-white",
};

interface BadgeProps {
  tone?: BadgeTone;
  dot?: boolean;
  icon?: ReactNode;
  children: ReactNode;
  className?: string;
  title?: string;
}

export function Badge({ tone = "neutral", dot = false, icon, children, className, title }: BadgeProps) {
  return (
    <span title={title} className={cn("inline-flex h-6 shrink-0 items-center gap-1.5 rounded-md px-2 text-xs font-medium whitespace-nowrap", TONES[tone], className)}>
      {dot && <span aria-hidden className={cn("size-1.5 rounded-full", DOTS[tone])} />}
      {icon}
      {children}
    </span>
  );
}
