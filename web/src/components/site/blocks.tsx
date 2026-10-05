import type { LucideIcon } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";
import { BrandLine, Container, Cta, Eyebrow, Section } from "./primitives";

interface PageHeroProps {
  crumb: string;
  title: ReactNode;
  lead: ReactNode;
  actions?: ReactNode;
  visual?: ReactNode;
  facts?: { value: string; label: string }[];
}

export function PageHero({ crumb, title, lead, actions, visual, facts }: PageHeroProps) {
  return (
    <section className="relative isolate overflow-hidden bg-white pt-32 pb-16 sm:pt-40 lg:pt-44 lg:pb-24">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="bg-grid mask-radial absolute inset-0 opacity-80" />
        <div className="absolute -top-56 right-[-12rem] size-[46rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.22),transparent_64%)]" data-parallax="0.2" />
        <div className="absolute bottom-[-18rem] left-[-14rem] size-[36rem] rounded-full bg-[radial-gradient(circle,rgb(123_111_255/0.14),transparent_66%)]" />
      </div>
      <BrandLine className="pointer-events-none absolute top-28 left-0 -z-10 hidden w-full text-primary-200 lg:block" variant={1} />
      <Container>
        <nav data-hero aria-label="Хлебные крошки" className="mb-8 flex items-center gap-2 font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">
          <Link href="/" className="transition-colors hover:text-primary">
            ФСП Таланты
          </Link>
          <span aria-hidden>/</span>
          <span className="text-primary">{crumb}</span>
        </nav>
        <div className={cn("grid grid-cols-1 items-center gap-14", visual && "lg:grid-cols-[1.1fr_0.9fr] lg:gap-16")}>
          <div>
            <h1 data-hero data-split="hero" className="display-hero text-balance text-ink">
              {title}
            </h1>
            <p data-hero className="mt-8 max-w-2xl text-lg leading-relaxed text-ink-muted sm:text-xl">
              {lead}
            </p>
            {actions && (
              <div data-hero className="mt-10 flex flex-wrap items-center gap-3">
                {actions}
              </div>
            )}
          </div>
          {visual && (
            <div data-hero className="relative">
              {visual}
            </div>
          )}
        </div>
        {facts && (
          <dl data-hero className="mt-16 grid grid-cols-2 gap-x-6 gap-y-8 border-t border-line pt-8 lg:mt-20 lg:grid-cols-4">
            {facts.map((f) => (
              <div key={f.value + f.label}>
                <dt className="sr-only">{f.label}</dt>
                <dd>
                  <span className="tabular block font-display text-3xl font-semibold tracking-[-0.03em] text-ink sm:text-4xl">{f.value}</span>
                  <span className="mt-2 block max-w-[16rem] text-sm leading-relaxed text-ink-muted">{f.label}</span>
                </dd>
              </div>
            ))}
          </dl>
        )}
      </Container>
    </section>
  );
}

export function SplitHeading({ eyebrow, title, children, tone = "light", id }: { eyebrow: string; title: ReactNode; children?: ReactNode; tone?: "light" | "night"; id?: string }) {
  const night = tone === "night";
  return (
    <div id={id} className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_1fr] lg:items-end lg:gap-16">
      <div>
        <div data-reveal="fade">
          <Eyebrow tone={tone}>{eyebrow}</Eyebrow>
        </div>
        <h2 data-split className={cn("display-2c mt-5 text-balance", night ? "text-white" : "text-ink")}>
          {title}
        </h2>
      </div>
      {children && (
        <div data-reveal className={cn("text-lg leading-relaxed", night ? "text-night-muted" : "text-ink-muted")}>
          {children}
        </div>
      )}
    </div>
  );
}

export interface FeatureItem {
  icon?: LucideIcon;
  title: string;
  text: ReactNode;
  badge?: string;
}

export function FeatureGrid({ items, columns = 3, tone = "light", className }: { items: FeatureItem[]; columns?: 2 | 3 | 4; tone?: "light" | "night"; className?: string }) {
  const night = tone === "night";
  return (
    <div
      data-stagger="0.07"
      className={cn(
        "grid grid-cols-1 gap-px overflow-hidden rounded-[24px] border",
        night ? "border-night-line bg-night-line" : "border-line bg-line",
        columns === 2 ? "sm:grid-cols-2" : columns === 3 ? "sm:grid-cols-2 lg:grid-cols-3" : "sm:grid-cols-2 lg:grid-cols-4",
        className,
      )}
    >
      {items.map(({ icon: Icon, title, text, badge }) => (
        <div key={title} className={cn("flex flex-col p-7 sm:p-8", night ? "bg-night-raised" : "bg-white")}>
          <div className="flex items-center justify-between gap-3">
            {Icon ? <Icon className={cn("size-6", night ? "text-primary-300" : "text-primary")} aria-hidden /> : <span />}
            {badge && (
              <span className={cn("rounded-full px-2.5 py-1 font-mono text-[10px] tracking-[0.1em] uppercase", night ? "bg-white/10 text-night-muted" : "bg-canvas text-ink-subtle")}>{badge}</span>
            )}
          </div>
          <h3 className={cn("mt-7 text-xl font-semibold tracking-tight", night ? "text-white" : "text-ink")}>{title}</h3>
          <div className={cn("mt-2.5 text-[15px] leading-relaxed", night ? "text-night-muted" : "text-ink-muted")}>{text}</div>
        </div>
      ))}
    </div>
  );
}

export function SpecList({ rows, tone = "light", className }: { rows: [string, ReactNode][]; tone?: "light" | "night"; className?: string }) {
  const night = tone === "night";
  return (
    <dl data-stagger="0.05" className={cn("border-t", night ? "border-night-line" : "border-line", className)}>
      {rows.map(([term, value]) => (
        <div key={term} className={cn("grid gap-2 border-b py-5 sm:grid-cols-[15rem_1fr] sm:gap-8", night ? "border-night-line" : "border-line")}>
          <dt className={cn("font-mono text-xs tracking-[0.1em] uppercase sm:pt-1", night ? "text-night-faint" : "text-ink-subtle")}>{term}</dt>
          <dd className={cn("text-base leading-relaxed sm:text-lg", night ? "text-white" : "text-ink")}>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

export function NumberedSteps({ steps, tone = "light" }: { steps: { title: string; text: ReactNode; meta?: string }[]; tone?: "light" | "night" }) {
  const night = tone === "night";
  return (
    <ol data-stagger="0.08" className="relative">
      {steps.map((s, i) => (
        <li key={s.title} className={cn("grid gap-4 border-t py-8 sm:grid-cols-[6rem_1fr] sm:gap-8", night ? "border-night-line" : "border-line")}>
          <span className={cn("tabular font-display text-5xl leading-none font-semibold tracking-[-0.04em]", night ? "text-primary-300" : "text-primary")}>{String(i + 1).padStart(2, "0")}</span>
          <div>
            <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
              <h3 className={cn("text-2xl font-semibold tracking-tight", night ? "text-white" : "text-ink")}>{s.title}</h3>
              {s.meta && <span className={cn("font-mono text-xs tracking-[0.08em] uppercase", night ? "text-night-faint" : "text-ink-subtle")}>{s.meta}</span>}
            </div>
            <div className={cn("mt-3 max-w-2xl text-base leading-relaxed sm:text-[17px]", night ? "text-night-muted" : "text-ink-muted")}>{s.text}</div>
          </div>
        </li>
      ))}
    </ol>
  );
}

export function WeightBars({ rows, unit = "", max, tone = "light" }: { rows: { label: string; value: number; display?: string; note?: string; muted?: boolean }[]; unit?: string; max?: number; tone?: "light" | "night" }) {
  const night = tone === "night";
  const top = max ?? Math.max(...rows.map((r) => r.value));
  return (
    <ul className="space-y-4">
      {rows.map((r) => (
        <li key={r.label}>
          <div className="mb-1.5 flex items-baseline justify-between gap-4 text-sm">
            <span className={night ? "text-white" : "text-ink"}>{r.label}</span>
            <span className={cn("tabular font-mono", night ? "text-night-muted" : "text-ink-muted")}>
              {r.display ?? r.value}
              {unit}
            </span>
          </div>
          <div className={cn("h-2.5 overflow-hidden rounded-full", night ? "bg-night-line" : "bg-primary-50")}>
            <div data-grow className={cn("h-full rounded-full", r.muted ? (night ? "bg-night-faint" : "bg-line-strong") : "bg-primary")} style={{ width: `${(r.value / top) * 100}%` }} />
          </div>
          {r.note && <p className={cn("mt-1.5 text-xs", night ? "text-night-faint" : "text-ink-subtle")}>{r.note}</p>}
        </li>
      ))}
    </ul>
  );
}

interface CtaBandProps {
  title: ReactNode;
  lead?: ReactNode;
  primary: { href: string; label: string };
  secondary?: { href: string; label: string };
}

export function CtaBand({ title, lead, primary, secondary }: CtaBandProps) {
  return (
    <Section tone="night" className="overflow-hidden">
      <div aria-hidden className="pointer-events-none absolute -top-48 left-1/2 size-[48rem] -translate-x-1/2 rounded-full bg-primary/35 blur-[160px]" />
      <BrandLine className="pointer-events-none absolute bottom-6 left-0 w-full text-primary-400/40" variant={2} />
      <Container className="relative text-center">
        <h2 data-split className="display-2 mx-auto max-w-[18ch] text-balance">
          {title}
        </h2>
        {lead && (
          <p data-reveal className="mx-auto mt-6 max-w-2xl text-lg leading-relaxed text-night-muted">
            {lead}
          </p>
        )}
        <div data-reveal className="mt-12 flex flex-wrap items-center justify-center gap-3">
          <Cta href={primary.href}>{primary.label}</Cta>
          {secondary && (
            <Cta href={secondary.href} variant="outline-night">
              {secondary.label}
            </Cta>
          )}
        </div>
      </Container>
    </Section>
  );
}

export function Note({ children, tone = "light" }: { children: ReactNode; tone?: "light" | "night" }) {
  return (
    <p data-reveal className={cn("max-w-3xl border-l-2 pl-5 text-sm leading-relaxed", tone === "night" ? "border-primary-400 text-night-muted" : "border-primary text-ink-subtle")}>
      {children}
    </p>
  );
}
