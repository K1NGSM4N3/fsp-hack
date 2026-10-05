import { Check } from "lucide-react";
import Link from "next/link";
import { cn } from "@/lib/cn";

export type StepStatus = "done" | "current" | "upcoming";

export interface Step {
  key: string;
  label: string;
  hint?: string;
  status: StepStatus;
  href?: string;
}

/** Brand cue: nodes joined by thin 1px lines, like the FSP node-graph mark. */
export function Stepper({ steps, ariaLabel = "Шаги", className }: { steps: Step[]; ariaLabel?: string; className?: string }) {
  return (
    <nav aria-label={ariaLabel} className={className}>
      <ol className="flex items-start">
        {steps.map((step, i) => {
          const last = i === steps.length - 1;
          const node = (
            <span
              className={cn(
                "relative z-10 flex size-8 shrink-0 items-center justify-center rounded-full border font-mono text-sm font-semibold transition-colors",
                step.status === "done" && "border-primary bg-primary text-white",
                step.status === "current" && "border-primary bg-white text-primary ring-4 ring-primary-100",
                step.status === "upcoming" && "border-line-strong bg-white text-ink-subtle",
              )}
            >
              {step.status === "done" ? <Check className="size-4" aria-hidden /> : i + 1}
            </span>
          );
          const label = (
            <span className="mt-2 block text-center">
              <span className={cn("block text-xs font-medium sm:text-sm", step.status === "upcoming" ? "text-ink-subtle" : "text-ink")}>{step.label}</span>
              {step.hint && <span className="mt-0.5 hidden text-xs text-ink-subtle sm:block">{step.hint}</span>}
            </span>
          );
          const content = (
            <>
              <span className="flex w-full items-center">
                <span aria-hidden className={cn("h-px flex-1", i === 0 ? "bg-transparent" : steps[i - 1]?.status === "done" ? "bg-primary" : "bg-line-strong")} />
                {node}
                <span aria-hidden className={cn("h-px flex-1", last ? "bg-transparent" : step.status === "done" ? "bg-primary" : "bg-line-strong")} />
              </span>
              {label}
            </>
          );
          const sr = step.status === "done" ? " (выполнено)" : step.status === "current" ? " (текущий шаг)" : "";
          return (
            <li key={step.key} className="flex min-w-0 flex-1 flex-col items-center" aria-current={step.status === "current" ? "step" : undefined}>
              {step.href && step.status !== "upcoming" ? (
                <Link href={step.href} className="group flex w-full flex-col items-center rounded-control focus-visible:outline-offset-4">
                  {content}
                  <span className="sr-only">{sr}</span>
                </Link>
              ) : (
                <span className="flex w-full flex-col items-center">
                  {content}
                  <span className="sr-only">{sr}</span>
                </span>
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}
