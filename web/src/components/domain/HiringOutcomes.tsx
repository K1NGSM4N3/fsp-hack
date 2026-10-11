import { Flag } from "lucide-react";
import type { HiringOutcome } from "@/api/types";
import { cn } from "@/lib/cn";
import { formatDateTime } from "@/lib/format";
import { HIRING_STAGE_LABEL } from "@/lib/labels";

/** Stages the company's hiring system reported after the contact: «Компания отметила: собеседование». */
export function HiringOutcomes({ outcomes, className }: { outcomes: HiringOutcome[]; className?: string }) {
  if (outcomes.length === 0) return null;
  return (
    <ol className={cn("space-y-2", className)} aria-label="Этапы найма от компании">
      {outcomes.map((o) => (
        <li key={`${o.stage}-${o.at}`} className="flex items-start gap-2 text-sm">
          <Flag className={cn("mt-0.5 size-4 shrink-0", o.stage === "rejected" ? "text-accent-strong" : "text-primary")} aria-hidden />
          <span className="min-w-0">
            <span className="text-ink">Компания отметила: {HIRING_STAGE_LABEL[o.stage]}</span>
            <span className="block text-xs text-ink-subtle">{formatDateTime(o.at)}</span>
            {o.reason && <span className="mt-0.5 block text-ink-muted">«{o.reason}»</span>}
          </span>
        </li>
      ))}
    </ol>
  );
}
