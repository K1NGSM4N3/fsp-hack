"use client";

import { ArrowRight } from "lucide-react";
import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";
import { Emblem, type DisciplineKey } from "./Emblems";

const DISCIPLINES: { key: DisciplineKey; name: string; text: string; strong: string[]; also: string[] }[] = [
  {
    key: "algo",
    name: "Алгоритмическое программирование",
    text: "Олимпиадные задачи на время. Нужно придумать алгоритм и реализовать его без единой ошибки.",
    strong: ["Бэкенд-разработка", "ML / Data Science"],
    also: ["Фронтенд-разработка", "Информационная безопасность"],
  },
  {
    key: "product",
    name: "Продуктовое программирование",
    text: "Командный хакатон: за ограниченное время собрать работающий продукт под задачу заказчика.",
    strong: ["Бэкенд-разработка", "Фронтенд-разработка"],
    also: ["DevOps / SRE", "Тестирование (QA)"],
  },
  {
    key: "security",
    name: "Программирование систем ИБ",
    text: "Соревнования по информационной безопасности: найти уязвимость у соперника и удержать свой сервис.",
    strong: ["Информационная безопасность"],
    also: ["DevOps / SRE", "Бэкенд-разработка"],
  },
  {
    key: "ai",
    name: "Программирование систем ИИ",
    text: "Задачи машинного обучения, где модель проверяют на данных, которых вы не видели.",
    strong: ["ML / Data Science"],
    also: ["Бэкенд-разработка"],
  },
  {
    key: "robotics",
    name: "Программирование робототехники",
    text: "Код для настоящего робота, которому нужно пройти трассу или выполнить задание.",
    strong: ["ML / Data Science"],
    also: ["Бэкенд-разработка", "DevOps / SRE", "Тестирование (QA)"],
  },
  {
    key: "uav",
    name: "Программирование БАС",
    text: "Беспилотные авиационные системы: автономный полёт и навигация по бортовым датчикам.",
    strong: ["ML / Data Science"],
    also: ["Тестирование (QA)", "Информационная безопасность"],
  },
];

const EDGE = "pl-4 sm:pl-6 lg:pl-[max(2rem,calc((100vw-1240px)/2+2rem))]";

/**
 * Desktop: the section pins and vertical scroll drives the row sideways; emblems draw as panels arrive.
 * Phones and tablets: the row stays a native snap-scrolling strip you swipe with a finger, the progress line follows
 * the swipe, and each emblem draws once when its panel is actually visible (an IntersectionObserver sees the
 * horizontal clipping too).
 */
export function DisciplineTrack() {
  const root = useRef<HTMLElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, gsap }) => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add(MOTION_MOBILE, () => {
        const q = gsap.utils.selector(el);
        const [viewport] = q("[data-viewport]");
        const [fill] = q("[data-progress]");
        if (!viewport) return;
        gsap.from(q("[data-intro] > *"), { autoAlpha: 0, y: 20, duration: 0.7, ease: "power3.out", stagger: 0.08, scrollTrigger: { trigger: el, start: "top 75%", once: true } });

        const draws = new Map<Element, gsap.core.Timeline>();
        for (const panel of q("[data-panel]")) {
          const shapes = panel.querySelectorAll("[data-emblem] path, [data-emblem] circle, [data-emblem] rect");
          const tl = gsap
            .timeline({ paused: true })
            .from(shapes, { drawSVG: 0, duration: 1.1, ease: "power2.inOut", stagger: 0.05 })
            .from(panel.querySelectorAll("[data-panel-copy]"), { autoAlpha: 0, y: 16, duration: 0.6, ease: "power3.out", stagger: 0.06 }, 0.2);
          draws.set(panel, tl);
        }
        const io = new IntersectionObserver(
          (entries) => {
            for (const entry of entries) {
              if (!entry.isIntersecting) continue;
              draws.get(entry.target)?.play();
              io.unobserve(entry.target);
            }
          },
          { threshold: 0.35 },
        );
        draws.forEach((_, panel) => io.observe(panel));

        const setFill = fill ? gsap.quickSetter(fill, "scaleX") : null;
        const onScroll = () => {
          const max = viewport.scrollWidth - viewport.clientWidth;
          setFill?.(max > 0 ? viewport.scrollLeft / max : 0);
        };
        viewport.addEventListener("scroll", onScroll, { passive: true });
        return () => {
          io.disconnect();
          viewport.removeEventListener("scroll", onScroll);
          if (fill) gsap.set(fill, { clearProps: "transform,scale" });
        };
      });
      mm.add(MOTION_DESKTOP, () => {
        const q = gsap.utils.selector(el);
        const [viewport] = q("[data-viewport]");
        const [track] = q("[data-track]");
        const [fill] = q("[data-progress]");
        if (!viewport || !track) return;
        viewport.scrollLeft = 0;
        gsap.set(viewport, { overflowX: "hidden", scrollSnapType: "none" });
        const distance = () => Math.max(0, track.scrollWidth - viewport.clientWidth);

        const tween = gsap.to(track, {
          x: () => -distance(),
          ease: "none",
          scrollTrigger: {
            trigger: el,
            pin: true,
            pinSpacing: true,
            start: "top top",
            end: () => `+=${distance()}`,
            scrub: 0.7,
            invalidateOnRefresh: true,
          },
        });
        if (fill) gsap.fromTo(fill, { scaleX: 0 }, { scaleX: 1, ease: "none", scrollTrigger: { trigger: el, start: "top top", end: () => `+=${distance()}`, scrub: true, invalidateOnRefresh: true } });

        gsap.from(q("[data-intro] > *"), { autoAlpha: 0, y: 40, duration: 1, ease: "expo.out", stagger: 0.08, scrollTrigger: { trigger: el, start: "top 70%", once: true } });

        for (const panel of q("[data-panel]")) {
          const emblem = panel.querySelector("[data-emblem]");
          const copy = panel.querySelectorAll("[data-panel-copy]");
          if (!emblem) continue;
          const shapes = emblem.querySelectorAll("path, circle, rect");
          const visibleAtStart = panel.offsetLeft < viewport.clientWidth * 0.85;
          const trigger: ScrollTrigger.Vars = visibleAtStart
            ? { trigger: el, start: "top 55%", once: true }
            : { trigger: panel, containerAnimation: tween, start: "left 82%", once: true };
          gsap
            .timeline({ scrollTrigger: trigger })
            .from(shapes, { drawSVG: 0, duration: 1.3, ease: "power2.inOut", stagger: 0.06 })
            .from(copy, { autoAlpha: 0, y: 24, duration: 0.8, ease: "expo.out", stagger: 0.07 }, 0.25);
          gsap.fromTo(emblem, { x: 36 }, { x: -36, ease: "none", scrollTrigger: { trigger: panel, containerAnimation: tween, start: "left right", end: "right left", scrub: true } });
        }
      });
    },
    { scope: root },
  );

  return (
    <section ref={root} aria-labelledby="disciplines-title" className="relative isolate overflow-hidden bg-night text-white">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10 bg-grid-night mask-fade-y" />
      <div className="flex min-h-svh flex-col justify-center py-8 sm:py-10">
        <div
          data-viewport
          tabIndex={0}
          role="region"
          aria-label="Дисциплины ФСП, листаются вбок"
          className="scrollbar-thin snap-x snap-mandatory overflow-x-auto overscroll-x-contain py-12 focus-visible:outline-offset-[-4px] sm:py-14"
        >
          <ol data-track className="flex w-max items-stretch pr-4 sm:pr-6 lg:pr-8">
            {/* On desktop the gutter in EDGE grows with the viewport; content-box sizing adds it to the text column instead of eating it. */}
            <li data-intro className={`flex w-[88vw] max-w-[620px] shrink-0 snap-start flex-col justify-center pr-8 sm:pr-14 lg:box-content lg:w-[32rem] lg:max-w-none ${EDGE}`}>
              <p className="eyebrow text-primary-300">Дисциплины</p>
              <h2 id="disciplines-title" className="display-3 mt-5 text-balance lg:text-[3.1rem]">
                Дисциплины ФСП и&nbsp;профессии, которым они ближе
              </h2>
              <p className="mt-6 text-base leading-relaxed text-pretty text-night-muted sm:text-lg">
                Результат в любой дисциплине идёт в плюс для любой специализации. Сильнее всего он помогает, когда дисциплина близка к работе: победа в программировании систем ИБ весомее для вакансии в безопасности, чем для фронтенда.
              </p>
              <p className="mt-8 inline-flex items-center gap-2 font-mono text-xs tracking-[0.14em] text-night-faint uppercase">
                Листайте
                <ArrowRight className="size-4" aria-hidden />
              </p>
            </li>
            {DISCIPLINES.map((d) => (
              <li key={d.key} data-panel className="flex w-[82vw] max-w-[420px] shrink-0 snap-start flex-col border-l border-night-line px-6 sm:px-9 lg:max-w-[400px] xl:max-w-[430px]">
                <div className="relative self-start">
                  <span aria-hidden className="absolute inset-[-30%] rounded-full bg-primary/25 blur-3xl" />
                  <Emblem kind={d.key} className="relative size-24 text-white sm:size-32 lg:size-40" />
                </div>
                <h3 data-panel-copy className="mt-8 text-2xl leading-tight font-semibold tracking-[-0.02em] text-balance sm:mt-10 sm:text-[1.7rem]">
                  {d.name}
                </h3>
                <p data-panel-copy className="mt-4 text-[15px] leading-relaxed text-night-muted">
                  {d.text}
                </p>
                <div data-panel-copy className="mt-auto pt-8">
                  <p className="font-mono text-xs tracking-[0.14em] text-primary-300 uppercase">Сильнее всего помогает</p>
                  <ul className="mt-3 space-y-1.5">
                    {d.strong.map((s) => (
                      <li key={s} className="flex items-center gap-2.5 text-base font-medium text-white">
                        <span aria-hidden className="size-1.5 rounded-full bg-accent-300" />
                        {s}
                      </li>
                    ))}
                  </ul>
                  <p className="mt-5 font-mono text-xs tracking-[0.14em] text-night-faint uppercase">Тоже идёт в плюс</p>
                  <p className="mt-2 text-sm leading-relaxed text-night-muted">{d.also.join(", ")}</p>
                </div>
              </li>
            ))}
          </ol>
        </div>
        <div className={`mt-2 pr-4 sm:pr-6 lg:pr-[max(2rem,calc((100vw-1240px)/2+2rem))] ${EDGE}`}>
          <div aria-hidden className="relative h-px bg-night-line">
            <span data-progress className="absolute inset-0 origin-left scale-x-0 bg-primary-300" />
          </div>
        </div>
      </div>
    </section>
  );
}
