"use client";

import { ArrowDown, ArrowRight } from "lucide-react";
import Link from "next/link";
import { useRef } from "react";
import { MOTION_OK, ScrollTrigger, SplitText, gsap, useGSAP } from "@/components/motion/gsap";
import { Container } from "@/components/site/primitives";
import { FlapBoard } from "./FlapBoard";

export function FspHero() {
  const root = useRef<HTMLElement>(null);

  useGSAP(
    () => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(el);
        const [title] = q("[data-hero-title]");
        const rest = q("[data-hero-fade]");
        const lanes = q("[data-lane]");
        gsap.set(title, { visibility: "visible" });
        SplitText.create(title, {
          type: "words",
          wordsClass: "split-word",
          autoSplit: true,
          onSplit: (self) =>
            gsap.from(self.words, {
              rotationX: -95,
              yPercent: 40,
              autoAlpha: 0,
              transformOrigin: "50% 0%",
              duration: 1.1,
              ease: "back.out(1.4)",
              stagger: 0.07,
              delay: 0.15,
            }),
        });
        gsap.fromTo(rest, { autoAlpha: 0, y: 24 }, { autoAlpha: 1, y: 0, duration: 1, ease: "expo.out", stagger: 0.1, delay: 0.55 });
        gsap.from(lanes, { drawSVG: "50% 50%", duration: 2.4, ease: "power3.inOut", stagger: 0.12, delay: 0.2 });
        gsap.to(q("[data-lanes]"), { yPercent: 18, ease: "none", scrollTrigger: { trigger: el, start: "top top", end: "bottom top", scrub: true } });
      });
      void document.fonts?.ready.then(() => ScrollTrigger.refresh());
    },
    { scope: root },
  );

  return (
    <section ref={root} className="relative isolate overflow-hidden bg-night pt-28 pb-16 text-white sm:pt-36 sm:pb-20 lg:pb-28">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="absolute -top-48 right-[-10%] size-[46rem] rounded-full bg-primary/35 blur-[160px]" />
        <svg data-lanes viewBox="0 0 1200 640" fill="none" preserveAspectRatio="xMidYMid slice" className="absolute inset-0 size-full text-primary-300/25">
          {[0, 1, 2, 3, 4].map((i) => {
            const inset = i * 34;
            return (
              <rect
                key={i}
                data-lane
                x={520 + inset}
                y={-140 + inset}
                width={1080 - inset * 2}
                height={620 - inset * 2}
                rx={310 - inset}
                stroke="currentColor"
                strokeWidth={i === 0 ? 1.5 : 1}
              />
            );
          })}
        </svg>
      </div>

      <Container>
        <p data-hero data-hero-fade className="eyebrow inline-flex items-center gap-2.5 text-primary-300">
          <span aria-hidden className="size-2 rounded-full bg-accent" />
          Участникам ФСП
        </p>
        <h1 data-hero data-hero-title className="display-1 mt-6 max-w-[15ch] text-balance perspective-[900px] sm:max-w-[18ch] lg:max-w-none">
          Результаты с&nbsp;соревнований теперь видят <span className="text-gradient-night">компании</span>
        </h1>
        <div className="mt-8 grid grid-cols-1 gap-8 lg:mt-10 lg:grid-cols-[1fr_auto] lg:items-end lg:gap-16">
          <p data-hero data-hero-fade className="max-w-2xl text-lg leading-relaxed text-pretty text-night-muted sm:text-xl">
            Привяжите ФСП ID к профилю, и ваши разряды и призовые места из реестра Федерации появятся в карточке. Рядом с каждым будет ссылка на официальный протокол.
          </p>
          <div data-hero data-hero-fade className="flex flex-col gap-3 sm:flex-row">
            <Link
              href="/register?role=candidate"
              className="group inline-flex h-14 items-center justify-center gap-2.5 rounded-full bg-primary px-7 font-medium text-white shadow-[0_14px_40px_-12px_rgb(64_47_255/0.9)] transition-colors hover:bg-primary-hover"
            >
              Создать профиль
              <ArrowRight className="size-4.5 transition-transform group-hover:translate-x-1" aria-hidden />
            </Link>
            <a
              href="#link"
              className="group inline-flex h-14 items-center justify-center gap-2.5 rounded-full border border-white/20 px-7 font-medium text-white transition-colors hover:border-white hover:bg-white/5"
            >
              Как привязать
              <ArrowDown className="size-4.5 transition-transform group-hover:translate-y-0.5" aria-hidden />
            </a>
          </div>
        </div>
        <div className="mt-12 sm:mt-16">
          <FlapBoard />
        </div>
      </Container>
    </section>
  );
}
