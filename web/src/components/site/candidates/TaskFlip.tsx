"use client";

import { useRef } from "react";
import { MOTION_DESKTOP, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

const TASKS = [
  {
    topic: "Go",
    kind: "один вариант",
    limit: "1:30",
    prompt: "Что напечатает программа?",
    code: "for i := 0; i < 3; i++ {\n    defer fmt.Print(i)\n}",
    options: ["012", "210", "333", "000"],
  },
  {
    topic: "SQL",
    kind: "число",
    limit: "2:30",
    prompt: "Сколько строк вернёт запрос по таблице из задания?",
    code: "SELECT city, COUNT(*)\nFROM orders\nWHERE amount > 700\nGROUP BY city\nHAVING COUNT(*) >= 2;",
    options: [],
  },
  {
    topic: "JavaScript",
    kind: "один вариант",
    limit: "1:30",
    prompt: "В каком порядке появятся буквы?",
    code: "console.log('A');\nsetTimeout(() => console.log('B'));\nPromise.resolve().then(() => console.log('C'));\nconsole.log('D');",
    options: ["A B C D", "A D C B", "A C D B", "A D B C"],
  },
  {
    topic: "Сети",
    kind: "число",
    limit: "1:00",
    prompt: "Сколько адресов можно выдать хостам в подсети 10.0.0.0/27?",
    code: "",
    options: [],
  },
];

const R = 22;
const C = 2 * Math.PI * R;

/** Pinned on desktop: task cards flip away one after another while each card's timer ring drains. */
export function TaskFlip() {
  const root = useRef<HTMLDivElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_DESKTOP, () => {
        const cards = gsap.utils.toArray<HTMLElement>("[data-task]", stage.current);
        const rings = gsap.utils.toArray<SVGCircleElement>("[data-ring]", stage.current);
        const steps = gsap.utils.toArray<HTMLElement>("[data-step]", root.current);
        gsap.set(stage.current, { display: "block", position: "relative", height: 470, perspective: 1400 });
        cards.forEach((card, i) => gsap.set(card, { position: "absolute", inset: 0, zIndex: cards.length - i, transformOrigin: "0% 50%" }));
        const tl = gsap.timeline({
          scrollTrigger: { trigger: root.current, start: "top top+=96", end: `+=${cards.length * 520}`, pin: true, scrub: 0.8 },
        });
        cards.forEach((card, i) => {
          const at = i * 1.2;
          tl.fromTo(rings[i] ?? null, { strokeDashoffset: 0 }, { strokeDashoffset: C, duration: 1, ease: "none" }, at);
          tl.to(steps[i] ?? null, { color: "var(--color-ink)", duration: 0.1 }, at);
          if (i < cards.length - 1) {
            tl.to(card, { rotationY: -100, autoAlpha: 0, x: -60, duration: 0.5, ease: "power2.in" }, at + 0.8);
            tl.fromTo(cards[i + 1] ?? null, { rotationY: 25, scale: 0.94 }, { rotationY: 0, scale: 1, duration: 0.5, ease: "power2.out" }, at + 0.95);
            tl.to(steps[i] ?? null, { color: "var(--color-ink-faint)", duration: 0.1 }, at + 1.1);
          }
        });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="grid grid-cols-1 gap-10 lg:grid-cols-[0.8fr_1.2fr] lg:items-center lg:gap-16">
      <ol className="space-y-5">
        {TASKS.map((t, i) => (
          <li key={t.prompt} data-step className="flex gap-4 text-ink-subtle">
            <span className="font-mono text-sm text-primary">0{i + 1}</span>
            <span>
              <span className="block font-semibold">{t.topic}</span>
              <span className="text-sm">{t.prompt}</span>
            </span>
          </li>
        ))}
        <li className="pt-3 text-sm leading-relaxed text-ink-muted">Так выглядят задания. Числа и код в вашем варианте будут другими: у каждого кандидата свой набор.</li>
      </ol>
      <div ref={stage} className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        {TASKS.map((t, i) => (
          <article key={t.prompt} data-task className="flex flex-col rounded-[24px] border border-line bg-white p-6 shadow-lift">
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">
                  Задание {i + 1} · {t.topic}
                </p>
                <p className="mt-1 text-xs text-ink-faint">ответ: {t.kind}</p>
              </div>
              <span className="relative flex size-14 items-center justify-center" aria-label={`Лимит времени ${t.limit}`}>
                <svg viewBox="0 0 52 52" className="absolute inset-0 -rotate-90" aria-hidden>
                  <circle cx="26" cy="26" r={R} fill="none" strokeWidth="4" className="stroke-primary-50" />
                  <circle data-ring cx="26" cy="26" r={R} fill="none" strokeWidth="4" strokeLinecap="round" strokeDasharray={C} strokeDashoffset={0} className="stroke-primary" />
                </svg>
                <span className="tabular font-mono text-xs font-semibold text-ink">{t.limit}</span>
              </span>
            </div>
            <p className="mt-5 text-lg font-semibold tracking-tight text-ink">{t.prompt}</p>
            {t.code && (
              <pre className="scrollbar-thin mt-4 overflow-x-auto rounded-2xl bg-night p-4 font-mono text-[13px] leading-6 text-night-muted">
                <code>{t.code}</code>
              </pre>
            )}
            {t.options.length > 0 ? (
              <ul className="mt-4 grid grid-cols-2 gap-2">
                {t.options.map((o) => (
                  <li key={o} className={cn("rounded-xl border border-line px-3 py-2 font-mono text-sm text-ink")}>
                    {o}
                  </li>
                ))}
              </ul>
            ) : (
              <p className="mt-4 rounded-xl border border-dashed border-line-strong px-3 py-2.5 font-mono text-sm text-ink-faint">Введите число</p>
            )}
          </article>
        ))}
      </div>
    </div>
  );
}
