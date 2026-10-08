"use client";

import { Check, Clock, LockOpen, MapPin, RotateCcw, X } from "lucide-react";
import { useRef, useState } from "react";
import { loadedKit, useMotion } from "@/components/motion/lazy";
import { cn } from "@/lib/cn";
import { DECLINE_REASON_LABEL, DECLINE_REASONS } from "@/lib/labels";

interface Invite {
  company: string;
  industry: string;
  tone: string;
  role: string;
  task: string;
  place: string;
  salary: string;
  deadline: string;
}

const INVITES: Invite[] = [
  {
    company: "Кедр Платежи",
    industry: "Финтех",
    tone: "bg-primary",
    role: "Бэкенд-разработчик в платёжную команду",
    task: "Сервис выплат партнёрам: расчёт комиссий, сверка с банком и очередь платежей.",
    place: "Гибрид · Москва",
    salary: "280 000 – 350 000 ₽",
    deadline: "Ответить в течение 14 дней",
  },
  {
    company: "Сота Логистика",
    industry: "Доставка",
    tone: "bg-night-soft",
    role: "Go-разработчик в сервис доставки",
    task: "Маршруты курьеров и статусы заказов в реальном времени, интеграции с партнёрами.",
    place: "Офис · Санкт-Петербург",
    salary: "250 000 – 310 000 ₽",
    deadline: "Осталось 9 дней",
  },
  {
    company: "Маяк Медиа",
    industry: "Медиа",
    tone: "bg-accent-strong",
    role: "Python-разработчик в команду рекомендаций",
    task: "Сервис ленты рекомендаций и фоновые пересчёты для двух мобильных приложений.",
    place: "Удалённо · любой город",
    salary: "230 000 – 290 000 ₽",
    deadline: "Осталось 3 дня",
  },
];

type Overlay = { kind: "accepted"; invite: Invite } | { kind: "reasons"; invite: Invite } | { kind: "declined"; invite: Invite; reason: string } | null;

const THRESHOLD = 110;

const loadDraggable = () => import("@/components/motion/extras/draggable");

function Pin({ n }: { n: number }) {
  return (
    <span aria-hidden className="inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-ink font-mono text-xs font-semibold text-white">
      {n}
    </span>
  );
}

function Card({ invite, top }: { invite: Invite; top: boolean }) {
  return (
    <article
      data-card={top ? "top" : undefined}
      aria-label={top ? `Приглашение от ${invite.company}` : undefined}
      className={cn("relative flex h-full touch-pan-y flex-col overflow-hidden rounded-[26px] border border-line bg-white p-5 shadow-pop select-none sm:p-6", top && "cursor-grab active:cursor-grabbing")}
    >
      <span data-yes aria-hidden className="pointer-events-none absolute top-5 left-4 z-10 -rotate-12 rounded-lg border-2 border-success bg-white px-2.5 py-1 font-mono text-sm font-semibold tracking-[0.14em] text-success uppercase" style={{ opacity: 0 }}>
        Принять
      </span>
      <span data-no aria-hidden className="pointer-events-none absolute top-5 right-4 z-10 rotate-12 rounded-lg border-2 border-accent-strong bg-white px-2.5 py-1 font-mono text-sm font-semibold tracking-[0.14em] text-accent-strong uppercase" style={{ opacity: 0 }}>
        Отклонить
      </span>
      <header className="flex items-center gap-3">
        <span className={cn("flex size-11 shrink-0 items-center justify-center rounded-[14px] text-base font-semibold text-white", invite.tone)}>{invite.company.charAt(0)}</span>
        <span className="min-w-0">
          <span className="flex items-center gap-2 font-semibold text-ink">
            <span className="truncate">{invite.company}</span>
            <Pin n={4} />
          </span>
          <span className="block text-xs text-ink-subtle">{invite.industry}</span>
        </span>
        <span className="ml-auto shrink-0 rounded-full bg-primary-50 px-2.5 py-1 text-xs font-medium text-primary-strong">Новое</span>
      </header>
      <h3 className="mt-6 text-[20px] leading-tight font-semibold tracking-tight text-ink sm:text-[21px]">
        {invite.role} <Pin n={1} />
      </h3>
      <p className="mt-2.5 text-[14.5px] leading-relaxed text-ink-muted">{invite.task}</p>
      <p className="mt-4 flex items-center gap-2 text-sm text-ink">
        <MapPin className="size-4 text-ink-subtle" aria-hidden />
        {invite.place}
        <Pin n={2} />
      </p>
      <div className="mt-auto rounded-2xl bg-canvas p-4">
        <p className="flex items-center justify-between text-xs text-ink-subtle">
          Зарплата, от и до
          <Pin n={3} />
        </p>
        <p className="tabular mt-1 font-mono text-[clamp(1rem,5.2vw,1.25rem)] font-semibold tracking-tight whitespace-nowrap text-ink sm:text-[22px]">{invite.salary}</p>
      </div>
      <p className="mt-3.5 flex items-center gap-1.5 text-xs text-ink-subtle">
        <Clock className="size-3.5" aria-hidden />
        {invite.deadline}
        <Pin n={5} />
      </p>
    </article>
  );
}

/** Invitation inbox: drag the top card right to accept or left to decline; the buttons below do the same. */
export function InviteSwipe() {
  const root = useRef<HTMLDivElement>(null);
  const busy = useRef(false);
  const taught = useRef(false);
  const [queue, setQueue] = useState<number[]>(() => INVITES.map((_, i) => i));
  const [overlay, setOverlay] = useState<Overlay>(null);

  const topIndex = queue[0];
  const topInvite = topIndex === undefined ? undefined : INVITES[topIndex];

  const finish = (dir: 1 | -1, invite: Invite) => {
    setQueue((q) => q.slice(1));
    setOverlay(dir > 0 ? { kind: "accepted", invite } : { kind: "reasons", invite });
    busy.current = false;
  };

  const decide = (dir: 1 | -1) => {
    if (busy.current || overlay || !topInvite) return;
    busy.current = true;
    const invite = topInvite;
    const card = root.current?.querySelector<HTMLElement>("[data-card=top]");
    const motion = loadedKit();
    if (!motion || !motion.matches(motion.MOTION_OK) || !card) {
      finish(dir, invite);
      return;
    }
    const { gsap } = motion;
    gsap.to(card.querySelector(dir > 0 ? "[data-yes]" : "[data-no]"), { opacity: 1, duration: 0.15 });
    gsap.to(card, {
      x: dir * (card.offsetWidth * 1.4 + 120),
      y: "+=36",
      rotation: dir * 24,
      autoAlpha: 0,
      duration: 0.55,
      ease: "power2.in",
      overwrite: true,
      onComplete: () => finish(dir, invite),
    });
  };

  useMotion(
    ({ MOTION_OK, gsap }, { Draggable, InertiaPlugin }) => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const box = root.current;
        if (!box) return;
        const panel = box.querySelector<HTMLElement>("[data-overlay]");
        if (panel) {
          gsap.fromTo(panel, { autoAlpha: 0, scale: 0.92, y: 12 }, { autoAlpha: 1, scale: 1, y: 0, duration: 0.55, ease: "back.out(1.6)" });
          gsap.fromTo(panel.querySelectorAll("[data-chip]"), { autoAlpha: 0, scale: 0.4, y: 10 }, { autoAlpha: 1, scale: 1, y: 0, duration: 0.5, ease: "back.out(2.6)", stagger: 0.05, delay: 0.15 });
          return;
        }
        const card = box.querySelector<HTMLElement>("[data-card=top]");
        if (!card) return;
        const yes = card.querySelector("[data-yes]");
        const no = card.querySelector("[data-no]");
        InertiaPlugin.track(card, "x");
        const drag: Draggable | undefined = Draggable.create(card, {
          type: "x",
          allowNativeTouchScrolling: true,
          zIndexBoost: false,
          minimumMovement: 3,
          onDrag: () => {
            const x = drag?.x ?? 0;
            gsap.set(card, { rotation: x * 0.06 });
            gsap.set(yes, { opacity: gsap.utils.clamp(0, 1, x / THRESHOLD) });
            gsap.set(no, { opacity: gsap.utils.clamp(0, 1, -x / THRESHOLD) });
          },
          onRelease: () => {
            const x = drag?.x ?? 0;
            const v = InertiaPlugin.getVelocity(card, "x");
            if (x > THRESHOLD || (x > 30 && v > 900)) decide(1);
            else if (x < -THRESHOLD || (x < -30 && v < -900)) decide(-1);
            else {
              gsap.to(card, { x: 0, y: 0, rotation: 0, duration: 0.9, ease: "elastic.out(1, 0.5)" });
              gsap.to([yes, no], { opacity: 0, duration: 0.25 });
            }
          },
        })[0];

        if (!taught.current) {
          taught.current = true;
          gsap
            .timeline({ scrollTrigger: { trigger: card, start: "top 70%", once: true }, delay: 0.3 })
            .to(card, { x: 46, rotation: 3, duration: 0.45, ease: "power2.out" })
            .to(yes, { opacity: 0.6, duration: 0.3 }, "<")
            .to(card, { x: -38, rotation: -2.5, duration: 0.55, ease: "power2.inOut" })
            .to(yes, { opacity: 0, duration: 0.2 }, "<")
            .to(no, { opacity: 0.6, duration: 0.3 }, "<0.15")
            .to(card, { x: 0, rotation: 0, duration: 0.8, ease: "elastic.out(1, 0.55)" })
            .to(no, { opacity: 0, duration: 0.25 }, "<");
        }

        return () => {
          drag?.kill();
          InertiaPlugin.untrack(card);
        };
      });
    },
    { scope: root, dependencies: [topIndex, overlay?.kind], revertOnUpdate: true, extra: loadDraggable },
  );



  const reset = () => {
    busy.current = false;
    setOverlay(null);
    setQueue(INVITES.map((_, i) => i));
  };

  const empty = !topInvite && !overlay;
  const last = queue.length === 0;

  return (
    <div ref={root} className="mx-auto w-full max-w-[25rem]">
      <div className="relative h-[468px] sm:h-[452px]">
        {queue
          .slice(0, 3)
          .map((idx, depth) => ({ idx, depth }))
          .reverse()
          .map(({ idx, depth }) => {
            const invite = INVITES[idx];
            if (!invite) return null;
            const covered = depth > 0 || overlay !== null;
            return (
              <div
                key={invite.company}
                aria-hidden={covered || undefined}
                inert={covered}
                className="absolute inset-0 transition-[translate,scale] duration-500 ease-out-expo"
                style={{ translate: `0 ${depth * 18}px`, scale: `${1 - depth * 0.05}`, zIndex: 10 - depth }}
              >
                <Card invite={invite} top={depth === 0 && overlay === null} />
              </div>
            );
          })}

        {empty && (
          <div className="absolute inset-0 flex flex-col items-center justify-center rounded-[26px] border border-dashed border-line-strong bg-canvas/60 p-8 text-center">
            <p className="text-lg font-semibold text-ink">Новых приглашений нет</p>
            <p className="mt-2 max-w-[17rem] text-sm leading-relaxed text-ink-muted">Так выглядит входящий ящик, когда вы ответили всем.</p>
            <button type="button" onClick={reset} className="mt-6 inline-flex h-11 items-center gap-2 rounded-full bg-ink px-5 text-sm font-medium text-white transition-colors hover:bg-primary">
              <RotateCcw className="size-4" aria-hidden />
              Показать снова
            </button>
          </div>
        )}

        {overlay && (
          <div data-overlay aria-live="polite" className="absolute inset-0 z-20 flex flex-col rounded-[26px] border border-line bg-white p-6 shadow-lift sm:p-7">
            {overlay.kind === "accepted" && (
              <>
                <span className="flex size-12 items-center justify-center rounded-2xl bg-success-50 text-success">
                  <LockOpen className="size-6" aria-hidden />
                </span>
                <p className="mt-5 text-[22px] leading-snug font-semibold tracking-tight text-ink">Приглашение принято</p>
                <p className="mt-2.5 text-[15px] leading-relaxed text-ink-muted">
                  {overlay.invite.company} получает ваше имя и контакты и пишет вам напрямую. Другие компании их по-прежнему не видят.
                </p>
                <ul className="mt-5 space-y-2">
                  {["Имя", "Почта", "Телефон"].map((f) => (
                    <li key={f} data-chip className="flex items-center justify-between rounded-xl bg-canvas px-3.5 py-2.5 text-sm">
                      <span className="text-ink">{f}</span>
                      <span className="flex items-center gap-1.5 text-xs font-medium text-success">
                        <Check className="size-3.5" aria-hidden /> открыто компании
                      </span>
                    </li>
                  ))}
                </ul>
              </>
            )}
            {overlay.kind === "reasons" && (
              <>
                <p className="font-mono text-xs tracking-[0.12em] text-accent-strong uppercase">Отклонено</p>
                <p className="mt-3 text-[22px] leading-snug font-semibold tracking-tight text-ink">Отметьте, что не подошло</p>
                <p className="mt-2.5 text-[15px] leading-relaxed text-ink-muted">{overlay.invite.company} увидит причину. Ваши имя и контакты остаются закрыты.</p>
                <div role="group" aria-label="Причина отказа" className="mt-6 flex flex-wrap gap-2">
                  {DECLINE_REASONS.map((r) => (
                    <button
                      key={r}
                      type="button"
                      data-chip
                      onClick={() => setOverlay({ kind: "declined", invite: overlay.invite, reason: DECLINE_REASON_LABEL[r] })}
                      className="inline-flex h-10 items-center rounded-full border border-line-strong px-4 text-sm font-medium text-ink transition-colors hover:border-ink hover:bg-canvas"
                    >
                      {DECLINE_REASON_LABEL[r]}
                    </button>
                  ))}
                </div>
                <div className="mt-auto flex items-center gap-3 rounded-2xl bg-canvas p-3.5 opacity-70">
                  <span className={cn("flex size-9 shrink-0 items-center justify-center rounded-xl text-sm font-semibold text-white", overlay.invite.tone)}>{overlay.invite.company.charAt(0)}</span>
                  <span className="min-w-0 text-[13px] leading-snug">
                    <span className="block truncate font-medium text-ink">{overlay.invite.role}</span>
                    <span className="tabular block text-ink-subtle">{overlay.invite.salary}</span>
                  </span>
                </div>
              </>
            )}
            {overlay.kind === "declined" && (
              <>
                <span className="flex size-12 items-center justify-center rounded-2xl bg-accent-50 text-accent-strong">
                  <X className="size-6" aria-hidden />
                </span>
                <p className="mt-5 text-[22px] leading-snug font-semibold tracking-tight text-ink">Причина отправлена</p>
                <p className="mt-2.5 text-[15px] leading-relaxed text-ink-muted">
                  {overlay.invite.company} увидит отметку «{overlay.reason}» в статистике отказов. Контакты так и остались закрыты.
                </p>
                <p data-chip className="mt-5 inline-flex w-fit items-center gap-2 rounded-full bg-canvas px-3.5 py-2 text-sm font-medium text-ink">
                  <X className="size-3.5 text-accent-strong" aria-hidden />
                  {overlay.reason}
                </p>
              </>
            )}
            {overlay.kind !== "reasons" && (
              <button
                type="button"
                onClick={() => setOverlay(null)}
                className="mt-auto inline-flex h-12 items-center justify-center rounded-full bg-ink px-5 text-sm font-medium text-white transition-colors hover:bg-primary"
              >
                {last ? "Готово" : "Следующее приглашение"}
              </button>
            )}
          </div>
        )}
      </div>

      <div className="mt-9 flex items-center justify-center gap-3">
        <button
          type="button"
          onClick={() => decide(-1)}
          disabled={!topInvite || overlay !== null}
          className="inline-flex h-14 items-center gap-2 rounded-full border border-line-strong bg-white px-6 text-[15px] font-medium text-ink transition-colors hover:border-accent-strong hover:text-accent-strong disabled:opacity-40 disabled:hover:border-line-strong disabled:hover:text-ink"
        >
          <X className="size-5" aria-hidden />
          Отклонить
        </button>
        <button
          type="button"
          onClick={() => decide(1)}
          disabled={!topInvite || overlay !== null}
          className="inline-flex h-14 items-center gap-2 rounded-full bg-primary px-6 text-[15px] font-medium text-white shadow-[0_14px_34px_-14px_rgb(64_47_255/0.9)] transition-colors hover:bg-primary-hover disabled:opacity-40 disabled:hover:bg-primary"
        >
          <Check className="size-5" aria-hidden />
          Принять
        </button>
      </div>
      <p className="mt-4 text-center text-xs text-ink-subtle">Карточку можно тянуть вправо или влево</p>
    </div>
  );
}
