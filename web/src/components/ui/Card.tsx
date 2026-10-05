import type { ComponentProps, ReactNode } from "react";
import { cn } from "@/lib/cn";

interface CardProps extends ComponentProps<"section"> {
  padding?: "none" | "sm" | "md" | "lg";
  interactive?: boolean;
}

const PADDING = { none: "", sm: "p-4", md: "p-5 sm:p-6", lg: "p-6 sm:p-8" };

export function Card({ padding = "md", interactive = false, className, ...props }: CardProps) {
  return (
    <section
      className={cn(
        "min-w-0 rounded-card border border-line bg-white shadow-card",
        PADDING[padding],
        interactive && "transition-colors hover:border-primary-300",
        className,
      )}
      {...props}
    />
  );
}

interface CardHeaderProps {
  title: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  eyebrow?: ReactNode;
  className?: string;
  as?: "h2" | "h3";
}

export function CardHeader({ title, description, action, eyebrow, className, as: Heading = "h2" }: CardHeaderProps) {
  return (
    <div className={cn("mb-4 flex flex-wrap items-start justify-between gap-3", className)}>
      <div className="min-w-0">
        {eyebrow && <div className="mb-1">{eyebrow}</div>}
        <Heading className="font-mono text-base font-semibold tracking-tight text-ink">{title}</Heading>
        {description && <p className="mt-1 text-sm text-ink-muted">{description}</p>}
      </div>
      {action && <div className="flex shrink-0 items-center gap-2">{action}</div>}
    </div>
  );
}

export function Eyebrow({ children, className }: { children: ReactNode; className?: string }) {
  return <span className={cn("font-mono text-[11px] font-medium tracking-[0.08em] text-ink-subtle uppercase", className)}>{children}</span>;
}

export function Divider({ className }: { className?: string }) {
  return <hr className={cn("border-line", className)} />;
}
