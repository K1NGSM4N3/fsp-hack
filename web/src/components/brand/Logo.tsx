import Image from "next/image";
import Link from "next/link";
import { cn } from "@/lib/cn";

export function Logo({ href = "/", className, compact = false }: { href?: string; className?: string; compact?: boolean }) {
  return (
    <Link href={href} className={cn("inline-flex items-center gap-2.5 rounded-control", className)} aria-label="ФСП Карьера — на главную">
      <Image src="/brand/fsp-logo.svg" alt="" width={98} height={33} priority unoptimized className="h-8 w-auto" />
      {!compact && (
        <span className="flex items-center gap-2.5">
          <span aria-hidden className="h-5 w-px bg-line-strong" />
          <span className="font-mono text-[15px] font-semibold tracking-tight text-primary">Карьера</span>
        </span>
      )}
    </Link>
  );
}

/** Decorative node graph in the brand style: thin 1px lines with node endpoints. */
export function NodeLines({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 400 300" fill="none" aria-hidden>
      <path d="M20 40 H180 a24 24 0 0 1 24 24 V150 a24 24 0 0 0 24 24 H380" stroke="currentColor" strokeWidth="1" />
      <path d="M60 280 V230 a24 24 0 0 1 24 -24 H250 a24 24 0 0 0 24 -24 V100" stroke="currentColor" strokeWidth="1" />
      <circle cx="20" cy="40" r="4" fill="currentColor" />
      <circle cx="380" cy="174" r="4" fill="currentColor" />
      <circle cx="60" cy="280" r="4" fill="currentColor" />
      <circle cx="274" cy="100" r="4" fill="currentColor" />
    </svg>
  );
}
