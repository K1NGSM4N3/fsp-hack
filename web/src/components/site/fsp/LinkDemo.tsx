"use client";

import { ArrowUpRight, BadgeCheck, CloudOff, Link2, Lock, Unlink, UserRound, UsersRound } from "lucide-react";
import { useRef, useState } from "react";
import { useMotion } from "@/components/motion/lazy";
import { Container } from "@/components/site/primitives";
import { cn } from "@/lib/cn";
import { Medal, type Metal } from "./Medal";

const loadFlip = () => import("@/components/motion/extras/flip");

type Phase = "idle" | "auth" | "drop" | "linked" | "leaving";

const ITEMS: { id: string; mark: string; metal: Metal; title: string; sub: string }[] = [
  { id: "ms", mark: "МС", metal: "gold", title: "Мастер спорта", sub: "Продуктовое программирование" },
  { id: "w1", mark: "1", metal: "gold", title: "1 место, всероссийские соревнования", sub: "Алгоритмическое программирование" },
  { id: "t2", mark: "2", metal: "silver", title: "2 место, команда «Нулевой указатель»", sub: "Программирование систем ИБ, капитан" },
  { id: "w3", mark: "3", metal: "bronze", title: "3 место, межрегиональные соревнования", sub: "Программирование систем ИИ" },
];

const BUTTON: Record<Phase, string> = {
  idle: "Привязать ФСП ID",
  auth: "Вход через ФСП ID…",
  drop: "Достижения в пути…",
  linked: "Отвязать ФСП ID",
  leaving: "Отвязываем…",
};

export function LinkDemo() {
  const root = useRef<HTMLElement>(null);
  const [phase, setPhase] = useState<Phase>("linked");
  const phaseRef = useRef<Phase>("linked");
  const motion = useRef(false);
  const touched = useRef(false);
  const flipState = useRef<Flip.FlipState | null>(null);

  const go = (next: Phase) => {
    phaseRef.current = next;
    setPhase(next);
  };

  useMotion(
    ({ MOTION_DESKTOP, MOTION_OK, ScrollTrigger, SplitText, gsap }) => {
      const el = root.current;
      if (!el) return;
      const mm = gsap.matchMedia();
      mm.add({ ok: MOTION_OK, desktop: MOTION_DESKTOP }, (ctx) => {
        if (!ctx.conditions?.ok) return;
        motion.current = true;
        go("idle");
        const q = gsap.utils.selector(el);
        const [title] = q("[data-link-title]");
        if (ctx.conditions.desktop) {
          SplitText.create(title, {
            type: "words",
            mask: "words",
            autoSplit: true,
            onSplit: (self) => gsap.from(self.words, { yPercent: 105, duration: 0.9, ease: "power4.out", stagger: 0.06, scrollTrigger: { trigger: title, start: "top 85%", once: true } }),
          });
        } else {
          gsap.from(title, { autoAlpha: 0, y: 24, duration: 0.7, ease: "power3.out", scrollTrigger: { trigger: title, start: "top 88%", once: true } });
        }
        gsap.from(q("[data-link-copy] > :not(h2)"), { autoAlpha: 0, y: 26, duration: 0.9, ease: "expo.out", stagger: 0.08, scrollTrigger: { trigger: el, start: "top 70%", once: true } });
        ScrollTrigger.create({
          trigger: q("[data-stage]")[0],
          start: "center 62%",
          once: true,
          onEnter: () => {
            if (!touched.current && phaseRef.current === "idle") gsap.delayedCall(0.5, () => phaseRef.current === "idle" && !touched.current && go("auth"));
          },
        });
        return () => {
          motion.current = false;
          go("linked");
        };
      });
    },
    { scope: root },
  );

  useMotion(
    ({ gsap }, { Flip }) => {
      const el = root.current;
      if (!el || !motion.current) return;
      const q = gsap.utils.selector(el);
      if (phase === "idle") {
        gsap.set(q("[data-row-text], [data-pill]"), { clearProps: "opacity,visibility,transform" });
        gsap.from(q("[data-slot], [data-skeleton]"), { autoAlpha: 0, duration: 0.4, stagger: 0.04 });
      }
      if (phase === "auth") {
        gsap
          .timeline({ onComplete: () => go("drop") })
          .from(q("[data-sheet]"), { yPercent: 115, duration: 0.6, ease: "expo.out" })
          .from(q("[data-dot]"), { scale: 0, autoAlpha: 0, duration: 0.18, stagger: 0.07, ease: "back.out(3)" }, "-=0.1")
          .from(q("[data-sheet-ok]"), { scale: 0, autoAlpha: 0, duration: 0.4, ease: "back.out(2.5)" })
          .to(q("[data-sheet]"), { yPercent: 115, duration: 0.45, ease: "power3.in" }, "+=0.35");
      }
      if (phase === "drop") {
        const medals = q("[data-tray] [data-flip-id]");
        gsap
          .timeline({
            onComplete: () => {
              flipState.current = Flip.getState(medals);
              go("linked");
            },
          })
          .from(medals, { y: -220, rotation: () => gsap.utils.random(-50, 50), duration: 1.1, ease: "bounce.out", stagger: 0.12 })
          .to({}, { duration: 0.25 });
      }
      if (phase === "leaving") {
        gsap
          .timeline({ onComplete: () => go("idle") })
          .to(q("[data-card] [data-flip-id]"), { y: -24, scale: 0.4, autoAlpha: 0, duration: 0.4, ease: "power2.in", stagger: 0.05 })
          .to(q("[data-row-text], [data-pill]"), { autoAlpha: 0, duration: 0.25 }, 0);
      }
      if (phase === "linked" && flipState.current) {
        const state = flipState.current;
        flipState.current = null;
        gsap
          .timeline()
          .add(Flip.from(state, { targets: q("[data-card] [data-flip-id]"), duration: 0.95, ease: "power3.inOut", scale: true, stagger: 0.08, zIndex: 20 }))
          .fromTo(q("[data-row-text]"), { autoAlpha: 0, x: -14 }, { autoAlpha: 1, x: 0, duration: 0.6, ease: "expo.out", stagger: 0.08 }, 0.45)
          .fromTo(q("[data-pill]"), { scale: 0.6, autoAlpha: 0 }, { scale: 1, autoAlpha: 1, duration: 0.7, ease: "back.out(2.6)" }, 0.7);
      }
    },
    { scope: root, dependencies: [phase], extra: loadFlip },
  );

  const onClick = () => {
    touched.current = true;
    const current = phaseRef.current;
    if (current === "auth" || current === "drop" || current === "leaving") return;
    if (!motion.current) {
      go(current === "linked" ? "idle" : "linked");
      return;
    }
    go(current === "linked" ? "leaving" : "auth");
  };

  const linked = phase === "linked" || phase === "leaving";
  const busy = phase === "auth" || phase === "drop" || phase === "leaving";

  return (
    <section ref={root} id="link" className="relative scroll-mt-20 overflow-hidden bg-white py-20 sm:py-28 lg:py-36">
      <Container className="grid grid-cols-1 items-center gap-14 lg:grid-cols-[0.85fr_1.15fr] lg:gap-20">
        <div data-link-copy>
          <p className="eyebrow text-primary">ФСП ID</p>
          <h2 data-link-title className="display-2 mt-5 text-balance text-ink">
            Привязка в&nbsp;одно касание
          </h2>
          <p className="mt-7 max-w-xl text-lg leading-relaxed text-pretty text-ink-muted">
            В настройках кабинета нажмите «Привязать ФСП ID». Откроется вход ФСП: логин и пароль вы вводите на стороне Федерации, и пароль остаётся у неё. После входа достижения сами появятся в профиле.
          </p>
          <ul className="mt-10 max-w-xl divide-y divide-line border-y border-line">
            <li className="flex gap-4 py-5">
              <UsersRound className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden />
              <p className="text-[15px] leading-relaxed text-ink-muted">
                <span className="font-medium text-ink">Один ФСП ID привязывается к одному человеку.</span> Медали друга к себе в профиль не добавить.
              </p>
            </li>
            <li className="flex gap-4 py-5">
              <CloudOff className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden />
              <p className="text-[15px] leading-relaxed text-ink-muted">
                <span className="font-medium text-ink">Привязка сохраняется, даже если реестр Федерации недоступен.</span> Достижения подтянутся, как только он снова ответит.
              </p>
            </li>
          </ul>
        </div>

        <div data-stage className="relative overflow-hidden rounded-[28px] border border-line bg-canvas p-4 sm:p-7">
          <div aria-hidden className="bg-grid mask-radial pointer-events-none absolute inset-0" />
          <div data-tray className="relative h-28 sm:h-32">
            <p className="font-mono text-xs tracking-[0.14em] text-ink-subtle uppercase sm:text-xs">Реестр ФСП</p>
            {phase !== "drop" && (
              <p aria-hidden className="absolute inset-x-0 bottom-5 text-center text-sm text-ink-faint">
                {linked ? "Достижения перенесены в профиль" : "Достижения появятся после входа"}
              </p>
            )}
            <div aria-hidden className="absolute inset-x-0 bottom-0 h-px bg-line-strong" />
            <div className="absolute inset-x-0 bottom-0 flex items-end justify-center gap-3 sm:gap-6">
              {phase === "drop" && ITEMS.map((it) => <Medal key={it.id} data-flip-id={it.id} mark={it.mark} metal={it.metal} size="lg" />)}
            </div>
          </div>

          <div data-card className="relative mt-5 rounded-2xl bg-white p-4 shadow-pop sm:p-5">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="flex min-w-[11rem] flex-1 items-center gap-3">
                <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary-50 text-primary">
                  <UserRound className="size-5" aria-hidden />
                </span>
                <span className="min-w-0">
                  <span className="block text-sm font-semibold text-ink">Ваш профиль</span>
                  <span className="block truncate text-xs text-ink-subtle">Бэкенд-разработка, Middle</span>
                </span>
              </div>
              <span className="grid shrink-0 justify-items-start sm:justify-items-end">
                <span data-pill className={cn("inline-flex items-center gap-1.5 rounded-full bg-success-50 px-2.5 py-1 text-xs font-medium text-success [grid-area:1/1]", !linked && "invisible")}>
                  <BadgeCheck className="size-3.5" aria-hidden />
                  ФСП ID привязан
                </span>
                <span className={cn("inline-flex items-center rounded-full border border-line px-2.5 py-1 text-xs text-ink-subtle [grid-area:1/1]", linked && "invisible")}>Нет данных ФСП</span>
              </span>
            </div>
            <ul className="mt-4">
              {ITEMS.map((it) => (
                <li key={it.id} className="flex items-center gap-3 border-t border-line py-2.5 sm:gap-4">
                  <span className="flex h-12 w-10 shrink-0 items-end justify-center">
                    {linked ? (
                      <Medal data-flip-id={it.id} mark={it.mark} metal={it.metal} size="sm" />
                    ) : (
                      <span data-slot className="mb-0.5 size-9 rounded-full border border-dashed border-line-strong" />
                    )}
                  </span>
                  <span className="grid min-w-0 flex-1">
                    <span data-row-text aria-hidden={!linked} className={cn("flex min-w-0 flex-col gap-1 [grid-area:1/1] sm:flex-row sm:items-center sm:justify-between sm:gap-4", !linked && "invisible")}>
                      <span className="min-w-0">
                        <span className="block text-sm leading-snug font-semibold text-ink">{it.title}</span>
                        <span className="block text-xs leading-snug text-ink-subtle">{it.sub}</span>
                      </span>
                      <span className="inline-flex shrink-0 items-center gap-1 text-xs font-medium text-primary-strong underline decoration-primary/30 underline-offset-4">
                        Протокол
                        <ArrowUpRight className="size-3.5" aria-hidden />
                      </span>
                    </span>
                    {!linked && (
                      <span data-skeleton aria-hidden className="flex flex-col justify-center gap-2 [grid-area:1/1]">
                        <span className="h-2.5 w-3/5 rounded-full bg-canvas" />
                        <span className="h-2 w-2/5 rounded-full bg-canvas" />
                      </span>
                    )}
                  </span>
                </li>
              ))}
            </ul>
          </div>

          <button
            type="button"
            onClick={onClick}
            aria-disabled={busy}
            className={cn(
              "relative mt-5 inline-flex h-14 w-full items-center justify-center gap-2.5 rounded-full text-base font-medium transition-colors",
              linked ? "border border-line-strong bg-white text-ink hover:border-ink" : "bg-primary text-white shadow-[0_14px_34px_-14px_rgb(64_47_255/0.9)] hover:bg-primary-hover",
              busy && "cursor-progress",
            )}
          >
            {linked ? <Unlink className="size-4.5" aria-hidden /> : <Link2 className="size-4.5" aria-hidden />}
            {BUTTON[phase]}
          </button>
          <p className="sr-only" aria-live="polite">
            {linked ? "ФСП ID привязан, достижения появились в профиле" : phase === "idle" ? "ФСП ID не привязан" : ""}
          </p>

          {phase === "auth" && (
            <div data-sheet aria-hidden className="absolute inset-x-3 bottom-3 rounded-2xl bg-night p-5 text-white shadow-pop sm:inset-x-5 sm:bottom-5 sm:p-6">
              <div className="flex items-center justify-between">
                <p className="font-mono text-xs tracking-[0.14em] text-night-muted uppercase">Вход · ФСП ID</p>
                <span data-sheet-ok className="flex size-7 items-center justify-center rounded-full bg-success-300 text-night">
                  <BadgeCheck className="size-4" />
                </span>
              </div>
              <div className="mt-4 space-y-2.5">
                {["Логин", "Пароль"].map((label, f) => (
                  <div key={label} className="flex h-11 items-center gap-3 rounded-xl border border-night-line bg-night-raised px-4">
                    <span className="w-16 text-xs text-night-faint">{label}</span>
                    <span className="flex gap-1.5">
                      {Array.from({ length: f ? 9 : 7 }, (_, i) => (
                        <span key={i} data-dot className="size-1.5 rounded-full bg-white" />
                      ))}
                    </span>
                  </div>
                ))}
              </div>
              <p className="mt-4 flex items-center gap-2 text-xs text-night-muted">
                <Lock className="size-3.5 text-primary-300" />
                Пароль остаётся у ФСП
              </p>
            </div>
          )}
        </div>
      </Container>
    </section>
  );
}
