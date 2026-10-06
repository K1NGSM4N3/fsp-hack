"use client";

import { ClipboardPaste } from "lucide-react";
import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";
import { Container, Cta } from "@/components/site/primitives";
import { cn } from "@/lib/cn";

const EXAMPLES = [
  "Ищем Python-разработчика уровня Middle в команду аналитики, гибрид в Казани, 220–280 тыс. ₽ на руки…",
  "Нужен Senior QA для мобильного банка: автотесты на Kotlin, удалённо, вилка 300–360 тыс. ₽…",
  "Frontend на React и TypeScript, грейд Junior, офис в Новосибирске, от 120 тыс. ₽…",
];

const SLOTS = ["Специализация", "Грейд", "Навыки", "Вилка", "Формат"];

/** Closing block: an empty vacancy field with a blinking caret and rotating examples as the placeholder; both loops sleep off screen. */
export function FirstVacancy() {
  const root = useRef<HTMLDivElement>(null);

  useMotion(
    ({ MOTION_OK, ScrollTrigger, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const el = root.current;
        const caret = el?.querySelector("[data-caret]");
        const examples = gsap.utils.toArray<HTMLElement>("[data-example]", el);
        const loops: gsap.core.Animation[] = [];
        if (caret) loops.push(gsap.to(caret, { opacity: 0, duration: 0.55, repeat: -1, yoyo: true, ease: "steps(1)", paused: true }));
        if (examples.length > 1) {
          gsap.set(examples.slice(1), { autoAlpha: 0, y: 24 });
          const tl = gsap.timeline({ repeat: -1, paused: true });
          examples.forEach((ex, i) => {
            const next = examples[(i + 1) % examples.length];
            tl.to(ex, { autoAlpha: 0, y: -24, duration: 0.6, ease: "power2.in" }, "+=3.2");
            tl.fromTo(next ?? ex, { autoAlpha: 0, y: 24 }, { autoAlpha: 1, y: 0, duration: 0.8, ease: "expo.out" }, "-=0.15");
          });
          loops.push(tl);
        }
        ScrollTrigger.create({ trigger: el, start: "top bottom", end: "bottom top", onToggle: (self) => loops.forEach((a) => void (self.isActive ? a.play() : a.pause())) });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root}>
      <Container className="relative">
        <div className="mx-auto max-w-3xl text-center">
          <h2 data-split className="display-1 text-balance text-white">
            Начните <span className="whitespace-nowrap">с одной</span> вакансии
          </h2>
          <p data-reveal className="mx-auto mt-6 max-w-xl text-lg leading-relaxed text-primary-100 sm:text-xl">
            Зарегистрируйте компанию и вставьте текст любой открытой вакансии. Подборку вы увидите сразу после разбора текста.
          </p>
        </div>

        <div data-reveal="scale" className="mx-auto mt-12 max-w-4xl sm:mt-16">
          <div data-tilt="3" aria-hidden className="rounded-panel bg-white p-2 text-ink shadow-[0_50px_100px_-40px_rgb(11_11_18/0.6)]">
            <div className="flex items-center justify-between gap-3 px-4 pt-3 pb-2 sm:px-5">
              <span className="text-[13px] text-ink-subtle">Текст вакансии</span>
              <span className="inline-flex items-center gap-1.5 rounded-full bg-canvas px-2.5 py-1 text-xs text-ink-muted">
                <ClipboardPaste className="size-3.5" />
                можно вставить как есть
              </span>
            </div>
            <div className="rounded-[22px] border border-line bg-canvas px-4 py-5 sm:px-6 sm:py-7">
              <div className="flex min-h-[7.5rem] items-start gap-1 sm:min-h-[8.5rem]">
                <span data-caret className="mt-[0.2em] h-[1.15em] w-[3px] shrink-0 rounded-full bg-primary text-xl sm:text-2xl lg:text-[1.75rem]" />
                <div className="grid flex-1 [grid-template-areas:'e'] *:[grid-area:e]">
                  {EXAMPLES.map((e, i) => (
                    <p key={e} data-example className={cn("text-xl leading-snug tracking-tight text-ink-faint sm:text-2xl lg:text-[1.75rem]", i > 0 && "opacity-0")}>
                      {e}
                    </p>
                  ))}
                </div>
              </div>
            </div>
            <div className="flex flex-col gap-3 px-3 pt-3 pb-2 sm:flex-row sm:items-center sm:justify-between">
              <div className="flex flex-wrap gap-1.5">
                {SLOTS.map((s) => (
                  <span key={s} className="rounded-full border border-dashed border-line-strong px-3 py-1 text-xs text-ink-faint">
                    {s}
                  </span>
                ))}
              </div>
              <span className="inline-flex h-10 shrink-0 items-center justify-center rounded-full bg-primary-100 px-5 text-sm font-medium text-primary">Разобрать текст</span>
            </div>
          </div>
        </div>

        <div data-reveal className="mt-12 flex flex-col items-center justify-center gap-3 sm:flex-row">
          <Cta href="/register?role=employer" variant="light">
            Зарегистрировать компанию
          </Cta>
          <Cta href="/login" variant="outline-night">
            Войти
          </Cta>
        </div>
      </Container>
    </div>
  );
}
