"use client";

import { X } from "lucide-react";
import { useEffect, useId, useRef, type ReactNode } from "react";
import { cn } from "@/lib/cn";
import { IconButton } from "./Button";

interface DialogProps {
  open: boolean;
  onClose: () => void;
  title: ReactNode;
  description?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  size?: "sm" | "md" | "lg";
}

/** Native <dialog>: focus trap, Esc and inert background come from the browser. */
export function Dialog({ open, onClose, title, description, children, footer, size = "md" }: DialogProps) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const descId = useId();

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (open && !el.open) el.showModal();
    if (!open && el.open) el.close();
  }, [open]);

  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      aria-describedby={description ? descId : undefined}
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      onClick={(e) => {
        if (e.target === ref.current) onClose();
      }}
      className={cn(
        "m-auto max-h-[calc(100dvh-2rem)] w-[calc(100vw-2rem)] overflow-hidden rounded-card border border-line bg-white p-0 text-ink shadow-pop open:animate-slide-up",
        size === "sm" ? "max-w-md" : size === "md" ? "max-w-xl" : "max-w-3xl",
      )}
    >
      {open && (
        <div className="flex max-h-[calc(100dvh-2rem)] flex-col">
          <header className="flex items-start justify-between gap-4 border-b border-line px-5 py-4 sm:px-6">
            <div className="min-w-0">
              <h2 id={titleId} className="font-mono text-lg font-semibold tracking-tight">
                {title}
              </h2>
              {description && (
                <p id={descId} className="mt-1 text-sm text-ink-muted">
                  {description}
                </p>
              )}
            </div>
            <IconButton label="Закрыть" size="sm" onClick={onClose} className="-mr-2">
              <X className="size-5" aria-hidden />
            </IconButton>
          </header>
          <div className="min-h-0 flex-1 overflow-y-auto px-5 py-5 sm:px-6">{children}</div>
          {footer && <footer className="flex flex-wrap justify-end gap-2 border-t border-line bg-canvas/60 px-5 py-3 sm:px-6">{footer}</footer>}
        </div>
      )}
    </dialog>
  );
}
