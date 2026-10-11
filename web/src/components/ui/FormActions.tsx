import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

/** Action bar of a long form: floats at the bottom of the screen while the form is in view, then settles under it. */
export function FormActions({ status, children, className }: { status?: ReactNode; children: ReactNode; className?: string }) {
  return (
    <div className={cn("sticky bottom-3 z-20 mt-2 sm:bottom-4 print:hidden", className)}>
      <div className="flex flex-wrap items-center justify-end gap-x-4 gap-y-2 rounded-2xl border border-line bg-white/90 px-4 py-3 shadow-pop backdrop-blur-xl sm:px-5">
        {status && <div className="mr-auto min-w-0 text-sm">{status}</div>}
        <div className="flex flex-wrap items-center gap-2">{children}</div>
      </div>
    </div>
  );
}

/** Status line for FormActions: the fields still to fix (each focuses on click), or whether there are unsaved edits. */
export function FormStatus({ missing, dirty, onFix }: { missing: string[]; dirty?: boolean; onFix: () => void }) {
  if (missing.length) {
    return (
      <button type="button" onClick={onFix} className="tap text-left text-accent-strong underline decoration-accent-strong/40 underline-offset-[3px] hover:decoration-accent-strong">
        Проверьте: {missing.join(", ")}
      </button>
    );
  }
  if (dirty) return <span className="text-ink-muted">Есть несохранённые изменения</span>;
  return null;
}
