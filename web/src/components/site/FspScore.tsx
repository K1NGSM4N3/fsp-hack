"use client";

import { Award, Medal, Trophy } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, formatCounter, gsap, useGSAP } from "@/components/motion/gsap";

const ITEMS = [
  { icon: Medal, title: "Мастер спорта", meta: "Продуктовое программирование · 2023", v: 0.38 },
  { icon: Trophy, title: "1 место, всероссийские соревнования", meta: "Алгоритмическое программирование · команда · 2024", v: 0.27 },
  { icon: Award, title: "Финалист, межрегиональные соревнования", meta: "Программирование систем ИБ · 2022", v: 0.15 },
];

const steps = ITEMS.reduce<number[]>((acc, item) => [...acc, 1 - (1 - (acc.at(-1) ?? 0)) * (1 - item.v)], []);
const FINAL = steps.at(-1) ?? 0;

/** Noisy-OR accumulation: every achievement adds, but the score saturates and never outweighs the test. */
export function FspScore() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const rows = gsap.utils.toArray<HTMLElement>("[data-ach]", el);
        const bar = el.querySelector("[data-meter]");
        const num = el.querySelector<HTMLElement>("[data-meter-value]");
        const state = { v: 0 };
        const render = () => num && (num.textContent = formatCounter(state.v, 2));
        const tl = gsap.timeline({ scrollTrigger: { trigger: el, start: "top 75%", once: true }, onStart: render });
        gsap.set(bar, { scaleX: 0 });
        state.v = 0;
        render();
        rows.forEach((row, i) => {
          tl.from(row, { autoAlpha: 0, y: 24, duration: 0.6, ease: "expo.out" }, i * 0.7);
          tl.to(bar, { scaleX: steps[i] ?? 0, duration: 0.6, ease: "power3.out" }, i * 0.7 + 0.2);
          tl.to(state, { v: steps[i] ?? 0, duration: 0.6, ease: "power3.out", onUpdate: render }, i * 0.7 + 0.2);
        });
        return () => {
          if (num) num.textContent = formatCounter(FINAL, 2);
        };
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[24px] border border-line bg-white p-5 shadow-lift sm:p-7">
      <div className="flex items-center justify-between gap-3">
        <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Снимок из реестра ФСП</p>
        <span className="rounded-full bg-success-50 px-2.5 py-1 font-mono text-[11px] text-success">ФСП ID привязан</span>
      </div>
      <ul className="mt-5 space-y-3">
        {ITEMS.map(({ icon: Icon, title, meta, v }) => (
          <li key={title} data-ach className="flex items-center gap-4 rounded-2xl border border-line bg-canvas/60 p-3.5">
            <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary-50 text-primary">
              <Icon className="size-5" aria-hidden />
            </span>
            <span className="min-w-0 flex-1">
              <span className="block text-sm font-semibold text-ink">{title}</span>
              <span className="block truncate text-xs text-ink-subtle">{meta}</span>
            </span>
            <span className="tabular shrink-0 font-mono text-xs text-ink-muted">+{formatCounter(v, 2)}</span>
          </li>
        ))}
      </ul>
      <div className="mt-6 border-t border-line pt-5">
        <div className="flex items-baseline justify-between">
          <p className="text-sm font-medium text-ink">ФСП-скор</p>
          <p className="tabular font-display text-3xl font-semibold tracking-tight text-ink">
            <span data-meter-value>{formatCounter(FINAL, 2)}</span>
          </p>
        </div>
        <div className="relative mt-3 h-2.5 overflow-hidden rounded-full bg-primary-50" aria-hidden>
          <span data-meter className="absolute inset-0 origin-left rounded-full bg-primary" style={{ transform: `scaleX(${FINAL})` }} />
        </div>
        <p className="mt-3 font-mono text-xs text-ink-subtle">1 − Π(1 − vᵢ) · полураспад вклада — 2 года · вес в подборе ≤ 10 из 100</p>
      </div>
    </div>
  );
}
