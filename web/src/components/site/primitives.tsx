import { ArrowRight, ArrowUpRight } from "lucide-react";
import Link from "next/link";
import type { ComponentProps, ReactNode } from "react";
import { cn } from "@/lib/cn";

type Tone = "light" | "canvas" | "night";

const TONE: Record<Tone, string> = {
  light: "bg-white text-ink",
  canvas: "bg-canvas text-ink",
  night: "bg-night text-white",
};

export function Container({ className, ...props }: ComponentProps<"div">) {
  return <div className={cn("mx-auto w-full max-w-[1240px] px-4 sm:px-6 lg:px-8", className)} {...props} />;
}

interface SectionProps extends Omit<ComponentProps<"section">, "title"> {
  tone?: Tone;
  bleed?: boolean;
}

export function Section({ tone = "light", bleed = false, className, children, ...props }: SectionProps) {
  return (
    <section className={cn("relative isolate", TONE[tone], !bleed && "py-20 sm:py-28 lg:py-36", className)} {...props}>
      {children}
    </section>
  );
}

export function Eyebrow({ children, tone = "light", className }: { children: ReactNode; tone?: "light" | "night"; className?: string }) {
  return (
    <p className={cn("eyebrow inline-flex items-center gap-2.5", tone === "night" ? "text-primary-300" : "text-primary", className)}>
      <span aria-hidden className="relative flex size-2">
        <span className={cn("absolute inset-0 rounded-full", tone === "night" ? "bg-primary-300" : "bg-primary")} />
        <span className={cn("absolute -inset-1 rounded-full border", tone === "night" ? "border-primary-300/40" : "border-primary/30")} />
      </span>
      {children}
    </p>
  );
}

interface SectionHeadProps {
  eyebrow?: ReactNode;
  title: ReactNode;
  lead?: ReactNode;
  tone?: "light" | "night";
  align?: "left" | "center";
  size?: "lg" | "md";
  aside?: ReactNode;
  className?: string;
  as?: "h1" | "h2";
}

export function SectionHead({ eyebrow, title, lead, tone = "light", align = "left", size = "lg", aside, className, as: Heading = "h2" }: SectionHeadProps) {
  const night = tone === "night";
  return (
    <div className={cn("flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between", align === "center" && "items-center text-center lg:flex-col lg:items-center", className)}>
      <div className={cn("min-w-0", align === "center" ? "max-w-4xl" : "max-w-3xl")}>
        {eyebrow && (
          <div data-reveal="fade" className="mb-5">
            <Eyebrow tone={tone}>{eyebrow}</Eyebrow>
          </div>
        )}
        <Heading data-split className={cn(size === "lg" ? "display-2" : "display-3", "text-balance", night ? "text-white" : "text-ink")}>
          {title}
        </Heading>
        {lead && (
          <p data-reveal className={cn("mt-6 max-w-2xl text-lg leading-relaxed text-pretty sm:text-xl", night ? "text-night-muted" : "text-ink-muted", align === "center" && "mx-auto")}>
            {lead}
          </p>
        )}
      </div>
      {aside && (
        <div data-reveal className="shrink-0">
          {aside}
        </div>
      )}
    </div>
  );
}

interface CtaProps {
  href: string;
  children: ReactNode;
  variant?: "primary" | "light" | "outline" | "outline-night" | "ink";
  size?: "md" | "lg";
  className?: string;
  external?: boolean;
  magnetic?: boolean;
}

const CTA_VARIANT = {
  primary: "bg-primary text-white shadow-[0_14px_34px_-14px_rgb(64_47_255/0.9)] hover:bg-primary-hover",
  light: "bg-white text-ink hover:bg-primary-50",
  ink: "bg-ink text-white hover:bg-primary",
  outline: "border border-line-strong bg-white/60 text-ink hover:border-ink",
  "outline-night": "border border-white/25 text-white hover:border-white hover:bg-white/5",
};

export function Cta({ href, children, variant = "primary", size = "lg", className, external = false, magnetic = true }: CtaProps) {
  const cls = cn(
    "group relative inline-flex items-center justify-center gap-2.5 overflow-hidden rounded-full font-medium whitespace-nowrap transition-colors",
    size === "lg" ? "h-14 px-7 text-base" : "h-11 px-5 text-sm",
    CTA_VARIANT[variant],
    className,
  );
  const inner = (
    <>
      <span>{children}</span>
      <span className="relative inline-flex size-5 items-center justify-center overflow-hidden" aria-hidden>
        <ArrowRight className="size-4.5 transition-transform duration-300 group-hover:translate-x-5" />
        <ArrowRight className="absolute size-4.5 -translate-x-5 transition-transform duration-300 group-hover:translate-x-0" />
      </span>
    </>
  );
  const mag = magnetic ? { "data-magnetic": "0.25" } : {};
  if (external) {
    return (
      <a href={href} className={cls} target="_blank" rel="noreferrer" {...mag}>
        {inner}
      </a>
    );
  }
  return (
    <Link href={href} className={cls} {...mag}>
      {inner}
    </Link>
  );
}

export function TextLink({ href, children, tone = "light", className }: { href: string; children: ReactNode; tone?: "light" | "night"; className?: string }) {
  return (
    <Link
      href={href}
      className={cn(
        "group inline-flex min-h-10 items-center gap-1.5 font-medium underline decoration-1 underline-offset-[6px] transition-colors",
        tone === "night" ? "text-white decoration-white/30 hover:decoration-white" : "text-ink decoration-ink/25 hover:text-primary hover:decoration-primary",
        className,
      )}
    >
      {children}
      <ArrowUpRight className="size-4 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5" aria-hidden />
    </Link>
  );
}

export function BrandLine({ className, variant = 1 }: { className?: string; variant?: 1 | 2 | 3 }) {
  const paths: Record<number, { d: string; nodes: [number, number][] }> = {
    1: { d: "M4 20 H520 a40 40 0 0 1 40 40 V180 a40 40 0 0 0 40 40 H1196", nodes: [[4, 20], [1196, 220]] },
    2: { d: "M4 220 H300 a40 40 0 0 0 40 -40 V60 a40 40 0 0 1 40 -40 H880 a40 40 0 0 1 40 40 V120 a40 40 0 0 0 40 40 H1196", nodes: [[4, 220], [880, 20], [1196, 160]] },
    3: { d: "M4 120 H420 a40 40 0 0 0 40 -40 V20 M460 120 V200 a40 40 0 0 0 40 40 H1196", nodes: [[4, 120], [460, 20], [1196, 240]] },
  };
  const p = paths[variant];
  return (
    <svg data-draw className={className} viewBox="0 0 1200 250" fill="none" aria-hidden>
      <path d={p.d} stroke="currentColor" strokeWidth="1.25" />
      {p.nodes.map(([x, y]) => (
        <circle key={`${x}-${y}`} cx={x} cy={y} r="4" fill="currentColor" />
      ))}
    </svg>
  );
}

export function Kicker({ index, children, tone = "light" }: { index: string; children: ReactNode; tone?: "light" | "night" }) {
  return (
    <span className={cn("inline-flex items-center gap-3 font-mono text-xs tracking-[0.12em] uppercase", tone === "night" ? "text-night-muted" : "text-ink-subtle")}>
      <span className={tone === "night" ? "text-primary-300" : "text-primary"}>{index}</span>
      <span aria-hidden className={cn("h-px w-8", tone === "night" ? "bg-night-line" : "bg-line-strong")} />
      {children}
    </span>
  );
}

export function Fact({ value, label, tone = "light", counter, decimals = 0, prefix, suffix }: { value: string; label: ReactNode; tone?: "light" | "night"; counter?: number; decimals?: number; prefix?: string; suffix?: string }) {
  return (
    <div className={cn("flex flex-col gap-3 border-t pt-6", tone === "night" ? "border-night-line" : "border-line")}>
      <p className={cn("tabular font-display text-5xl font-semibold tracking-[-0.04em] sm:text-6xl", tone === "night" ? "text-white" : "text-ink")}>
        {prefix && <span className={tone === "night" ? "text-primary-300" : "text-primary"}>{prefix}</span>}
        <span data-counter={counter} data-decimals={decimals}>
          {value}
        </span>
        {suffix && <span className={cn("ml-1 text-[0.55em]", tone === "night" ? "text-night-muted" : "text-ink-subtle")}>{suffix}</span>}
      </p>
      <p className={cn("max-w-xs text-sm leading-relaxed", tone === "night" ? "text-night-muted" : "text-ink-muted")}>{label}</p>
    </div>
  );
}
