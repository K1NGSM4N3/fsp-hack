"use client";

import { ArrowRight, EyeOff } from "lucide-react";
import Link from "next/link";
import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";
import { Container } from "@/components/site/primitives";

const LANES = ["Алгоритмы", "Продукт", "ИБ", "ИИ", "Роботы", "БАС"];
const COUNT = ["3", "2", "1"];

export function StartLine() {
  const root = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(el);
        const [count] = q("[data-count]");
        const reds = q("[data-red]");
        const greens = q("[data-green]");
        const lanes = q("[data-lane-line]");
        const tags = q("[data-lane-tag]");
        const items = q("[data-go] > *");
        gsap.set(greens, { autoAlpha: 0, scale: 0.5 });
        gsap.set(items, { autoAlpha: 0, y: 50 });
        gsap.set(lanes, { scaleX: 0 });
        gsap.set(tags, { autoAlpha: 0, x: -16 });

        const tl = gsap.timeline({ scrollTrigger: { trigger: q("[data-lights]")[0], start: "top 70%", once: true } });
        COUNT.forEach((n, i) => {
          const at = i * 0.72;
          tl.call(() => void (count.textContent = n), undefined, at)
            .fromTo(count, { scale: 1.9, autoAlpha: 0 }, { scale: 1, autoAlpha: 1, duration: 0.34, ease: "expo.out" }, at)
            .fromTo(reds[i], { autoAlpha: 0, scale: 0.4 }, { autoAlpha: 1, scale: 1, duration: 0.3, ease: "back.out(3)" }, at)
            .to(count, { scale: 0.65, autoAlpha: 0, duration: 0.22, ease: "power2.in" }, at + 0.48);
        });
        const go = COUNT.length * 0.72;
        tl.to(reds, { autoAlpha: 0, duration: 0.12 }, go)
          .to(greens, { autoAlpha: 1, scale: 1, duration: 0.4, ease: "back.out(3)" }, go)
          .to(lanes, { scaleX: 1, duration: 1.3, ease: "expo.out", stagger: 0.05 }, go)
          .to(tags, { autoAlpha: 1, x: 0, duration: 0.8, ease: "expo.out", stagger: 0.05 }, go + 0.2)
          .to(items, { autoAlpha: 1, y: 0, duration: 1.1, ease: "expo.out", stagger: 0.1 }, go + 0.05);

        return () => {
          count.textContent = COUNT[0];
        };
      });
    },
    { scope: root },
  );

  return (
    <section ref={root} aria-labelledby="start-title" className="relative isolate overflow-hidden bg-night text-white">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="absolute -bottom-64 left-1/2 size-[52rem] -translate-x-1/2 rounded-full bg-primary/30 blur-[170px]" />
        <span className="absolute top-1/2 left-[8%] hidden -translate-x-[calc(100%+0.75rem)] -translate-y-1/2 font-mono text-[11px] tracking-[0.4em] text-white/30 uppercase [writing-mode:vertical-rl] rotate-180 lg:block">
          Старт
        </span>
        <div className="absolute inset-y-0 left-0 w-2 bg-[conic-gradient(var(--color-white)_25%,transparent_0_50%,var(--color-white)_0_75%,transparent_0)] bg-size-[8px_8px] opacity-25 sm:left-3 sm:w-3 sm:bg-size-[12px_12px] lg:left-[8%] lg:w-4 lg:bg-size-[16px_16px]" />
        <div className="absolute inset-0 flex flex-col justify-evenly">
          {LANES.map((lane) => (
            <div key={lane} className="relative h-px">
              <span data-lane-line className="absolute inset-0 origin-left bg-white/[0.06] sm:bg-white/10" />
              <span data-lane-tag className="absolute -top-6 left-[calc(8%+2.25rem)] hidden font-mono text-[11px] tracking-[0.16em] text-white/25 uppercase lg:block">
                {lane}
              </span>
            </div>
          ))}
        </div>
      </div>

      <Container className="relative flex flex-col items-center py-24 text-center sm:py-32 lg:py-40">
        <div data-lights aria-hidden className="inline-flex items-center gap-3 rounded-full border border-night-line bg-night-raised px-4 py-3 shadow-[0_20px_40px_-20px_rgb(0_0_0/0.8)] sm:gap-4 sm:px-5">
          {COUNT.map((n) => (
            <span key={n} className="relative block size-5 rounded-full bg-night-soft sm:size-6">
              <span data-red className="absolute inset-0 rounded-full bg-accent opacity-0 shadow-[0_0_18px_4px_rgb(236_29_53/0.6)]" />
              <span data-green className="absolute inset-0 rounded-full bg-success-300 shadow-[0_0_18px_4px_rgb(95_211_155/0.55)]" />
            </span>
          ))}
        </div>

        <div className="relative mt-10 w-full sm:mt-12">
          <span
            data-count
            aria-hidden
            className="pointer-events-none absolute inset-x-0 top-0 font-display text-[9rem] leading-none font-semibold tracking-[-0.06em] text-white opacity-0 sm:text-[13rem]"
          >
            {COUNT[0]}
          </span>
          <div data-go className="flex flex-col items-center">
            <h2 id="start-title" className="display-1 text-balance">
              Старт за&nbsp;вами
            </h2>
            <p className="mt-6 max-w-xl text-lg leading-relaxed text-pretty text-night-muted sm:text-xl">
              Создайте профиль и пройдите тест. Потом нажмите «Привязать ФСП ID» в настройках, и ваши результаты появятся в карточке.
            </p>
            <Link
              href="/register?role=candidate"
              className="group mt-10 inline-flex min-h-16 max-w-full items-center justify-center gap-3 rounded-full bg-primary px-7 py-4 text-lg font-medium text-white shadow-[0_20px_50px_-14px_rgb(64_47_255/0.95)] transition-colors hover:bg-primary-hover sm:min-h-20 sm:px-10 sm:text-xl"
            >
              <span className="text-balance">Создать профиль и привязать ФСП&nbsp;ID</span>
              <ArrowRight className="size-5 shrink-0 transition-transform group-hover:translate-x-1" aria-hidden />
            </Link>
            <p className="mt-8 inline-flex max-w-md items-start gap-2.5 text-left text-sm leading-relaxed text-night-faint sm:items-center">
              <EyeOff className="mt-0.5 size-4 shrink-0 sm:mt-0" aria-hidden />
              Достижения можно скрыть от компаний или отвязать ФСП ID в настройках, в любой момент.
            </p>
          </div>
        </div>
      </Container>
    </section>
  );
}
