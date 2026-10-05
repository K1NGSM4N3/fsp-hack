"use client";

import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";

const ITEMS: { title: string; text: string }[] = [
  { title: "Стек, с которым работаете", text: "Языки, базы и инструменты последних лет. По ним компании находят вас внутри категории." },
  { title: "Резюме в PDF, если есть", text: "Загрузите файл, и поля профиля заполнятся сами. Останется проверить и поправить." },
  { title: "Зарплата, от которой готовы говорить", text: "Ожидания видны в карточке, и подбор их учитывает." },
  { title: "Формат и город", text: "Офис, гибрид или удалённая работа. Вас показывают компаниям, у которых формат и город совпадают с вашими." },
  { title: "Полчаса без отвлечений", text: "На каждое задание идёт свой таймер, поэтому тест удобнее пройти спокойно и за один раз." },
  { title: "ФСП ID, если выступали на соревнованиях ФСП", text: "Достижения подтянутся из реестра и поднимут вас внутри категории. Без них вы в поиске на общих основаниях." },
];

/** Checklist whose boxes fill and tick in step with the scroll. */
export function PrepChecklist() {
  const root = useRef<HTMLOListElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const rows = gsap.utils.toArray<HTMLElement>("[data-item]", root.current);
        const tl = gsap.timeline({ scrollTrigger: { trigger: root.current, start: "top 72%", end: "bottom 62%", scrub: 0.6 } });
        rows.forEach((row, i) => {
          const fill = row.querySelector("[data-fill]");
          const check = row.querySelector("[data-check]");
          const text = row.querySelectorAll("[data-text]");
          tl.fromTo(text, { opacity: 0.32 }, { opacity: 1, duration: 0.5, ease: "none" }, i)
            .fromTo(fill, { scale: 0.3, autoAlpha: 0, transformOrigin: "50% 50%" }, { scale: 1, autoAlpha: 1, duration: 0.35, ease: "back.out(2)" }, i + 0.2)
            .fromTo(check, { drawSVG: "0%", opacity: 0 }, { drawSVG: "100%", opacity: 1, duration: 0.45, ease: "power2.out" }, i + 0.4);
        });
      });
    },
    { scope: root },
  );

  return (
    <ol ref={root} className="border-t border-line">
      {ITEMS.map((it) => (
        <li key={it.title} data-item className="grid grid-cols-[auto_1fr] gap-x-5 border-b border-line py-7 sm:gap-x-7">
          <svg viewBox="0 0 40 40" className="mt-0.5 size-9 sm:size-10" aria-hidden>
            <rect x="1.5" y="1.5" width="37" height="37" rx="11" fill="none" strokeWidth="1.5" className="stroke-line-strong" />
            <rect data-fill x="1.5" y="1.5" width="37" height="37" rx="11" className="fill-primary" />
            <path data-check d="M12 20.5 L17.5 26 L28.5 14" fill="none" strokeWidth="3.2" strokeLinecap="round" strokeLinejoin="round" className="stroke-white" />
          </svg>
          <div className="grid gap-2 sm:grid-cols-[minmax(0,0.85fr)_minmax(0,1.15fr)] sm:gap-8">
            <h3 data-text className="text-xl leading-snug font-semibold tracking-tight text-ink sm:text-[22px]">
              {it.title}
            </h3>
            <p data-text className="text-[15.5px] leading-relaxed text-ink-muted">
              {it.text}
            </p>
          </div>
        </li>
      ))}
    </ol>
  );
}
