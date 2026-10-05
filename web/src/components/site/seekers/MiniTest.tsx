"use client";

import { ArrowDown, ArrowUp, RotateCcw, TimerOff } from "lucide-react";
import { useRef, useState } from "react";
import { MOTION_OK, ScrollTrigger, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";
import { motionAllowed } from "./plugins";

interface Question {
  text: string;
  options: string[];
  correct: number;
}

const BANK: Question[][] = [
  [
    {
      text: "Поле «Возраст» принимает значения от 18 до 60. Какой набор лучше всего проверит границы?",
      options: ["18 и 60", "17, 18, 60 и 61", "20, 30, 40 и 50", "0 и 100"],
      correct: 1,
    },
    {
      text: "Пользователь забыл пароль. Что безопаснее всего отправить ему на почту?",
      options: ["Его старый пароль", "Новый пароль открытым текстом", "Одноразовую ссылку для смены пароля", "Подсказку к паролю"],
      correct: 2,
    },
  ],
  [
    {
      text: "Пользователь дважды быстро нажал «Оплатить». Как не списать деньги два раза?",
      options: ["Сделать кнопку крупнее", "Передавать уникальный ключ операции и не проводить её повторно", "Вернуть лишнее на следующий день", "Дольше ждать ответа банка"],
      correct: 1,
    },
    {
      text: "В списке заказов десятки тысяч строк, и страница грузится долго. Как отдать список быстрее?",
      options: ["Отдавать заказы частями, по страницам", "Сжать шрифты на странице", "Обновлять страницу реже", "Показывать список только утром"],
      correct: 0,
    },
  ],
  [
    {
      text: "Таблица заказов выросла до миллионов строк, и поиск заказов одного клиента стал медленным. Что поможет первым делом?",
      options: ["Перенести таблицу на другой сервер", "Удалять заказы старше месяца", "Добавить индекс по полю с клиентом", "Кэшировать главную страницу"],
      correct: 2,
    },
    {
      text: "Сервис отправляет письма через внешний сервис, и тот иногда отвечает ошибкой. Как не потерять письма?",
      options: ["Просить пользователя нажать ещё раз", "Отправлять каждое письмо трижды", "Не проверять ответ сервиса", "Класть письма в очередь и повторять отправку с паузой"],
      correct: 3,
    },
  ],
  [
    {
      text: "Сервис падает раз в сутки примерно в одно и то же время. С чего разумно начать?",
      options: ["Сравнить время падений с расписанием фоновых задач и памятью", "Перезапускать сервис каждый час", "Добавить ещё два сервера", "Переписать сервис с нуля"],
      correct: 0,
    },
    {
      text: "После выкладки новой версии у части пользователей начались ошибки. Что сделать первым?",
      options: ["Подождать, вдруг пройдёт само", "Попросить пользователей очистить кэш", "Вернуть прошлую версию и разбираться по логам", "Срочно выложить ещё одну версию"],
      correct: 2,
    },
  ],
  [
    {
      text: "Два менеджера одновременно меняют остаток одного товара, и одно изменение пропадает. Как этого не допустить?",
      options: ["Сохранять изменения реже", "Разрешить правку только по утрам", "Показывать остаток крупнее", "Сохранять с проверкой версии записи и повторять при конфликте"],
      correct: 3,
    },
    {
      text: "При сбое базы десять сервисов повторяют запросы без пауз, и база не может подняться. Что поменять?",
      options: ["Повторять с растущей паузой и случайным разбросом", "Повторять ещё чаще", "Добавить ещё десять сервисов", "Отключить базу на ночь"],
      correct: 0,
    },
  ],
];

const LEVELS = BANK.length;
const START = 2;
const ROUNDS = 3;
const LIMIT = 90;
const LETTERS = ["А", "Б", "В", "Г"];
const R = 19;
const BANDS = [
  { label: "Senior", at: 4 },
  { label: "Middle", at: 2 },
  { label: "Junior", at: 0 },
];

type Move = "up" | "down" | "timeout";

interface State {
  round: number;
  level: number;
  asked: string[];
  moves: Move[];
  picked: number | null;
}

const INITIAL: State = { round: 0, level: START, asked: [`${START}-0`], moves: [], picked: null };

const LEVEL_NAME = ["Junior", "между Junior и Middle", "Middle", "между Middle и Senior", "Senior"];

function nextId(level: number, asked: string[]): string {
  const pool = BANK[level] ?? [];
  const free = pool.findIndex((_, i) => !asked.includes(`${level}-${i}`));
  return `${level}-${Math.max(0, free)}`;
}

function clock(seconds: number): string {
  const s = Math.max(0, Math.ceil(seconds));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/** Three-question taste of the adaptive test: a right answer lifts the level marker and serves a harder question, a wrong one lowers it. */
export function MiniTest() {
  const root = useRef<HTMLDivElement>(null);
  const body = useRef<HTMLDivElement>(null);
  const marker = useRef<HTMLDivElement>(null);
  const ring = useRef<SVGCircleElement>(null);
  const time = useRef<HTMLSpanElement>(null);
  const hint = useRef<HTMLParagraphElement>(null);
  const prevLevel = useRef(START);
  const busy = useRef(false);
  const [state, setState] = useState<State>(INITIAL);

  const current = state.asked[state.asked.length - 1] ?? `${START}-0`;
  const [lvl, idx] = current.split("-").map(Number);
  const question = BANK[lvl ?? START]?.[idx ?? 0] ?? BANK[START]?.[0];
  const done = state.round >= ROUNDS;
  const lastMove = state.moves[state.moves.length - 1];

  const advance = (choice: number) => {
    setState((s) => {
      if (s.round >= ROUNDS) return s;
      const id = s.asked[s.asked.length - 1] ?? "";
      const [l, i] = id.split("-").map(Number);
      const q = BANK[l ?? START]?.[i ?? 0];
      const right = q !== undefined && choice === q.correct;
      const move: Move = choice < 0 ? "timeout" : right ? "up" : "down";
      const level = Math.max(0, Math.min(LEVELS - 1, s.level + (right ? 1 : -1)));
      const round = s.round + 1;
      const asked = round < ROUNDS ? [...s.asked, nextId(level, s.asked)] : s.asked;
      return { round, level, asked, moves: [...s.moves, move], picked: null };
    });
  };

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const box = body.current;
        if (box) {
          gsap.fromTo(box, { autoAlpha: 0, x: 28 }, { autoAlpha: 1, x: 0, duration: 0.55, ease: "expo.out" });
          gsap.fromTo(box.querySelectorAll("[data-option]"), { autoAlpha: 0, y: 14 }, { autoAlpha: 1, y: 0, duration: 0.5, ease: "back.out(1.6)", stagger: 0.06, delay: 0.1 });
        }
        if (marker.current && prevLevel.current !== state.level) {
          gsap.fromTo(
            marker.current,
            { y: 0, yPercent: -(prevLevel.current / (LEVELS - 1)) * 100 },
            { y: 0, yPercent: -(state.level / (LEVELS - 1)) * 100, duration: 1.1, ease: "elastic.out(1, 0.55)" },
          );
        }
        if (hint.current && state.moves.length) {
          gsap.fromTo(hint.current, { autoAlpha: 0, y: 8 }, { autoAlpha: 1, y: 0, duration: 0.5, ease: "back.out(2)" });
        }
        if (!done && ring.current && time.current) {
          const t = time.current;
          const left = { s: LIMIT };
          const timer = gsap
            .timeline({ paused: true })
            .fromTo(ring.current, { drawSVG: "0% 100%" }, { drawSVG: "0% 0%", duration: LIMIT, ease: "none" }, 0)
            .to(
              left,
              {
                s: 0,
                duration: LIMIT,
                ease: "none",
                onUpdate: () => {
                  t.textContent = clock(left.s);
                },
                onComplete: () => {
                  if (!busy.current) advance(-1);
                },
              },
              0,
            );
          const inView = ScrollTrigger.create({
            trigger: root.current,
            start: "top 85%",
            end: "bottom 15%",
            onToggle: (self) => void (self.isActive ? timer.play() : timer.pause()),
          });
          if (inView.isActive) timer.play();
          return () => {
            t.textContent = clock(LIMIT);
          };
        }
      });
      prevLevel.current = state.level;
      busy.current = false;
    },
    { scope: root, dependencies: [state.round], revertOnUpdate: true },
  );

  const answer = (choice: number) => {
    if (busy.current || done) return;
    busy.current = true;
    setState((s) => ({ ...s, picked: choice }));
    if (!motionAllowed() || !body.current) {
      advance(choice);
      return;
    }
    gsap.to(body.current, { autoAlpha: 0, x: -28, duration: 0.3, ease: "power2.in", delay: 0.35, onComplete: () => advance(choice) });
  };

  const restart = () => {
    busy.current = false;
    setState(INITIAL);
  };

  const ringLength = 2 * Math.PI * R;

  return (
    <div ref={root} className="overflow-hidden rounded-[26px] bg-white text-ink shadow-[0_40px_90px_-40px_rgb(64_47_255/0.65),0_0_0_1px_rgb(255_255_255/0.06)]">
      <div className="flex h-12 items-center gap-3 border-b border-line bg-canvas px-5">
        <span aria-hidden className="flex gap-1.5">
          <span className="size-2.5 rounded-full bg-line-strong" />
          <span className="size-2.5 rounded-full bg-line-strong" />
          <span className="size-2.5 rounded-full bg-line-strong" />
        </span>
        <span className="truncate text-[13px] font-medium text-ink-muted">Тест · Бэкенд · Middle</span>
        <span className="ml-auto rounded-full bg-white px-2.5 py-0.5 font-mono text-[10px] tracking-[0.1em] text-ink-subtle uppercase shadow-card">пример</span>
      </div>

      <div className="grid grid-cols-[1fr_auto]">
        <div className="min-h-[468px] p-5 sm:p-7">
          {done ? (
            <div ref={body} aria-live="polite">
              <p className="font-mono text-[11px] tracking-[0.12em] text-primary uppercase">Пример окончен</p>
              <p className="mt-3 text-[22px] leading-snug font-semibold tracking-tight text-ink sm:text-2xl">Так тест подбирает задания под ваши ответы</p>
              <p className="mt-3 text-[15px] leading-relaxed text-ink-muted">
                Здесь три задания. В настоящем тесте их от 8 до 20: он идёт, пока ваш уровень не станет понятен, а затем показывает результат.
              </p>
              <ol className="mt-6 space-y-2" aria-label="Ваш путь по заданиям">
                {state.moves.map((m, i) => (
                  <li key={i} data-option className="flex items-center gap-3 rounded-xl border border-line px-3.5 py-2.5 text-[14px]">
                    <span className={cn("flex size-7 items-center justify-center rounded-full", m === "up" ? "bg-success-50 text-success" : "bg-accent-50 text-accent-strong")}>
                      {m === "up" ? <ArrowUp className="size-4" aria-hidden /> : m === "down" ? <ArrowDown className="size-4" aria-hidden /> : <TimerOff className="size-4" aria-hidden />}
                    </span>
                    Задание {i + 1}: {m === "up" ? "дальше сложнее" : m === "down" ? "дальше проще" : "время вышло, дальше проще"}
                  </li>
                ))}
              </ol>
              <button
                type="button"
                onClick={restart}
                className="mt-7 inline-flex h-11 items-center gap-2 rounded-full bg-ink px-5 text-sm font-medium text-white transition-colors hover:bg-primary"
              >
                <RotateCcw className="size-4" aria-hidden />
                Пройти ещё раз
              </button>
            </div>
          ) : (
            <div ref={body}>
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Задание {state.round + 1}</p>
                  <p ref={hint} aria-live="polite" className={cn("mt-1 flex items-center gap-1 text-[12.5px] font-medium", lastMove === "up" ? "text-success" : "text-accent-strong", !lastMove && "invisible")}>
                    {lastMove === "up" ? <ArrowUp className="size-3.5" aria-hidden /> : <ArrowDown className="size-3.5" aria-hidden />}
                    {lastMove === "up" ? "Это задание сложнее" : lastMove === "timeout" ? "Время вышло, это задание проще" : "Это задание проще"}
                  </p>
                </div>
                <span className="relative flex size-12 shrink-0 items-center justify-center" role="timer" aria-label={`На задание ${clock(LIMIT)}`}>
                  <svg viewBox="0 0 46 46" className="absolute inset-0 -rotate-90" aria-hidden>
                    <circle cx="23" cy="23" r={R} fill="none" strokeWidth="3.5" className="stroke-primary-100" />
                    <circle ref={ring} cx="23" cy="23" r={R} fill="none" strokeWidth="3.5" strokeLinecap="round" strokeDasharray={`${ringLength} ${ringLength}`} className="stroke-primary" />
                  </svg>
                  <span ref={time} aria-hidden className="tabular font-mono text-[11px] font-semibold text-ink">
                    {clock(LIMIT)}
                  </span>
                </span>
              </div>
              <p className="mt-5 text-[18px] leading-snug font-semibold tracking-tight text-pretty text-ink sm:text-xl">{question?.text}</p>
              <div role="group" aria-label="Варианты ответа" className="mt-6 space-y-2.5">
                {question?.options.map((o, i) => {
                  const picked = state.picked === i;
                  return (
                    <button
                      key={o}
                      type="button"
                      data-option
                      onClick={() => answer(i)}
                      aria-pressed={picked}
                      className={cn(
                        "flex w-full items-start gap-3 rounded-[14px] border px-3.5 py-3 text-left text-[14.5px] leading-snug transition-colors",
                        picked ? "border-primary bg-primary-50 text-ink" : "border-line text-ink hover:border-primary-300 hover:bg-primary-50/50",
                      )}
                    >
                      <span
                        className={cn(
                          "flex size-6 shrink-0 items-center justify-center rounded-full font-mono text-[11px] font-semibold",
                          picked ? "bg-primary text-white" : "bg-canvas text-ink-muted",
                        )}
                      >
                        {LETTERS[i]}
                      </span>
                      <span className="pt-0.5">{o}</span>
                    </button>
                  );
                })}
              </div>
            </div>
          )}
        </div>

        <aside aria-label="Уровень" className="relative w-[64px] border-l border-line bg-canvas/60 sm:w-[92px]">
          <p className="pt-5 text-center font-mono text-[8.5px] tracking-[0.12em] text-ink-subtle uppercase sm:text-[9.5px]">Уровень</p>
          <div className="absolute inset-x-0 top-14 bottom-10">
            {BANDS.map((b) => (
              <span
                key={b.label}
                aria-hidden
                className="absolute right-[30px] -translate-y-1/2 font-mono text-[8px] text-ink-faint sm:right-10 sm:text-[9.5px]"
                style={{ top: `${100 - (b.at / (LEVELS - 1)) * 100}%` }}
              >
                {b.label}
              </span>
            ))}
            <div className="absolute inset-y-0 right-1.5 w-5 sm:right-4">
              <span aria-hidden className="absolute inset-y-0 left-1/2 w-px -translate-x-1/2 bg-line-strong" />
              {BANDS.map((b) => (
                <span
                  key={b.label}
                  aria-hidden
                  className="absolute left-1/2 h-px w-2.5 -translate-x-1/2 bg-line-strong"
                  style={{ top: `${100 - (b.at / (LEVELS - 1)) * 100}%` }}
                />
              ))}
              <div ref={marker} className="absolute inset-0" style={{ transform: `translateY(${-(state.level / (LEVELS - 1)) * 100}%)` }}>
                <span className="absolute bottom-0 left-1/2 flex size-5 -translate-x-1/2 translate-y-1/2 items-center justify-center rounded-full bg-primary shadow-[0_0_0_6px_rgb(64_47_255/0.18),0_8px_18px_-4px_rgb(64_47_255/0.7)]">
                  <span className="size-1.5 rounded-full bg-white" />
                </span>
              </div>
            </div>
          </div>
          <span className="sr-only">Сейчас: {LEVEL_NAME[state.level]}</span>
        </aside>
      </div>
    </div>
  );
}
