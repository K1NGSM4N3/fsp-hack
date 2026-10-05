"use client";

import { Link2, RotateCcw, Trophy, UserRound } from "lucide-react";
import { useRef, useState } from "react";
import { Flip, MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

interface Entry {
  code: string;
  score: number;
  fsp: string | null;
  you?: boolean;
}

const OTHERS: Entry[] = [
  { code: "K-3A91F0", score: 79.4, fsp: "КМС, алгоритмическое" },
  { code: "K-5D20B7", score: 74.8, fsp: null },
  { code: "K-8C11E2", score: 73.0, fsp: "Участник, межрегиональные" },
  { code: "K-0F7AA4", score: 69.5, fsp: null },
];
const BASE = 71.0;
const FSP_POINTS = 6.2;

const format = (v: number) => v.toLocaleString("ru-RU", { minimumFractionDigits: 1, maximumFractionDigits: 1 });

/** Pressing «Привязать ФСП ID» adds the FSP points and the candidate's card climbs the ranked list (Flip). */
export function RankClimb() {
  const [linked, setLinked] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const before = useRef<Flip.FlipState | null>(null);
  const touched = useRef(false);

  const entries: Entry[] = [...OTHERS, { code: "Вы", score: linked ? BASE + FSP_POINTS : BASE, fsp: linked ? "Мастер спорта и ещё 2" : null, you: true }].sort((a, b) => b.score - a.score);

  const toggle = (auto = false) => {
    if (auto && touched.current) return;
    if (!auto) touched.current = true;
    if (root.current && window.matchMedia(MOTION_OK).matches) before.current = Flip.getState(root.current.querySelectorAll("[data-rank]"));
    setLinked((v) => (auto ? true : !v));
  };

  useGSAP(
    () => {
      const state = before.current;
      before.current = null;
      if (state) {
        Flip.from(state, { duration: 0.9, ease: "power3.inOut", stagger: 0.04, spin: false });
        const you = root.current?.querySelector("[data-you]");
        if (you && linked) gsap.fromTo(you, { boxShadow: "0 0 0 0 rgb(64 47 255 / 0.5)" }, { boxShadow: "0 0 0 10px rgb(64 47 255 / 0)", duration: 1.2, delay: 0.6 });
      }
    },
    { scope: root, dependencies: [linked] },
  );

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        ScrollTrigger.create({ trigger: root.current, start: "top 45%", once: true, onEnter: () => gsap.delayedCall(0.8, () => toggle(true)) });
      });
    },
    { scope: root },
  );

  return (
    <div ref={root} className="rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-7">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Подборка · Бэкенд Middle</p>
          <p className="mt-1 text-sm text-ink-muted">Кандидаты с похожим результатом теста</p>
        </div>
        <button
          type="button"
          onClick={() => toggle()}
          className={cn(
            "inline-flex h-10 items-center gap-2 rounded-full px-4 text-sm font-medium transition-colors",
            linked ? "border border-line-strong text-ink hover:border-ink" : "bg-primary text-white hover:bg-primary-hover",
          )}
        >
          {linked ? <RotateCcw className="size-4" aria-hidden /> : <Link2 className="size-4" aria-hidden />}
          {linked ? "Отвязать" : "Привязать ФСП ID"}
        </button>
      </div>
      <ol className="mt-5 space-y-2" aria-live="polite">
        {entries.map((e, i) => (
          <li
            key={e.code}
            data-rank
            data-flip-id={e.code}
            data-you={e.you ? "" : undefined}
            className={cn("flex items-center gap-3 rounded-2xl border px-3 py-2.5", e.you ? "border-primary bg-primary-50" : "border-line bg-white")}
          >
            <span className="tabular w-5 text-center font-mono text-sm text-ink-subtle">{i + 1}</span>
            <span className={cn("flex size-9 shrink-0 items-center justify-center rounded-full", e.you ? "bg-primary text-white" : "bg-canvas text-ink-subtle")}>
              <UserRound className="size-4" aria-hidden />
            </span>
            <span className="min-w-0 flex-1">
              <span className={cn("block font-mono text-sm font-semibold", e.you ? "text-primary-strong" : "text-ink")}>{e.code}</span>
              <span className="flex items-center gap-1 truncate text-xs text-ink-subtle">
                {e.fsp ? (
                  <>
                    <Trophy className="size-3 shrink-0 text-primary" aria-hidden /> {e.fsp}
                  </>
                ) : (
                  "Нет данных ФСП"
                )}
              </span>
            </span>
            <span className="tabular shrink-0 font-mono text-sm font-semibold text-ink">{format(e.score)}</span>
          </li>
        ))}
      </ol>
      <p className="mt-4 text-xs leading-relaxed text-ink-subtle">
        За достижения кандидат получил {format(FSP_POINTS)} балла из 10 возможных. У остальных баллы не изменились.
      </p>
    </div>
  );
}
