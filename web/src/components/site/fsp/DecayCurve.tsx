"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";

const W = 520;
const H = 280;
const PAD = { l: 44, r: 20, t: 20, b: 44 };
const YEARS = 6;
const x = (t: number) => PAD.l + (t / YEARS) * (W - PAD.l - PAD.r);
const y = (v: number) => PAD.t + (1 - v / 100) * (H - PAD.t - PAD.b);
const weight = (t: number) => 100 * 0.5 ** (t / 2);
const CURVE = Array.from({ length: 61 }, (_, i) => {
  const t = (i / 60) * YEARS;
  return `${i ? "L" : "M"}${x(t).toFixed(1)} ${y(weight(t)).toFixed(1)}`;
}).join(" ");
const MARKS = [
  { t: 0, label: "в этом году", value: "100 %" },
  { t: 2, label: "2 года назад", value: "50 %" },
  { t: 4, label: "4 года назад", value: "25 %" },
  { t: 6, label: "6 лет назад", value: "12,5 %" },
];

/** Half-life of two years, drawn as the reader scrolls; markers appear as the line reaches them. */
export function DecayCurve() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        const tl = gsap.timeline({ scrollTrigger: { trigger: root.current, start: "top 75%", end: "bottom 55%", scrub: 0.7 } });
        tl.from(q("[data-curve]"), { drawSVG: "0%", duration: 1, ease: "none" }, 0);
        q("[data-mark]").forEach((m, i) => tl.from(m, { scale: 0, autoAlpha: 0, transformOrigin: "50% 50%", duration: 0.12, ease: "back.out(3)" }, (MARKS[i]?.t ?? 0) / YEARS - 0.02));
      });
    },
    { scope: root },
  );

  return (
    <figure ref={root} className="rounded-[24px] border border-line bg-white p-5 sm:p-7">
      <figcaption className="text-lg font-semibold tracking-tight text-ink">Сколько весит достижение со временем</figcaption>
      <svg viewBox={`0 0 ${W} ${H}`} className="mt-4 block h-auto w-full overflow-visible" role="img" aria-label="Вклад достижения: 100 % в этом году, 50 % через 2 года, 25 % через 4 года, 12,5 % через 6 лет">
        {[0, 25, 50, 75, 100].map((v) => (
          <g key={v} aria-hidden>
            <line x1={PAD.l} x2={W - PAD.r} y1={y(v)} y2={y(v)} stroke="var(--color-line)" />
            <text x={PAD.l - 8} y={y(v) + 4} textAnchor="end" className="fill-ink-subtle font-mono text-[12px]">
              {v}
            </text>
          </g>
        ))}
        {[0, 2, 4, 6].map((t) => (
          <text key={t} x={x(t)} y={H - PAD.b + 24} textAnchor="middle" className="fill-ink-subtle font-mono text-[12px]" aria-hidden>
            {t === 0 ? "сейчас" : `${t} г.`}
          </text>
        ))}
        <path d={`${CURVE} L${x(YEARS)} ${y(0)} L${x(0)} ${y(0)} Z`} className="fill-primary/8" />
        <path data-curve d={CURVE} fill="none" stroke="var(--color-primary)" strokeWidth="2.5" strokeLinecap="round" />
        {MARKS.map((m) => (
          <g key={m.t} data-mark>
            <circle cx={x(m.t)} cy={y(weight(m.t))} r="5.5" fill="var(--color-primary)" stroke="white" strokeWidth="2" />
            <text x={x(m.t) + (m.t === YEARS ? -8 : 10)} y={y(weight(m.t)) - 12} textAnchor={m.t === YEARS ? "end" : "start"} className="fill-ink font-mono text-[13px] font-semibold">
              {m.value}
            </text>
          </g>
        ))}
      </svg>
      <ul className="mt-4 grid grid-cols-2 gap-2 text-xs text-ink-muted sm:grid-cols-4">
        {MARKS.map((m) => (
          <li key={m.t}>
            <span className="tabular font-mono font-semibold text-ink">{m.value}</span> {m.label}
          </li>
        ))}
      </ul>
    </figure>
  );
}
