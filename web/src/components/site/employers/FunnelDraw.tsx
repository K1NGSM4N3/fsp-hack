"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";

const STAGES = [
  { label: "Отправлено", value: 30 },
  { label: "Просмотрено", value: 24 },
  { label: "Ответили", value: 20 },
  { label: "Приняли", value: 11 },
];

const W = 520;
const H = 240;
const COL = 100;
const GAP = 40;
const top = (v: number) => H / 2 - (v / 30) * (H / 2 - 8);
const bottom = (v: number) => H / 2 + (v / 30) * (H / 2 - 8);

const OUTLINE = (() => {
  const upper: string[] = [];
  const lower: string[] = [];
  STAGES.forEach((s, i) => {
    const x = 10 + i * (COL + GAP);
    upper.push(`${i ? "L" : "M"}${x} ${top(s.value)} L${x + COL} ${top(s.value)}`);
    lower.unshift(`L${x + COL} ${bottom(s.value)} L${x} ${bottom(s.value)}`);
  });
  return `${upper.join(" ")} ${lower.join(" ")} Z`;
})();

/** The invitation funnel outline is drawn with DrawSVG, then filled; stage numbers scramble in. */
export function FunnelDraw() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        gsap
          .timeline({ scrollTrigger: { trigger: root.current, start: "top 75%", end: "center 45%", scrub: 0.8 } })
          .from(q("[data-outline]"), { drawSVG: "0%", duration: 1, ease: "none" })
          .from(q("[data-fill]"), { opacity: 0, duration: 0.4 }, ">-0.2")
          .from(q("[data-divider]"), { drawSVG: "50% 50%", stagger: 0.1, duration: 0.3 }, "<");
      });
    },
    { scope: root },
  );

  return (
    <div ref={root}>
      <svg viewBox={`0 0 ${W} ${H}`} className="block h-auto w-full" role="img" aria-label="Воронка: отправлено 30, просмотрено 24, ответили 20, приняли 11">
        <defs>
          <linearGradient id="funnel-fill" x1="0" x2="1" y1="0" y2="0">
            <stop offset="0" stopColor="var(--color-primary)" stopOpacity="0.22" />
            <stop offset="1" stopColor="var(--color-primary)" stopOpacity="0.5" />
          </linearGradient>
        </defs>
        <path data-fill d={OUTLINE} fill="url(#funnel-fill)" />
        <path data-outline d={OUTLINE} fill="none" stroke="var(--color-primary)" strokeWidth="2" strokeLinejoin="round" />
        {STAGES.slice(1).map((s, i) => {
          const x = 10 + (i + 1) * (COL + GAP) - GAP / 2;
          return <line key={s.label} data-divider x1={x} x2={x} y1={8} y2={H - 8} stroke="var(--color-line-strong)" strokeDasharray="3 5" />;
        })}
      </svg>
      <dl className="mt-5 grid grid-cols-4 gap-2">
        {STAGES.map((s) => (
          <div key={s.label}>
            <dt className="text-xs text-ink-subtle">{s.label}</dt>
            <dd data-scramble className="tabular mt-1 font-display text-2xl font-semibold tracking-tight text-ink sm:text-3xl">
              {s.value}
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
