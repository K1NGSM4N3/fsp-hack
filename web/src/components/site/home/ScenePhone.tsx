import { BatteryFull, Briefcase, Check, ChevronLeft, Clock, MapPin, Signal, Wifi } from "lucide-react";
import { cn } from "@/lib/cn";

const LIVE_HIDDEN = "invisible opacity-0";

function StatusBar({ time, dark = false }: { time?: string; dark?: boolean }) {
  return (
    <div className={cn("relative z-20 flex h-[2.5em] items-end justify-between px-[1.7em] pb-[0.3em] text-[0.74em] font-semibold", dark ? "text-white" : "text-ink")}>
      <span>{time}</span>
      <span className="flex items-center gap-[0.3em]">
        <Signal className="size-[1.15em]" />
        <Wifi className="size-[1.15em]" />
        <BatteryFull className="size-[1.3em]" />
      </span>
    </div>
  );
}

const OPTIONS = ["Добавлю ещё серверов", "Найду, где запрос ждёт дольше всего", "Перепишу сервис с нуля"];

/** Job seeker's phone. Server markup shows the accepted invitation; the scene timeline rewinds it to the test screen. */
const PLACEMENT =
  "-mt-[22%] ml-[4%] w-[48%] lg:mt-0 lg:ml-0 lg:w-[25%] group-data-live/scene:absolute group-data-live/scene:bottom-0 group-data-live/scene:left-0 group-data-live/scene:m-0 group-data-live/scene:w-[min(50cqw,35cqh)] lg:group-data-live/scene:right-[2%] lg:group-data-live/scene:left-auto lg:group-data-live/scene:w-[min(25cqw,44cqh)]";

export function ScenePhone() {
  return (
    <div data-s-phone className={cn("@container relative", PLACEMENT)} aria-hidden>
      <div className="rounded-[15cqw] bg-ink p-[2.6cqw] shadow-[0_50px_90px_-40px_rgb(11_11_18/0.7)] ring-1 ring-night-line">
        <div className="relative aspect-[9/19.4] overflow-hidden rounded-[12.6cqw] bg-white text-[5.3cqw] leading-[1.35] text-ink">
          <span className="absolute top-[0.55em] left-1/2 z-30 -ml-[2.9em] h-[1.5em] w-[5.8em] rounded-full bg-black" />

          <div data-s-scr-test className={cn("absolute inset-0 bg-white", LIVE_HIDDEN)}>
            <StatusBar time="10:31" />
            <div className="px-[1.1em] pt-[0.7em]">
              <p className="text-[0.78em] text-ink-subtle">Тест · Бэкенд, Middle</p>
              <div className="mt-[0.8em] flex items-end justify-between">
                <p>
                  <span className="block text-[0.72em] text-ink-subtle">Задание</span>
                  <span className="font-mono text-[1.6em] leading-none font-semibold">9</span>
                </p>
                <span className="relative size-[2.9em]">
                  <svg viewBox="0 0 36 36" className="size-full -rotate-90">
                    <circle cx="18" cy="18" r="15" fill="none" strokeWidth="3" className="stroke-surface" />
                    <circle data-s-timer cx="18" cy="18" r="15" fill="none" strokeWidth="3" strokeLinecap="round" className="stroke-primary" />
                  </svg>
                  <span className="absolute inset-0 flex items-center justify-center font-mono text-[0.68em] font-semibold">1:24</span>
                </span>
              </div>
              <p className="mt-[1em] text-[1em] leading-[1.3] font-semibold tracking-[-0.01em]">Сервис стал отвечать медленно под нагрузкой. С чего вы начнёте?</p>
              <ul className="mt-[0.9em] space-y-[0.45em] text-[0.8em]">
                {OPTIONS.map((o, i) => (
                  <li key={o} className="relative flex items-center gap-[0.6em] rounded-[0.75em] border border-line px-[0.8em] py-[0.6em]">
                    {i === 1 && <span data-s-opt-hl className="absolute -inset-px rounded-[0.75em] border-2 border-primary bg-primary-50" />}
                    <span className="relative flex size-[1.15em] shrink-0 items-center justify-center rounded-full border-2 border-line-strong bg-white">
                      {i === 1 && <span data-s-opt-dot className="size-[0.5em] rounded-full bg-primary" />}
                    </span>
                    <span className="relative">{o}</span>
                  </li>
                ))}
              </ul>
            </div>
            <span data-s-answer className="absolute inset-x-[1.1em] bottom-[1.5em] flex h-[2.6em] items-center justify-center rounded-[0.8em] bg-primary text-[0.88em] font-medium text-white">
              Ответить
            </span>
          </div>

          <div data-s-scr-cat className={cn("absolute inset-0 bg-primary text-white", LIVE_HIDDEN)}>
            <StatusBar time="10:44" dark />
            <div className="flex h-[calc(100%-3em)] flex-col items-center justify-center px-[1.3em] text-center">
              <span data-s-check className="flex size-[3.5em] items-center justify-center rounded-full bg-white text-primary">
                <Check className="size-[1.7em]" strokeWidth={3} />
              </span>
              <p className="mt-[1em] text-[1.3em] leading-[1.1] font-semibold tracking-[-0.02em]">Категория подтверждена</p>
              <p className="mt-[0.8em] rounded-full bg-white/15 px-[0.9em] py-[0.3em] font-mono text-[0.88em]">Бэкенд · Middle</p>
              <p className="mt-[1.1em] text-[0.8em] leading-[1.4] text-primary-100">Компании видят ваш уровень и навыки. Имя и контакты скрыты.</p>
            </div>
          </div>

          <div data-s-scr-lock className={cn("absolute inset-0 bg-night text-white", LIVE_HIDDEN)}>
            <span className="absolute inset-0 bg-[radial-gradient(120%_70%_at_50%_0%,rgb(64_47_255/0.7),transparent_70%)]" />
            <StatusBar dark />
            <div className="relative mt-[1.6em] text-center">
              <p className="text-[0.8em] text-night-muted">вторник, 6 октября</p>
              <p className="mt-[0.1em] font-display text-[4em] leading-none font-semibold tracking-[-0.05em]">9:15</p>
            </div>
            <div data-s-push className="absolute inset-x-[0.6em] top-[8.6em] rounded-[1.1em] bg-white/95 p-[0.75em] text-ink shadow-[0_14px_30px_-12px_rgb(0_0_0/0.6)]">
              <div className="flex items-center gap-[0.5em]">
                <span className="flex size-[1.55em] items-center justify-center rounded-[0.45em] bg-primary">
                  <span className="size-[0.55em] rounded-full bg-white" />
                </span>
                <span className="text-[0.66em] font-semibold tracking-[0.06em] text-ink-subtle uppercase">ФСП Таланты</span>
                <span className="ml-auto text-[0.66em] text-ink-subtle">сейчас</span>
              </div>
              <p className="mt-[0.45em] text-[0.84em] leading-tight font-semibold">Приглашение: Go-разработчик</p>
              <p className="mt-[0.2em] font-mono text-[0.88em] font-semibold text-primary">280 000 – 350 000 ₽</p>
              <p className="text-[0.72em] text-ink-muted">Финтех · гибрид, Москва</p>
            </div>
            <span className="absolute bottom-[1.3em] left-[1.6em] size-[2.3em] rounded-full bg-white/15" />
            <span className="absolute right-[1.6em] bottom-[1.3em] size-[2.3em] rounded-full bg-white/15" />
          </div>

          <div data-s-scr-inv className="absolute inset-0 bg-white">
            <StatusBar time="9:16" />
            <div className="px-[1.1em] pt-[0.5em]">
              <p className="flex items-center gap-[0.15em] text-[0.8em] text-primary">
                <ChevronLeft className="size-[1.2em]" /> Приглашения
              </p>
              <p className="mt-[0.9em] text-[0.76em] text-ink-subtle">Финтех · команда платежей</p>
              <p className="mt-[0.15em] text-[1.4em] leading-[1.1] font-semibold tracking-[-0.03em]">Go-разработчик</p>
              <div className="mt-[0.9em] rounded-[0.9em] bg-primary-50 p-[0.8em]">
                <p className="text-[0.7em] text-primary-strong">Зарплата</p>
                <p className="mt-[0.1em] font-mono text-[1.08em] font-semibold tracking-[-0.02em]">280 000 – 350 000 ₽</p>
              </div>
              <ul className="mt-[0.9em] space-y-[0.5em] text-[0.8em] text-ink-muted">
                <li className="flex items-center gap-[0.5em]">
                  <MapPin className="size-[1.1em] shrink-0 text-ink-faint" /> Гибрид, Москва
                </li>
                <li className="flex items-center gap-[0.5em]">
                  <Briefcase className="size-[1.1em] shrink-0 text-ink-faint" /> Сервис выплат для партнёров
                </li>
                <li className="flex items-center gap-[0.5em]">
                  <Clock className="size-[1.1em] shrink-0 text-ink-faint" /> Ответить до 20 октября
                </li>
              </ul>
            </div>
            <div className="absolute inset-x-[1.1em] bottom-[1.5em] h-[3.4em]">
              <div data-s-actions className={cn("absolute inset-x-0 bottom-0 grid grid-cols-2 gap-[0.5em] text-[0.88em] font-medium", LIVE_HIDDEN)}>
                <span data-s-accept-btn className="relative flex h-[2.6em] items-center justify-center overflow-hidden rounded-[0.8em] bg-primary text-white">
                  <span data-s-tap className={cn("absolute top-1/2 left-1/2 -mt-[1.6em] -ml-[1.6em] size-[3.2em] rounded-full bg-white/45", LIVE_HIDDEN)} />
                  <span className="relative">Принять</span>
                </span>
                <span className="flex h-[2.6em] items-center justify-center rounded-[0.8em] border border-line-strong">Отклонить</span>
              </div>
              <div data-s-accepted className="absolute inset-0 flex items-center gap-[0.6em] rounded-[0.9em] bg-success-50 px-[0.75em] text-success">
                <span className="flex size-[1.9em] shrink-0 items-center justify-center rounded-full bg-success text-white">
                  <Check className="size-[1.1em]" strokeWidth={3} />
                </span>
                <span className="text-[0.76em] leading-[1.3]">
                  <span className="block font-semibold">Приглашение принято</span>
                  Контакты видит только эта компания
                </span>
              </div>
            </div>
          </div>

          <span data-s-dim className="pointer-events-none absolute inset-0 z-[25] bg-night opacity-0" />
          <span className="absolute bottom-[0.45em] left-1/2 z-30 -ml-[3em] h-[0.26em] w-[6em] rounded-full bg-ink/30" />
        </div>
      </div>
    </div>
  );
}
