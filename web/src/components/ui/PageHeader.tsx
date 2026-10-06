import { ArrowLeft } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

interface PageHeaderProps {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  back?: { href: string; label: string };
  eyebrow?: ReactNode;
  className?: string;
}

export function PageHeader({ title, description, actions, back, eyebrow, className }: PageHeaderProps) {
  return (
    <header className={cn("mb-7 sm:mb-9", className)}>
      {back && (
        <Link href={back.href} className="tap mb-3 inline-flex items-center gap-1.5 rounded text-sm text-ink-muted hover:text-primary">
          <ArrowLeft className="size-4" aria-hidden />
          {back.label}
        </Link>
      )}
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="min-w-0">
          {eyebrow && <div className="mb-2">{typeof eyebrow === "string" ? <span className="eyebrow text-primary">{eyebrow}</span> : eyebrow}</div>}
          <h1 className="text-[26px] leading-tight font-semibold tracking-[-0.03em] text-ink sm:text-[32px]">{title}</h1>
          {description && <p className="mt-2 max-w-2xl text-sm leading-relaxed text-ink-muted sm:text-[15px]">{description}</p>}
        </div>
        {actions && <div className="flex flex-wrap items-center gap-2">{actions}</div>}
      </div>
    </header>
  );
}
