"use client";

import { useRef, useState } from "react";
import { ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

export interface PageTab {
  id: string;
  label: string;
}

export function PageTabs({ items }: { items: PageTab[] }) {
  const [active, setActive] = useState<string | null>(null);
  const bar = useRef<HTMLDivElement>(null);
  const scroller = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      for (const item of items) {
        const el = document.getElementById(item.id);
        if (!el) continue;
        ScrollTrigger.create({
          trigger: el,
          start: "top 45%",
          end: "bottom 45%",
          onToggle: (self) => {
            if (!self.isActive) return;
            setActive(item.id);
            const link = scroller.current?.querySelector<HTMLElement>(`[data-tab="${item.id}"]`);
            const box = scroller.current;
            if (link && box) gsap.to(box, { scrollLeft: link.offsetLeft - 16, duration: 0.4, ease: "power2.out" });
          },
        });
      }
    },
    { scope: bar, dependencies: [items] },
  );

  return (
    <div ref={bar} className="sticky top-0 z-40 border-y border-line bg-white/85 backdrop-blur-xl backdrop-saturate-150 print:hidden">
      <div ref={scroller} className="scrollbar-none mx-auto flex max-w-[1240px] gap-1 overflow-x-auto px-4 sm:px-6 lg:px-8">
        <span className="hidden shrink-0 items-center pr-4 font-mono text-[11px] tracking-[0.12em] text-ink-faint uppercase md:flex">На странице</span>
        {items.map((item) => (
          <a
            key={item.id}
            href={`#${item.id}`}
            data-tab={item.id}
            aria-current={active === item.id ? "true" : undefined}
            className={cn(
              "relative inline-flex h-14 shrink-0 items-center px-3 text-sm font-medium transition-colors",
              active === item.id ? "text-ink" : "text-ink-subtle hover:text-ink",
            )}
          >
            {item.label}
            <span
              aria-hidden
              className={cn("absolute inset-x-3 bottom-0 h-0.5 origin-left rounded-full bg-primary transition-transform duration-500 ease-[var(--ease-out-expo)]", active === item.id ? "scale-x-100" : "scale-x-0")}
            />
          </a>
        ))}
      </div>
    </div>
  );
}
