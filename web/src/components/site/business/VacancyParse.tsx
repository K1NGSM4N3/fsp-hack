"use client";

import { ArrowDown, Check, PenLine, RotateCcw } from "lucide-react";
import { useRef } from "react";
import { type Kit, useMotion } from "@/components/motion/lazy";
import { cn } from "@/lib/cn";

interface Piece {
  text: string;
  chip?: string;
}

const VACANCY: Piece[] = [
  { text: "Ищем " },
  { text: "бэкенд-разработчика", chip: "role" },
  { text: " уровня " },
  { text: "Middle", chip: "grade" },
  { text: " в команду платежей. Пишем на " },
  { text: "Go", chip: "go" },
  { text: ", данные храним в " },
  { text: "PostgreSQL", chip: "pg" },
  { text: ", события идут через " },
  { text: "Kafka", chip: "kafka" },
  { text: ". Опыт с " },
  { text: "Kubernetes", chip: "k8s" },
  { text: " будет плюсом. Платим " },
  { text: "280–350 тыс. ₽", chip: "salary" },
  { text: " на руки, работать можно " },
  { text: "удалённо или в офисе в Москве", chip: "format" },
  { text: "." },
];

interface Field {
  label: string;
  chips: { id: string; label: string; optional?: boolean }[];
}

const PUNCT = /^[,.]/;

const loadMotionPath = () => import("@/components/motion/extras/motion-path");

/** Punctuation right after a phrase is glued to it, so a line never starts with a comma once the text is split into words. */
const SEGMENTS = VACANCY.map((p, i) => {
  const next = VACANCY[i + 1]?.text ?? "";
  const prevIsPhrase = Boolean(VACANCY[i - 1]?.chip);
  return {
    chip: p.chip,
    text: !p.chip && prevIsPhrase && PUNCT.test(p.text) ? p.text.slice(1) : p.text,
    tail: p.chip && PUNCT.test(next) ? next.charAt(0) : "",
  };
});

const FIELDS: Field[] = [
  { label: "Специализация", chips: [{ id: "role", label: "Бэкенд-разработка" }] },
  { label: "Грейд", chips: [{ id: "grade", label: "Middle" }] },
  {
    label: "Навыки",
    chips: [
      { id: "go", label: "Go" },
      { id: "pg", label: "PostgreSQL" },
      { id: "kafka", label: "Kafka" },
      { id: "k8s", label: "Kubernetes", optional: true },
    ],
  },
  { label: "Вилка", chips: [{ id: "salary", label: "280–350 тыс. ₽" }] },
  { label: "Формат", chips: [{ id: "format", label: "Удалённо или офис, Москва" }] },
];

const TYPE_STEP = 0.021;
const LEAD = 0.7;

interface Box {
  left: number;
  top: number;
  height: number;
}

function boxOf(el: Element): Box {
  const r = el.getBoundingClientRect();
  return { left: r.left, top: r.top, height: r.height };
}

/** Flies a chip from the phrase it was read from to its field along a bowed arc. */
function flight({ gsap }: Pick<Kit, "gsap">, chip: HTMLElement, from: Box) {
  const to = boxOf(chip);
  const dx = from.left - to.left;
  const dy = from.top - to.top + (from.height - to.height) / 2;
  const len = Math.hypot(dx, dy) || 1;
  let nx = -dy / len;
  let ny = dx / len;
  if (ny > 0) {
    nx = -nx;
    ny = -ny;
  }
  const bow = Math.min(110, len * 0.28);
  return gsap
    .timeline()
    .set(chip, { x: dx, y: dy, scale: 0.86, autoAlpha: 0 })
    .to(chip, { autoAlpha: 1, duration: 0.12 }, 0)
    .to(
      chip,
      {
        motionPath: {
          path: [
            { x: dx, y: dy },
            { x: dx / 2 + nx * bow, y: dy / 2 + ny * bow },
            { x: 0, y: 0 },
          ],
          curviness: 1.3,
        },
        scale: 1,
        duration: 0.95,
        ease: "power3.inOut",
      },
      0,
    );
}

/**
 * Phones: the text is already there; recognised phrases light up one by one and each chip pops into its field.
 * No per-letter split, no flight paths, a dozen short tweens in total.
 */
function mobileParse({ gsap, ScrollTrigger }: Pick<Kit, "gsap" | "ScrollTrigger">, el: HTMLElement, ctx: gsap.Context, replay: HTMLButtonElement) {
  const q = gsap.utils.selector(el);
  const go = q("[data-go]");
  let tl: gsap.core.Timeline | null = null;
  const build = () => {
    tl = gsap.timeline({ onComplete: () => void (replay.disabled = false) });
    tl.set(q("[data-mark]"), { scaleX: 0, transformOrigin: "0% 50%" })
      .set(q("[data-chip]"), { autoAlpha: 0 })
      .set(q("[data-empty]"), { autoAlpha: 1 })
      .set(q("[data-check]"), { autoAlpha: 0, scale: 0.4 })
      .set(go, { autoAlpha: 0.35 });
    gsap.utils.toArray<HTMLElement>("[data-phrase]", el).forEach((phrase, i) => {
      const chip = el.querySelector<HTMLElement>(`[data-chip="${phrase.dataset.phrase}"]`);
      const field = chip?.closest("[data-field]");
      const at = 0.35 + i * 0.42;
      tl?.to(phrase.querySelector("[data-mark]"), { scaleX: 1, duration: 0.3, ease: "power2.out" }, at);
      if (!chip || !field) return;
      tl?.fromTo(chip, { autoAlpha: 0, y: -8, scale: 0.7 }, { autoAlpha: 1, y: 0, scale: 1, duration: 0.45, ease: "back.out(2.2)" }, at + 0.15);
      tl?.to(field.querySelector("[data-empty]"), { autoAlpha: 0, duration: 0.2 }, at + 0.15);
      if (field.querySelector("[data-chip]:last-of-type") === chip) {
        tl?.to(field.querySelector("[data-check]"), { autoAlpha: 1, scale: 1, duration: 0.35, ease: "back.out(2.4)" }, at + 0.4);
      }
    });
    tl.to(go, { autoAlpha: 1, duration: 0.4 }, "+=0.1").fromTo(q("[data-go-ring]"), { autoAlpha: 0.7, scale: 1 }, { autoAlpha: 0, scale: 1.35, duration: 1.1, ease: "power2.out" }, "<");
  };
  const onReplay = () => {
    replay.disabled = true;
    tl?.restart();
  };
  replay.disabled = true;
  replay.addEventListener("click", onReplay);
  ScrollTrigger.create({ trigger: el, start: "top 80%", once: true, onEnter: () => ctx.add(build) });
  return () => replay.removeEventListener("click", onReplay);
}

/**
 * Hero demo: a vacancy types itself, recognised phrases light up and fly into the structured need.
 * The server markup is the finished state, so without JS or with reduced motion the result is simply shown.
 */
export function VacancyParse() {
  const root = useRef<HTMLDivElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, ScrollTrigger, SplitText, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_MOBILE, (ctx) => {
        const el = root.current;
        const replay = el?.querySelector<HTMLButtonElement>("[data-replay]");
        if (el && replay) return mobileParse({ gsap, ScrollTrigger }, el, ctx, replay);
      });
      mm.add(MOTION_DESKTOP, (ctx) => {
        const el = root.current;
        const text = el?.querySelector<HTMLElement>("[data-text]");
        const caret = el?.querySelector<HTMLElement>("[data-caret]");
        const replay = el?.querySelector<HTMLButtonElement>("[data-replay]");
        if (!el || !text || !caret || !replay) return;

        // Split letters are aria-hidden; screen readers get this one plain sentence instead (server HTML has it once).
        const copy = document.createElement("span");
        copy.className = "sr-only";
        copy.textContent = text.textContent;
        text.after(copy);
        const split = SplitText.create(text, { type: "words,chars", charsClass: "vp-char", wordsClass: "vp-word", aria: "hidden" });
        const chars = split.chars as HTMLElement[];
        const phrases = gsap.utils.toArray<HTMLElement>("[data-phrase]", el);
        const marks = gsap.utils.toArray<HTMLElement>("[data-mark]", el);
        const chips = gsap.utils.toArray<HTMLElement>("[data-chip]", el);
        const empties = gsap.utils.toArray<HTMLElement>("[data-empty]", el);
        const checks = gsap.utils.toArray<HTMLElement>("[data-check]", el);
        const go = el.querySelector<HTMLElement>("[data-go]");
        const goRing = el.querySelector<HTMLElement>("[data-go-ring]");

        gsap.set(chars, { autoAlpha: 0 });
        gsap.set(marks, { scaleX: 0, transformOrigin: "0% 50%" });
        gsap.set(chips, { autoAlpha: 0 });
        gsap.set(empties, { autoAlpha: 1 });
        gsap.set(checks, { autoAlpha: 0, scale: 0.4 });
        gsap.set(caret, { autoAlpha: 0 });
        if (go) gsap.set(go, { autoAlpha: 0.35 });

        let tl: gsap.core.Timeline | null = null;

        const build = () => {
          const origin = text.getBoundingClientRect();
          const at = chars.map((c) => {
            const r = c.getBoundingClientRect();
            return { x: r.right - origin.left, y: r.top - origin.top + r.height * 0.12, h: r.height * 0.76 };
          });
          const first = chars[0]?.getBoundingClientRect();
          tl = gsap.timeline({ paused: true });
          tl.set(caret, { autoAlpha: 1, x: first ? first.left - origin.left : 0, y: at[0]?.y ?? 0, height: at[0]?.h ?? 20 }, 0);
          chars.forEach((c, i) => {
            const p = at[i];
            if (!p) return;
            tl?.set(c, { autoAlpha: 1 }, LEAD + i * TYPE_STEP).set(caret, { x: p.x, y: p.y }, LEAD + i * TYPE_STEP);
          });

          const typed = LEAD + chars.length * TYPE_STEP;
          for (const phrase of phrases) {
            const id = phrase.dataset.phrase;
            const own = phrase.querySelectorAll(".vp-char");
            const last = own[own.length - 1];
            const chip = el.querySelector<HTMLElement>(`[data-chip="${id}"]`);
            const mark = phrase.querySelector("[data-mark]");
            if (!last || !chip || !mark) continue;
            const t = LEAD + (chars.indexOf(last as HTMLElement) + 1) * TYPE_STEP;
            const field = chip.closest("[data-field]");
            const empty = field?.querySelector("[data-empty]");
            const check = field?.querySelector("[data-check]");
            const lastInField = field?.querySelector("[data-chip]:last-of-type") === chip;
            tl.to(mark, { scaleX: 1, duration: 0.35, ease: "power2.out" }, t + 0.05);
            tl.add(flight({ gsap }, chip, boxOf(phrase)), t + 0.18);
            if (empty) tl.to(empty, { autoAlpha: 0, duration: 0.2 }, t + 0.7);
            if (check && lastInField) tl.to(check, { autoAlpha: 1, scale: 1, duration: 0.45, ease: "back.out(2.4)" }, t + 1.1);
          }

          tl.to(caret, { autoAlpha: 0, duration: 0.12, repeat: 5, yoyo: true, ease: "steps(1)" }, typed + 0.1);
          tl.to(caret, { autoAlpha: 0, duration: 0.2 }, ">");
          if (go) tl.to(go, { autoAlpha: 1, duration: 0.4 }, "<");
          if (goRing) tl.fromTo(goRing, { autoAlpha: 0.7, scale: 1 }, { autoAlpha: 0, scale: 1.35, duration: 1.1, ease: "power2.out", repeat: 1 }, "<");
          tl.add(() => {
            replay.disabled = false;
          });
          const run = tl;
          ScrollTrigger.create({ trigger: el, start: "top 80%", once: true, onEnter: () => void run.play() });
        };

        const onReplay = () => {
          if (!tl) return;
          replay.disabled = true;
          tl.restart(true);
        };
        replay.disabled = true;
        replay.addEventListener("click", onReplay);

        void document.fonts?.ready.then(() => {
          if (!ctx.isReverted) ctx.add(build);
        });

        return () => {
          replay.removeEventListener("click", onReplay);
          copy.remove();
        };
      });
    },
    { scope: root, extra: loadMotionPath },
  );

  return (
    <div ref={root} className="relative overflow-hidden rounded-panel border border-white/10 bg-night-raised/80 shadow-[0_40px_120px_-40px_rgb(64_47_255/0.6)] backdrop-blur">
      <div className="flex items-center justify-between gap-4 border-b border-white/8 px-4 py-3 sm:px-6">
        <p className="font-mono text-xs tracking-[0.14em] text-night-faint uppercase">Новая потребность</p>
        <button
          type="button"
          data-replay
          className="tap hidden items-center gap-1.5 rounded-full px-2.5 py-1 text-xs text-night-muted transition-colors hover:bg-white/5 hover:text-white disabled:pointer-events-none disabled:opacity-0 [html.motion_&]:inline-flex"
        >
          <RotateCcw className="size-3.5" aria-hidden />
          Повторить
        </button>
      </div>

      <div className="relative border-b border-white/8 p-4 sm:p-6">
        <p className="text-[13px] text-night-faint">Текст вакансии, вставлен как есть</p>
        <div className="mt-3 rounded-card border border-white/10 bg-night px-4 py-3.5 sm:px-5 sm:py-4">
          <p data-text className="relative isolate text-[15px] leading-[1.75] text-night-muted sm:text-base">
            {SEGMENTS.map((p, i) =>
              p.chip ? (
                <span key={i} className="whitespace-nowrap">
                  <span data-phrase={p.chip} className="relative text-white">
                    <span data-mark aria-hidden className="absolute -inset-x-0.5 inset-y-[0.14em] -z-10 rounded-[5px] bg-primary/55" />
                    {p.text}
                  </span>
                  {p.tail}
                </span>
              ) : (
                <span key={i}>{p.text}</span>
              ),
            )}
            <span aria-hidden className="ml-0.5 inline-block h-[1.1em] w-[2px] translate-y-[0.18em] bg-primary-300 motion-desk:hidden" />
            <span data-caret aria-hidden className="pointer-events-none absolute top-0 left-0 hidden w-[2px] bg-primary-300 motion-desk:block" />
          </p>
        </div>
        <span aria-hidden className="absolute -bottom-4 left-1/2 z-10 flex size-8 -translate-x-1/2 items-center justify-center rounded-full border border-white/15 bg-night-soft text-primary-300">
          <ArrowDown className="size-4" />
        </span>
      </div>

      <div className="p-4 pt-6 sm:p-6 sm:pt-7">
        <p className="text-[13px] text-night-faint">Что сервис понял из текста</p>
        <dl className="mt-2">
          {FIELDS.map((f) => (
            <div
              key={f.label}
              data-field
              className="grid grid-cols-[1fr_auto] items-center gap-x-3 gap-y-1.5 border-b border-white/8 py-2.5 last:border-b-0 sm:grid-cols-[7.5rem_1fr_auto]"
            >
              <dt className="col-start-1 row-start-1 text-sm text-night-muted">{f.label}</dt>
              <dd className="relative col-span-2 row-start-2 flex min-h-8 flex-wrap items-center gap-1.5 sm:col-span-1 sm:col-start-2 sm:row-start-1">
                <span data-empty aria-hidden className="absolute top-1/2 left-0 h-7 w-24 -translate-y-1/2 rounded-full border border-dashed border-white/15 opacity-0" />
                {f.chips.map((c) => (
                  <span
                    key={c.id}
                    data-chip={c.id}
                    className={cn(
                      "relative inline-flex h-8 items-center gap-1.5 rounded-full px-3 text-sm font-medium whitespace-nowrap will-change-transform",
                      c.optional ? "border border-dashed border-white/35 text-white" : "bg-white text-ink",
                    )}
                  >
                    {c.label}
                    {c.optional && <span className="text-xs font-normal text-night-muted">желательно</span>}
                  </span>
                ))}
              </dd>
              <dd className="col-start-2 row-start-1 flex items-center gap-1.5 sm:col-start-3">
                <span data-check className="flex size-5 items-center justify-center rounded-full bg-success-300 text-night">
                  <Check className="size-3" strokeWidth={3} aria-hidden />
                </span>
                <PenLine className="size-4 text-night-faint" aria-hidden />
                <span className="sr-only">распознано, можно поправить</span>
              </dd>
            </div>
          ))}
        </dl>
        <div className="mt-5 flex flex-wrap items-center justify-between gap-3">
          <p className="text-sm text-night-faint">Любое поле можно поправить вручную</p>
          <a href="#compare" data-go className="relative inline-flex h-10 items-center gap-2 rounded-full bg-primary px-4 text-sm font-medium text-white transition-colors hover:bg-primary-hover">
            <span data-go-ring aria-hidden className="pointer-events-none absolute inset-0 rounded-full border border-primary-300 opacity-0" />
            Показать подборку
            <ArrowDown className="size-4" aria-hidden />
          </a>
        </div>
      </div>
    </div>
  );
}
