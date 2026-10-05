import { BadgeCheck, Check, KeyRound, Pin, ScrollText, ShieldCheck, Trophy, UserRound } from "lucide-react";
import { cn } from "@/lib/cn";

const INVITES = [
  { title: "Go-разработчик в команду платежей", meta: "Финтех · гибрид, Москва", salary: "280 000 – 350 000 ₽", status: "Новое", fresh: true },
  { title: "Backend в маркетплейс: заказы и склад", meta: "E-commerce · удалённо", salary: "300 000 – 380 000 ₽", status: "Просмотрено", fresh: false },
  { title: "Платформенная команда, Kafka и PostgreSQL", meta: "Телеком · офис, Казань", salary: "260 000 – 320 000 ₽", status: "Просмотрено", fresh: false },
];

export function InviteStack() {
  return (
    <div className="relative mx-auto h-[25rem] w-full max-w-[27rem] sm:h-[27rem]">
      {INVITES.map((inv, i) => (
        <div
          key={inv.title}
          data-tilt={i === 0 ? "6" : undefined}
          className={cn(
            "absolute inset-x-0 rounded-[24px] border border-line p-5 sm:p-6",
            i === 0 && "top-24 z-30 bg-white shadow-lift",
            i === 1 && "top-12 z-20 mx-5 bg-canvas shadow-pop",
            i === 2 && "top-0 z-10 mx-10 bg-surface shadow-card",
          )}
        >
          <div className="flex items-center justify-between gap-3 text-xs">
            <span className={cn("inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 font-medium", inv.fresh ? "bg-primary text-white" : "bg-canvas text-ink-subtle")}>{inv.status}</span>
            <span className={cn("font-mono text-ink-faint", i > 0 && "hidden sm:inline")}>истекает через {14 - i * 3} дн.</span>
          </div>
          <p className="mt-4 text-lg leading-snug font-semibold tracking-tight text-ink">{inv.title}</p>
          <p className="mt-1 text-sm text-ink-muted">{inv.meta}</p>
          {i === 0 && (
            <>
              <div className="mt-5 rounded-2xl bg-canvas p-4">
                <p className="text-xs text-ink-subtle">Зарплатная вилка</p>
                <p className="tabular mt-1 font-mono text-2xl font-semibold tracking-tight text-ink">{inv.salary}</p>
              </div>
              <p className="mt-4 flex items-center gap-2 text-sm text-ink">
                <Check className="size-4 text-success" aria-hidden /> Связь: созвон с тимлидом в удобное время
              </p>
              <div className="mt-5 grid grid-cols-2 gap-2">
                <span className="inline-flex h-11 items-center justify-center rounded-xl bg-primary text-sm font-medium text-white">Принять</span>
                <span className="inline-flex h-11 items-center justify-center rounded-xl border border-line-strong text-sm font-medium text-ink">Отклонить</span>
              </div>
            </>
          )}
        </div>
      ))}
    </div>
  );
}

const ROWS = [
  { code: "K-5F8A1F", score: 88, note: "Все обязательные навыки, 2 подтверждены тестом", pinned: true, isNew: false },
  { code: "K-9073DC", score: 68, note: "Вместо PostgreSQL — близкий навык MySQL", pinned: false, isNew: true },
  { code: "K-C42234", score: 76, note: "Грейд Senior — на ступень выше запроса", pinned: false, isNew: false },
];

export function ShortlistMock() {
  return (
    <div data-tilt="5" className="rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-6">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Потребность</p>
          <p className="mt-1 text-lg font-semibold tracking-tight text-ink">Middle Go-разработчик · платежи</p>
        </div>
        <span className="tabular shrink-0 rounded-full bg-canvas px-2.5 py-1 font-mono text-xs text-ink-muted">250–320 тыс. ₽</span>
      </div>
      <div className="mt-4 grid grid-cols-3 gap-2 rounded-2xl bg-canvas p-3 text-center">
        {[
          ["11", "кандидатов в Backend · Middle"],
          ["2", "с достижениями ФСП"],
          ["280 тыс.", "медиана ожиданий"],
        ].map(([v, l]) => (
          <div key={l}>
            <p className="tabular font-mono text-base font-semibold text-ink">{v}</p>
            <p className="mt-0.5 text-[11px] leading-tight text-ink-subtle">{l}</p>
          </div>
        ))}
      </div>
      <ul className="mt-4 divide-y divide-line">
        {ROWS.map((r) => (
          <li key={r.code} className="flex items-center gap-3 py-3">
            <span className="flex size-9 shrink-0 items-center justify-center rounded-full bg-canvas text-ink-subtle">
              <UserRound className="size-4" aria-hidden />
            </span>
            <div className="min-w-0 flex-1">
              <p className="flex items-center gap-2 font-mono text-sm font-semibold text-ink">
                {r.code}
                {r.pinned && <Pin className="size-3.5 text-primary" aria-label="закреплён" />}
                {r.isNew && <span className="rounded bg-primary-50 px-1.5 py-0.5 font-sans text-[10px] font-medium text-primary-strong">новый</span>}
              </p>
              <p className="truncate text-xs text-ink-muted">{r.note}</p>
            </div>
            <span className="relative flex size-11 shrink-0 items-center justify-center">
              <svg viewBox="0 0 44 44" className="absolute inset-0 -rotate-90" aria-hidden>
                <circle cx="22" cy="22" r="18" fill="none" strokeWidth="3.5" className="stroke-surface" />
                <circle cx="22" cy="22" r="18" fill="none" strokeWidth="3.5" strokeLinecap="round" strokeDasharray="113.1" strokeDashoffset={113.1 * (1 - r.score / 100)} className="stroke-primary" />
              </svg>
              <span className="tabular font-mono text-xs font-semibold text-ink">{r.score}</span>
            </span>
          </li>
        ))}
      </ul>
      <div className="mt-2 flex items-center justify-between gap-3 border-t border-line pt-4">
        <span className="text-xs text-ink-subtle">Квота: 12 из 30 за 7 дней</span>
        <span className="inline-flex h-9 items-center rounded-full bg-ink px-4 text-xs font-medium text-white">Пригласить с вилкой</span>
      </div>
    </div>
  );
}

const FLOW = [
  { icon: UserRound, title: "Кандидат", text: "нажимает «Привязать ФСП ID»" },
  { icon: KeyRound, title: "ФСП ID · Keycloak", text: "вход участника, OIDC + PKCE S256, state и nonce" },
  { icon: ShieldCheck, title: "Проверка id_token", text: "подпись по JWKS, iss, aud, nonce → claim fsp_id" },
  { icon: ScrollText, title: "Реестр ФСП", text: "разряды и соревнования по fsp_id" },
  { icon: Trophy, title: "Снимок в профиле", text: "ФСП-скор → карточка и ранжирование" },
];

export function FspFlow() {
  return (
    <div className="relative rounded-[26px] border border-line bg-white p-5 shadow-lift sm:p-7">
      <p className="font-mono text-[11px] tracking-[0.12em] text-ink-subtle uppercase">Привязка ФСП ID</p>
      <ol className="relative mt-5 space-y-3">
        <svg data-draw className="absolute top-6 bottom-6 left-[21px] h-[calc(100%-3rem)] w-2" viewBox="0 0 8 100" preserveAspectRatio="none" aria-hidden>
          <line x1="4" y1="0" x2="4" y2="100" stroke="currentColor" strokeWidth="1.5" className="text-primary-300" />
        </svg>
        {FLOW.map(({ icon: Icon, title, text }, i) => (
          <li key={title} className="relative flex items-center gap-4">
            <span className={cn("relative z-10 flex size-11 shrink-0 items-center justify-center rounded-full border", i === FLOW.length - 1 ? "border-primary bg-primary text-white" : "border-line bg-white text-primary")}>
              <Icon className="size-4.5" aria-hidden />
            </span>
            <span className="min-w-0 rounded-2xl border border-line bg-canvas/60 px-4 py-2.5">
              <span className="block text-sm font-semibold text-ink">{title}</span>
              <span className="block text-xs text-ink-muted">{text}</span>
            </span>
          </li>
        ))}
      </ol>
      <p className="mt-5 flex items-center gap-2 text-xs text-ink-subtle">
        <BadgeCheck className="size-4 text-success" aria-hidden /> Один ФСП ID — один профиль. Отвязка удаляет снимок достижений.
      </p>
    </div>
  );
}
