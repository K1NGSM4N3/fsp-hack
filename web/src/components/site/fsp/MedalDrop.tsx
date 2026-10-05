"use client";

import { BadgeCheck } from "lucide-react";
import { useRef } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

const MEDALS = [
  { mark: "МС", title: "Мастер спорта", meta: "2023", tone: "gold" },
  { mark: "1", title: "1 место", meta: "2024", tone: "gold" },
  { mark: "2", title: "2 место", meta: "2023", tone: "silver" },
  { mark: "КМС", title: "КМС", meta: "2022", tone: "silver" },
  { mark: "Ф", title: "Финалист", meta: "2022", tone: "bronze" },
];

const TONE: Record<string, string> = {
  gold: "from-primary-400 to-primary text-white",
  silver: "from-primary-200 to-primary-300 text-primary-900",
  bronze: "from-accent-100 to-accent-300 text-ink",
};

/** Medals fall into the profile with a bounce, then sway on their ribbons. */
export function MedalDrop() {
  const root = useRef<HTMLDivElement>(null);
  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        const medals = q("[data-medal]");
        const tl = gsap.timeline({ delay: 0.6 });
        tl.from(medals, { y: -320, rotation: () => gsap.utils.random(-40, 40), duration: 1.3, ease: "bounce.out", stagger: 0.16 })
          .from(q("[data-linked]"), { scale: 0, autoAlpha: 0, duration: 0.9, ease: "elastic.out(1, 0.45)" }, "-=0.5")
          .add(() => {
            medals.forEach((m, i) => gsap.to(m, { rotation: i % 2 ? 4 : -4, duration: 1.6 + i * 0.2, ease: "sine.inOut", yoyo: true, repeat: -1 }));
          });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-7">
      <div className="flex items-center justify-between gap-3">
        <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Профиль · достижения ФСП</p>
        <span data-linked className="inline-flex items-center gap-1.5 rounded-full bg-success-50 px-2.5 py-1 text-[11px] font-medium text-success">
          <BadgeCheck className="size-3.5" aria-hidden /> ФСП ID привязан
        </span>
      </div>
      <ul className="mt-6 grid grid-cols-3 gap-x-3 gap-y-6 sm:grid-cols-5">
        {MEDALS.map((m) => (
          <li key={m.title} className="flex flex-col items-center text-center">
            <span data-medal className="relative flex origin-top flex-col items-center">
              <span aria-hidden className="h-7 w-5 bg-[linear-gradient(90deg,var(--color-primary)_0_33%,white_33%_66%,var(--color-accent)_66%)] [clip-path:polygon(0_0,100%_0,100%_100%,50%_70%,0_100%)]" />
              <span className={cn("-mt-1 flex size-14 items-center justify-center rounded-full bg-linear-to-br font-mono text-sm font-bold shadow-pop ring-4 ring-white", TONE[m.tone])}>{m.mark}</span>
            </span>
            <span className="mt-3 text-xs font-semibold text-ink">{m.title}</span>
            <span className="text-[11px] leading-tight text-ink-subtle">{m.meta}</span>
          </li>
        ))}
      </ul>
      <div className="mt-6 flex items-center justify-between gap-3 rounded-2xl bg-canvas px-4 py-3">
        <span className="text-sm text-ink-muted">Баллы за ФСП в подборке</span>
        <span className="tabular font-display text-2xl font-semibold text-ink">
          6,2 <span className="text-sm font-medium text-ink-subtle">из 10</span>
        </span>
      </div>
    </div>
  );
}
