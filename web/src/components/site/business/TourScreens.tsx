import { Check, Clock3, Inbox, LayoutGrid, LockOpen, Mail, Minus, Search, Send, UsersRound, X } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/lib/cn";

const NAV = [
  { icon: Search, label: "Подбор" },
  { icon: UsersRound, label: "Подборки" },
  { icon: Send, label: "Приглашения" },
  { icon: LayoutGrid, label: "Вакансии" },
];

function Frame({ active, title, children }: { active: number; title: string; children: ReactNode }) {
  return (
    <div className="@container flex h-full min-h-0 flex-col bg-canvas text-[12px] text-ink">
      <div className="flex h-9 shrink-0 items-center justify-between gap-3 border-b border-line bg-white px-3.5">
        <span className="flex items-center gap-2 font-semibold">
          <span className="size-2.5 rounded-[3px] bg-primary" aria-hidden />
          ФСП Таланты
          <span className="font-normal text-ink-faint">/</span>
          <span className="truncate font-normal text-ink-muted">{title}</span>
        </span>
        <span className="hidden items-center gap-1.5 text-ink-subtle @md:flex">
          <span className="flex size-5 items-center justify-center rounded-full bg-ink text-[9px] font-semibold text-white">ВК</span>
          Ваша компания
        </span>
      </div>
      <div className="flex min-h-0 flex-1">
        <nav aria-hidden className="hidden w-32 shrink-0 flex-col gap-0.5 border-r border-line bg-white p-2 @xl:flex">
          {NAV.map((n, i) => (
            <span key={n.label} className={cn("flex items-center gap-2 rounded-md px-2 py-1.5", i === active ? "bg-primary-50 font-medium text-primary" : "text-ink-muted")}>
              <n.icon className="size-3.5" />
              {n.label}
            </span>
          ))}
        </nav>
        <div className="min-w-0 flex-1 p-3.5 @xl:p-4">{children}</div>
      </div>
    </div>
  );
}

function Chip({ children, tone = "line" }: { children: ReactNode; tone?: "line" | "ok" | "warn" | "primary" }) {
  const cls = {
    line: "border border-line bg-white text-ink",
    ok: "bg-success-50 text-success",
    warn: "bg-warning-50 text-warning",
    primary: "bg-primary text-white",
  }[tone];
  return <span className={cn("inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium whitespace-nowrap", cls)}>{children}</span>;
}

const NEED = ["Go", "PostgreSQL", "Kafka", "280–350 тыс. ₽", "удалённо"];

const CATEGORIES = [
  { name: "Бэкенд · Middle", people: "46 специалистов", wait: "250–320 тыс. ₽", fit: "в вашей вилке", tone: "ok" as const, best: true },
  { name: "Бэкенд · Senior", people: "19 специалистов", wait: "340–420 тыс. ₽", fit: "выше вилки", tone: "warn" as const },
  { name: "Бэкенд · Junior", people: "63 специалиста", wait: "150–210 тыс. ₽", fit: "ниже уровня", tone: "line" as const },
];

export function ScreenCategories() {
  return (
    <Frame active={0} title="Новый подбор">
      <p className="text-[15px] font-semibold tracking-tight">Бэкенд-разработчик, Middle</p>
      <div data-pop className="mt-2 flex flex-wrap gap-1">
        {NEED.map((n) => (
          <Chip key={n}>{n}</Chip>
        ))}
      </div>
      <p className="mt-4 text-[11px] font-medium tracking-wide text-ink-subtle uppercase">Подходящие категории</p>
      <ul className="mt-2 space-y-1.5">
        {CATEGORIES.map((c) => (
          <li data-pop key={c.name} className={cn("rounded-lg border bg-white p-2.5", c.best ? "border-primary-300 shadow-[0_0_0_3px_rgb(64_47_255/0.08)]" : "border-line")}>
            <div className="flex items-center justify-between gap-2">
              <span className="font-semibold">{c.name}</span>
              <Chip tone={c.tone}>{c.fit}</Chip>
            </div>
            <div className="mt-1 flex flex-wrap gap-x-4 gap-y-0.5 text-ink-muted">
              <span>{c.people}</span>
              <span>ожидают {c.wait}</span>
            </div>
          </li>
        ))}
      </ul>
      <div className="mt-3 flex justify-end">
        <span className="rounded-md bg-primary px-3 py-1.5 font-medium text-white">Открыть подборку</span>
      </div>
    </Frame>
  );
}

const LIST = [
  { code: "K-5F8A", score: 91, on: true },
  { code: "K-71B0", score: 84 },
  { code: "K-C422", score: 78 },
  { code: "K-2D55", score: 73 },
];

const SKILLS: { name: string; src: string; tone: "ok" | "primary" | "line" }[] = [
  { name: "Go", src: "тест", tone: "ok" },
  { name: "Kafka", src: "тест", tone: "ok" },
  { name: "PostgreSQL", src: "резюме", tone: "line" },
  { name: "Docker", src: "заявлен", tone: "line" },
];

function Ring({ value, size = "md" }: { value: number; size?: "sm" | "md" }) {
  const c = 2 * Math.PI * 17;
  return (
    <span className={cn("relative flex shrink-0 items-center justify-center", size === "sm" ? "size-7" : "size-11")}>
      <svg viewBox="0 0 40 40" className="absolute inset-0 -rotate-90" aria-hidden>
        <circle cx="20" cy="20" r="17" fill="none" strokeWidth="3.5" className="stroke-primary-100" />
        <circle cx="20" cy="20" r="17" fill="none" strokeWidth="3.5" strokeLinecap="round" className="stroke-primary" strokeDasharray={`${(c * value) / 100} ${c}`} />
      </svg>
      <span className={cn("tabular font-semibold", size === "sm" ? "text-[9px]" : "text-[12px]")}>{value}</span>
    </span>
  );
}

export function ScreenShortlist() {
  return (
    <Frame active={1} title="Подборка · Бэкенд, Middle">
      <div className="flex h-full min-h-0 gap-3">
        <ul className="hidden w-36 shrink-0 space-y-1 @lg:block">
          {LIST.map((l) => (
            <li key={l.code} className={cn("flex items-center gap-2 rounded-lg border px-2 py-1.5", l.on ? "border-primary-300 bg-white" : "border-transparent text-ink-muted")}>
              <Ring value={l.score} size="sm" />
              {l.code}
            </li>
          ))}
        </ul>
        <div data-pop className="min-w-0 flex-1 rounded-lg border border-line bg-white p-3">
          <div className="flex items-center gap-3">
            <Ring value={91} />
            <div className="min-w-0 flex-1">
              <p className="text-[14px] font-semibold">Специалист K-5F8A</p>
              <p className="text-ink-subtle">Бэкенд · Middle · тест сдан уверенно</p>
            </div>
            <span className="hidden rounded-md bg-primary px-2.5 py-1 font-medium text-white @md:inline">Пригласить</span>
          </div>
          <div className="mt-2.5 flex flex-wrap gap-1">
            {SKILLS.map((s) => (
              <Chip key={s.name} tone={s.tone}>
                {s.name}
                <span className="font-normal">· {s.src}</span>
              </Chip>
            ))}
          </div>
          <div className="mt-3 grid grid-cols-1 gap-2.5 @md:grid-cols-2">
            <div data-pop>
              <p className="text-[11px] font-medium text-ink-subtle">Почему в подборке</p>
              <ul className="mt-1 space-y-1">
                {["Все обязательные навыки: Go, PostgreSQL, Kafka", "Ожидания внутри вашей вилки", "Призёр соревнований ФСП"].map((r) => (
                  <li key={r} className="flex items-start gap-1.5">
                    <Check className="mt-px size-3.5 shrink-0 text-success" strokeWidth={2.5} aria-hidden />
                    {r}
                  </li>
                ))}
              </ul>
            </div>
            <div data-pop>
              <p className="text-[11px] font-medium text-ink-subtle">Чего не хватает</p>
              <ul className="mt-1 space-y-1 text-ink-muted">
                {["Нет Kubernetes", "Docker только заявлен"].map((r) => (
                  <li key={r} className="flex items-start gap-1.5">
                    <Minus className="mt-px size-3.5 shrink-0 text-warning" strokeWidth={2.5} aria-hidden />
                    {r}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </div>
    </Frame>
  );
}

function Row({ label, children, accent = false }: { label: string; children: ReactNode; accent?: boolean }) {
  return (
    <div className="grid grid-cols-[6.5rem_1fr] items-center gap-2">
      <span className="text-ink-subtle">
        {label}
        {accent && <span className="block text-[10px] font-medium text-primary">обязательно</span>}
      </span>
      <span className={cn("rounded-md border bg-white px-2 py-1.5", accent ? "border-primary text-ink shadow-[0_0_0_3px_rgb(64_47_255/0.1)]" : "border-line")}>{children}</span>
    </div>
  );
}

export function ScreenInvite() {
  return (
    <Frame active={2} title="Приглашение для K-5F8A">
      <div className="grid min-h-0 grid-cols-1 items-start gap-3 @xl:grid-cols-[1.25fr_1fr]">
        <div className="space-y-1.5">
          <Row label="Что предлагаем">Go-разработчик в команду платежей</Row>
          <div data-pop>
            <Row label="Вилка, ₽" accent>
              <span className="font-semibold">
                от <span className="whitespace-nowrap">280 000</span> до <span className="whitespace-nowrap">350 000</span>
              </span>
            </Row>
          </div>
          <Row label="Формат">Удалённо, можно приезжать в офис</Row>
          <Row label="Сообщение">
            <span className="line-clamp-2 text-ink-muted">Здравствуйте. Мы переносим платежи на Kafka и ищем человека, который будет отвечать за очередь событий.</span>
          </Row>
          <div className="flex justify-end pt-1">
            <span className="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 font-medium text-white">
              <Send className="size-3.5" aria-hidden />
              Отправить
            </span>
          </div>
        </div>
        <div data-pop className="hidden rounded-lg border border-line bg-white p-3 @xl:block">
          <p className="text-[11px] font-medium text-ink-subtle">Так приглашение увидит специалист</p>
          <p className="mt-2 font-display text-[18px] font-semibold tracking-tight text-primary">280–350 тыс. ₽</p>
          <p className="mt-1 font-semibold">Go-разработчик в команду платежей</p>
          <p className="text-ink-muted">Финтех · удалённо</p>
          <div className="mt-3 flex gap-1.5">
            <span className="rounded-md bg-ink px-2.5 py-1 font-medium text-white">Принять</span>
            <span className="rounded-md border border-line px-2.5 py-1 text-ink-muted">Отклонить</span>
          </div>
        </div>
      </div>
    </Frame>
  );
}

export function ScreenAnswer() {
  return (
    <Frame active={2} title="Приглашения">
      <div className="grid min-h-0 grid-cols-1 items-start gap-3 @xl:grid-cols-[1.3fr_1fr]">
        <div className="space-y-1.5">
          <div data-pop className="rounded-lg border border-success-200 bg-white p-2.5">
            <div className="flex items-center justify-between gap-2">
              <span className="font-semibold">K-5F8A принял приглашение</span>
              <Chip tone="ok">
                <LockOpen className="size-3" aria-hidden />
                контакты открыты
              </Chip>
            </div>
            <dl className="mt-2 grid grid-cols-[4.5rem_1fr] gap-x-2 gap-y-1">
              <dt className="text-ink-subtle">Имя</dt>
              <dd data-reveal-text="Андрей Соколов" className="font-medium">Андрей Соколов</dd>
              <dt className="text-ink-subtle">Telegram</dt>
              <dd data-reveal-text="@sokolov_go" className="font-mono">@sokolov_go</dd>
              <dt className="text-ink-subtle">Почта</dt>
              <dd data-reveal-text="andrey@example.com" className="font-mono">andrey@example.com</dd>
            </dl>
          </div>
          <div data-pop className="flex items-center justify-between gap-2 rounded-lg border border-line bg-white p-2.5">
            <span className="flex items-center gap-2">
              <X className="size-3.5 text-accent-strong" aria-hidden />
              K-71B0 отклонил
            </span>
            <Chip tone="warn">причина: формат работы</Chip>
          </div>
          <div data-pop className="flex items-center justify-between gap-2 rounded-lg border border-line bg-white p-2.5">
            <span className="flex items-center gap-2 text-ink-muted">
              <Clock3 className="size-3.5" aria-hidden />
              K-C422 ещё думает
            </span>
            <span className="text-ink-subtle">ответ в течение 14 дней</span>
          </div>
          <div data-pop className="flex items-center justify-between gap-2 rounded-lg border border-line bg-white p-2.5">
            <span className="flex items-center gap-2">
              <Check className="size-3.5 text-success" strokeWidth={2.5} aria-hidden />
              K-2D55 принял приглашение
            </span>
            <Chip tone="ok">
              <LockOpen className="size-3" aria-hidden />
              контакты открыты
            </Chip>
          </div>
        </div>
        <div data-pop className="hidden rounded-lg border border-line bg-white p-3 @xl:block">
          <p className="flex items-center gap-1.5 text-[11px] font-medium text-ink-subtle">
            <Inbox className="size-3.5" aria-hidden />
            Почему отказывают
          </p>
          <ol className="mt-2 space-y-1.5">
            {["Формат работы", "Зарплата", "Стек"].map((r, i) => (
              <li key={r} className="flex items-center gap-2">
                <span className="flex size-4 items-center justify-center rounded-full bg-canvas text-[10px] text-ink-subtle">{i + 1}</span>
                {r}
              </li>
            ))}
          </ol>
          <p className="mt-3 flex items-start gap-1.5 text-ink-muted">
            <Mail className="mt-px size-3.5 shrink-0" aria-hidden />
            Чаще всего мешает офис. Попробуйте гибрид.
          </p>
        </div>
      </div>
    </Frame>
  );
}

export const SCREENS = [ScreenCategories, ScreenShortlist, ScreenInvite, ScreenAnswer];
