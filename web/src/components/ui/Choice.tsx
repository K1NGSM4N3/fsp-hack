"use client";

import { useId, type ComponentProps, type ReactNode } from "react";
import { cn } from "@/lib/cn";
import { useFieldControl } from "./Field";

interface CheckboxProps extends Omit<ComponentProps<"input">, "type"> {
  label: ReactNode;
  description?: ReactNode;
}

export function Checkbox({ label, description, className, id, ...props }: CheckboxProps) {
  const autoId = useId();
  const inputId = id ?? autoId;
  return (
    <div className={cn("flex items-start gap-3", className)}>
      <label htmlFor={inputId} className="tap mt-0.5 flex size-4 shrink-0 cursor-pointer">
        <input
          id={inputId}
          type="checkbox"
          className="size-4 cursor-pointer rounded border-line-strong accent-primary"
          aria-describedby={description ? `${inputId}-desc` : undefined}
          {...props}
        />
      </label>
      <div className="min-w-0 text-sm">
        <label htmlFor={inputId} className="cursor-pointer text-ink">
          {label}
        </label>
        {description && (
          <p id={`${inputId}-desc`} className="mt-0.5 text-xs text-ink-subtle">
            {description}
          </p>
        )}
      </div>
    </div>
  );
}

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label: ReactNode;
  description?: ReactNode;
  disabled?: boolean;
  className?: string;
}

export function Switch({ checked, onChange, label, description, disabled, className }: SwitchProps) {
  const id = useId();
  return (
    <div className={cn("flex items-start justify-between gap-4", className)}>
      <div className="min-w-0">
        <label htmlFor={id} className="cursor-pointer text-sm font-medium text-ink">
          {label}
        </label>
        {description && (
          <p id={`${id}-desc`} className="mt-0.5 text-sm text-ink-subtle">
            {description}
          </p>
        )}
      </div>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        aria-describedby={description ? `${id}-desc` : undefined}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        className={cn(
          "tap mt-0.5 inline-flex h-6 w-11 shrink-0 items-center rounded-full border transition-colors disabled:opacity-50",
          checked ? "border-primary bg-primary" : "border-line-strong bg-surface",
        )}
      >
        <span className={cn("inline-block size-4.5 rounded-full bg-white shadow-sm transition-transform", checked ? "translate-x-5.5" : "translate-x-0.5")} />
      </button>
    </div>
  );
}

export interface RadioCardOption<V extends string> {
  value: V;
  title: ReactNode;
  description?: ReactNode;
  icon?: ReactNode;
  disabled?: boolean;
  aside?: ReactNode;
}

interface RadioCardsProps<V extends string> {
  name: string;
  value: V | null | undefined;
  onChange: (v: V) => void;
  options: RadioCardOption<V>[];
  columns?: 1 | 2 | 3;
  className?: string;
}

export function RadioCards<V extends string>({ name, value, onChange, options, columns = 2, className }: RadioCardsProps<V>) {
  const field = useFieldControl({});
  return (
    <div
      role="radiogroup"
      aria-describedby={field["aria-describedby"]}
      className={cn("grid gap-3", columns === 1 ? "grid-cols-1" : columns === 2 ? "grid-cols-1 sm:grid-cols-2" : "grid-cols-1 sm:grid-cols-3", className)}
    >
      {options.map((o) => {
        const checked = value === o.value;
        return (
          <label
            key={o.value}
            className={cn(
              "relative flex cursor-pointer gap-3 rounded-card border bg-white p-4 transition-colors",
              checked ? "border-primary bg-primary-50/60 ring-1 ring-primary" : "border-line hover:border-line-strong",
              o.disabled && "cursor-not-allowed opacity-60",
            )}
          >
            <input
              type="radio"
              name={name}
              value={o.value}
              checked={checked}
              disabled={o.disabled}
              onChange={() => onChange(o.value)}
              className="peer sr-only"
            />
            {o.icon && <span className={cn("mt-0.5 shrink-0", checked ? "text-primary" : "text-ink-subtle")}>{o.icon}</span>}
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-ink">{o.title}</span>
              {o.description && <span className="mt-1 block text-sm text-ink-muted">{o.description}</span>}
              {o.aside}
            </span>
            <span
              aria-hidden
              className={cn(
                "mt-0.5 flex size-4.5 shrink-0 items-center justify-center rounded-full border",
                checked ? "border-primary bg-primary" : "border-line-strong bg-white",
              )}
            >
              {checked && <span className="size-1.5 rounded-full bg-white" />}
            </span>
            <span className="pointer-events-none absolute inset-0 rounded-card peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-primary" />
          </label>
        );
      })}
    </div>
  );
}
