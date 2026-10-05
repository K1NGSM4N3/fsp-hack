"use client";

import { Plus } from "lucide-react";
import type { MouseEvent, ReactNode } from "react";
import { MOTION_OK, ScrollTrigger, gsap } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

export interface FaqItem {
  q: string;
  a: ReactNode;
}

/** Native <details>, so answers open without JS; GSAP only smooths the height change. */
export function Faq({ items, tone = "light", className }: { items: FaqItem[]; tone?: "light" | "night"; className?: string }) {
  const night = tone === "night";

  const onToggle = (e: MouseEvent<HTMLElement>) => {
    if (!window.matchMedia(MOTION_OK).matches) return;
    const details = e.currentTarget.parentElement as HTMLDetailsElement | null;
    const body = details?.querySelector<HTMLElement>("[data-faq-body]");
    if (!details || !body) return;
    e.preventDefault();
    if (details.open) {
      gsap.to(body, {
        height: 0,
        autoAlpha: 0,
        duration: 0.45,
        ease: "power3.inOut",
        onComplete: () => {
          details.open = false;
          gsap.set(body, { clearProps: "height,opacity,visibility" });
          ScrollTrigger.refresh();
        },
      });
    } else {
      details.open = true;
      gsap.fromTo(
        body,
        { height: 0, autoAlpha: 0 },
        { height: "auto", autoAlpha: 1, duration: 0.55, ease: "expo.out", onComplete: () => void ScrollTrigger.refresh() },
      );
    }
  };

  return (
    <div className={cn("border-t", night ? "border-night-line" : "border-line", className)}>
      {items.map((item) => (
        <details key={item.q} className={cn("group border-b", night ? "border-night-line" : "border-line")}>
          <summary
            onClick={onToggle}
            className={cn(
              "flex cursor-pointer list-none items-center justify-between gap-6 py-6 text-left text-lg font-medium tracking-tight transition-colors sm:text-xl [&::-webkit-details-marker]:hidden",
              night ? "text-white hover:text-primary-300" : "text-ink hover:text-primary",
            )}
          >
            {item.q}
            <span
              className={cn(
                "flex size-10 shrink-0 items-center justify-center rounded-full border transition-colors",
                night ? "border-night-line group-open:border-primary-400 group-open:bg-primary" : "border-line-strong group-open:border-primary group-open:bg-primary group-open:text-white",
              )}
            >
              <Plus className="size-4.5 transition-[rotate] duration-300 group-open:rotate-45" aria-hidden />
            </span>
          </summary>
          <div data-faq-body className="overflow-hidden">
            <div className={cn("max-w-3xl pb-7 text-base leading-relaxed sm:text-lg", night ? "text-night-muted" : "text-ink-muted")}>{item.a}</div>
          </div>
        </details>
      ))}
    </div>
  );
}
