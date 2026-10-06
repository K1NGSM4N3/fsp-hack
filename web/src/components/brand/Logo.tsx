import Image from "next/image";
import Link from "next/link";
import { cn } from "@/lib/cn";

interface LogoProps {
  href?: string;
  className?: string;
  compact?: boolean;
  tone?: "light" | "dark";
}

export function Logo({ href = "/", className, compact = false, tone = "light" }: LogoProps) {
  const dark = tone === "dark";
  return (
    <Link href={href} className={cn("inline-flex min-h-10 items-center gap-2.5 rounded-control", className)} aria-label="ФСП Таланты — на главную">
      <Image src={dark ? "/brand/fsp-logo-dark.svg" : "/brand/fsp-logo.svg"} alt="" width={98} height={33} priority unoptimized className="h-7 w-auto sm:h-8" />
      {!compact && (
        <span className="flex items-center gap-2.5">
          <span aria-hidden className={cn("h-5 w-px", dark ? "bg-night-line" : "bg-line-strong")} />
          <span className={cn("font-mono text-[15px] font-semibold tracking-tight", dark ? "text-white" : "text-primary")}>Таланты</span>
        </span>
      )}
    </Link>
  );
}

/** Brand motif: thin 1px orthogonal lines with rounded corners and node endpoints. */
export function NodeLines({ className, draw = false }: { className?: string; draw?: boolean }) {
  return (
    <svg className={className} viewBox="0 0 400 300" fill="none" aria-hidden data-draw={draw ? "" : undefined}>
      <path d="M20 40 H180 a24 24 0 0 1 24 24 V150 a24 24 0 0 0 24 24 H380" stroke="currentColor" strokeWidth="1" />
      <path d="M60 280 V230 a24 24 0 0 1 24 -24 H250 a24 24 0 0 0 24 -24 V100" stroke="currentColor" strokeWidth="1" />
      <circle cx="20" cy="40" r="4" fill="currentColor" />
      <circle cx="380" cy="174" r="4" fill="currentColor" />
      <circle cx="60" cy="280" r="4" fill="currentColor" />
      <circle cx="274" cy="100" r="4" fill="currentColor" />
    </svg>
  );
}
