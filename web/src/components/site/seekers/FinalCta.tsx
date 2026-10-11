"use client";

import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";
import { Cta, TextLink } from "@/components/site/primitives";

/**
 * The last, still empty box of the checklist. With a mouse it ticks while the pointer or focus is on «Создать профиль»;
 * on touch screens there is no hover, so it ticks by itself right after the box lands.
 */
export function FinalCta({ href }: { href: string }) {
  const root = useRef<HTMLDivElement>(null);
  const tick = useRef<gsap.core.Timeline | null>(null);

  useMotion(
    ({ FINE_POINTER, MOTION_OK, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        const check = gsap
          .timeline({ paused: true })
          .fromTo(q("[data-fill]"), { scale: 0.3, autoAlpha: 0, transformOrigin: "50% 50%" }, { scale: 1, autoAlpha: 1, duration: 0.35, ease: "back.out(2)" })
          .fromTo(q("[data-check]"), { drawSVG: "0%", opacity: 0 }, { drawSVG: "100%", opacity: 1, duration: 0.45, ease: "power2.out" }, 0.15);
        tick.current = check;
        gsap.from(q("[data-box]"), {
          rotation: -24,
          scale: 0.4,
          autoAlpha: 0,
          duration: 1.1,
          ease: "back.out(2)",
          scrollTrigger: { trigger: root.current, start: "top 80%", once: true },
          onComplete: window.matchMedia(FINE_POINTER).matches ? undefined : () => void check.play(),
        });
        return () => {
          tick.current = null;
        };
      });
    },
    { scope: root },
  );

  const on = () => void tick.current?.play();
  const off = () => void tick.current?.reverse();

  return (
    <div ref={root}>
      <div className="flex items-start gap-4 sm:gap-6">
        <svg data-box viewBox="0 0 40 40" className="mt-1 size-11 shrink-0 sm:mt-2 sm:size-16 lg:size-[4.5rem]" aria-hidden>
          <rect x="1.5" y="1.5" width="37" height="37" rx="11" fill="none" strokeWidth="1.6" className="stroke-white/60" />
          <rect data-fill x="1.5" y="1.5" width="37" height="37" rx="11" className="fill-white" style={{ opacity: 0 }} />
          <path data-check d="M12 20.5 L17.5 26 L28.5 14" fill="none" strokeWidth="3.2" strokeLinecap="round" strokeLinejoin="round" className="stroke-primary" style={{ opacity: 0 }} />
        </svg>
        <h2 data-split className="display-2 text-balance text-white">
          Осталось создать профиль
        </h2>
      </div>
      <div className="mt-8 grid grid-cols-1 gap-10 border-t border-white/20 pt-8 lg:mt-12 lg:grid-cols-[1fr_auto] lg:items-center lg:gap-16 lg:pt-10">
        <p data-reveal className="max-w-xl text-lg leading-relaxed text-white/80 sm:text-xl">
          Регистрация по почте и короткая анкета. Тест можно пройти сразу или вернуться к нему позже из личного кабинета.
        </p>
        <div data-reveal className="flex flex-wrap items-center gap-x-7 gap-y-5">
          <span onPointerEnter={on} onPointerLeave={off} onFocus={on} onBlur={off} className="inline-flex">
            <Cta href={href} variant="light">
              Создать профиль
            </Cta>
          </span>
          <TextLink href="/login" tone="night">
            Уже есть профиль
          </TextLink>
        </div>
      </div>
    </div>
  );
}
