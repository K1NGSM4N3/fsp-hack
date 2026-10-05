"use client";

import { Lock, LockOpen, ScrollText } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";

const FIELDS = [
  ["Имя", "Имя Фамилия"],
  ["Почта", "name@example.ru"],
  ["Телефон", "+7 900 000-00-00"],
];

const STEPS = ["Контакты скрыты", "Кандидат нажал «Принять»", "Компания видит контакты"];

/** Scroll-driven: blurred contacts, the «Принять» press, then the contacts and the access record appear. */
export function ContactReveal() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        gsap.set(q("[data-value]"), { filter: "blur(7px)" });
        gsap.set(q("[data-open]"), { autoAlpha: 0 });
        gsap.set(q("[data-locked]"), { autoAlpha: 1 });
        gsap.set(q("[data-log]"), { autoAlpha: 0, y: 16 });
        gsap.set(q("[data-step]"), { opacity: 0.35 });
        const tl = gsap.timeline({ scrollTrigger: { trigger: root.current, start: "top 70%", end: "center 40%", scrub: 0.8 } });
        tl.to(q("[data-step='0']"), { opacity: 1, duration: 0.2 })
          .to(q("[data-accept]"), { scale: 0.92, duration: 0.15, ease: "power2.in" }, "+=0.3")
          .to(q("[data-accept]"), { scale: 1, backgroundColor: "var(--color-success)", duration: 0.2, ease: "back.out(3)" })
          .to(q("[data-step='1']"), { opacity: 1, duration: 0.2 }, "<")
          .to(q("[data-locked]"), { autoAlpha: 0, rotate: -30, duration: 0.2 })
          .to(q("[data-open]"), { autoAlpha: 1, duration: 0.2 }, "<")
          .to(q("[data-value]"), { filter: "blur(0px)", duration: 0.5, stagger: 0.12 })
          .to(q("[data-step='2']"), { opacity: 1, duration: 0.2 }, "<")
          .to(q("[data-log]"), { autoAlpha: 1, y: 0, duration: 0.3 });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-7">
      <ol className="mb-6 grid grid-cols-3 gap-2">
        {STEPS.map((s, i) => (
          <li key={s} data-step={i} className="border-t-2 border-primary pt-2 text-xs leading-snug font-medium text-ink">
            <span className="block font-mono text-[10px] text-primary">0{i + 1}</span>
            {s}
          </li>
        ))}
      </ol>
      <div className="rounded-2xl border border-line bg-canvas/60 p-4">
        <div className="flex items-center justify-between gap-3">
          <p className="text-sm font-semibold text-ink">Приглашение: Go-разработчик в команду платежей</p>
          <span data-accept className="inline-flex h-9 shrink-0 items-center rounded-lg bg-primary px-4 text-xs font-medium text-white">
            Принять
          </span>
        </div>
        <p className="mt-1 font-mono text-xs text-ink-subtle">280 000 – 350 000 ₽ · гибрид</p>
      </div>
      <div className="mt-5 flex items-center justify-between">
        <p className="text-sm font-medium text-ink">Контакты кандидата K-5F8A1F</p>
        <span className="relative inline-flex size-9 items-center justify-center rounded-full bg-ink text-white">
          <Lock data-locked className="invisible absolute size-4" aria-hidden />
          <LockOpen data-open className="absolute size-4 text-success-300" aria-hidden />
        </span>
      </div>
      <dl className="mt-3 space-y-2">
        {FIELDS.map(([k, v]) => (
          <div key={k} className="flex items-center justify-between gap-4 rounded-xl border border-line px-4 py-2.5">
            <dt className="text-xs text-ink-subtle">{k}</dt>
            <dd data-value className="font-mono text-sm text-ink select-none">
              {v}
            </dd>
          </div>
        ))}
      </dl>
      <p data-log className="mt-4 flex items-start gap-2.5 rounded-xl bg-ink px-4 py-3 text-xs leading-relaxed text-white">
        <ScrollText className="mt-0.5 size-4 shrink-0 text-primary-300" aria-hidden />
        Записано: компания открыла контакты по принятому приглашению, сегодня в 10:42.
      </p>
      <p className="mt-3 text-xs text-ink-subtle">Данные на схеме условные.</p>
    </div>
  );
}
