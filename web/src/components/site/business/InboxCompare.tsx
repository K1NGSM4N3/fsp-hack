"use client";

import { Check, ChevronsLeftRight, Inbox, Minus, Paperclip, Sparkles } from "lucide-react";
import { useRef } from "react";
import { useMotion } from "@/components/motion/lazy";
import { cn } from "@/lib/cn";

interface Mail {
  from: string;
  subject: string;
  preview: string;
  file?: string;
  x: number;
  y: number;
  r: number;
  tone: "a" | "b" | "c";
}

const MAIL: Mail[] = [
  { from: "Отклик с сайта", subject: "Резюме", preview: "Добрый день, прошу рассмотреть мою кандидатуру на любую подходящую вакансию", x: 3, y: 12, r: -5, tone: "a" },
  { from: "Без имени", subject: "Re: Re: Fwd: вакансия", preview: "Здравствуйте, вакансия ещё актуальна? Резюме во вложении", file: "резюме_новое(2).pdf", x: 19, y: 23, r: 4, tone: "b" },
  { from: "Отклик с сайта", subject: "Frontend, React", preview: "Бэкенд не пробовал, но быстро разберусь", x: 6, y: 41, r: 3, tone: "c" },
  { from: "Почта вакансии", subject: "Без темы", preview: "Сколько платите?", x: 27, y: 50, r: -6, tone: "a" },
  { from: "Отклик с сайта", subject: "Стажировка", preview: "Коммерческого опыта пока нет, зато очень мотивирован", x: 2, y: 64, r: -2, tone: "b" },
  { from: "Почта вакансии", subject: "Отклик: Backend", preview: "Опыт в 1С и немного Python, готов переучиться на Go", file: "CV.docx", x: 22, y: 73, r: 5, tone: "c" },
  { from: "Отклик с сайта", subject: "Senior, 12 лет опыта", preview: "Рассматриваю предложения от 600 тыс., только офис в Петербурге", x: 44, y: 16, r: 6, tone: "a" },
  { from: "Без имени", subject: "Портфолио", preview: "Прикрепляю работы, я дизайнер интерфейсов", file: "portfolio.zip", x: 58, y: 38, r: -4, tone: "b" },
  { from: "Почта вакансии", subject: "Резюме", preview: "Ищу удалёнку в любой сфере, зарплата обсуждается", x: 48, y: 60, r: 3, tone: "c" },
  { from: "Отклик с сайта", subject: "Отклик на вакансию", preview: "Сопроводительное письмо: «Здравствуйте»", x: 66, y: 74, r: -6, tone: "a" },
];

const TONE: Record<Mail["tone"], string> = {
  a: "bg-primary-100 text-primary-strong",
  b: "bg-accent-50 text-accent-strong",
  c: "bg-warning-50 text-warning",
};

interface Pick {
  code: string;
  category: string;
  score: number;
  pros: string[];
  gap: string;
}

const PICKS: Pick[] = [
  { code: "K-5F8A", category: "Бэкенд · Middle", score: 91, pros: ["Все обязательные навыки: Go, PostgreSQL, Kafka", "Ожидания внутри вашей вилки"], gap: "Нет Kubernetes" },
  { code: "K-71B0", category: "Бэкенд · Middle", score: 84, pros: ["Go и Kafka подтверждены тестом", "Призёр соревнований ФСП"], gap: "Вместо PostgreSQL — MySQL" },
  { code: "K-C422", category: "Бэкенд · Senior", score: 78, pros: ["Грейд на ступень выше запроса"], gap: "Ожидания чуть выше вашей вилки" },
];

const START = 0.5;

const loadDraggable = () => import("@/components/motion/extras/draggable");

function ScoreRing({ value }: { value: number }) {
  const c = 2 * Math.PI * 17;
  return (
    <span className="relative flex size-11 shrink-0 items-center justify-center">
      <svg viewBox="0 0 40 40" className="absolute inset-0 -rotate-90" aria-hidden>
        <circle cx="20" cy="20" r="17" fill="none" strokeWidth="3" className="stroke-primary-100" />
        <circle cx="20" cy="20" r="17" fill="none" strokeWidth="3" strokeLinecap="round" className="stroke-primary" strokeDasharray={`${(c * value) / 100} ${c}`} />
      </svg>
      <span className="tabular text-[13px] font-semibold text-ink">{value}</span>
    </span>
  );
}

function MailCard({ m }: { m: Mail }) {
  return (
    <div
      data-mail
      className="absolute w-[80%] rounded-[14px] border border-line bg-white p-3 shadow-pop sm:w-[46%] lg:w-[31%]"
      style={{ left: `${m.x}%`, top: `${m.y}%`, transform: `rotate(${m.r}deg)` }}
    >
      <div className="flex items-center gap-2">
        <span className={cn("flex size-7 shrink-0 items-center justify-center rounded-full text-[11px] font-semibold", TONE[m.tone])}>{m.subject.slice(0, 1)}</span>
        <span className="min-w-0 flex-1 truncate text-[11px] text-ink-subtle">{m.from}</span>
        <span className="size-2 shrink-0 rounded-full bg-accent" aria-hidden />
      </div>
      <p className="mt-2 truncate text-[13px] font-semibold text-ink">{m.subject}</p>
      <p className="mt-0.5 line-clamp-2 text-[12px] leading-snug text-ink-muted">{m.preview}</p>
      {m.file && (
        <span className="mt-2 inline-flex items-center gap-1 rounded-md bg-canvas px-1.5 py-0.5 text-[11px] text-ink-subtle">
          <Paperclip className="size-3" aria-hidden />
          {m.file}
        </span>
      )}
    </div>
  );
}

/**
 * Before/after comparison. The "after" scene is revealed by translating a clipping wrapper and counter-translating
 * its content, so the divider moves with transforms only. Drag, click, or use the arrow keys on the handle.
 * Touch drags are horizontal only, vertical swipes scroll the page; the idle floating of the mails is desktop-only.
 */
export function InboxCompare() {
  const root = useRef<HTMLDivElement>(null);

  useMotion(
    ({ MOTION_DESKTOP, MOTION_OK, ScrollTrigger, gsap }, { Draggable }) => {
      const el = root.current;
      const clip = el?.querySelector<HTMLElement>("[data-clip]");
      const inner = el?.querySelector<HTMLElement>("[data-inner]");
      const track = el?.querySelector<HTMLElement>("[data-track]");
      const handle = el?.querySelector<HTMLButtonElement>("[data-handle]");
      if (!el || !clip || !inner || !track || !handle) return;

      const rest = el.offsetWidth < 640 ? 0.12 : START;
      const state = { p: rest };
      const apply = () => {
        const v = gsap.utils.clamp(0, 1, state.p) * 100;
        gsap.set([clip, track], { xPercent: v, x: 0 });
        gsap.set(inner, { xPercent: -v, x: 0 });
        handle.setAttribute("aria-valuenow", String(Math.round(v)));
      };
      let target = state.p;
      const glide = (to: number, duration = 0.6) => {
        target = gsap.utils.clamp(0, 1, to);
        gsap.to(state, { p: target, duration: window.matchMedia(MOTION_OK).matches ? duration : 0, ease: "power3.out", overwrite: true, onUpdate: apply });
      };
      apply();

      const mm = gsap.matchMedia();
      mm.add({ motion: MOTION_OK, desktop: MOTION_DESKTOP, reduce: "(prefers-reduced-motion: reduce)" }, (ctx) => {
        const motion = Boolean(ctx.conditions?.motion);
        const desktop = Boolean(ctx.conditions?.desktop);
        const proxy = document.createElement("div");
        const width = () => el.offsetWidth || 1;
        const [drag] = Draggable.create(proxy, {
          type: "x",
          trigger: el,
          inertia: motion,
          allowNativeTouchScrolling: true,
          minimumMovement: 4,
          onPressInit() {
            gsap.killTweensOf(state);
            gsap.set(proxy, { x: state.p * width() });
            this.applyBounds({ minX: 0, maxX: width() });
          },
          onDrag() {
            state.p = this.x / width();
            apply();
          },
          onThrowUpdate() {
            state.p = this.x / width();
            apply();
          },
          onClick(e: PointerEvent) {
            if (handle.contains(e.target as Node)) return;
            const box = el.getBoundingClientRect();
            glide((e.clientX - box.left) / box.width);
          },
        });

        if (motion) {
          state.p = 0.88;
          apply();
          ScrollTrigger.create({
            trigger: el,
            start: "top 70%",
            once: true,
            onEnter: () => {
              gsap.to(state, { p: rest, duration: 1.8, ease: "expo.inOut", delay: 0.2, onUpdate: apply });
            },
          });
          if (desktop) {
            for (const card of gsap.utils.toArray<HTMLElement>("[data-mail]", el)) {
              gsap.to(card, { y: gsap.utils.random(-7, 7), rotation: `+=${gsap.utils.random(-1.6, 1.6)}`, duration: gsap.utils.random(2.2, 3.6), ease: "sine.inOut", yoyo: true, repeat: -1 });
            }
          }
        }
        return () => drag?.kill();
      });

      const onKey = (e: KeyboardEvent) => {
        const step = e.shiftKey || e.key === "PageUp" || e.key === "PageDown" ? 0.2 : 0.05;
        const from = gsap.isTweening(state) ? target : state.p;
        if (e.key === "ArrowRight" || e.key === "PageUp") glide(from + step, 0.35);
        else if (e.key === "ArrowLeft" || e.key === "PageDown") glide(from - step, 0.35);
        else if (e.key === "Home") glide(0);
        else if (e.key === "End") glide(1);
        else return;
        e.preventDefault();
      };
      handle.addEventListener("keydown", onKey);
      return () => handle.removeEventListener("keydown", onKey);
    },
    { scope: root, extra: loadDraggable },
  );

  return (
    <div
      ref={root}
      className="relative h-[38rem] cursor-ew-resize touch-pan-y overflow-hidden rounded-panel border border-line bg-surface select-none sm:h-[39rem] lg:h-[40rem]"
    >
      <div aria-hidden className="absolute inset-0">
        <div className="bg-grid absolute inset-0 opacity-60" />
        {MAIL.map((m) => (
          <MailCard key={m.subject + m.x} m={m} />
        ))}
        <span className="absolute top-4 left-4 z-10 inline-flex items-center gap-2 rounded-full bg-ink px-3.5 py-2 text-[13px] font-medium text-white shadow-pop">
          <Inbox className="size-4" />
          <span>
            Отклики<span className="hidden sm:inline"> на вакансию</span>
          </span>
        </span>
      </div>

      <div data-clip className="absolute inset-0 overflow-hidden" style={{ transform: `translateX(${START * 100}%)` }}>
        <div data-inner className="absolute inset-0 bg-white" style={{ transform: `translateX(${-START * 100}%)` }}>
          <div className="bg-grid mask-radial absolute inset-0 opacity-50" />
          <div className="absolute top-4 right-4 z-10 inline-flex items-center gap-2 rounded-full bg-primary px-3.5 py-2 text-[13px] font-medium text-white shadow-pop">
            <Sparkles className="size-4" aria-hidden />
            <span>
              Подборка<span className="hidden sm:inline"> в ФСП Таланты</span>
            </span>
          </div>

          <div className="absolute top-1/2 left-[5%] hidden w-[40%] -translate-y-1/2 sm:block lg:left-[6%] lg:w-[38%]" aria-hidden>
            <p className="font-mono text-[11px] tracking-[0.14em] text-ink-subtle uppercase">Ваш запрос</p>
            <p className="mt-3 font-display text-2xl leading-tight font-semibold tracking-tight text-ink lg:text-3xl">Бэкенд-разработчик, Middle</p>
            <div className="mt-5 flex flex-wrap gap-1.5">
              {["Go", "PostgreSQL", "Kafka", "280–350 тыс. ₽", "удалённо"].map((t) => (
                <span key={t} className="rounded-full border border-line bg-white px-3 py-1 text-sm text-ink">
                  {t}
                </span>
              ))}
            </div>
            <p className="mt-6 max-w-sm text-sm leading-relaxed text-ink-muted">Сюда попадают только специалисты, сдавшие тест на бэкенд уровня Middle или соседний грейд.</p>
          </div>

          <ol className="absolute top-[4.5rem] right-3 left-[15%] flex flex-col gap-2.5 sm:top-1/2 sm:right-[3%] sm:left-[52%] sm:-translate-y-1/2 lg:right-[4%]" aria-label="Пример подборки">
            {PICKS.map((p) => (
              <li key={p.code} className="rounded-[16px] border border-line bg-white p-3.5 shadow-card sm:p-4">
                <div className="flex items-center gap-3">
                  <ScoreRing value={p.score} />
                  <div className="min-w-0 flex-1">
                    <p className="text-[14px] font-semibold text-ink">Специалист {p.code}</p>
                    <p className="text-[12px] text-ink-subtle">{p.category} · тест сдан</p>
                  </div>
                  <span className="hidden rounded-full bg-primary-50 px-2.5 py-1 text-[12px] font-medium text-primary xl:inline">Пригласить</span>
                </div>
                <ul className="mt-3 space-y-1 text-[12.5px] leading-snug">
                  {p.pros.map((r) => (
                    <li key={r} className="flex items-start gap-2 text-ink">
                      <Check className="mt-0.5 size-3.5 shrink-0 text-success" strokeWidth={2.5} aria-hidden />
                      {r}
                    </li>
                  ))}
                  <li className="flex items-start gap-2 text-ink-muted">
                    <Minus className="mt-0.5 size-3.5 shrink-0 text-warning" strokeWidth={2.5} aria-hidden />
                    {p.gap}
                  </li>
                </ul>
              </li>
            ))}
          </ol>
        </div>
      </div>

      <div data-track className="pointer-events-none absolute inset-0" style={{ transform: `translateX(${START * 100}%)` }}>
        <span aria-hidden className="absolute inset-y-0 left-0 w-[3px] -translate-x-1/2 bg-ink" />
        <button
          type="button"
          data-handle
          role="slider"
          aria-label="Шторка между откликами и подборкой"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round(START * 100)}
          className="pointer-events-auto absolute top-1/2 left-0 flex size-14 -translate-x-1/2 -translate-y-1/2 cursor-grab items-center justify-center rounded-full bg-ink text-white shadow-pop ring-4 ring-white/70 transition-colors hover:bg-primary active:cursor-grabbing"
        >
          <ChevronsLeftRight className="size-6" aria-hidden />
        </button>
      </div>
    </div>
  );
}
