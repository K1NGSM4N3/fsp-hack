"use client";

import { RefreshCw } from "lucide-react";
import { memo, useRef, useState } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

const CITIES = ["Казань", "Пермь", "Томск", "Самара", "Омск", "Тверь", "Сочи", "Уфа"];

interface Instance {
  seed: string;
  threshold: number;
  minCount: number;
  rows: { id: number; city: string; amount: number }[];
  answer: number;
}

function rng(seed: number) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** Same generator shape as the real `sql.group_by_having` template: the answer is computed, never stored. */
function makeInstance(seed: number): Instance {
  const r = rng(seed);
  const threshold = 500 + Math.floor(r() * 9) * 50;
  const minCount = 2 + Math.floor(r() * 2);
  const pool = [...CITIES];
  for (let i = pool.length - 1; i > 0; i--) {
    const j = Math.floor(r() * (i + 1));
    [pool[i], pool[j]] = [pool[j] ?? "", pool[i] ?? ""];
  }
  pool.length = 4;
  const rows = Array.from({ length: 12 }, (_, i) => ({ id: 101 + i, city: pool[Math.floor(r() * pool.length)] ?? "Казань", amount: 300 + Math.round(r() * 14) * 50 }));
  const counts = new Map<string, number>();
  for (const row of rows) if (row.amount > threshold) counts.set(row.city, (counts.get(row.city) ?? 0) + 1);
  const answer = [...counts.values()].filter((n) => n >= minCount).length;
  return { seed: `0x${(seed >>> 0).toString(16).toUpperCase().padStart(8, "0")}`, threshold, minCount, rows, answer };
}

const INITIAL: [Instance, Instance] = [makeInstance(0x5f3a91c2), makeInstance(0xa0417e6b)];

const plural = (n: number) => (n % 10 === 1 && n % 100 !== 11 ? "город" : n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 10 || n % 100 >= 20) ? "города" : "городов");

function valuesOf(inst: Instance): Record<string, string> {
  const out: Record<string, string> = {
    seed: inst.seed,
    threshold: String(inst.threshold),
    min: String(inst.minCount),
    answer: `${inst.answer} ${plural(inst.answer)}`,
  };
  inst.rows.slice(0, 4).forEach((row, i) => {
    out[`city-${i}`] = row.city;
    out[`amount-${i}`] = String(row.amount);
  });
  return out;
}

const Panel = memo(function Panel({ who, inst, index }: { who: string; inst: Instance; index: number }) {
  const v = (k: string, value: string, className?: string) => (
    <span data-k={`${index}:${k}`} className={className}>
      {value}
    </span>
  );
  return (
    <div className="min-w-0 overflow-hidden rounded-[20px] border border-night-line bg-night-raised text-white">
      <div className="flex items-center justify-between gap-3 border-b border-night-line px-4 py-3">
        <span className="flex items-center gap-2 text-sm font-medium">
          <span className={cn("size-2 rounded-full", index === 0 ? "bg-primary-400" : "bg-white")} aria-hidden />
          {who}
        </span>
        <span className="tabular truncate font-mono text-[11px] text-night-faint">seed {v("seed", inst.seed, "text-night-muted")}</span>
      </div>
      <pre className="scrollbar-thin overflow-x-auto px-4 py-4 font-mono text-[12.5px] leading-6 text-night-muted">
        <code>
          <span className="text-primary-300">SELECT</span> c.city, <span className="text-primary-300">COUNT</span>(*){"\n"}
          <span className="text-primary-300">FROM</span> orders o <span className="text-primary-300">JOIN</span> customers c{"\n"}
          {"  "}
          <span className="text-primary-300">ON</span> c.id = o.customer_id{"\n"}
          <span className="text-primary-300">WHERE</span> o.amount &gt; {v("threshold", String(inst.threshold), "rounded bg-primary/30 px-1 text-white")}
          {"\n"}
          <span className="text-primary-300">GROUP BY</span> c.city{"\n"}
          <span className="text-primary-300">HAVING</span> <span className="text-primary-300">COUNT</span>(*) &gt;= {v("min", String(inst.minCount), "rounded bg-primary/30 px-1 text-white")};
        </code>
      </pre>
      <div className="border-t border-night-line px-4 py-3">
        <table className="w-full font-mono text-[12px]">
          <caption className="sr-only">Фрагмент сгенерированной таблицы заказов</caption>
          <thead>
            <tr className="text-left text-night-faint">
              <th className="pb-1.5 font-normal">id</th>
              <th className="pb-1.5 font-normal">city</th>
              <th className="pb-1.5 text-right font-normal">amount</th>
            </tr>
          </thead>
          <tbody className="text-night-muted">
            {inst.rows.slice(0, 4).map((row, i) => (
              <tr key={row.id}>
                <td className="py-0.5">{row.id}</td>
                <td className="py-0.5">{v(`city-${i}`, row.city)}</td>
                <td className="tabular py-0.5 text-right">{v(`amount-${i}`, String(row.amount))}</td>
              </tr>
            ))}
            <tr>
              <td colSpan={3} className="pt-1 text-night-faint">
                … ещё 8 строк
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div className="flex items-center justify-between gap-3 border-t border-night-line bg-night/50 px-4 py-3 text-sm">
        <span className="text-night-muted">Ответ генератора</span>
        <span className="font-mono font-semibold text-white">{v("answer", `${inst.answer} ${plural(inst.answer)}`)}</span>
      </div>
    </div>
  );
});

export function LeakDemo() {
  const root = useRef<HTMLDivElement>(null);
  const [count, setCount] = useState(2);

  const regenerate = () => {
    const el = root.current;
    if (!el) return;
    const animate = window.matchMedia(MOTION_OK).matches;
    [0, 1].forEach((index) => {
      const values = valuesOf(makeInstance(Math.floor(Math.random() * 0xffffffff)));
      for (const [k, text] of Object.entries(values)) {
        const node = el.querySelector<HTMLElement>(`[data-k="${index}:${k}"]`);
        if (!node) continue;
        if (!animate) node.textContent = text;
        else {
          const chars = /^\d+$/.test(text) ? "0123456789" : k === "seed" ? "0123456789ABCDEF" : "абвгдежзиклмнопрстуф";
          gsap.to(node, { duration: 0.9, delay: index * 0.12, scrambleText: { text, chars, speed: 0.6 }, overwrite: true });
        }
      }
    });
    setCount((c) => c + 2);
  };

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        let loop: gsap.core.Tween | null = null;
        const cycle = () => {
          regenerate();
          loop = gsap.delayedCall(4.2, cycle);
        };
        ScrollTrigger.create({
          trigger: root.current,
          start: "top 80%",
          end: "bottom 20%",
          onToggle: (self) => {
            loop?.kill();
            loop = self.isActive ? gsap.delayedCall(2.4, cycle) : null;
          },
        });
        return () => loop?.kill();
      });
    },
    { scope: root },
  );

  return (
    <div ref={root}>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <Panel who="Кандидат А" inst={INITIAL[0]} index={0} />
        <Panel who="Кандидат Б" inst={INITIAL[1]} index={1} />
      </div>
      <div className="mt-5 flex flex-wrap items-center justify-between gap-4">
        <p className="font-mono text-xs text-night-muted">
          Шаблон <span className="text-white">sql.group_by_having</span> · одна тема, один уровень сложности · экземпляров показано: <span className="tabular text-white">{count}</span>
        </p>
        <button
          type="button"
          onClick={regenerate}
          className="inline-flex h-11 items-center gap-2 rounded-full border border-white/25 px-5 text-sm font-medium text-white transition-colors hover:border-white hover:bg-white/5"
        >
          <RefreshCw className="size-4" aria-hidden />
          Новый seed
        </button>
      </div>
    </div>
  );
}

const LEAK = [0, 25, 50, 100];
const FIXED = [9.4, 65.2, 99.9, 100];
const OURS = [9.7, 9.6, 9.3, 10.4];
const W = 520;
const H = 320;
const PAD = { l: 40, r: 70, t: 16, b: 46 };
const xAt = (leak: number) => PAD.l + (leak / 100) * (W - PAD.l - PAD.r);
const yAt = (pct: number) => PAD.t + (1 - pct / 100) * (H - PAD.t - PAD.b);
const path = (values: number[]) => values.map((v, i) => `${i ? "L" : "M"}${xAt(LEAK[i] ?? 0).toFixed(1)} ${yAt(v).toFixed(1)}`).join(" ");
const pct = (v: number) => `${v.toLocaleString("ru-RU", { maximumFractionDigits: 1 })} %`;

/** Share of weak candidates passing a Middle test as more answers leak (validation report §6.2). */
export function LeakChart() {
  const [hover, setHover] = useState<number | null>(null);
  const series = [
    { name: "Фиксированный тест", values: FIXED, color: "var(--color-accent)" },
    { name: "Параметрический тест ФСП Таланты", values: OURS, color: "var(--color-primary)" },
  ];
  return (
    <figure className="rounded-[24px] border border-line bg-white p-5 sm:p-7">
      <figcaption>
        <p className="text-lg font-semibold tracking-tight text-ink">Сколько слабых кандидатов проходит тест на Middle</p>
        <p className="mt-1 text-sm text-ink-muted">Доля прошедших в зависимости от того, какая часть ответов утекла</p>
        <ul className="mt-4 flex flex-wrap gap-x-5 gap-y-2 text-sm text-ink">
          {series.map((s) => (
            <li key={s.name} className="flex items-center gap-2">
              <span className="h-0.5 w-5 rounded-full" style={{ background: s.color }} aria-hidden />
              {s.name}
            </li>
          ))}
        </ul>
      </figcaption>
      <div className="relative mt-4">
        <svg data-draw viewBox={`0 0 ${W} ${H}`} className="block h-auto w-full overflow-visible" role="img" aria-label="Фиксированный тест: 9,4 %, 65,2 %, 99,9 %, 100 % при утечке 0, 25, 50, 100 % ответов. Параметрический тест: 9,7 %, 9,6 %, 9,3 %, 10,4 %.">
          <g aria-hidden>
            {[0, 25, 50, 75, 100].map((t) => (
              <g key={t}>
                <rect data-skip x={PAD.l} y={yAt(t)} width={W - PAD.l - PAD.r} height="1" className="fill-line" />
                <text x={PAD.l - 10} y={yAt(t) + 4} textAnchor="end" className="fill-ink-subtle font-mono text-[13px]">
                  {t}
                </text>
              </g>
            ))}
            {LEAK.map((l) => (
              <text key={l} x={xAt(l)} y={H - PAD.b + 22} textAnchor="middle" className="fill-ink-subtle font-mono text-[13px]">
                {l} %
              </text>
            ))}
            <text x={xAt(50)} y={H - 4} textAnchor="middle" className="fill-ink-subtle text-[13px]">
              утекло ответов
            </text>
            {hover !== null && <rect x={xAt(LEAK[hover] ?? 0) - 0.5} y={PAD.t} width="1" height={H - PAD.t - PAD.b} className="fill-ink-faint" />}
          </g>
          {series.map((s) => (
            <path key={s.name} d={path(s.values)} fill="none" stroke={s.color} strokeWidth="2.5" strokeLinejoin="round" strokeLinecap="round" />
          ))}
          {series.map((s) =>
            s.values.map((v, i) => <circle key={`${s.name}-${i}`} cx={xAt(LEAK[i] ?? 0)} cy={yAt(v)} r={hover === i ? 6 : 4.5} fill={s.color} stroke="white" strokeWidth="2" />),
          )}
          <g aria-hidden className="text-[13px]">
            <text x={xAt(100) + 12} y={yAt(100) + 4} className="fill-ink font-mono font-semibold">
              100 %
            </text>
            <text x={xAt(100) + 12} y={yAt(10.4) + 4} className="fill-ink font-mono font-semibold">
              10,4 %
            </text>
          </g>
          {LEAK.map((l, i) => (
            <rect
              key={l}
              x={xAt(l) - 40}
              y={0}
              width={80}
              height={H}
              fill="transparent"
              onPointerEnter={() => setHover(i)}
              onPointerLeave={() => setHover((h) => (h === i ? null : h))}
            />
          ))}
        </svg>
        {hover !== null && (
          <div
            className="pointer-events-none absolute top-2 z-10 w-52 -translate-x-1/2 rounded-xl border border-line bg-white p-3 text-xs shadow-pop"
            style={{ left: `${(xAt(LEAK[hover] ?? 0) / W) * 100}%` }}
          >
            <p className="font-mono text-ink-subtle">Утекло {LEAK[hover]} % ответов</p>
            {series.map((s) => (
              <p key={s.name} className="mt-1.5 flex items-center justify-between gap-3 text-ink">
                <span className="flex items-center gap-1.5">
                  <span className="size-2 rounded-full" style={{ background: s.color }} aria-hidden />
                  {s.name.startsWith("Фикс") ? "Фиксированный" : "Наш"}
                </span>
                <span className="tabular font-mono font-semibold">{pct(s.values[hover] ?? 0)}</span>
              </p>
            ))}
          </div>
        )}
      </div>
      <details className="group mt-4 text-sm">
        <summary className="cursor-pointer font-medium text-primary marker:text-primary">Показать таблицей</summary>
        <table className="mt-3 w-full text-left font-mono text-[13px]">
          <thead className="text-ink-subtle">
            <tr>
              <th className="py-1.5 font-normal">Утекло</th>
              <th className="py-1.5 font-normal">Фиксированный тест</th>
              <th className="py-1.5 font-normal">Наш тест</th>
            </tr>
          </thead>
          <tbody className="text-ink">
            {LEAK.map((l, i) => (
              <tr key={l} className="border-t border-line">
                <td className="py-1.5">{l} %</td>
                <td className="py-1.5">{pct(FIXED[i] ?? 0)}</td>
                <td className="py-1.5">{pct(OURS[i] ?? 0)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </figure>
  );
}
