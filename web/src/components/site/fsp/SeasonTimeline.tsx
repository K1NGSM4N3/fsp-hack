"use client";

import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";
import { Container } from "@/components/site/primitives";
import { Medal } from "./Medal";

const SEASONS = [
  { label: "Давно", scale: 0.52, opacity: 0.32 },
  { label: "Пару сезонов назад", scale: 0.68, opacity: 0.5 },
  { label: "Прошлый сезон", scale: 0.84, opacity: 0.74 },
  { label: "Этот сезон", scale: 1, opacity: 1 },
];

const AT = (i: number) => 12.5 + i * 25;
/** The medal carries the fade; its caption stops at the dimmest opacity that still reads at 4.5:1 (WCAG AA). */
const NAME_FLOOR = 0.74;
const nameOpacity = (opacity: number) => Math.max(opacity, NAME_FLOOR);

/**
 * The «now» cursor walks from the oldest season to this one while earlier medals shrink and fade.
 * Desktop pins the section and scrubs the walk with the scroll; phones play the same walk once, eased, in under three
 * seconds when the scale comes into view, and the page keeps scrolling normally.
 */
export function SeasonTimeline() {
  const root = useRef<HTMLElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, ScrollTrigger, gsap }) => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add({ desktop: MOTION_DESKTOP, mobile: MOTION_MOBILE }, (ctx) => {
        const { desktop, mobile } = ctx.conditions ?? {};
        if (!desktop && !mobile) return;
        const q = gsap.utils.selector(el);
        const medals = q("[data-season-medal]");
        const names = q("[data-season-name]");
        const labels = q("[data-season-label]");
        const last = SEASONS.length - 1;
        const step = 1 / last;
        const pop = 0.07;

        gsap.from(q("[data-fresh-head] > *"), { autoAlpha: 0, y: desktop ? 30 : 20, duration: desktop ? 1 : 0.7, ease: desktop ? "expo.out" : "power3.out", stagger: 0.1, scrollTrigger: { trigger: el, start: "top 70%", once: true } });

        const tl = gsap.timeline({
          defaults: { ease: "none" },
          paused: !desktop,
          scrollTrigger: desktop ? { trigger: el, pin: true, pinSpacing: true, start: "top top", end: "+=140%", scrub: 0.6, invalidateOnRefresh: true } : undefined,
        });
        if (!desktop) {
          ScrollTrigger.create({
            trigger: q("[data-scale]")[0],
            start: "top 75%",
            once: true,
            onEnter: () => ctx.add(() => gsap.to(tl, { progress: 1, duration: 2.8, ease: "power1.inOut", delay: 0.2 })),
          });
        }
        tl.fromTo(q("[data-cursor]"), { x: 0, xPercent: AT(0) }, { xPercent: AT(last), duration: 1 }, 0)
          .fromTo(q("[data-fill]"), { scaleX: AT(0) / 100 }, { scaleX: AT(last) / 100, duration: 1 }, 0)
          .fromTo(labels, { opacity: 0.35 }, { opacity: 1, duration: pop, stagger: step }, 0);
        SEASONS.forEach((s, i) => {
          const at = i * step;
          if (i > 0) {
            tl.fromTo(medals[i], { scale: 0, autoAlpha: 0, y: -30 }, { scale: 1, autoAlpha: 1, y: 0, duration: pop, ease: "back.out(2)" }, at - pop);
            tl.fromTo(names[i], { autoAlpha: 0 }, { autoAlpha: 1, duration: pop }, at - pop);
          } else {
            tl.set(medals[i], { scale: 1, autoAlpha: 1 }, 0);
            tl.set(names[i], { autoAlpha: 1 }, 0);
          }
          if (i < last) {
            tl.fromTo(medals[i], { scale: 1, opacity: 1 }, { scale: s.scale, opacity: s.opacity, duration: 1 - at, immediateRender: false }, at);
            tl.fromTo(names[i], { opacity: 1 }, { opacity: nameOpacity(s.opacity), duration: 1 - at, immediateRender: false }, at);
          }
        });
        tl.fromTo(q("[data-fresh-glow]"), { scale: 0.4, autoAlpha: 0 }, { scale: 1, autoAlpha: 1, duration: pop, ease: "power2.out" }, 1 - pop / 2);
      });
    },
    { scope: root },
  );

  return (
    <section ref={root} className="relative overflow-hidden bg-white">
      <div className="flex min-h-svh flex-col justify-center py-20 sm:py-24">
        <Container>
          <div data-fresh-head className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_0.85fr] lg:items-end lg:gap-16">
            <div>
              <p className="eyebrow text-primary">Свежесть</p>
              <h2 className="display-2 mt-5 text-balance text-ink">Свежие результаты весят больше давних</h2>
            </div>
            <p className="max-w-xl text-base leading-relaxed text-pretty text-ink-muted sm:text-lg">
              Одна и та же победа поднимает вас сильнее, если она случилась в этом сезоне. Давние результаты остаются в профиле и продолжают учитываться, просто со временем их вклад становится скромнее.
            </p>
          </div>

          <div data-scale className="relative mt-14 h-[320px] sm:mt-20 sm:h-[360px]">
            <div aria-hidden className="absolute inset-x-0 top-[76%] h-[3px] -translate-y-1/2 rounded-full bg-line">
              <span data-fill className="absolute inset-0 origin-left rounded-full bg-primary" style={{ transform: `scaleX(${AT(SEASONS.length - 1) / 100})` }} />
            </div>
            <ol className="absolute inset-0">
              {SEASONS.map((s, i) => (
                <li key={s.label} className="absolute inset-y-0 w-[25%] -translate-x-1/2" style={{ left: `${AT(i)}%` }}>
                  <span data-season-label className="absolute inset-x-0 top-0 px-1 text-center font-mono text-xs leading-snug tracking-[0.04em] text-ink-muted uppercase sm:tracking-[0.1em]">
                    {s.label}
                  </span>
                  <div className="absolute inset-x-0 bottom-[24%] flex flex-col items-center pb-5">
                    {i === SEASONS.length - 1 && <span data-fresh-glow aria-hidden className="absolute bottom-6 size-36 rounded-full bg-primary/15 blur-2xl" />}
                    <span data-season-medal className="relative origin-bottom" style={{ transform: `scale(${s.scale})`, opacity: s.opacity }}>
                      <Medal mark="1" metal="gold" size="xl" />
                    </span>
                    <span data-season-name className="relative mt-3 text-sm font-semibold whitespace-nowrap text-ink" style={{ opacity: nameOpacity(s.opacity) }}>
                      1 место
                    </span>
                  </div>
                </li>
              ))}
            </ol>
            <div data-cursor aria-hidden className="pointer-events-none absolute inset-y-0 left-0 w-full" style={{ transform: `translateX(${AT(SEASONS.length - 1)}%)` }}>
              <span className="absolute top-[76%] bottom-7 left-0 w-px bg-primary" />
              <span className="absolute top-[76%] left-0 size-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-[3px] border-white bg-primary shadow-glow" />
              <span className="absolute bottom-0 left-0 -translate-x-1/2 rounded-full bg-primary px-3 py-1 font-mono text-[10px] tracking-[0.14em] whitespace-nowrap text-white uppercase sm:text-[11px]">
                Сейчас
              </span>
            </div>
          </div>
          <p className="mt-6 text-sm text-ink-subtle">Из разрядов в каждой дисциплине учитывается самый высокий.</p>
        </Container>
      </div>
    </section>
  );
}
