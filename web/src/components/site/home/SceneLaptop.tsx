import { Check, Clock, Inbox, Lock, Mail, Phone, Search, Settings, Users } from "lucide-react";
import { cn } from "@/lib/cn";

const ROWS = [
  { code: "К-5F8A", meta: "4 года опыта", skills: ["Go", "PostgreSQL"], pay: "от 270 тыс." },
  { code: "К-91C0", meta: "6 лет опыта", skills: ["Go", "Kafka"], pay: "от 320 тыс." },
  { code: "К-3D27", meta: "3 года опыта", skills: ["Go", "Redis"], pay: "от 240 тыс." },
  { code: "К-E4B9", meta: "5 лет опыта", skills: ["Python", "Go"], pay: "от 300 тыс." },
  { code: "К-07AD", meta: "4 года опыта", skills: ["Go", "gRPC"], pay: "от 290 тыс." },
];

const LIVE_HIDDEN = "invisible opacity-0";

function Chip({ children, confirmed = false }: { children: string; confirmed?: boolean }) {
  return (
    <span className={cn("inline-flex items-center gap-[0.25em] rounded-[0.4em] border px-[0.45em] py-[0.12em] text-[0.78em]", confirmed ? "border-success-200 bg-success-50 text-success" : "border-line bg-white text-ink-muted")}>
      {confirmed && <Check className="size-[1em]" strokeWidth={3} aria-hidden />}
      {children}
    </span>
  );
}

/** Company cabinet on a laptop. Server markup shows the final state (contacts open); the scene timeline rewinds it. */
const PLACEMENT =
  "w-full lg:w-[72%] group-data-live/scene:absolute group-data-live/scene:top-0 group-data-live/scene:right-0 group-data-live/scene:w-[min(100cqw,82cqh)] lg:group-data-live/scene:top-auto lg:group-data-live/scene:right-auto lg:group-data-live/scene:bottom-[5%] lg:group-data-live/scene:left-0 lg:group-data-live/scene:w-[min(63cqw,132cqh)]";

export function SceneLaptop() {
  return (
    <div data-s-laptop className={cn("@container relative", PLACEMENT)} aria-hidden>
      <div className="rounded-t-[2.4cqw] bg-ink p-[1.3cqw] pb-[1.1cqw] shadow-[0_40px_80px_-40px_rgb(27_28_33/0.55)]">
        <div data-s-screen className="relative aspect-[16/10.4] overflow-hidden rounded-[0.7cqw] bg-white text-[2.8cqw] leading-[1.35] text-ink @min-[36rem]:text-[1.8cqw]">
          <div className="flex h-[2.3em] items-center gap-[0.4em] border-b border-line bg-canvas px-[0.9em]">
            <span className="size-[0.6em] rounded-full bg-line-strong" />
            <span className="size-[0.6em] rounded-full bg-line-strong" />
            <span className="size-[0.6em] rounded-full bg-line-strong" />
            <span className="mx-auto rounded-[0.4em] border border-line bg-white px-[1.6em] py-[0.15em] text-[0.78em] text-ink-subtle">Кабинет компании · Подборка</span>
          </div>

          <div className="flex h-[calc(100%-2.3em)]">
            <div className="hidden w-[3.4em] shrink-0 flex-col items-center gap-[1.15em] bg-night py-[1em] text-night-faint @min-[36rem]:flex">
              <span className="size-[1.3em] rounded-[0.35em] bg-primary" />
              <Users className="size-[1.15em] text-white" />
              <Inbox className="size-[1.15em]" />
              <Search className="size-[1.15em]" />
              <Settings className="mt-auto size-[1.15em]" />
            </div>

            <div className="min-w-0 flex-1 px-[0.9em] py-[0.8em] @min-[36rem]:px-[1.2em] @min-[36rem]:py-[1em]">
              <div className="flex items-baseline justify-between gap-[0.6em]">
                <p className="text-[1.3em] font-semibold tracking-[-0.02em]">Подборка</p>
                <p className="truncate text-[0.78em] text-ink-subtle">Бэкенд · Middle</p>
              </div>
              <div className="mt-[0.5em] flex flex-wrap gap-[0.35em]">
                <Chip>Гибрид</Chip>
                <Chip>Москва</Chip>
                <Chip>до 350 тыс. ₽</Chip>
              </div>
              <ul className="mt-[0.8em] border-t border-line">
                {ROWS.map((r, i) => (
                  <li key={r.code} data-s-row={i} className={cn("relative flex items-center gap-[0.6em] border-b border-line px-[0.4em] py-[0.55em]", i >= 3 && "hidden @min-[36rem]:flex")}>
                    {i === 0 && <span data-s-row-hl className="absolute inset-0 rounded-[0.5em] bg-primary-50 ring-1 ring-primary-200" />}
                    <span className="relative flex size-[2em] shrink-0 items-center justify-center rounded-full bg-surface text-[0.85em] font-semibold text-ink-muted">К</span>
                    <span className="relative min-w-0 flex-1">
                      <span className="block truncate font-semibold">Кандидат {r.code}</span>
                      <span className="block truncate text-[0.8em] text-ink-subtle">{r.meta}</span>
                    </span>
                    <span className="relative hidden gap-[0.3em] @min-[36rem]:flex">
                      {r.skills.map((s) => (
                        <Chip key={s} confirmed>
                          {s}
                        </Chip>
                      ))}
                    </span>
                    <span className="relative shrink-0 font-mono text-[0.82em] text-ink-muted">{r.pay}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="flex w-[44%] shrink-0 flex-col border-l border-line bg-canvas p-[0.8em] @min-[36rem]:w-[37%] @min-[36rem]:p-[1em]">
              <div className="relative min-h-0 flex-1">
                <p data-s-empty className={cn("absolute inset-0 flex items-center justify-center px-[1em] text-center text-[0.85em] text-ink-faint", LIVE_HIDDEN)}>
                  Выберите человека из подборки
                </p>
                <div data-s-profile className="flex h-full flex-col">
                  <p className="font-mono text-[0.68em] tracking-[0.08em] text-ink-subtle uppercase">Профиль без имени</p>
                  <p className="mt-[0.25em] text-[1.12em] font-semibold tracking-[-0.01em]">Кандидат К-5F8A</p>
                  <p className="mt-[0.2em] flex items-center gap-[0.3em] text-[0.8em] text-success">
                    <Check className="size-[1.1em] shrink-0" strokeWidth={3} />
                    Бэкенд · Middle, тест пройден
                  </p>
                  <div className="mt-[0.6em] hidden flex-wrap gap-[0.3em] @min-[36rem]:flex">
                    <Chip confirmed>Go</Chip>
                    <Chip confirmed>PostgreSQL</Chip>
                    <Chip>Kafka</Chip>
                  </div>
                  <div className="mt-[0.6em] grid grid-cols-2 gap-[0.4em] text-[0.8em]">
                    <p>
                      <span className="block text-ink-subtle">Ожидания</span>
                      <span className="font-mono font-semibold">от 270 000 ₽</span>
                    </p>
                    <p>
                      <span className="block text-ink-subtle">Формат</span>
                      <span className="font-semibold">гибрид</span>
                    </p>
                  </div>
                  <div className="relative mt-auto h-[5.6em] rounded-[0.6em] border border-line bg-white">
                    <div data-s-lock className={cn("absolute inset-0 flex flex-col justify-center gap-[0.4em] p-[0.7em]", LIVE_HIDDEN)}>
                      <span className="flex items-center gap-[0.4em] text-[0.78em] text-ink-muted">
                        <Lock className="size-[1.1em] shrink-0" />
                        Контакты откроются после ответа
                      </span>
                      <span className="h-[0.5em] w-[72%] rounded-full bg-surface" />
                      <span className="h-[0.5em] w-[54%] rounded-full bg-surface" />
                    </div>
                    <div data-s-open className="absolute inset-0 flex flex-col justify-center gap-[0.22em] p-[0.7em] text-[0.8em]">
                      <span data-s-open-item className="text-[1.12em] font-semibold">
                        Анна Соколова
                      </span>
                      <span data-s-open-item className="flex items-center gap-[0.4em] text-ink-muted">
                        <Mail className="size-[1.05em] shrink-0" /> anna.s@example.com
                      </span>
                      <span data-s-open-item className="flex items-center gap-[0.4em] text-ink-muted">
                        <Phone className="size-[1.05em] shrink-0" /> +7 912 345-67-89
                      </span>
                    </div>
                  </div>
                </div>
              </div>
              <div data-s-origin className="relative mt-[0.7em] h-[2.4em] shrink-0 text-[0.85em] font-medium">
                <span data-s-btn-invite className={cn("absolute inset-0 flex items-center justify-center rounded-[0.55em] bg-primary text-white", LIVE_HIDDEN)}>
                  Пригласить
                </span>
                <span data-s-btn-sent className={cn("absolute inset-0 flex items-center justify-center gap-[0.35em] rounded-[0.55em] border border-line-strong bg-white text-ink-muted", LIVE_HIDDEN)}>
                  <Clock className="size-[1.1em]" /> Ждём ответа
                </span>
                <span data-s-btn-accepted className="absolute inset-0 flex items-center justify-center gap-[0.35em] rounded-[0.55em] bg-success-50 text-success">
                  <Check className="size-[1.1em]" strokeWidth={3} /> Приглашение принято
                </span>
              </div>
            </div>
          </div>

          <span data-s-dim className="pointer-events-none absolute inset-0 z-[5] bg-night opacity-0" />
          <svg data-s-cursor viewBox="0 0 24 24" className={cn("absolute top-0 left-0 z-10 size-[1.9em] drop-shadow-[0_2px_3px_rgb(0_0_0/0.25)]", LIVE_HIDDEN)}>
            <path d="M3 2.5v17.2l4.6-4.4 3.1 7 3-1.3-3-6.8h6.3z" className="fill-ink stroke-white" strokeWidth="1.4" strokeLinejoin="round" />
          </svg>
        </div>
      </div>
      <div className="relative mx-[-4.5%] h-[2.1cqw] rounded-t-[0.4cqw] rounded-b-[2.2cqw] bg-linear-to-b from-line-strong to-ink-faint">
        <span className="absolute top-0 left-1/2 h-[0.75cqw] w-[15%] -translate-x-1/2 rounded-b-[0.8cqw] bg-ink-faint" />
      </div>
    </div>
  );
}
