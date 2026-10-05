"use client";

import { Check, RotateCcw, X } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

const LEVEL = [0, 0.45, 0.2, 0.55, 0.38, 0.62, 0.5, 0.58, 0.49, 0.56, 0.52, 0.55, 0.54];
const SPREAD = [1, 0.82, 0.7, 0.62, 0.55, 0.5, 0.45, 0.41, 0.38, 0.35, 0.33, 0.3, 0.22];
const RIGHT = [true, false, true, true, false, true, false, true, true, false, true, true, true];
const MIN = -2;
const MAX = 2;
const pos = (v: number) => ((Math.max(MIN, Math.min(MAX, v)) - MIN) / (MAX - MIN)) * 100;
const LAST = LEVEL.length - 1;
const ZONES = [
  { label: "Ниже Junior", from: MIN, to: -1 },
  { label: "Junior", from: -1, to: 0 },
  { label: "Middle", from: 0, to: 1 },
  { label: "Senior", from: 1, to: MAX },
];

function play(el: HTMLElement | null, delay = 0.3) {
  if (!el || !window.matchMedia(MOTION_OK).matches) return null;
  const marker = el.querySelector("[data-marker]");
  const band = el.querySelector("[data-band]");
  const answers = gsap.utils.toArray<HTMLElement>("[data-answer]", el);
  const counter = el.querySelector<HTMLElement>("[data-count]");
  const verdict = el.querySelector("[data-verdict]");
  const state = { p: 0 };
  const render = () => {
    const i = Math.min(LAST, Math.floor(state.p));
    const f = state.p - i;
    const j = Math.min(LAST, i + 1);
    const level = (LEVEL[i] ?? 0) + ((LEVEL[j] ?? 0) - (LEVEL[i] ?? 0)) * f;
    const spread = 1.96 * ((SPREAD[i] ?? 0) + ((SPREAD[j] ?? 0) - (SPREAD[i] ?? 0)) * f);
    gsap.set(marker, { left: `${pos(level)}%` });
    gsap.set(band, { left: `${pos(level - spread)}%`, width: `${pos(level + spread) - pos(level - spread)}%` });
    if (counter) counter.textContent = String(i + 1);
    answers.forEach((a, k) => gsap.set(a, { autoAlpha: k <= i ? 1 : 0.18, scale: k === i ? 1.15 : 1 }));
  };
  return gsap
    .timeline({ delay })
    .set(verdict, { autoAlpha: 0, y: 12 })
    .fromTo(state, { p: 0 }, { p: LAST, duration: 5, ease: "none", onUpdate: render })
    .to(verdict, { autoAlpha: 1, y: 0, duration: 0.7, ease: "expo.out" });
}

/** The level estimate and its uncertainty range narrowing with every answer until the range clears the claimed grade's threshold. */
export function LevelFinder({ className }: { className?: string }) {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        ScrollTrigger.create({ trigger: root.current, start: "top 75%", once: true, onEnter: () => play(root.current) });
      });
    },
    { scope: root },
  );
  const final = LEVEL[LAST] ?? 0;
  const finalSpread = 1.96 * (SPREAD[LAST] ?? 0);
  return (
    <div ref={root} className={cn("rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-7", className)}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Бэкенд · заявлен Middle</p>
          <p className="mt-1.5 text-lg font-semibold tracking-tight text-ink">
            Ответ <span data-count className="tabular">13</span> из ~13, уровень уточняется
          </p>
        </div>
        <button
          type="button"
          onClick={() => play(root.current, 0)}
          className="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-full border border-line-strong px-3 text-xs font-medium text-ink-muted transition-colors hover:border-ink hover:text-ink"
        >
          <RotateCcw className="size-3.5" aria-hidden />
          Ещё раз
        </button>
      </div>

      <div className="relative mt-10 mb-4" aria-hidden>
        <div className="flex h-12 overflow-hidden rounded-2xl border border-line">
          {ZONES.map((z) => (
            <div key={z.label} className={cn("flex flex-1 items-end justify-center border-r border-line pb-1.5 text-[11px] font-medium last:border-r-0", z.label === "Middle" ? "bg-primary-50 text-primary-strong" : "bg-canvas text-ink-subtle")}>
              {z.label}
            </div>
          ))}
        </div>
        <span data-band className="absolute top-0 h-12 rounded-2xl bg-primary/15 ring-1 ring-primary/30" style={{ left: `${pos(final - finalSpread)}%`, width: `${pos(final + finalSpread) - pos(final - finalSpread)}%` }} />
        <span data-marker className="absolute -top-3 flex -translate-x-1/2 flex-col items-center" style={{ left: `${pos(final)}%` }}>
          <span className="rounded-full bg-ink px-2 py-0.5 text-[10px] font-medium whitespace-nowrap text-white">вы здесь</span>
          <span className="h-14 w-0.5 rounded-full bg-ink" />
        </span>
      </div>
      <p className="text-xs text-ink-subtle">Подсвеченная полоса показывает, где ваш уровень. С каждым ответом она сужается.</p>

      <ol className="mt-6 flex flex-wrap gap-1.5" aria-label="Ответы по порядку">
        {RIGHT.map((ok, i) => (
          <li
            key={i}
            data-answer
            className={cn("flex size-7 items-center justify-center rounded-full text-white", ok ? "bg-primary" : "bg-accent")}
            aria-label={`Ответ ${i + 1}: ${ok ? "верно" : "неверно"}`}
          >
            {ok ? <Check className="size-3.5" strokeWidth={3} aria-hidden /> : <X className="size-3.5" strokeWidth={3} aria-hidden />}
          </li>
        ))}
      </ol>

      <p data-verdict className="mt-6 rounded-2xl bg-success-50 px-4 py-3 text-sm text-ink">
        <span className="font-semibold text-success">Готово на 13-м ответе.</span> Полоса целиком выше порога Middle, категория «Бэкенд · Middle» подтверждена.
      </p>
    </div>
  );
}
