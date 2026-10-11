"use client";

import type { ComponentProps, ReactNode } from "react";
import { cn } from "@/lib/cn";
import { controlBase, useFieldControl } from "./Field";

interface InputProps extends ComponentProps<"input"> {
  leading?: ReactNode;
  trailing?: ReactNode;
}

export function Input({ className, leading, trailing, ...props }: InputProps) {
  const field = useFieldControl(props);
  const input = <input {...props} {...field} className={cn(controlBase, "h-10 px-3 text-base sm:text-sm", leading ? "pl-9" : "", trailing ? "pr-10" : "", className)} />;
  if (!leading && !trailing) return input;
  return (
    <div className="relative">
      {leading && <span className="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-ink-subtle">{leading}</span>}
      {input}
      {trailing && <span className="absolute top-1/2 right-3 -translate-y-1/2 text-sm text-ink-subtle">{trailing}</span>}
    </div>
  );
}

export function Textarea({ className, rows = 4, ...props }: ComponentProps<"textarea">) {
  const field = useFieldControl(props);
  return <textarea rows={rows} {...props} {...field} className={cn(controlBase, "min-h-20 px-3 py-2 text-base leading-relaxed sm:text-sm", className)} />;
}

export function Select({ className, children, ...props }: ComponentProps<"select">) {
  const field = useFieldControl(props);
  return (
    <div className="relative">
      <select {...props} {...field} className={cn(controlBase, "h-10 appearance-none pr-9 pl-3 text-base sm:text-sm", className)}>
        {children}
      </select>
      <svg className="pointer-events-none absolute top-1/2 right-3 size-4 -translate-y-1/2 text-ink-subtle" viewBox="0 0 16 16" fill="none" aria-hidden>
        <path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  );
}

/** Integer rubles; shows thin-space grouping while typing, emits a number or null. */
export function MoneyInput({
  value,
  onChange,
  className,
  ...props
}: Omit<ComponentProps<"input">, "value" | "onChange" | "type"> & { value: number | null | undefined; onChange: (v: number | null) => void }) {
  const display = value === null || value === undefined || Number.isNaN(value) ? "" : new Intl.NumberFormat("ru-RU").format(value);
  return (
    <Input
      {...props}
      inputMode="numeric"
      autoComplete="off"
      value={display}
      onChange={(e) => {
        const digits = e.target.value.replace(/\D/g, "").slice(0, 9);
        onChange(digits ? Number(digits) : null);
      }}
      trailing="₽"
      className={cn("tabular font-mono", className)}
    />
  );
}
