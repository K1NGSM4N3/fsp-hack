"use client";

import { ChevronsUp, Trophy, UserRound } from "lucide-react";
import { useRef, useState } from "react";
import type { Flip as FlipPlugin } from "gsap/Flip";
import { useMotion } from "@/components/motion/lazy";
import { Container } from "@/components/site/primitives";
import { cn } from "@/lib/cn";

const loadFlip = () => import("@/components/motion/extras/flip");

interface Row {
  id: string;
  label: string;
  test: number;
  fsp: string | null;
  note?: string;
  you?: boolean;
}

const OTHERS: Row[] = [
  { id: "a", label: "K-7F2C1A", test: 0.96, fsp: "КМС, алгоритмическое", note: "Тест заметно сильнее" },
  { id: "b", label: "K-3A91F0", test: 0.84, fsp: null },
  { id: "c", label: "K-5D20B7", test: 0.83, fsp: "Участник, региональные" },
  { id: "d", label: "K-0F7AA4", test: 0.82, fsp: null },
  { id: "e", label: "K-8C11E2", test: 0.78, fsp: null },
];

const YOU_BEFORE: Row = { id: "you", label: "Вы", test: 0.81, fsp: null, you: true };
const YOU_AFTER: Row = { ...YOU_BEFORE, fsp: "Мастер спорта и ещё три" };

const order = (linked: boolean): Row[] => {
  const [a, b, c, d, e] = OTHERS;
  return linked ? [a, YOU_AFTER, b, c, d, e] : [a, b, c, d, YOU_BEFORE, e];
};

const ROW_H = "h-[104px] sm:h-[76px]";

export function RankLadder() {
  const root = useRef<HTMLElement>(null);
  const [linked, setLinked] = useState(true);
  const before = useRef<Flip.FlipState | null>(null);
  const flip = useRef<typeof FlipPlugin | null>(null);
  const motion = useRef(false);
  const touched = useRef(false);

  const toggle = (next: boolean) => {
    if (root.current && motion.current && flip.current) before.current = flip.current.getState(root.current.querySelectorAll("[data-row]"));
    setLinked(next);
  };

  useMotion(
    ({ MOTION_DESKTOP, MOTION_OK, ScrollTrigger, gsap }, { Flip }) => {
      const el = root.current;
      if (!el) return;
      flip.current = Flip;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        motion.current = true;
        setLinked(false);
        const q = gsap.utils.selector(el);
        const wide = window.matchMedia(MOTION_DESKTOP).matches;
        gsap.from(q("[data-rank-head] > *"), {
          autoAlpha: 0,
          x: wide ? -40 : 0,
          y: wide ? 0 : 20,
          duration: wide ? 1 : 0.7,
          ease: wide ? "expo.out" : "power3.out",
          stagger: 0.1,
          scrollTrigger: { trigger: el, start: "top 75%", once: true },
        });
        gsap.from(q("[data-bar]"), { scaleX: 0, duration: 1.2, ease: "expo.out", stagger: 0.06, scrollTrigger: { trigger: q("[data-board]")[0], start: "top 80%", once: true } });
        ScrollTrigger.create({
          trigger: q("[data-board]")[0],
          start: "center 65%",
          once: true,
          onEnter: () => gsap.delayedCall(0.9, () => !touched.current && toggle(true)),
        });
        return () => {
          motion.current = false;
          setLinked(true);
        };
      });
    },
    { scope: root, extra: loadFlip },
  );

  useMotion(
    ({ gsap }, { Flip }) => {
      const state = before.current;
      before.current = null;
      const el = root.current;
      if (!state || !el) return;
      const q = gsap.utils.selector(el);
      Flip.from(state, { duration: 1.1, ease: "power3.inOut", stagger: 0.03, zIndex: 5 });
      if (linked) {
        gsap.fromTo(q("[data-you-inner]"), { scale: 1 }, { scale: 1.035, duration: 0.55, ease: "power2.out", yoyo: true, repeat: 1 });
        gsap.from(q("[data-climb]"), { autoAlpha: 0, y: 10, duration: 0.6, delay: 0.9, ease: "back.out(2.5)" });
      }
    },
    { scope: root, dependencies: [linked], extra: loadFlip },
  );

  const rows = order(linked);
  const youAt = rows.findIndex((r) => r.you);

  return (
    <section ref={root} className="relative overflow-hidden bg-canvas py-20 sm:py-28 lg:py-36">
      <Container>
        <div data-rank-head className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_0.8fr] lg:items-end lg:gap-16">
          <div>
            <p className="eyebrow text-primary">Место в подборке</p>
            <h2 className="display-2 mt-5 text-balance text-ink">Выше среди равных</h2>
          </div>
          <p className="max-w-xl text-lg leading-relaxed text-pretty text-ink-muted">
            Категорию по-прежнему определяет только тест. Достижения меняют порядок внутри неё: среди людей с похожим результатом компания сначала увидит тех, у кого есть подтверждённые победы.
          </p>
        </div>

        <div data-board className="mt-12 sm:mt-16">
          <div className="flex flex-col gap-4 border-b-2 border-ink pb-4 sm:flex-row sm:items-center sm:justify-between">
            <p className="font-mono text-xs tracking-[0.12em] text-ink-subtle uppercase">Подборка компании · Бэкенд-разработка, Middle</p>
            <button
              type="button"
              role="switch"
              aria-checked={linked}
              onClick={() => {
                touched.current = true;
                toggle(!linked);
              }}
              className="group inline-flex min-h-10 items-center gap-3 self-start rounded-full text-sm font-medium text-ink sm:self-auto"
            >
              <span className={cn("relative h-8 w-14 rounded-full transition-colors duration-300", linked ? "bg-primary" : "bg-line-strong")}>
                <span className={cn("absolute top-1 left-1 size-6 rounded-full bg-white shadow-card transition-transform duration-300 ease-out", linked && "translate-x-6")} />
              </span>
              ФСП ID привязан
            </button>
          </div>

          <div className="hidden grid-cols-[3.5rem_1.2fr_1fr_1fr] gap-6 border-b border-line py-3 font-mono text-xs tracking-[0.12em] text-ink-faint uppercase sm:grid">
            <span>Место</span>
            <span>Профиль</span>
            <span>Результат теста</span>
            <span>Достижения ФСП</span>
          </div>

          <div className="relative grid grid-cols-[2.5rem_1fr] sm:grid-cols-[3.5rem_1fr]">
            <ol aria-hidden>
              {rows.map((r, i) => (
                <li key={i} className={cn("flex items-center border-b border-line font-display text-3xl font-semibold tracking-tight tabular sm:text-4xl", ROW_H, i === youAt ? "text-primary" : "text-ink-faint")}>
                  {i + 1}
                </li>
              ))}
            </ol>
            <ol aria-label="Подборка соискателей" className="relative">
              {rows.map((r) => (
                <li key={r.id} data-row data-flip-id={r.id} className={cn("relative", ROW_H, r.you && "z-10")}>
                  <div
                    data-you-inner={r.you ? "" : undefined}
                    className={cn(
                      "grid h-full grid-cols-1 content-center gap-x-6 gap-y-2 border-b px-3 sm:grid-cols-[1.2fr_1fr_1fr] sm:items-center sm:gap-y-0 sm:px-0",
                      r.you ? "rounded-2xl border-transparent bg-primary text-white shadow-glow sm:-mx-4 sm:px-4" : "border-line",
                    )}
                  >
                    <span className="flex min-w-0 items-center gap-3">
                      <span className={cn("flex size-8 shrink-0 items-center justify-center rounded-full", r.you ? "bg-white/15 text-white" : "bg-white text-ink-subtle")}>
                        <UserRound className="size-4" aria-hidden />
                      </span>
                      <span className={cn("truncate font-mono text-sm font-semibold", r.you ? "text-white" : "text-ink")}>{r.label}</span>
                      {r.note && <span className="hidden rounded-full border border-line-strong px-2 py-0.5 text-xs whitespace-nowrap text-ink-muted lg:inline">{r.note}</span>}
                      {r.you && linked && (
                        <span data-climb className="inline-flex items-center gap-1 rounded-full bg-white/15 px-2 py-0.5 text-xs font-medium whitespace-nowrap text-white">
                          <ChevronsUp className="size-3.5" aria-hidden />
                          выше
                        </span>
                      )}
                    </span>
                    <span className="flex items-center gap-3">
                      <span className="sr-only">Результат теста</span>
                      <span className={cn("relative h-1.5 flex-1 overflow-hidden rounded-full", r.you ? "bg-white/20" : "bg-line")}>
                        <span data-bar className={cn("absolute inset-0 origin-left rounded-full", r.you ? "bg-white" : "bg-ink")} style={{ transform: `scaleX(${r.test})` }} />
                      </span>
                    </span>
                    <span className={cn("flex min-w-0 items-center gap-2 text-sm", r.you ? "text-white" : r.fsp ? "text-ink" : "text-ink-faint")}>
                      {r.fsp ? <Trophy className={cn("size-4 shrink-0", r.you ? "text-warning-200" : "text-primary")} aria-hidden /> : <span aria-hidden className="size-4 shrink-0 rounded-full border border-dashed border-current" />}
                      <span className="truncate">{r.fsp ?? "Нет данных ФСП"}</span>
                    </span>
                  </div>
                </li>
              ))}
            </ol>
          </div>
        </div>

        <div className="mt-12 grid grid-cols-1 gap-8 sm:grid-cols-2 sm:gap-12 lg:mt-16">
          <p className="max-w-lg border-l-2 border-primary pl-5 text-[15px] leading-relaxed text-ink-muted">
            <span className="font-medium text-ink">Если у соседа тест заметно сильнее, достижения его не обгонят.</span> Каждая следующая награда добавляет меньше предыдущей, так что даже стопка дипломов тест не заменит.
          </p>
          <p className="max-w-lg border-l-2 border-line-strong pl-5 text-[15px] leading-relaxed text-ink-muted">
            <span className="font-medium text-ink">Нет достижений, значит нет и штрафа.</span> В карточке будет нейтральная пометка «Нет данных ФСП», а ваше место определит результат теста.
          </p>
        </div>
      </Container>
    </section>
  );
}
