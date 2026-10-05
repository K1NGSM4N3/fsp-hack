"use client";

import { Check } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";

const ROWS: { topic: string; before: string; after: string }[] = [
  { topic: "Кто пишет первым", before: "Кандидат — отклик за откликом, в пустоту", after: "Работодатель — точечное приглашение конкретному человеку" },
  { topic: "Чем подтверждён уровень", before: "Строчкой в резюме и «уверенным знанием»", after: "Адаптивным тестом: категория = специализация × грейд" },
  { topic: "Когда видна зарплата", before: "На финальном собеседовании, если повезёт", after: "В первом сообщении: вилка «от–до» обязательна" },
  { topic: "Что видит компания", before: "Ключевые слова и годы стажа", after: "Анонимную карточку и балл 0–100 с расшифровкой" },
  { topic: "Контакты", before: "Сразу всем, кто открыл резюме", after: "Только после вашего «Принять», каждый доступ — в журнале" },
  { topic: "Если ответы утекли", before: "Тест можно выбрасывать", after: "Ничего не меняется: у каждого свои числа, код и ответы" },
];

export function FlipTable() {
  const ref = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const rows = gsap.utils.toArray<HTMLElement>("[data-row]", ref.current);
        const tl = gsap.timeline({ scrollTrigger: { trigger: ref.current, start: "top 72%", end: "bottom 55%", scrub: 0.8 } });
        rows.forEach((row, i) => {
          const at = i * 0.6;
          tl.from(row.querySelector("[data-before]"), { backgroundSize: "0% 1px", duration: 0.5, ease: "none" }, at)
            .from(row.querySelector("[data-after]"), { autoAlpha: 0.15, x: 24, duration: 0.6, ease: "power2.out" }, at + 0.15)
            .from(row.querySelector("[data-tick]"), { scale: 0, rotate: -90, duration: 0.4, ease: "back.out(3)" }, at + 0.35);
        });
      });
    },
    { scope: ref },
  );

  return (
    <div ref={ref} className="overflow-hidden rounded-panel border border-line bg-white shadow-card">
      <div className="hidden grid-cols-[13rem_1fr_1fr] border-b border-line bg-canvas/70 px-8 py-4 font-mono text-xs tracking-[0.12em] uppercase md:grid">
        <span className="text-ink-subtle">Вопрос</span>
        <span className="text-ink-subtle">Как обычно</span>
        <span className="text-primary">Как у нас</span>
      </div>
      <ul className="divide-y divide-line">
        {ROWS.map((r) => (
          <li key={r.topic} data-row className="grid grid-cols-1 gap-3 px-5 py-6 sm:px-8 md:grid-cols-[13rem_1fr_1fr] md:items-center md:gap-8">
            <span className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">{r.topic}</span>
            <span className="text-base text-ink-faint md:text-lg">
              <span className="sr-only">Как обычно: </span>
              <span data-before className="strike-draw text-accent">
                <span className="text-ink-faint">{r.before}</span>
              </span>
            </span>
            <span data-after className="flex items-start gap-3 text-base font-medium text-ink md:text-lg">
              <span data-tick aria-hidden className="mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-full bg-primary text-white">
                <Check className="size-3.5" strokeWidth={3} />
              </span>
              <span>
                <span className="sr-only">Как у нас: </span>
                {r.after}
              </span>
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
