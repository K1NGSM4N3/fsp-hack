import Link from "next/link";
import type { ReactNode } from "react";
import { Logo } from "@/components/brand/Logo";

const ROUTES = [
  { href: "/seekers", title: "Соискателям", text: "Один тест, и компании пишут сами" },
  { href: "/business", title: "Компаниям", text: "Подборка с проверенным уровнем" },
  { href: "/vacancies", title: "Вакансии", text: "Открытые вакансии с вилкой от и до" },
  { href: "/fsp", title: "Участникам ФСП", text: "Разряды и призовые места в профиле" },
];

export const stateButton = "inline-flex h-12 items-center justify-center gap-2 rounded-full px-6 font-medium transition-colors";

/**
 * Full-screen night page for 404 and fatal errors: brand header, a large code, the message and the main routes.
 * Kept to plain markup without inline icons: Next embeds the root not-found tree in the payload of every page.
 */
export function SiteState({ code, title, text, actions }: { code: string; title: string; text: ReactNode; actions: ReactNode }) {
  return (
    <div className="relative isolate flex min-h-dvh flex-1 flex-col overflow-hidden bg-night bg-[radial-gradient(46rem_46rem_at_100%_-10rem,rgb(64_47_255/0.5),transparent_62%)] text-white">
      <header className="mx-auto flex w-full max-w-[1240px] items-center justify-between px-4 pt-5 sm:px-6 lg:px-8">
        <Logo tone="dark" />
        <Link href="/login" className="inline-flex min-h-10 items-center px-2 text-sm text-night-muted hover:text-white">
          Войти в кабинет
        </Link>
      </header>
      <main id="main" className="mx-auto flex w-full max-w-[1240px] flex-1 flex-col justify-center px-4 py-16 sm:px-6 lg:px-8">
        <p aria-hidden className="font-display text-[clamp(5.5rem,22vw,13rem)] leading-[0.85] font-semibold tracking-[-0.06em] text-transparent [-webkit-text-stroke:1.5px_rgb(169_163_255/0.8)]">
          {code}
        </p>
        <h1 className="mt-8 max-w-2xl text-[clamp(2rem,1.4rem+2vw,3rem)] leading-[1.05] font-semibold tracking-[-0.035em] text-balance">{title}</h1>
        <div className="mt-4 max-w-xl text-lg leading-relaxed text-night-muted">{text}</div>
        <div className="mt-9 flex flex-wrap gap-3">{actions}</div>
        <nav aria-label="Разделы сайта" className="mt-16 border-t border-night-line pt-8">
          <p className="eyebrow text-night-faint">Куда дальше</p>
          <ul className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            {ROUTES.map((r) => (
              <li key={r.href}>
                <Link href={r.href} className="block h-full rounded-card border border-white/10 bg-white/[0.03] p-4 hover:border-white/25 hover:bg-white/[0.06]">
                  <span className="block font-medium">{r.title}</span>
                  <span className="mt-1 block text-sm text-night-muted">{r.text}</span>
                </Link>
              </li>
            ))}
          </ul>
        </nav>
      </main>
    </div>
  );
}
