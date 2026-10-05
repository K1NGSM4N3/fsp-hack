"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";

const ITEMS = [
  { year: "2021", title: "III разряд", meta: "Алгоритмическое программирование", detail: "Первый разряд в профиле. Учитывается только самый высокий в дисциплине." },
  { year: "2022", title: "Финалист межрегиональных соревнований", meta: "Программирование систем ИБ", detail: "Кандидату в безопасность такая строчка даст больше, чем фронтендеру." },
  { year: "2023", title: "Мастер спорта", meta: "Продуктовое программирование", detail: "В карточке компания увидит её именно так, со ссылкой на протокол." },
  { year: "2024", title: "1 место на всероссийских соревнованиях", meta: "Алгоритмическое программирование · команда, капитан", detail: "Командные результаты считаются, в профиле сохраняется и ваша роль." },
];

/** The timeline line is drawn by scroll, and each node lights up as the line reaches it. */
export function AchievementPath() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        gsap.from(q("[data-line]"), { drawSVG: "0%", ease: "none", scrollTrigger: { trigger: root.current, start: "top 70%", end: "bottom 60%", scrub: 0.6 } });
        for (const node of q("[data-node]")) {
          gsap.from(node, { scale: 0, backgroundColor: "var(--color-white)", duration: 0.5, ease: "back.out(3)", scrollTrigger: { trigger: node, start: "top 62%", toggleActions: "play none none reverse" } });
        }
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="relative pl-14 sm:pl-20">
      <svg className="absolute top-2 bottom-2 left-[19px] h-[calc(100%-1rem)] w-2 sm:left-[27px]" viewBox="0 0 8 100" preserveAspectRatio="none" aria-hidden>
        <line data-line x1="4" y1="0" x2="4" y2="100" stroke="var(--color-primary)" strokeWidth="2" />
      </svg>
      <ol className="space-y-6">
        {ITEMS.map((it) => (
          <li key={it.year} data-skew className="relative rounded-[22px] border border-line bg-white p-6">
            <span data-node aria-hidden className="absolute top-7 -left-[42px] size-4 rounded-full border-4 border-white bg-primary shadow-[0_0_0_1px_var(--color-primary-200)] sm:-left-[58px]" />
            <p className="font-mono text-xs tracking-[0.1em] text-primary uppercase">{it.year}</p>
            <h3 className="mt-2 text-lg font-semibold tracking-tight text-ink">{it.title}</h3>
            <p className="text-sm text-ink-subtle">{it.meta}</p>
            <p className="mt-3 text-[15px] leading-relaxed text-ink-muted">{it.detail}</p>
          </li>
        ))}
      </ol>
    </div>
  );
}
