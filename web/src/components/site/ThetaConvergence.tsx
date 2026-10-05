"use client";

import { RotateCcw } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, formatCounter, gsap, useGSAP } from "@/components/motion/gsap";

const EST = [0, 0.45, 0.2, 0.55, 0.38, 0.62, 0.5, 0.58, 0.49, 0.56, 0.52, 0.55, 0.54];
const SE = [1, 0.82, 0.7, 0.62, 0.55, 0.5, 0.45, 0.41, 0.38, 0.35, 0.33, 0.3, 0.26];
const CORRECT = [true, false, true, true, false, true, false, true, true, false, true, true, true];
const W = 560;
const H = 270;
const X0 = 44;
const X1 = 470;
const x = (i: number) => X0 + (i * (X1 - X0)) / (EST.length - 1);
const y = (theta: number) => 135 - Math.max(-2.2, Math.min(2.2, theta)) * 55;
const LINE = EST.map((t, i) => `${i ? "L" : "M"}${x(i).toFixed(1)} ${y(t).toFixed(1)}`).join(" ");
const BAND = [
  ...EST.map((t, i) => `${i ? "L" : "M"}${x(i).toFixed(1)} ${y(t + 1.96 * (SE[i] ?? 0)).toFixed(1)}`),
  ...EST.map((_, k) => {
    const i = EST.length - 1 - k;
    return `L${x(i).toFixed(1)} ${y((EST[i] ?? 0) - 1.96 * (SE[i] ?? 0)).toFixed(1)}`;
  }),
  "Z",
].join(" ");
const LAST = EST.length - 1;

function playTheta(el: HTMLElement | null) {
  if (!el || !window.matchMedia(MOTION_OK).matches) return;
  const clip = el.querySelector("[data-clip]");
  const line = el.querySelector("[data-line]");
  const dot = el.querySelector("[data-dot]");
  const items = gsap.utils.toArray<SVGCircleElement>("[data-item]", el);
  const step = el.querySelector<HTMLElement>("[data-step]");
  const est = el.querySelector<HTMLElement>("[data-est]");
  const ci = el.querySelector<HTMLElement>("[data-ci]");
  const verdict = el.querySelector("[data-verdict]");
  const state = { p: 0 };
  const render = () => {
    const i = Math.min(LAST, Math.floor(state.p));
    const f = state.p - i;
    const t = (EST[i] ?? 0) + ((EST[Math.min(LAST, i + 1)] ?? 0) - (EST[i] ?? 0)) * f;
    const s = (SE[i] ?? 0) + ((SE[Math.min(LAST, i + 1)] ?? 0) - (SE[i] ?? 0)) * f;
    gsap.set(dot, { attr: { cx: x(state.p), cy: y(t) } });
    if (step) step.textContent = String(i + 1);
    if (est) est.textContent = formatCounter(t, 2);
    if (ci) ci.textContent = formatCounter(1.96 * s, 2);
    items.forEach((c, k) => c.setAttribute("opacity", k <= i ? "1" : "0.25"));
  };
  gsap
    .timeline({ delay: 0.9 })
    .set(verdict, { autoAlpha: 0, y: 10 })
    .fromTo(clip, { attr: { width: 0 } }, { attr: { width: X1 - X0 + 24 }, duration: 5.2, ease: "none" }, 0)
    .fromTo(line, { drawSVG: "0%" }, { drawSVG: "100%", duration: 5.2, ease: "none" }, 0)
    .fromTo(state, { p: 0 }, { p: LAST, duration: 5.2, ease: "none", onUpdate: render }, 0)
    .to(verdict, { autoAlpha: 1, y: 0, duration: 0.6, ease: "expo.out" });
}

/** EAP estimate and its 95 % interval narrowing item by item until the interval clears the claimed grade threshold. */
export function ThetaConvergence() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => playTheta(root.current));
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Адаптивный тест · Бэкенд</p>
          <p className="mt-1 text-sm font-medium text-ink">Заявлен грейд Middle</p>
        </div>
        <button
          type="button"
          onClick={() => playTheta(root.current)}
          className="inline-flex h-9 items-center gap-1.5 rounded-full border border-line-strong px-3 text-xs font-medium text-ink-muted transition-colors hover:border-ink hover:text-ink"
        >
          <RotateCcw className="size-3.5" aria-hidden />
          Повторить
        </button>
      </div>
      <svg viewBox={`0 0 ${W} ${H}`} className="mt-4 block h-auto w-full" role="img" aria-label="Оценка уровня θ сходится за 13 заданий; 95-процентный интервал сужается и оказывается выше порога Middle.">
        <defs>
          <clipPath id="theta-clip">
            <rect data-clip x={X0 - 12} y="0" width={X1 - X0 + 24} height={H} />
          </clipPath>
        </defs>
        {[
          [-1, "Junior"],
          [0, "Middle"],
          [1, "Senior"],
        ].map(([t, label]) => (
          <g key={label}>
            <line x1={X0} x2={X1} y1={y(Number(t))} y2={y(Number(t))} stroke="currentColor" strokeDasharray="3 5" className={label === "Middle" ? "text-ink-faint" : "text-line-strong"} />
            <text x={X1 + 10} y={y(Number(t)) + 4} className={label === "Middle" ? "fill-ink font-mono text-[12px] font-semibold" : "fill-ink-subtle font-mono text-[12px]"}>
              {label}
            </text>
          </g>
        ))}
        <g clipPath="url(#theta-clip)">
          <path d={BAND} className="fill-primary/12" />
          <path data-line d={LINE} fill="none" strokeWidth="2.5" strokeLinejoin="round" className="stroke-primary" />
        </g>
        <circle data-dot cx={x(LAST)} cy={y(EST[LAST] ?? 0)} r="6" className="fill-primary stroke-white" strokeWidth="2.5" />
        <g transform={`translate(0 ${H - 22})`}>
          {CORRECT.map((ok, i) => (
            <circle key={i} data-item cx={x(i)} cy="0" r="5" className={ok ? "fill-primary" : "fill-accent"} />
          ))}
        </g>
        <text x={X0} y={H - 2} className="fill-ink-subtle text-[11px]">
          задания: синие — верно, красные — неверно
        </text>
      </svg>
      <div className="mt-4 grid grid-cols-3 gap-3 border-t border-line pt-4">
        <p>
          <span className="block text-xs text-ink-subtle">Задание</span>
          <span data-step className="tabular font-mono text-lg font-semibold text-ink">
            13
          </span>
        </p>
        <p>
          <span className="block text-xs text-ink-subtle">Оценка θ̂</span>
          <span data-est className="tabular font-mono text-lg font-semibold text-ink">
            0,54
          </span>
        </p>
        <p>
          <span className="block text-xs text-ink-subtle">95 % интервал, ±</span>
          <span data-ci className="tabular font-mono text-lg font-semibold text-ink">
            0,51
          </span>
        </p>
      </div>
      <p data-verdict className="mt-4 rounded-2xl bg-success-50 px-4 py-3 text-sm text-ink">
        <span className="font-semibold text-success">Стоп на 13-м задании.</span> Интервал целиком выше порога Middle — категория «Бэкенд · Middle» присвоена.
      </p>
    </div>
  );
}
