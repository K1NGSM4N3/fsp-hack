"use client";

import { X } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

type ChipTone = "neutral" | "primary" | "success" | "warning" | "danger";

const STATIC_TONES: Record<ChipTone, string> = {
  neutral: "border-line bg-white text-ink",
  primary: "border-primary-200 bg-primary-50 text-primary-strong",
  success: "border-success-200 bg-success-50 text-success",
  warning: "border-warning-200 bg-warning-50 text-warning",
  danger: "border-accent-100 bg-accent-50 text-accent-strong",
};

interface ChipProps {
  children: ReactNode;
  tone?: ChipTone;
  icon?: ReactNode;
  onRemove?: () => void;
  removeLabel?: string;
  className?: string;
  title?: string;
}

export function Chip({ children, tone = "neutral", icon, onRemove, removeLabel, className, title }: ChipProps) {
  return (
    <span title={title} className={cn("inline-flex h-7 max-w-full items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium", STATIC_TONES[tone], onRemove && "pr-1", className)}>
      {icon}
      <span className="truncate">{children}</span>
      {onRemove && (
        <button
          type="button"
          onClick={onRemove}
          aria-label={removeLabel ?? `Убрать «${typeof children === "string" ? children : ""}»`}
          className="inline-flex size-5 items-center justify-center rounded-full text-current opacity-70 hover:bg-black/5 hover:opacity-100"
        >
          <X className="size-3.5" aria-hidden />
        </button>
      )}
    </span>
  );
}

interface ToggleChipProps {
  selected: boolean;
  onToggle: () => void;
  children: ReactNode;
  icon?: ReactNode;
  disabled?: boolean;
  className?: string;
}

export function ToggleChip({ selected, onToggle, children, icon, disabled, className }: ToggleChipProps) {
  return (
    <button
      type="button"
      aria-pressed={selected}
      disabled={disabled}
      onClick={onToggle}
      className={cn(
        "inline-flex h-9 items-center gap-1.5 rounded-full border px-3.5 text-sm font-medium transition-colors disabled:opacity-50",
        selected ? "border-primary bg-primary text-white" : "border-line-strong bg-white text-ink hover:border-primary hover:text-primary",
        className,
      )}
    >
      {icon}
      {children}
    </button>
  );
}

interface ChipSelectProps<V extends string> {
  options: { value: V; label: ReactNode }[];
  value: V[];
  onChange: (next: V[]) => void;
  single?: boolean;
  ariaLabel?: string;
  className?: string;
}

export function ChipSelect<V extends string>({ options, value, onChange, single = false, ariaLabel, className }: ChipSelectProps<V>) {
  return (
    <div role="group" aria-label={ariaLabel} className={cn("flex flex-wrap gap-2", className)}>
      {options.map((o) => {
        const selected = value.includes(o.value);
        return (
          <ToggleChip
            key={o.value}
            selected={selected}
            onToggle={() => {
              if (single) onChange([o.value]);
              else onChange(selected ? value.filter((v) => v !== o.value) : [...value, o.value]);
            }}
          >
            {o.label}
          </ToggleChip>
        );
      })}
    </div>
  );
}
