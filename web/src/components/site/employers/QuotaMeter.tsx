"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

const LIMIT = 30;
const STATIC_USED = 12;

/** Thirty weekly invitations fill up as the section scrolls past. */
export function QuotaMeter() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        const cells = q("[data-cell]");
        const num = q("[data-used]")[0];
        const done = q("[data-done]");
        gsap.set(cells, { backgroundColor: "rgb(255 255 255 / 0.08)", scaleY: 0.55 });
        gsap.set(done, { autoAlpha: 0, y: 10 });
        const state = { v: 0 };
        if (num) num.textContent = "0";
        const tl = gsap.timeline({ scrollTrigger: { trigger: root.current, start: "top 80%", end: "bottom 35%", scrub: 0.6 } });
        tl.to(cells, { backgroundColor: "var(--color-primary-400)", scaleY: 1, stagger: 0.1, duration: 0.3, ease: "none" }, 0)
          .to(state, { v: LIMIT, duration: cells.length * 0.1 + 0.2, ease: "none", onUpdate: () => num && (num.textContent = String(Math.round(state.v))) }, 0)
          .to(done, { autoAlpha: 1, y: 0, duration: 0.3 });
        return () => {
          if (num) num.textContent = String(STATIC_USED);
        };
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[24px] border border-night-line bg-night-raised p-6 sm:p-8">
      <div className="flex items-end justify-between gap-4">
        <div>
          <p className="text-sm text-night-muted">Приглашений на этой неделе</p>
          <p className="tabular mt-1 font-display text-6xl font-semibold tracking-[-0.04em] text-white">
            <span data-used>{STATIC_USED}</span>
            <span className="text-2xl text-night-faint"> / {LIMIT}</span>
          </p>
        </div>
        <p data-done className="max-w-[12rem] text-right text-sm text-night-muted">
          Новые 30 появятся через 7 дней
        </p>
      </div>
      <div className="mt-6 grid grid-cols-[repeat(30,minmax(0,1fr))] gap-1" aria-hidden>
        {Array.from({ length: LIMIT }, (_, i) => (
          <span key={i} data-cell className={cn("h-10 origin-bottom rounded-[4px]", i < STATIC_USED ? "bg-primary-400" : "bg-white/10")} />
        ))}
      </div>
      <p className="mt-5 text-sm leading-relaxed text-night-muted">Квота заставляет выбирать. Поэтому кандидаты открывают приглашения: их присылают не всем подряд.</p>
    </div>
  );
}
