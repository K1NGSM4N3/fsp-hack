"use client";

import { Check, Clock, FileText, Lock, LockOpen, MailCheck } from "lucide-react";
import { useRef, type ReactNode } from "react";
import { MOTION_DESKTOP, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

type Side = "candidate" | "employer";

interface Step {
  side: Side;
  title: string;
  text: string;
  visual: ReactNode;
}

function Tag({ children, active = false }: { children: ReactNode; active?: boolean }) {
  return (
    <span className={cn("inline-flex h-7 items-center rounded-full border px-2.5 font-mono text-xs", active ? "border-primary-400 bg-primary/25 text-white" : "border-night-line text-night-muted")}>
      {children}
    </span>
  );
}

const STEPS: Step[] = [
  {
    side: "candidate",
    title: "Регистрация и профиль",
    text: "Email с подтверждением и согласия на обработку данных. Профиль — формой или из PDF-резюме: текст распознаётся, опыт и навыки подставляются сами.",
    visual: (
      <div className="space-y-2 text-sm">
        <p className="flex items-center gap-2 text-white">
          <MailCheck className="size-4 text-success-300" aria-hidden /> Email подтверждён
        </p>
        <p className="flex items-center gap-2 text-night-muted">
          <FileText className="size-4" aria-hidden /> resume.pdf → 9 навыков распознано
        </p>
      </div>
    ),
  },
  {
    side: "candidate",
    title: "Опрос: стек и грейд",
    text: "Направление, специализация, стек и грейд, на который вы претендуете. Стек решает, какие темы попадут в тест, но шкала и пороги одинаковы для всех.",
    visual: (
      <div className="space-y-3">
        <div className="flex flex-wrap gap-1.5">
          <Tag active>Go</Tag>
          <Tag active>PostgreSQL</Tag>
          <Tag active>Kafka</Tag>
          <Tag>Docker</Tag>
        </div>
        <div className="inline-flex rounded-full border border-night-line p-0.5 font-mono text-xs">
          <span className="rounded-full px-3 py-1 text-night-muted">Junior</span>
          <span className="rounded-full bg-white px-3 py-1 text-ink">Middle</span>
          <span className="rounded-full px-3 py-1 text-night-muted">Senior</span>
        </div>
      </div>
    ),
  },
  {
    side: "candidate",
    title: "Адаптивный тест",
    text: "От 8 до 20 заданий, в среднем около 13. Каждое следующее подбирается под ваш текущий уровень, и тест останавливается, как только результат статистически ясен.",
    visual: (
      <div>
        <div className="flex gap-1" aria-hidden>
          {Array.from({ length: 13 }, (_, i) => (
            <span key={i} className={cn("h-1.5 flex-1 rounded-full", i < 7 ? "bg-primary-400" : i === 7 ? "bg-white" : "bg-night-line")} />
          ))}
        </div>
        <p className="mt-3 flex items-center justify-between font-mono text-xs text-night-muted">
          <span>задание 8 · go.defer_order</span>
          <span className="inline-flex items-center gap-1 text-white">
            <Clock className="size-3.5" aria-hidden /> 01:24
          </span>
        </p>
      </div>
    ),
  },
  {
    side: "candidate",
    title: "Категория",
    text: "Сдали — получаете категорию, например «Бэкенд · Middle». Навыки из решённых заданий отмечаются как «подтверждён тестом». Без категории профиль в поиске не участвует.",
    visual: (
      <div className="space-y-3">
        <span className="inline-flex h-9 items-center overflow-hidden rounded-lg border border-primary-400 font-mono text-sm text-white">
          <span className="px-3">Бэкенд</span>
          <span className="h-full w-px bg-primary-400" />
          <span className="px-3">Middle</span>
        </span>
        <p className="flex items-center gap-2 text-sm text-night-muted">
          <Check className="size-4 text-success-300" aria-hidden /> Go, PostgreSQL — подтверждены тестом
        </p>
      </div>
    ),
  },
  {
    side: "employer",
    title: "Потребность текстом",
    text: "Компания описывает, кого ищет, — формой или просто вставляет текст вакансии. Система распознаёт специализацию, грейд, обязательные и желательные навыки, вилку и формат.",
    visual: (
      <p className="rounded-xl border border-night-line bg-night/60 p-3 text-sm leading-relaxed text-night-muted">
        Ищем <mark className="rounded bg-primary/30 px-1 text-white">Middle</mark> <mark className="rounded bg-primary/30 px-1 text-white">Go</mark>-разработчика в платёжную команду:{" "}
        <mark className="rounded bg-primary/30 px-1 text-white">PostgreSQL</mark>, <mark className="rounded bg-primary/30 px-1 text-white">Kafka</mark>, гибрид, <mark className="rounded bg-primary/30 px-1 text-white">250–320 тыс.</mark>
      </p>
    ),
  },
  {
    side: "employer",
    title: "Подборка с объяснением",
    text: "Сначала — рекомендованные категории: сколько кандидатов, медианные ожидания, покрытие стека. Затем — анонимные карточки по баллу 0–100 с причинами и пробелами.",
    visual: (
      <ul className="space-y-1.5 font-mono text-xs">
        {[
          ["K-5F8A1F", 88],
          ["K-9073DC", 74],
          ["K-C42234", 69],
        ].map(([code, score]) => (
          <li key={code} className="flex items-center gap-3">
            <span className="w-20 text-night-muted">{code}</span>
            <span className="h-1.5 flex-1 overflow-hidden rounded-full bg-night-line">
              <span className="block h-full rounded-full bg-primary-400" style={{ width: `${score}%` }} />
            </span>
            <span className="w-6 text-right text-white">{score}</span>
          </li>
        ))}
      </ul>
    ),
  },
  {
    side: "employer",
    title: "Приглашение с вилкой",
    text: "Что предлагаете, сообщение, вилка «от–до», формат и способ связи. Не больше 30 приглашений за 7 дней и одно действующее на пару «компания–кандидат». Срок жизни — 14 дней.",
    visual: (
      <div>
        <p className="text-xs text-night-muted">Зарплата</p>
        <p className="tabular font-mono text-xl font-semibold text-white">280 000 – 350 000 ₽</p>
        <p className="mt-2 font-mono text-xs text-night-muted">квота: 12 / 30 · истекает через 14 дней</p>
      </div>
    ),
  },
  {
    side: "candidate",
    title: "Контакт после «Принять»",
    text: "Кандидат видит условия до начала общения и принимает или отклоняет с причиной. Только после согласия компания получает контакты — и каждое обращение к ним попадает в журнал.",
    visual: (
      <div className="space-y-2 font-mono text-sm">
        <p className="flex items-center gap-2 text-night-faint">
          <Lock className="size-4" aria-hidden /> +7 ••• ••• •• ••
        </p>
        <p className="flex items-center gap-2 text-white">
          <LockOpen className="size-4 text-success-300" aria-hidden /> +7 999 111-22-33
        </p>
        <p className="text-xs text-night-muted">доступ записан в журнал</p>
      </div>
    ),
  },
];

const SIDE_LABEL: Record<Side, string> = { candidate: "Кандидат", employer: "Работодатель" };

export function Journey() {
  const section = useRef<HTMLElement>(null);
  const viewport = useRef<HTMLDivElement>(null);
  const track = useRef<HTMLOListElement>(null);
  const fill = useRef<HTMLSpanElement>(null);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_DESKTOP, () => {
        const vp = viewport.current;
        const list = track.current;
        if (!vp || !list) return;
        vp.style.overflow = "visible";
        gsap.set(fill.current, { scaleX: 0 });
        const distance = () => Math.max(0, list.scrollWidth - vp.clientWidth);
        const move = gsap.to(list, {
          x: () => -distance(),
          ease: "none",
          scrollTrigger: {
            trigger: section.current,
            start: "top top",
            end: () => `+=${distance()}`,
            pin: true,
            scrub: 1,
            invalidateOnRefresh: true,
            onUpdate: (self) => gsap.set(fill.current, { scaleX: self.progress }),
          },
        });
        for (const card of gsap.utils.toArray<HTMLElement>("[data-step]", list)) {
          gsap.fromTo(
            card,
            { autoAlpha: 0.35, scale: 0.94 },
            { autoAlpha: 1, scale: 1, ease: "none", scrollTrigger: { trigger: card, containerAnimation: move, start: "left 95%", end: "left 55%", scrub: true } },
          );
        }
        return () => {
          vp.style.overflow = "";
        };
      });
      void document.fonts?.ready.then(() => ScrollTrigger.refresh());
    },
    { scope: section },
  );

  return (
    <section ref={section} id="how" className="relative isolate overflow-hidden bg-night py-20 text-white lg:flex lg:min-h-svh lg:flex-col lg:justify-center lg:py-16">
      <div aria-hidden className="pointer-events-none absolute -top-48 left-1/3 size-[36rem] rounded-full bg-primary/30 blur-[140px]" />
      <div aria-hidden className="bg-grid-night mask-radial pointer-events-none absolute inset-0 opacity-60" />
      <div className="relative mx-auto w-full max-w-[1240px] px-4 sm:px-6 lg:px-8">
        <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:justify-between">
          <div className="max-w-3xl">
            <p data-reveal="fade" className="eyebrow mb-5 text-primary-300">
              Путь от регистрации до контакта
            </p>
            <h2 data-split className="display-2 text-balance">
              Восемь шагов. Ни одного слепого отклика.
            </h2>
          </div>
          <p data-reveal className="max-w-sm text-night-muted">
            Кандидат подтверждает уровень один раз. Дальше инициатива у работодателя — а условия видны до начала разговора.
          </p>
        </div>
      </div>

      <div
        ref={viewport}
        tabIndex={0}
        role="region"
        aria-label="Шаги сценария, листаются горизонтально"
        className="scrollbar-none relative mt-12 snap-x snap-mandatory overflow-x-auto focus-visible:outline-offset-[-2px] lg:mt-14"
      >
        <ol ref={track} className="flex w-max gap-4 px-4 sm:px-6 lg:gap-5 lg:px-[max(2rem,calc((100vw-1240px)/2+2rem))]">
          {STEPS.map((s, i) => (
            <li
              key={s.title}
              data-step
              className={cn(
                "flex h-[29rem] w-[min(84vw,24rem)] shrink-0 snap-start flex-col justify-between rounded-[24px] border p-6 sm:p-7 lg:h-[min(31rem,58vh)] lg:min-h-[27rem] lg:w-[25rem]",
                s.side === "candidate" ? "border-night-line bg-night-raised" : "border-primary-400/40 bg-primary-900/50",
              )}
            >
              <div>
                <div className="flex items-center justify-between">
                  <span className="tabular font-mono text-sm text-night-faint">{String(i + 1).padStart(2, "0")} / 08</span>
                  <span
                    className={cn(
                      "rounded-full px-2.5 py-1 font-mono text-[11px] tracking-[0.1em] uppercase",
                      s.side === "candidate" ? "bg-white/10 text-white" : "bg-primary text-white",
                    )}
                  >
                    {SIDE_LABEL[s.side]}
                  </span>
                </div>
                <h3 className="mt-8 text-2xl font-semibold tracking-tight">{s.title}</h3>
                <p className="mt-3 text-[15px] leading-relaxed text-night-muted">{s.text}</p>
              </div>
              <div className="mt-6 border-t border-night-line pt-5">{s.visual}</div>
            </li>
          ))}
        </ol>
      </div>

      <div className="relative mx-auto mt-10 hidden w-full max-w-[1240px] px-8 lg:block" aria-hidden>
        <div className="relative h-px bg-night-line">
          <span ref={fill} className="absolute inset-0 origin-left bg-primary-400" />
        </div>
        <div className="mt-3 flex justify-between font-mono text-[11px] tracking-[0.1em] text-night-faint uppercase">
          <span>Кандидат готовит профиль</span>
          <span>Работодатель выходит на контакт</span>
        </div>
      </div>
    </section>
  );
}
