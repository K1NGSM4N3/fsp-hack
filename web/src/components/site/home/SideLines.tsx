"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";
import { Container } from "../primitives";

type Side = "seeker" | "company";

const LINES: { side: Side; text: string }[] = [
  { side: "seeker", text: "Зарплата видна до первого созвона." },
  { side: "company", text: "Уровень проверен до первого письма." },
  { side: "seeker", text: "Отказ занимает одно нажатие." },
  { side: "company", text: "Причина отказа приходит сразу." },
  { side: "seeker", text: "Контакты открываете только вы." },
  { side: "company", text: "Приглашения читают, потому что в них есть сумма." },
];

const LABEL: Record<Side, string> = { seeker: "Соискателю", company: "Компании" };

/**
 * Seeker and company lines alternate like a dialogue. Each line is an outline with a filled copy on top;
 * the copy is uncovered by two opposite translations (mask moves in, text moves back), so only transforms animate.
 * Without motion the filled copy is simply in place.
 */
export function SideLines() {
  const root = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        for (const line of gsap.utils.toArray<HTMLElement>("[data-line]", root.current)) {
          const dir = line.dataset.side === "company" ? 1 : -1;
          const mask = line.querySelector("[data-fill]");
          const inner = line.querySelector("[data-fill-inner]");
          const label = line.querySelector("[data-label]");
          const tl = gsap.timeline({ scrollTrigger: { trigger: line, start: "top 85%", end: "top 40%", scrub: 0.6 } });
          tl.fromTo(line, { x: dir * 48 }, { x: 0, ease: "power2.out", duration: 1 }, 0)
            .fromTo(label, { autoAlpha: 0, x: dir * 24 }, { autoAlpha: 1, x: 0, ease: "power2.out", duration: 0.35 }, 0)
            .fromTo(mask, { xPercent: dir * 101 }, { xPercent: 0, ease: "none", duration: 1 }, 0)
            .fromTo(inner, { xPercent: -dir * 101 }, { xPercent: 0, ease: "none", duration: 1 }, 0);
        }
      });
    },
    { scope: root },
  );

  return (
    <section ref={root} aria-labelledby="sides-title" className="relative isolate overflow-hidden bg-night py-24 text-white sm:py-32 lg:py-40">
      <div aria-hidden className="pointer-events-none absolute top-1/3 -left-60 size-[44rem] rounded-full bg-primary/20 blur-[160px]" />
      <div aria-hidden className="pointer-events-none absolute -right-60 bottom-0 size-[36rem] rounded-full bg-accent/10 blur-[160px]" />
      <Container className="relative">
        <div className="flex flex-wrap items-baseline justify-between gap-x-10 gap-y-4 border-b border-night-line pb-6">
          <h2 id="sides-title" className="eyebrow text-night-muted">
            Что получает каждая сторона
          </h2>
          <p className="flex gap-6 font-mono text-xs tracking-[0.12em] uppercase">
            <span className="inline-flex items-center gap-2 text-primary-300">
              <span aria-hidden className="size-1.5 rounded-full bg-primary-300" />
              {LABEL.seeker}
            </span>
            <span className="inline-flex items-center gap-2 text-accent-300">
              <span aria-hidden className="size-1.5 rounded-full bg-accent-300" />
              {LABEL.company}
            </span>
          </p>
        </div>

        <ul className="mt-14 space-y-12 sm:mt-20 sm:space-y-16 lg:space-y-14">
          {LINES.map((l) => {
            const company = l.side === "company";
            return (
              <li key={l.text} data-line data-side={l.side} className={cn("flex flex-col", company && "lg:items-end lg:text-right")}>
                <span data-label className={cn("mb-3 font-mono text-xs tracking-[0.14em] uppercase", company ? "text-accent-300" : "text-primary-300")}>
                  {LABEL[l.side]}
                </span>
                <p className="relative max-w-[15ch] text-[clamp(2.1rem,0.9rem+4.6vw,5rem)] leading-[1.02] font-semibold tracking-[-0.045em] text-balance">
                  <span className="text-transparent [-webkit-text-stroke:1px_var(--color-night-faint)]">{l.text}</span>
                  <span data-fill aria-hidden className="absolute inset-0 overflow-hidden">
                    <span data-fill-inner className="block text-white">
                      {l.text}
                    </span>
                  </span>
                </p>
              </li>
            );
          })}
        </ul>
      </Container>
    </section>
  );
}
