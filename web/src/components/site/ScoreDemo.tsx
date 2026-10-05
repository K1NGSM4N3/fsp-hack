"use client";

import { Check, MapPin, Trophy, UserRound, X } from "lucide-react";
import { useRef, useState } from "react";
import { MOTION_OK, formatCounter, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";

type Source = "test" | "fsp" | "resume" | "claimed";

interface Profile {
  tab: string;
  code: string;
  grade: string;
  strength: number;
  experience: string;
  salary: string;
  activity: string;
  city: string;
  skills: { name: string; source: Source }[];
  fsp: string | null;
  features: number[];
  reasons: string[];
  gaps: string[];
}

const FEATURES: [string, number][] = [
  ["Обязательные навыки", 41],
  ["Грейд", 15],
  ["Результат теста", 10],
  ["Достижения ФСП", 10],
  ["Желательные навыки", 8],
  ["Зарплата", 7],
  ["Актуальность", 5],
  ["Близость текстов", 4],
];

const SOURCE: Record<Source, { label: string; cls: string }> = {
  test: { label: "тест", cls: "border-primary-200 bg-primary-50 text-primary-strong" },
  fsp: { label: "ФСП", cls: "border-primary-200 bg-primary-50 text-primary-strong" },
  resume: { label: "резюме", cls: "border-line bg-canvas text-ink-muted" },
  claimed: { label: "заявлен", cls: "border-line bg-white text-ink-subtle" },
};

const PROFILES: Profile[] = [
  {
    tab: "Сильное совпадение",
    code: "K-5F8A1F",
    grade: "Middle",
    strength: 81,
    experience: "5 лет",
    salary: "от 300 000 ₽",
    activity: "2 дня назад",
    city: "Казань · гибрид",
    skills: [
      { name: "Go", source: "test" },
      { name: "PostgreSQL", source: "test" },
      { name: "Kafka", source: "fsp" },
      { name: "Docker", source: "resume" },
      { name: "gRPC", source: "claimed" },
    ],
    fsp: "Мастер спорта по продуктовому программированию, 2023",
    features: [38.9, 15, 8.1, 6.4, 5.6, 7, 4.6, 2.3],
    reasons: ["Все обязательные навыки: Go, PostgreSQL, Kafka; 2 подтверждены тестом", "Результат теста выше 81 % диапазона грейда Middle", "Ожидания внутри вилки"],
    gaps: ["Нет желательного навыка: Kubernetes"],
  },
  {
    tab: "Близкий стек",
    code: "K-9073DC",
    grade: "Middle",
    strength: 52,
    experience: "4 года",
    salary: "от 260 000 ₽",
    activity: "вчера",
    city: "Москва · офис, гибрид",
    skills: [
      { name: "Go", source: "test" },
      { name: "MySQL", source: "resume" },
      { name: "Kafka", source: "test" },
      { name: "Docker", source: "claimed" },
    ],
    fsp: null,
    features: [30.8, 15, 5.2, 0, 3.2, 7, 4.8, 1.8],
    reasons: ["Обязательные навыки Go и Kafka подтверждены тестом", "Ожидания внутри вилки", "Профиль актуален: активность вчера"],
    gaps: ["Вместо PostgreSQL близкий навык MySQL", "Навык Docker только заявлен"],
  },
  {
    tab: "Грейд выше запроса",
    code: "K-C42234",
    grade: "Senior",
    strength: 94,
    experience: "9 лет",
    salary: "от 410 000 ₽",
    activity: "5 дней назад",
    city: "Санкт-Петербург · удалённо",
    skills: [
      { name: "Go", source: "test" },
      { name: "PostgreSQL", source: "test" },
      { name: "Kafka", source: "resume" },
      { name: "Kubernetes", source: "test" },
    ],
    fsp: "Призёр всероссийских соревнований по алгоритмическому программированию, 2022",
    features: [36.6, 9, 9.4, 3.1, 8, 3, 4.1, 2.9],
    reasons: ["Все обязательные навыки, 2 подтверждены тестом", "Желательный Kubernetes подтверждён тестом", "Результат теста выше 94 % диапазона грейда Senior"],
    gaps: ["Грейд Senior, на ступень выше запроса", "Ожидания выше вилки на 17 %"],
  },
];

const R = 58;
const CIRC = 2 * Math.PI * R;
const total = (p: Profile) => Math.round(p.features.reduce((a, b) => a + b, 0));

export function ScoreDemo() {
  const [active, setActive] = useState(0);
  const root = useRef<HTMLDivElement>(null);
  const prev = useRef<{ score: number; features: number[] } | null>(null);
  const p = PROFILES[active];
  const score = total(p);

  useGSAP(
    () => {
      const el = root.current;
      if (!el) return;
      const before = prev.current;
      const from = before ?? { score: 0, features: FEATURES.map(() => 0) };
      const first = before === null;
      prev.current = { score, features: p.features };
      let finished = false;
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const ring = el.querySelector<SVGCircleElement>("[data-ring]");
        const num = el.querySelector<HTMLElement>("[data-score]");
        const bars = gsap.utils.toArray<HTMLElement>("[data-bar]", el);
        const values = gsap.utils.toArray<HTMLElement>("[data-bar-value]", el);
        const lines = gsap.utils.toArray<HTMLElement>("[data-line]", el);
        const tl = gsap.timeline({
          defaults: { ease: "power3.out" },
          scrollTrigger: first ? { trigger: el, start: "top 75%", once: true } : undefined,
        });
        const state = { v: from.score };
        tl.fromTo(ring, { strokeDashoffset: CIRC * (1 - from.score / 100) }, { strokeDashoffset: CIRC * (1 - score / 100), duration: 1.6 }, 0);
        tl.to(state, { v: score, duration: 1.6, onUpdate: () => num && (num.textContent = formatCounter(state.v, 0)) }, 0);
        bars.forEach((bar, i) => {
          const max = FEATURES[i]?.[1] ?? 1;
          tl.fromTo(bar, { width: `${((from.features[i] ?? 0) / max) * 100}%` }, { width: `${((p.features[i] ?? 0) / max) * 100}%`, duration: 1.1 }, 0.15 + i * 0.06);
        });
        values.forEach((v, i) => {
          const s = { v: from.features[i] ?? 0 };
          tl.to(s, { v: p.features[i] ?? 0, duration: 1.1, onUpdate: () => (v.textContent = formatCounter(s.v, 1)) }, 0.15 + i * 0.06);
        });
        tl.from(lines, { autoAlpha: 0, x: -14, duration: 0.6, stagger: 0.09 }, 0.35);
        tl.eventCallback("onComplete", () => {
          finished = true;
        });
      });
      return () => {
        if (first && !finished) prev.current = before;
      };
    },
    { scope: root, dependencies: [active] },
  );

  return (
    <div ref={root} className="relative">
      <div role="tablist" aria-label="Примеры кандидатов" className="mb-5 flex flex-wrap gap-2">
        {PROFILES.map((x, i) => (
          <button
            key={x.code}
            type="button"
            role="tab"
            aria-selected={i === active}
            onClick={() => setActive(i)}
            className={cn(
              "inline-flex h-10 items-center gap-2 rounded-full border px-4 text-sm font-medium transition-colors",
              i === active ? "border-ink bg-ink text-white" : "border-line-strong bg-white text-ink-muted hover:border-ink hover:text-ink",
            )}
          >
            {x.tab}
            <span className={cn("tabular font-mono text-xs", i === active ? "text-primary-300" : "text-ink-faint")}>{total(x)}</span>
          </button>
        ))}
      </div>

      <article role="tabpanel" aria-label={`Карточка кандидата ${p.code}`} className="rounded-[24px] border border-line bg-white p-5 shadow-lift sm:p-7">
        <div className="flex items-start justify-between gap-5">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2.5">
              <span className="flex size-9 items-center justify-center rounded-full bg-canvas text-ink-subtle" aria-hidden>
                <UserRound className="size-4.5" />
              </span>
              <span className="font-mono text-base font-semibold text-ink">Кандидат {p.code}</span>
              <span className="tabular rounded-full bg-primary px-2.5 py-0.5 font-mono text-xs font-semibold text-white sm:hidden">{score}/100</span>
              <span className="inline-flex h-6 items-center overflow-hidden rounded-md border border-primary-200 bg-primary-50 font-mono text-xs font-medium text-primary-strong">
                <span className="px-2">Backend</span>
                <span className="h-full w-px bg-primary-200" />
                <span className="px-2">{p.grade}</span>
              </span>
            </div>
            <dl className="mt-5 grid grid-cols-2 gap-x-6 gap-y-3 text-sm sm:grid-cols-4">
              {[
                ["Сила теста", `${p.strength}/100`],
                ["Опыт", p.experience],
                ["Ожидания", p.salary],
                ["Активность", p.activity],
              ].map(([k, v]) => (
                <div key={k}>
                  <dt className="text-xs text-ink-subtle">{k}</dt>
                  <dd className="tabular mt-0.5 font-mono font-medium text-ink">{v}</dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="relative hidden shrink-0 sm:block" role="img" aria-label={`Соответствие: ${score} из 100`}>
            <svg width="132" height="132" viewBox="0 0 132 132" className="-rotate-90" aria-hidden>
              <circle cx="66" cy="66" r={R} fill="none" strokeWidth="8" className="stroke-surface" />
              <circle
                data-ring
                cx="66"
                cy="66"
                r={R}
                fill="none"
                strokeWidth="8"
                strokeLinecap="round"
                className="stroke-primary"
                strokeDasharray={CIRC}
                strokeDashoffset={CIRC * (1 - score / 100)}
              />
            </svg>
            <span className="absolute inset-0 flex flex-col items-center justify-center">
              <span data-score className="tabular font-display text-4xl font-semibold tracking-tight text-ink">
                {score}
              </span>
              <span className="font-mono text-[10px] tracking-[0.12em] text-ink-subtle uppercase">из 100</span>
            </span>
          </div>
        </div>

        <ul className="mt-5 flex flex-wrap gap-1.5" aria-label="Навыки и источник подтверждения">
          {p.skills.map((s) => (
            <li key={s.name} className={cn("inline-flex h-7 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium", SOURCE[s.source].cls)}>
              {s.name}
              <span className="font-mono text-[10px] opacity-70">· {SOURCE[s.source].label}</span>
            </li>
          ))}
        </ul>

        <div className="mt-6 grid grid-cols-1 gap-6 border-t border-line pt-6 lg:grid-cols-[1fr_1.05fr]">
          <ul className="space-y-2.5 text-sm">
            {p.reasons.map((r) => (
              <li key={`${p.code}-${r}`} data-line className="flex items-start gap-2.5">
                <span className="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full bg-success-50 text-success">
                  <Check className="size-3.5" strokeWidth={3} aria-label="Плюс:" />
                </span>
                <span className="text-ink">{r}</span>
              </li>
            ))}
            {p.gaps.map((g) => (
              <li key={`${p.code}-${g}`} data-line className="flex items-start gap-2.5">
                <span className="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full bg-accent-50 text-accent-strong">
                  <X className="size-3.5" strokeWidth={3} aria-label="Пробел:" />
                </span>
                <span className="text-ink-muted">{g}</span>
              </li>
            ))}
            <li data-line className="flex items-start gap-2.5 pt-1 text-ink-muted">
              <Trophy className={cn("mt-0.5 size-4.5 shrink-0", p.fsp ? "text-primary" : "text-ink-faint")} aria-hidden />
              <span>{p.fsp ?? "Нет данных ФСП. Это не штраф"}</span>
            </li>
          </ul>
          <div>
            <p className="mb-3 flex items-center justify-between font-mono text-[11px] tracking-[0.1em] text-ink-subtle uppercase">
              <span>Как посчитан балл</span>
              <span>вклад / вес</span>
            </p>
            <ul className="space-y-2">
              {FEATURES.map(([label, max], i) => (
                <li key={label} className="grid grid-cols-[8.5rem_1fr_3.6rem] items-center gap-3 text-xs">
                  <span className="truncate text-ink">{label}</span>
                  <span className="relative h-2 overflow-hidden rounded-full bg-primary-50" aria-hidden>
                    <span data-bar className="absolute inset-y-0 left-0 rounded-full bg-primary" style={{ width: `${((p.features[i] ?? 0) / max) * 100}%` }} />
                  </span>
                  <span className="tabular text-right font-mono text-ink">
                    <span data-bar-value>{formatCounter(p.features[i] ?? 0, 1)}</span>
                    <span className="text-ink-faint">/{max}</span>
                  </span>
                </li>
              ))}
            </ul>
            <p className="mt-4 flex items-center gap-1.5 text-xs text-ink-subtle">
              <MapPin className="size-3.5" aria-hidden />
              {p.city} · контакты скрыты до согласия
            </p>
          </div>
        </div>
      </article>
    </div>
  );
}
