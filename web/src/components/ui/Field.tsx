"use client";

import { createContext, useContext, useId, type AriaAttributes, type ReactNode } from "react";
import { cn } from "@/lib/cn";

interface FieldContextValue {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
  required: boolean;
}

const FieldContext = createContext<FieldContextValue | null>(null);

/** Inputs inside a Field pick up id, aria-describedby and aria-invalid automatically. */
export function useFieldControl(props: { id?: string; "aria-describedby"?: string; "aria-invalid"?: AriaAttributes["aria-invalid"]; required?: boolean }) {
  const ctx = useContext(FieldContext);
  return {
    id: props.id ?? ctx?.id,
    "aria-describedby": props["aria-describedby"] ?? ctx?.describedBy,
    "aria-invalid": props["aria-invalid"] ?? (ctx?.invalid ? true : undefined),
    "aria-required": props.required || ctx?.required || undefined,
  };
}

interface FieldProps {
  label: ReactNode;
  hint?: ReactNode;
  error?: string;
  required?: boolean;
  optional?: boolean;
  className?: string;
  labelAction?: ReactNode;
  children: ReactNode;
}

export function Field({ label, hint, error, required = false, optional = false, className, labelAction, children }: FieldProps) {
  const id = useId();
  const hintId = hint ? `${id}-hint` : undefined;
  const errorId = error ? `${id}-error` : undefined;
  const describedBy = errorId ?? hintId;
  return (
    <FieldContext.Provider value={{ id, describedBy, invalid: Boolean(error), required }}>
      <div data-field className={cn("flex min-w-0 flex-col gap-1.5", className)}>
        <div className="flex items-baseline justify-between gap-2">
          <label htmlFor={id} className="text-sm font-medium text-ink">
            {label}
            {required && (
              <span className="ml-0.5 text-accent-strong" aria-hidden>
                *
              </span>
            )}
            {optional && <span className="ml-1.5 text-xs font-normal text-ink-subtle">необязательно</span>}
          </label>
          {labelAction}
        </div>
        {children}
        {error ? (
          <p id={errorId} data-field-error className="text-sm text-accent-strong" role="alert">
            {error}
          </p>
        ) : hint ? (
          <p id={hintId} className="text-xs text-ink-subtle">
            {hint}
          </p>
        ) : null}
      </div>
    </FieldContext.Provider>
  );
}

/** Group label for checkbox/radio/chip sets (renders a fieldset legend). */
interface FieldSetProps {
  legend: ReactNode;
  hint?: ReactNode;
  error?: string;
  required?: boolean;
  className?: string;
  children: ReactNode;
}

export function FieldSet({ legend, hint, error, required, className, children }: FieldSetProps) {
  const id = useId();
  return (
    <fieldset data-field className={cn("flex min-w-0 flex-col gap-2", className)} aria-describedby={error ? `${id}-error` : hint ? `${id}-hint` : undefined}>
      <legend className="mb-1.5 text-sm font-medium text-ink">
        {legend}
        {required && (
          <span className="ml-0.5 text-accent-strong" aria-hidden>
            *
          </span>
        )}
      </legend>
      {children}
      {error ? (
        <p id={`${id}-error`} data-field-error className="text-sm text-accent-strong" role="alert">
          {error}
        </p>
      ) : hint ? (
        <p id={`${id}-hint`} className="text-xs text-ink-subtle">
          {hint}
        </p>
      ) : null}
    </fieldset>
  );
}

/** After a failed submit: bring the first field with an error into view and put the caret in it. */
export function focusFirstError(root: HTMLElement | null) {
  requestAnimationFrame(() => {
    const field = root?.querySelector("[data-field-error]")?.closest("[data-field]");
    if (!field) return;
    const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    field.scrollIntoView({ block: "center", behavior: still ? "auto" : "smooth" });
    field.querySelector<HTMLElement>("input:not([type=hidden]), select, textarea, button")?.focus({ preventScroll: true });
  });
}

export const controlBase =
  "w-full rounded-control border border-line-strong bg-white text-ink placeholder:text-ink-faint transition-colors hover:border-ink-faint/60 focus:border-primary focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-0 focus-visible:outline-primary disabled:cursor-not-allowed disabled:bg-canvas disabled:text-ink-subtle aria-[invalid=true]:border-accent-strong";
