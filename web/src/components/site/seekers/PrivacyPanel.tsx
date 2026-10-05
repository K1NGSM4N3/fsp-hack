"use client";

import { ArrowDown, EyeOff, Trash2 } from "lucide-react";
import { useRef, useState } from "react";
import { MOTION_OK, gsap, useGSAP } from "@/components/motion/gsap";
import { cn } from "@/lib/cn";
import "./plugins";

type Status = "active" | "open" | "closed";

const STATUSES: { value: Status; label: string; hint: string }[] = [
  { value: "active", label: "Активно ищу", hint: "Компании видят отметку «Активно ищу работу» в вашей карточке." },
  { value: "open", label: "Открыт к предложениям", hint: "Вы в поиске без отметки. Подходит, если текущая работа устраивает, но хорошее предложение вы рассмотрите." },
  { value: "closed", label: "Не ищу", hint: "Профиль скрыт из поиска, новые приглашения не приходят." },
];

const SHACKLE_CLOSED = "M23 32 V23 C23 15.8 28.8 10 36 10 C43.2 10 49 15.8 49 23 V32";
const SHACKLE_OPEN = "M23 32 V17 C23 9.8 28.8 4 36 4 C43.2 4 49 9.8 49 17 V21";

const CONTACTS: [string, string][] = [
  ["Имя", "Анна Соколова"],
  ["Почта", "anna@example.ru"],
  ["Телефон", "+7 900 000-00-00"],
];

function Toggle({ checked, onChange, label, hint }: { checked: boolean; onChange: (v: boolean) => void; label: string; hint: string }) {
  return (
    <div className="flex items-start justify-between gap-5 py-5">
      <div>
        <p className="text-[15px] font-medium text-white">{label}</p>
        <p className="mt-1 text-sm leading-relaxed text-night-muted">{hint}</p>
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        onClick={() => onChange(!checked)}
        className={cn("relative mt-0.5 inline-flex h-7 w-12 shrink-0 items-center rounded-full transition-colors duration-300", checked ? "bg-primary" : "bg-night-line")}
      >
        <span className={cn("ml-1 size-5 rounded-full bg-white shadow-[0_2px_6px_rgb(11_11_18/0.4)] transition-transform duration-300 ease-out-expo", checked && "translate-x-5")} />
      </button>
    </div>
  );
}

/** What a company sees versus what you control: «Принять» morphs the lock open and lifts the redaction; the switches rewrite the card live. */
export function PrivacyPanel() {
  const root = useRef<HTMLDivElement>(null);
  const [accepted, setAccepted] = useState(false);
  const [status, setStatus] = useState<Status>("open");
  const [showFsp, setShowFsp] = useState(true);
  const [showCompanies, setShowCompanies] = useState(true);
  const prev = useRef({ accepted, status, showFsp, showCompanies });

  useGSAP(
    () => {
      const was = prev.current;
      prev.current = { accepted, status, showFsp, showCompanies };
      const mm = gsap.matchMedia();
      mm.add(MOTION_OK, () => {
        const q = gsap.utils.selector(root);
        if (was.accepted !== accepted) {
          const shackle = q("[data-shackle]");
          const bars = q("[data-bar]");
          if (accepted) {
            gsap.fromTo(shackle, { morphSVG: SHACKLE_CLOSED }, { morphSVG: SHACKLE_OPEN, duration: 0.8, ease: "back.out(2.2)" });
            gsap.fromTo(q("[data-keyhole]"), { rotation: 0 }, { rotation: 90, svgOrigin: "36 47", duration: 0.5, ease: "power2.out" });
            gsap.fromTo(q("[data-lock]"), { rotation: 0 }, { keyframes: { rotation: [0, -8, 6, -3, 0] }, svgOrigin: "36 60", duration: 0.6, ease: "none" });
            gsap.fromTo(q("[data-glow]"), { autoAlpha: 0, scale: 0.6 }, { autoAlpha: 1, scale: 1, duration: 0.7, ease: "expo.out" });
            gsap.fromTo(bars, { scaleX: 1 }, { scaleX: 0, transformOrigin: "100% 50%", duration: 0.65, ease: "power3.inOut", stagger: 0.12, delay: 0.25 });
            gsap.fromTo(q("[data-granted]"), { autoAlpha: 0, y: 10 }, { autoAlpha: 1, y: 0, duration: 0.5, ease: "back.out(1.8)", delay: 0.6 });
          } else {
            gsap.fromTo(shackle, { morphSVG: SHACKLE_OPEN }, { morphSVG: SHACKLE_CLOSED, duration: 0.6, ease: "power3.inOut" });
            gsap.fromTo(q("[data-keyhole]"), { rotation: 90 }, { rotation: 0, svgOrigin: "36 47", duration: 0.4, ease: "power2.out" });
            gsap.fromTo(bars, { scaleX: 0 }, { scaleX: 1, transformOrigin: "0% 50%", duration: 0.5, ease: "power3.inOut", stagger: 0.08 });
            gsap.to(q("[data-glow]"), { autoAlpha: 0, duration: 0.3 });
          }
        }
        const flash = (sel: string) => gsap.fromTo(q(sel), { autoAlpha: 0, y: 8 }, { autoAlpha: 1, y: 0, duration: 0.5, ease: "back.out(1.8)" });
        if (was.showFsp !== showFsp) flash("[data-row=fsp]");
        if (was.showCompanies !== showCompanies) flash("[data-row=job]");
        if (was.status !== status) {
          flash("[data-status]");
          if (status === "closed" || was.status === "closed") gsap.fromTo(q("[data-hidden-note]"), { autoAlpha: 0, scale: 0.94 }, { autoAlpha: 1, scale: 1, duration: 0.5, ease: "back.out(1.8)" });
        }
      });
    },
    { scope: root, dependencies: [accepted, status, showFsp, showCompanies] },
  );

  const activeStatus = STATUSES.findIndex((s) => s.value === status);
  const hidden = status === "closed";

  return (
    <div ref={root} className="grid grid-cols-1 gap-8 lg:grid-cols-[1.05fr_0.95fr] lg:gap-10">
      <div>
        <div className="flex flex-wrap items-center gap-x-4 gap-y-3 rounded-[20px] border border-night-line bg-night-raised p-3 pl-4 sm:flex-nowrap">
          <div className="min-w-0 flex-1">
            <p className="font-mono text-[10.5px] tracking-[0.12em] text-night-faint uppercase">Ваш экран</p>
            <p className="mt-1 text-[15px] leading-snug font-medium text-white">Кедр Платежи приглашает</p>
            <p className="tabular text-sm text-night-muted">Бэкенд · 280–350 тыс. ₽</p>
          </div>
          <button
            type="button"
            onClick={() => setAccepted((v) => !v)}
            aria-pressed={accepted}
            className={cn(
              "inline-flex h-11 shrink-0 items-center rounded-full px-5 text-sm font-medium transition-colors",
              accepted ? "border border-night-line text-white hover:border-white" : "bg-primary text-white shadow-[0_10px_28px_-10px_rgb(64_47_255/0.9)] hover:bg-primary-hover",
            )}
          >
            {accepted ? "Вернуть замок" : "Принять"}
          </button>
        </div>
        <div aria-hidden className="flex items-center gap-2 py-3 pl-6 font-mono text-[10.5px] tracking-[0.12em] text-night-faint uppercase">
          <ArrowDown className="size-3.5" />
          экран компании
        </div>

        <div className="relative overflow-hidden rounded-[26px] bg-white p-5 text-ink shadow-[0_40px_80px_-40px_rgb(64_47_255/0.6)] sm:p-7">
          <div className={cn("transition-opacity duration-300", hidden && "opacity-20")} aria-hidden={hidden || undefined}>
            <div className="flex items-start gap-4">
              <span className="relative flex size-[68px] shrink-0 items-center justify-center rounded-[20px] bg-canvas">
                <span data-glow aria-hidden className={cn("absolute inset-0 rounded-[20px] bg-success-50 ring-2 ring-success-300", !accepted && "opacity-0")} />
                <svg viewBox="0 0 72 72" className="relative size-14" aria-hidden>
                  <g data-lock>
                    <path data-shackle d={accepted ? SHACKLE_OPEN : SHACKLE_CLOSED} fill="none" strokeWidth="6" strokeLinecap="round" className="stroke-ink" />
                    <rect x="13" y="31" width="46" height="33" rx="10" className="fill-ink" />
                    <g data-keyhole>
                      <circle cx="36" cy="45" r="4.5" className="fill-white" />
                      <path d="M36 47 V54" strokeWidth="4" strokeLinecap="round" className="stroke-white" />
                    </g>
                  </g>
                </svg>
              </span>
              <div className="min-w-0 pt-1">
                <p className="text-lg font-semibold tracking-tight">Профиль К-5F8A</p>
                <div className="mt-2 flex flex-wrap items-center gap-2">
                  <span className="rounded-full bg-primary px-2.5 py-1 text-xs font-medium text-white">Бэкенд · Middle</span>
                  <span
                    data-status
                    className={cn(
                      "rounded-full px-2.5 py-1 text-xs font-medium",
                      status === "active" ? "bg-success-50 text-success" : "bg-canvas text-ink-muted",
                    )}
                  >
                    {status === "active" ? "Активно ищу работу" : status === "open" ? "Открыт к предложениям" : "Не ищет"}
                  </span>
                </div>
              </div>
            </div>

            <dl className="mt-6 divide-y divide-line border-y border-line text-[14.5px]">
              {CONTACTS.map(([k, v]) => (
                <div key={k} className="flex items-center justify-between gap-4 py-3">
                  <dt className="text-ink-subtle">{k}</dt>
                  <dd className="relative min-w-0 font-medium">
                    <span aria-hidden={!accepted || undefined} className="tabular">
                      {v}
                    </span>
                    {!accepted && <span className="sr-only">скрыто до принятия приглашения</span>}
                    <span data-bar aria-hidden className="absolute -inset-x-1.5 -inset-y-0.5 rounded-md bg-ink" style={{ transform: accepted ? "scaleX(0)" : "scaleX(1)" }} />
                  </dd>
                </div>
              ))}
              <div data-row="job" className="flex items-start justify-between gap-4 py-3">
                <dt className="text-ink-subtle">Опыт</dt>
                <dd className="text-right">
                  Бэкенд-разработчик,{" "}
                  {showCompanies ? <span className="font-medium">Сота Логистика</span> : <span className="text-ink-faint">компания скрыта</span>}
                </dd>
              </div>
              <div data-row="fsp" className="flex items-start justify-between gap-4 py-3">
                <dt className="text-ink-subtle">ФСП</dt>
                <dd className="text-right">{showFsp ? "Призёр всероссийских соревнований по продуктовому программированию" : <span className="text-ink-faint">скрыто</span>}</dd>
              </div>
              <div className="flex items-start justify-between gap-4 py-3">
                <dt className="text-ink-subtle">Ожидания</dt>
                <dd className="tabular font-medium">от 280 000 ₽</dd>
              </div>
            </dl>
            <p data-granted className={cn("mt-4 text-[13px] leading-relaxed", accepted ? "text-success" : "text-ink-subtle")}>
              {accepted ? "Кедр Платежи видит контакты, потому что вы приняли её приглашение. Открытие записано в журнал." : "Навыки, результат теста и ожидания компании видят. Имя и контакты нет."}
            </p>
          </div>
          {hidden && (
            <div data-hidden-note className="absolute inset-0 flex flex-col items-center justify-center bg-white/80 p-8 text-center">
              <span className="flex size-12 items-center justify-center rounded-2xl bg-ink text-white">
                <EyeOff className="size-5" aria-hidden />
              </span>
              <p className="mt-4 text-lg font-semibold">Профиль скрыт из поиска</p>
              <p className="mt-1.5 max-w-xs text-sm leading-relaxed text-ink-muted">Компании не находят вас и не могут прислать новое приглашение.</p>
            </div>
          )}
        </div>
      </div>

      <div className="rounded-[26px] border border-night-line bg-night-raised p-4 sm:p-7 lg:mt-[7.5rem] lg:self-start">
        <p className="text-xl font-semibold tracking-tight text-white">Что настраиваете вы</p>
        <div className="mt-6">
          <p id="status-label" className="text-sm text-night-muted">
            Статус поиска
          </p>
          <div role="radiogroup" aria-labelledby="status-label" className="relative mt-3 grid grid-cols-3 gap-1 rounded-2xl bg-night p-1">
            <span
              aria-hidden
              className="absolute top-1 bottom-1 left-1 w-[calc((100%-0.5rem-0.5rem)/3)] rounded-xl bg-white transition-transform duration-500 ease-out-expo"
              style={{ transform: `translateX(calc(${activeStatus} * (100% + 0.25rem)))` }}
            />
            {STATUSES.map((s) => (
              <button
                key={s.value}
                type="button"
                role="radio"
                aria-checked={status === s.value}
                onClick={() => setStatus(s.value)}
                className={cn(
                  "relative min-h-12 rounded-xl px-1 py-2 text-[12px] leading-tight font-medium transition-colors duration-300 sm:px-2 sm:text-[13px]",
                  status === s.value ? "text-ink" : "text-night-muted hover:text-white",
                )}
              >
                {s.label}
              </button>
            ))}
          </div>
          <p className="mt-3 min-h-[2.75rem] text-sm leading-relaxed text-night-muted">{STATUSES[activeStatus]?.hint}</p>
        </div>
        <div className="mt-2 divide-y divide-night-line border-t border-night-line">
          <Toggle checked={showFsp} onChange={setShowFsp} label="Показывать достижения ФСП" hint="Скрытые достижения не видны компаниям и не учитываются в подборе." />
          <Toggle checked={showCompanies} onChange={setShowCompanies} label="Показывать названия прошлых компаний" hint="Если выключить, в опыте останутся должность и задачи." />
          <div className="flex items-start gap-4 py-5">
            <span className="flex size-9 shrink-0 items-center justify-center rounded-xl bg-night text-accent-300">
              <Trash2 className="size-4" aria-hidden />
            </span>
            <div>
              <p className="text-[15px] font-medium text-white">Удалить аккаунт</p>
              <p className="mt-1 text-sm leading-relaxed text-night-muted">В настройках, в любой момент. Персональные данные при этом стираются.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
