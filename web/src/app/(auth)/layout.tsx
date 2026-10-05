import { BadgeCheck, Banknote, ShieldCheck, Trophy } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { Logo } from "@/components/brand/Logo";
import { HeroGraph } from "@/components/site/HeroGraph";

const POINTS = [
  { icon: BadgeCheck, text: "Категорию даёт адаптивный тест, обычно за 10–25 минут" },
  { icon: Banknote, text: "Зарплата указана в первом же сообщении от компании" },
  { icon: Trophy, text: "Достижения ФСП поднимают в подборке, без них штрафа нет" },
  { icon: ShieldCheck, text: "Контакты компания увидит, только когда вы нажмёте «Принять»" },
];

export default function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div className="grid min-h-dvh flex-1 lg:grid-cols-[1fr_minmax(0,34rem)]">
      <div className="relative flex flex-col bg-white">
        <div aria-hidden className="bg-grid mask-radial pointer-events-none absolute inset-0 opacity-50" />
        <header className="relative flex h-16 items-center justify-between px-4 sm:px-8">
          <Logo />
          <Link href="/" className="text-sm text-ink-muted transition-colors hover:text-primary">
            На сайт
          </Link>
        </header>
        <main id="main" className="relative flex flex-1 items-start justify-center px-4 pt-6 pb-12 sm:px-8 sm:pt-12">
          <div className="w-full max-w-md animate-rise">{children}</div>
        </main>
      </div>
      <aside className="relative isolate hidden overflow-hidden bg-night text-white lg:flex lg:flex-col lg:justify-center lg:px-12">
        <div aria-hidden className="pointer-events-none absolute -top-40 -right-40 -z-10 size-[34rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.6),transparent_65%)]" />
        <div aria-hidden className="absolute inset-0 -z-10 opacity-45">
          <HeroGraph />
        </div>
        <div className="relative">
          <p className="font-mono text-xs tracking-[0.14em] text-primary-300 uppercase">ФСП Таланты</p>
          <p className="mt-5 text-[2.4rem] leading-[1.05] font-semibold tracking-[-0.04em]">Один тест, и компании пишут вам сами.</p>
          <ul className="mt-10 space-y-5">
            {POINTS.map(({ icon: Icon, text }) => (
              <li key={text} className="flex gap-4 text-night-muted">
                <span className="flex size-9 shrink-0 items-center justify-center rounded-[10px] border border-white/10 bg-white/5 text-primary-300">
                  <Icon className="size-4.5" aria-hidden />
                </span>
                <span className="pt-1.5">{text}</span>
              </li>
            ))}
          </ul>
        </div>
      </aside>
    </div>
  );
}
