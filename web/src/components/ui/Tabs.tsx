"use client";

import { useId, useRef, type KeyboardEvent, type ReactNode } from "react";
import { cn } from "@/lib/cn";

export interface TabItem<V extends string> {
  value: V;
  label: ReactNode;
  count?: number;
}

interface TabsProps<V extends string> {
  items: TabItem<V>[];
  value: V;
  onChange: (v: V) => void;
  ariaLabel: string;
  className?: string;
  idPrefix?: string;
}

/** WAI-ARIA tabs with roving tabindex; pair each panel with `tabPanelProps`. */
export function Tabs<V extends string>({ items, value, onChange, ariaLabel, className, idPrefix }: TabsProps<V>) {
  const autoId = useId();
  const prefix = idPrefix ?? autoId;
  const refs = useRef<Array<HTMLButtonElement | null>>([]);

  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const idx = items.findIndex((i) => i.value === value);
    let next = idx;
    if (e.key === "ArrowRight") next = (idx + 1) % items.length;
    else if (e.key === "ArrowLeft") next = (idx - 1 + items.length) % items.length;
    else if (e.key === "Home") next = 0;
    else if (e.key === "End") next = items.length - 1;
    else return;
    e.preventDefault();
    const item = items[next];
    if (item) {
      onChange(item.value);
      refs.current[next]?.focus();
    }
  };

  return (
    <div role="tablist" aria-label={ariaLabel} onKeyDown={onKeyDown} className={cn("scrollbar-thin -mx-1 flex gap-1 overflow-x-auto border-b border-line px-1", className)}>
      {items.map((item, i) => {
        const selected = item.value === value;
        return (
          <button
            key={item.value}
            ref={(el) => {
              refs.current[i] = el;
            }}
            id={`${prefix}-tab-${item.value}`}
            role="tab"
            type="button"
            aria-selected={selected}
            aria-controls={`${prefix}-panel-${item.value}`}
            tabIndex={selected ? 0 : -1}
            onClick={() => onChange(item.value)}
            className={cn(
              "relative -mb-px inline-flex h-11 shrink-0 items-center gap-2 border-b-2 px-3 text-sm font-medium transition-colors",
              selected ? "border-primary text-primary" : "border-transparent text-ink-muted hover:text-ink",
            )}
          >
            {item.label}
            {item.count !== undefined && (
              <span className={cn("tabular rounded-full px-1.5 py-0.5 font-mono text-[11px]", selected ? "bg-primary-50 text-primary-strong" : "bg-canvas text-ink-subtle")}>
                {item.count}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export function tabPanelProps(prefix: string, value: string) {
  return { id: `${prefix}-panel-${value}`, role: "tabpanel" as const, "aria-labelledby": `${prefix}-tab-${value}`, tabIndex: 0 };
}
