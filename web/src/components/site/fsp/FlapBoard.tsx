"use client";

import { useRef } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";

const WIDTH = 14;
const ROWS = 3;
const GLYPHS = "АБВГДЕЖЗИКЛМНОПРСТУФХЦЧШЭЮЯ0123456789";
const STEP = 0.088;

const POOL: ReadonlyArray<readonly [string, string]> = [
  ["МАСТЕР СПОРТА", "ПРОДУКТОВОЕ"],
  ["1 МЕСТО", "ВСЕРОССИЙСКИЕ"],
  ["КМС", "АЛГОРИТМЫ"],
  ["ФИНАЛИСТ", "СИСТЕМЫ ИБ"],
  ["2 МЕСТО", "РОБОТОТЕХНИКА"],
  ["I РАЗРЯД", "СИСТЕМЫ ИИ"],
  ["ПРИЗЁР", "МЕЖДУНАРОДНЫЕ"],
  ["3 МЕСТО", "В КОМАНДЕ"],
  ["II РАЗРЯД", "БАС"],
  ["1 МЕСТО", "РЕГИОНАЛЬНЫЕ"],
  ["ЮНОШЕСКИЙ", "АЛГОРИТМЫ"],
];

const pad = (s: string) => s.padEnd(WIDTH, " ").slice(0, WIDTH);
const rowText = ([a, b]: readonly [string, string]) => pad(a) + pad(b);

interface Cell {
  top: HTMLElement;
  bottom: HTMLElement;
  flapTop: HTMLElement;
  flapBottom: HTMLElement;
  topGlyph: HTMLElement;
  bottomGlyph: HTMLElement;
  flapTopGlyph: HTMLElement;
  flapBottomGlyph: HTMLElement;
  ch: string;
  initial: string;
}

const HALF = "absolute inset-x-0 h-1/2 overflow-hidden bg-linear-to-b";
const GLYPH = "absolute inset-x-0 h-(--ch) text-center leading-(--ch)";

function Flap({ ch, tone }: { ch: string; tone: "lead" | "meta" }) {
  const color = tone === "lead" ? "text-white" : "text-primary-200";
  return (
    <span data-flap className={`relative block h-(--ch) w-(--cw) shrink-0 font-mono text-(length:--cf) font-bold ${color} perspective-[420px]`}>
      <span className={`${HALF} top-0 rounded-t-[4px] from-night-line to-night-soft`}>
        <span className={`${GLYPH} top-0`}>{ch}</span>
      </span>
      <span className={`${HALF} bottom-0 rounded-b-[4px] from-night-soft to-night-raised`}>
        <span className={`${GLYPH} bottom-0`}>{ch}</span>
      </span>
      <span className={`${HALF} top-0 origin-bottom rounded-t-[4px] from-night-line to-night-soft backface-hidden [transform:rotateX(-90deg)]`}>
        <span className={`${GLYPH} top-0`}>{ch}</span>
      </span>
      <span className={`${HALF} bottom-0 origin-top rounded-b-[4px] from-night-soft to-night-raised backface-hidden [transform:rotateX(90deg)]`}>
        <span className={`${GLYPH} bottom-0`}>{ch}</span>
      </span>
      <span aria-hidden className="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-night" />
    </span>
  );
}

function readCell(el: Element): Cell {
  const [top, bottom, flapTop, flapBottom] = Array.from(el.children) as HTMLElement[];
  const glyph = (h: HTMLElement) => h.firstElementChild as HTMLElement;
  const ch = glyph(top).textContent ?? " ";
  return { top, bottom, flapTop, flapBottom, topGlyph: glyph(top), bottomGlyph: glyph(bottom), flapTopGlyph: glyph(flapTop), flapBottomGlyph: glyph(flapBottom), ch, initial: ch };
}

function paint(c: Cell, ch: string) {
  c.ch = ch;
  c.topGlyph.textContent = ch;
  c.bottomGlyph.textContent = ch;
  c.flapTopGlyph.textContent = ch;
  c.flapBottomGlyph.textContent = ch;
}

/** Each flip shows a run of random glyphs before the target, like a Solari board drum. */
function cellTimeline(c: Cell, target: string, spins: number): gsap.core.Timeline {
  const tl = gsap.timeline();
  if (c.ch === target && target === " ") return tl;
  const seq: string[] = [];
  for (let i = 0; i < spins; i++) seq.push(GLYPHS[Math.floor(Math.random() * GLYPHS.length)]);
  seq.push(target);
  let from = c.ch;
  for (const to of seq) {
    const a = from;
    tl.call(() => {
      c.topGlyph.textContent = to;
      c.flapTopGlyph.textContent = a;
      c.flapBottomGlyph.textContent = to;
    })
      .fromTo(c.flapTop, { rotationX: 0 }, { rotationX: -90, duration: STEP / 2, ease: "power1.in", immediateRender: false })
      .fromTo(c.flapBottom, { rotationX: 90 }, { rotationX: 0, duration: STEP / 2, ease: "power1.out", immediateRender: false })
      .call(() => {
        c.bottomGlyph.textContent = to;
        c.ch = to;
      })
      .set(c.flapBottom, { rotationX: 90 });
    from = to;
  }
  return tl;
}

export function FlapBoard() {
  const root = useRef<HTMLDivElement>(null);
  const initial = POOL.slice(0, ROWS);

  useGSAP(
    () => {
      const board = root.current;
      if (!board) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const rows = gsap.utils.toArray<HTMLElement>("[data-row]", board).map((row) => ({
          cells: Array.from(row.querySelectorAll("[data-flap]")).map(readCell),
          lamp: row.querySelector<HTMLElement>("[data-lamp-on]"),
        }));
        const flaps = rows.flatMap((r) => r.cells.flatMap((c) => [c.flapTop, c.flapBottom]));
        const lamps = rows.map((r) => r.lamp).filter((l): l is HTMLElement => l !== null);
        const live = new Set<gsap.core.Animation>();
        let loop: gsap.core.Tween | null = null;
        let visible = true;

        gsap.set(flaps, { rotationX: (i: number) => (i % 2 ? 90 : -90) });
        gsap.set(lamps, { autoAlpha: 0, scale: 0.4 });
        for (const r of rows) for (const c of r.cells) paint(c, " ");

        const flipRow = (index: number, text: string, delay = 0, spins: [number, number] = [3, 8]) => {
          const row = rows[index];
          const tl = gsap.timeline({ delay, onComplete: () => void live.delete(tl) });
          if (row.lamp) tl.to(row.lamp, { autoAlpha: 0, scale: 0.4, duration: 0.2, ease: "power2.in" }, 0);
          row.cells.forEach((c, i) => {
            const target = text[i] ?? " ";
            const spin = target === " " && c.ch === " " ? 0 : gsap.utils.random(spins[0], spins[1], 1);
            tl.add(cellTimeline(c, target, spin), 0.05 + (i % WIDTH) * 0.032 + (i >= WIDTH ? 0.12 : 0));
          });
          if (row.lamp) tl.to(row.lamp, { autoAlpha: 1, scale: 1, duration: 0.5, ease: "back.out(3)" });
          live.add(tl);
          return tl;
        };

        gsap.set(board, { visibility: "visible" });
        const intro = gsap.timeline({ delay: 0.5 });
        intro.from(board, { autoAlpha: 0, y: 40, duration: 1, ease: "expo.out" });
        initial.forEach((entry, i) => intro.add(flipRow(i, rowText(entry), 0, [4, 10]), 0.35 + i * 0.22));

        let poolIndex = ROWS;
        let rowIndex = 0;
        const cycle = () => {
          if (visible && !document.hidden) {
            flipRow(rowIndex, rowText(POOL[poolIndex % POOL.length]));
            poolIndex += 1;
            rowIndex = (rowIndex + 1) % ROWS;
          }
          loop = gsap.delayedCall(2.6, cycle);
        };
        loop = gsap.delayedCall(5.2, cycle);

        ScrollTrigger.create({ trigger: board, start: "top bottom", end: "bottom top", onToggle: (self) => (visible = self.isActive) });

        return () => {
          loop?.kill();
          for (const a of live) a.kill();
          gsap.set(flaps, { clearProps: "transform" });
          for (const r of rows) for (const c of r.cells) paint(c, c.initial);
        };
      });
    },
    { scope: root },
  );

  return (
    <div
      ref={root}
      data-hero
      className="relative rounded-[22px] border border-night-line bg-night-raised p-3 shadow-[0_40px_80px_-40px_rgb(0_0_0/0.8)] [--cf:17px] [--ch:30px] [--cw:19px] sm:p-6 sm:[--cf:28px] sm:[--ch:48px] sm:[--cw:32px] lg:p-8 lg:[--cf:22px] lg:[--ch:38px] lg:[--cw:25px] xl:p-10 xl:[--cf:25px] xl:[--ch:44px] xl:[--cw:29px]"
    >
      {[
        "top-2.5 left-2.5",
        "top-2.5 right-2.5",
        "bottom-2.5 left-2.5",
        "right-2.5 bottom-2.5",
      ].map((pos) => (
        <span key={pos} aria-hidden className={`absolute size-1.5 rounded-full bg-night-line ${pos}`} />
      ))}
      <div aria-hidden className="flex items-center justify-between gap-4 px-1 pb-3 font-mono text-[10px] tracking-[0.16em] text-night-faint uppercase sm:text-[11px]">
        <span className="inline-flex items-center gap-2">
          <span className="size-1.5 rounded-full bg-accent" />
          Табло достижений
        </span>
        <span className="hidden xl:block">Протокол</span>
      </div>
      <ul className="sr-only">
        {initial.map(([a, b]) => (
          <li key={a + b}>
            {a.toLowerCase()}, {b.toLowerCase()}
          </li>
        ))}
      </ul>
      <div aria-hidden>
        {initial.map((entry, r) => {
          const text = rowText(entry);
          return (
            <div key={r} data-row className="flex items-center justify-between gap-3 border-t border-night-line py-2.5 sm:py-3.5">
              <div className="flex flex-col gap-[3px] sm:gap-1 lg:flex-row lg:items-center lg:gap-4">
                <div className="flex gap-[2px] sm:gap-[3px]">
                  {Array.from(text.slice(0, WIDTH), (ch, i) => (
                    <Flap key={i} ch={ch} tone="lead" />
                  ))}
                </div>
                <span className="hidden size-1.5 shrink-0 rounded-full bg-primary-300 lg:block" />
                <div className="flex gap-[2px] sm:gap-[3px]">
                  {Array.from(text.slice(WIDTH), (ch, i) => (
                    <Flap key={i} ch={ch} tone="meta" />
                  ))}
                </div>
              </div>
              <span className="relative block size-2.5 shrink-0 self-start rounded-full bg-night-line sm:size-3 lg:self-center xl:mr-7">
                <span data-lamp-on className="absolute inset-0 rounded-full bg-success-300 shadow-[0_0_14px_2px_rgb(95_211_155/0.6)]" />
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
