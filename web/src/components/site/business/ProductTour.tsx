"use client";

import { useRef, type ReactNode } from "react";
import { useMotion } from "@/components/motion/lazy";
import { useLenis } from "@/components/motion/SmoothScroll";
import { Container, Eyebrow } from "@/components/site/primitives";
import { cn } from "@/lib/cn";

const STEPS = [
  {
    title: "Рынок до первого письма",
    text: "Сервис покажет подходящие категории: сколько там специалистов и на какую зарплату они рассчитывают. Если ваша вилка ниже рынка, это видно до начала поиска.",
  },
  {
    title: "Подборка с причинами",
    text: "У каждого специалиста написано, почему он в подборке и чего ему не хватает. Навыки помечены: подтверждён тестом, указан в резюме или только заявлен.",
  },
  {
    title: "Приглашение с вилкой",
    text: "Вы пишете первым и сразу называете зарплату от и до. Специалист видит условия до разговора, поэтому расхождения выясняются в первый же день.",
  },
  {
    title: "Ответ и контакты",
    text: "Если специалист согласен, вы получаете имя и контакты. Если нет, он выбирает причину отказа, и в кабинете видно, что мешает чаще всего.",
  },
];

const HEAD = {
  eyebrow: "Кабинет компании",
  title: "Четыре экрана от задачи до контакта",
};

function Laptop({ children }: { children: ReactNode }) {
  return (
    <div className="relative">
      <div data-lid className="relative origin-bottom rounded-t-[20px] rounded-b-[6px] border border-night-line bg-night p-[1.5%] pb-[2.2%] shadow-[0_60px_120px_-50px_rgb(11_11_18/0.55)]">
        <span aria-hidden className="absolute top-[0.55%] left-1/2 size-1.5 -translate-x-1/2 rounded-full bg-night-soft" />
        <div className="relative aspect-[16/10] overflow-hidden rounded-[6px] bg-canvas">{children}</div>
      </div>
      <div aria-hidden className="relative mx-[-4%] h-3.5 rounded-t-[2px] rounded-b-[16px] bg-linear-to-b from-line to-line-strong">
        <span className="absolute top-0 left-1/2 h-1.5 w-[14%] -translate-x-1/2 rounded-b-md bg-line-strong" />
      </div>
    </div>
  );
}

/**
 * Desktop: pinned, the laptop screen changes as you scroll. Phones and tablets: the four steps stack, and each screen
 * fills in once as it enters (its cards rise one after another). Without motion the steps simply stack.
 * The four screens are server components passed in as props, so their markup ships no JS.
 */
export function ProductTour({ screens: shots }: { screens: ReactNode[] }) {
  const root = useRef<HTMLDivElement>(null);
  const getLenis = useLenis();
  const trigger = useRef<ScrollTrigger | null>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_MOBILE, ScrollTrigger, gsap }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_MOBILE, () => {
        for (const shot of gsap.utils.toArray<HTMLElement>("[data-stack] [data-shot]", root.current)) {
          gsap.fromTo(
            shot.querySelectorAll("[data-pop]"),
            { autoAlpha: 0, y: 12 },
            { autoAlpha: 1, y: 0, duration: 0.55, ease: "power3.out", stagger: 0.07, delay: 0.25, scrollTrigger: { trigger: shot, start: "top 80%", once: true } },
          );
        }
      });
      mm.add(MOTION_DESKTOP, () => {
        const wrap = root.current?.querySelector<HTMLElement>("[data-pin-wrap]");
        const pin = wrap?.querySelector<HTMLElement>("[data-pin]");
        if (!wrap || !pin) return;
        const screens = gsap.utils.toArray<HTMLElement>("[data-screen]", wrap);
        const captions = gsap.utils.toArray<HTMLElement>("[data-caption]", wrap);
        const steps = gsap.utils.toArray<HTMLElement>("[data-step]", wrap);
        const fill = wrap.querySelector("[data-fill]");
        const lid = wrap.querySelector("[data-lid]");
        let current = 0;

        const reveal = (screen: HTMLElement | undefined) => {
          if (!screen) return;
          gsap.fromTo(screen.querySelectorAll("[data-pop]"), { autoAlpha: 0, y: 14 }, { autoAlpha: 1, y: 0, duration: 0.7, ease: "expo.out", stagger: 0.07, delay: 0.18, overwrite: true });
          for (const t of screen.querySelectorAll<HTMLElement>("[data-reveal-text]")) {
            const text = t.dataset.revealText ?? "";
            t.textContent = "•".repeat(text.length);
            gsap.to(t, { duration: 1.1, delay: 0.5, scrambleText: { text, chars: "абвгдеклмнопрстxyz0123456789@._", speed: 0.5 }, overwrite: true });
          }
        };

        const show = (i: number) => {
          if (i === current) return;
          const dir = i > current ? 1 : -1;
          const prevScreen = screens[current];
          const prevCaption = captions[current];
          current = i;
          if (prevScreen) gsap.to(prevScreen, { autoAlpha: 0, y: -20 * dir, scale: 0.985, duration: 0.4, ease: "power2.in", overwrite: true });
          if (prevCaption) gsap.to(prevCaption, { autoAlpha: 0, y: -12 * dir, duration: 0.3, ease: "power2.in", overwrite: true });
          const nextScreen = screens[i];
          if (nextScreen) gsap.fromTo(nextScreen, { autoAlpha: 0, y: 28 * dir, scale: 0.985 }, { autoAlpha: 1, y: 0, scale: 1, duration: 0.75, ease: "expo.out", delay: 0.1, overwrite: true });
          const nextCaption = captions[i];
          if (nextCaption) gsap.fromTo(nextCaption, { autoAlpha: 0, y: 16 * dir }, { autoAlpha: 1, y: 0, duration: 0.6, ease: "expo.out", delay: 0.12, overwrite: true });
          steps.forEach((s, k) => s.toggleAttribute("data-active", k === i));
          reveal(nextScreen);
        };

        gsap.set(screens.slice(1), { autoAlpha: 0 });
        gsap.set(captions.slice(1), { autoAlpha: 0 });

        const length = () => `+=${Math.round(window.innerHeight * 3)}`;
        trigger.current = ScrollTrigger.create({
          trigger: wrap,
          pin,
          start: "top top",
          end: length,
          anticipatePin: 1,
          onUpdate: (self) => show(Math.min(STEPS.length - 1, Math.floor(self.progress * STEPS.length))),
        });
        if (fill) gsap.fromTo(fill, { scaleY: 0 }, { scaleY: 1, ease: "none", scrollTrigger: { trigger: wrap, start: "top top", end: length, scrub: true } });
        if (lid) gsap.fromTo(lid, { rotationX: 24, y: 40, transformPerspective: 1600 }, { rotationX: 0, y: 0, ease: "power2.out", scrollTrigger: { trigger: wrap, start: "top 85%", end: "top top", scrub: 0.6 } });

        return () => {
          trigger.current = null;
          for (const t of wrap.querySelectorAll<HTMLElement>("[data-reveal-text]")) t.textContent = t.dataset.revealText ?? "";
        };
      });
    },
    { scope: root },
  );

  const jump = (i: number) => {
    const st = trigger.current;
    if (!st) return;
    const y = st.start + ((i + 0.5) / STEPS.length) * (st.end - st.start);
    const lenis = getLenis();
    if (lenis) lenis.scrollTo(y, { duration: 1.1 });
    else window.scrollTo({ top: y, behavior: "smooth" });
  };

  return (
    <div ref={root}>
      <div data-pin-wrap className="hidden motion-desk:block">
        <div data-pin className="flex h-svh items-center pt-16">
          <Container className="grid grid-cols-[minmax(0,0.72fr)_minmax(0,1.28fr)] items-center gap-10 xl:gap-14">
            <div>
              <Eyebrow>{HEAD.eyebrow}</Eyebrow>
              <h2 className="display-3 mt-5 max-w-md text-balance text-ink">{HEAD.title}</h2>
              <div className="relative mt-10 pl-7">
                <span aria-hidden className="absolute top-2 bottom-2 left-[7px] w-px bg-line-strong" />
                <span data-fill aria-hidden className="absolute top-2 bottom-2 left-[7px] w-px origin-top bg-primary" />
                <ol className="space-y-1">
                  {STEPS.map((s, i) => (
                    <li key={s.title}>
                      <button
                        type="button"
                        data-step
                        data-active={i === 0 ? "" : undefined}
                        onClick={() => jump(i)}
                        className="group relative flex w-full items-center gap-3 rounded-lg py-2 text-left text-lg font-medium text-ink-faint transition-colors hover:text-ink data-[active]:text-ink"
                      >
                        <span aria-hidden className="absolute -left-7 flex size-[15px] items-center justify-center rounded-full border-2 border-line-strong bg-canvas transition-colors group-data-[active]:border-primary group-data-[active]:bg-primary" />
                        <span className="font-mono text-xs text-ink-faint group-data-[active]:text-primary">0{i + 1}</span>
                        {s.title}
                      </button>
                    </li>
                  ))}
                </ol>
              </div>
              <div className="mt-8 grid max-w-md [grid-template-areas:'c'] *:[grid-area:c]">
                {STEPS.map((s) => (
                  <p key={s.title} data-caption className="text-lg leading-relaxed text-ink-muted">
                    {s.text}
                  </p>
                ))}
              </div>
            </div>
            <Laptop>
              <div className="grid h-full [grid-template-areas:'s'] *:[grid-area:s]">
                {shots.map((shot, i) => (
                  <div key={i} data-screen className="h-full min-h-0" aria-hidden>
                    {shot}
                  </div>
                ))}
              </div>
            </Laptop>
          </Container>
        </div>
      </div>

      <Container data-stack className="py-20 sm:py-28 motion-desk:hidden">
        <div className="max-w-2xl">
          <div data-reveal="fade">
            <Eyebrow>{HEAD.eyebrow}</Eyebrow>
          </div>
          <h2 data-split className="display-2c mt-5 text-balance text-ink">
            {HEAD.title}
          </h2>
        </div>
        <ol className="mt-14 space-y-16 sm:space-y-20">
          {STEPS.map((s, i) => {
            const shot = shots[i];
            return (
              <li key={s.title} className={cn("grid grid-cols-1 gap-6 md:items-center md:gap-12", i % 2 === 1 ? "md:grid-cols-[1.2fr_0.8fr]" : "md:grid-cols-[0.8fr_1.2fr]")}>
                <div data-reveal>
                  <span className="font-mono text-xs text-primary">0{i + 1}</span>
                  <h3 className="mt-2 text-2xl font-semibold tracking-tight text-ink">{s.title}</h3>
                  <p className="mt-3 text-base leading-relaxed text-ink-muted sm:text-lg">{s.text}</p>
                </div>
                <div data-reveal data-shot aria-hidden className={cn("overflow-hidden rounded-[18px] border border-line shadow-lift", i % 2 === 1 && "md:order-first")}>
                  {shot}
                </div>
              </li>
            );
          })}
        </ol>
      </Container>
    </div>
  );
}
