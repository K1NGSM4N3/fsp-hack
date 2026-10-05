"use client";

import { Menu, X } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { Logo } from "@/components/brand/Logo";
import { ButtonLink, IconButton } from "@/components/ui";
import { useMe } from "@/features/auth/hooks";
import { cn } from "@/lib/cn";

const LINKS = [
  { href: "/#candidates", label: "Кандидатам" },
  { href: "/#employers", label: "Работодателям" },
  { href: "/#how", label: "Как это работает" },
  { href: "/vacancies", label: "Вакансии" },
];

function AuthButtons({ stacked = false }: { stacked?: boolean }) {
  const me = useMe();
  if (me.data) {
    const href = me.data.role === "employer" ? "/employer" : "/candidate";
    return (
      <ButtonLink href={href} size="sm" className={cn(stacked && "w-full")}>
        Мой кабинет
      </ButtonLink>
    );
  }
  return (
    <div className={cn("flex gap-2", stacked && "flex-col")}>
      <ButtonLink href="/login" variant="ghost" size="sm" className={cn(stacked && "w-full")}>
        Войти
      </ButtonLink>
      <ButtonLink href="/register" size="sm" className={cn(stacked && "w-full")}>
        Регистрация
      </ButtonLink>
    </div>
  );
}

export function PublicHeader() {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);
  return (
    <header className="sticky top-0 z-30 border-b border-line bg-white/95 backdrop-blur">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between gap-4 px-4 sm:px-6">
        <Logo />
        <nav aria-label="Основная навигация" className="hidden items-center gap-1 md:flex">
          {LINKS.map((l) => (
            <Link
              key={l.href}
              href={l.href}
              className={cn(
                "rounded-control px-3 py-2 text-sm font-medium transition-colors hover:text-primary",
                pathname.startsWith(l.href) && l.href !== "/" ? "text-primary" : "text-ink-muted",
              )}
            >
              {l.label}
            </Link>
          ))}
        </nav>
        <div className="hidden md:block">
          <AuthButtons />
        </div>
        <IconButton label={open ? "Закрыть меню" : "Открыть меню"} className="md:hidden" onClick={() => setOpen((v) => !v)} aria-expanded={open}>
          {open ? <X className="size-5" aria-hidden /> : <Menu className="size-5" aria-hidden />}
        </IconButton>
      </div>
      {open && (
        <div className="border-t border-line bg-white px-4 py-4 md:hidden">
          <nav aria-label="Основная навигация" className="mb-4 flex flex-col">
            {LINKS.map((l) => (
              <Link key={l.href} href={l.href} onClick={() => setOpen(false)} className="rounded-control px-2 py-2.5 text-sm font-medium text-ink hover:text-primary">
                {l.label}
              </Link>
            ))}
          </nav>
          <AuthButtons stacked />
        </div>
      )}
    </header>
  );
}

export function PublicFooter() {
  return (
    <footer className="mt-auto bg-ink text-surface">
      <div className="mx-auto grid max-w-6xl gap-8 px-4 py-10 sm:px-6 md:grid-cols-[1.5fr_1fr_1fr]">
        <div>
          <Image src="/brand/fsp-logo-full-dark.svg" alt="Федерация спортивного программирования России" width={320} height={40} unoptimized loading="eager" className="h-8 w-auto" />
          <p className="mt-4 max-w-sm text-sm text-ink-faint">
            Платформа подбора ИТ-специалистов: категория подтверждена тестом, достижения ФСП усиливают профиль, работодатель приходит с вилкой.
          </p>
        </div>
        <div>
          <p className="font-mono text-xs tracking-wider text-ink-faint uppercase">Кандидатам</p>
          <ul className="mt-3 space-y-2 text-sm">
            <li>
              <Link href="/register?role=candidate" className="hover:text-white hover:underline">
                Пройти тест и получить категорию
              </Link>
            </li>
            <li>
              <Link href="/vacancies" className="hover:text-white hover:underline">
                Вакансии
              </Link>
            </li>
          </ul>
        </div>
        <div>
          <p className="font-mono text-xs tracking-wider text-ink-faint uppercase">Работодателям</p>
          <ul className="mt-3 space-y-2 text-sm">
            <li>
              <Link href="/register?role=employer" className="hover:text-white hover:underline">
                Найти специалиста
              </Link>
            </li>
            <li>
              <Link href="/login" className="hover:text-white hover:underline">
                Войти в кабинет
              </Link>
            </li>
          </ul>
        </div>
      </div>
      <div className="border-t border-white/10">
        <p className="mx-auto max-w-6xl px-4 py-4 text-xs text-ink-faint sm:px-6">Хакатон «Лидеры цифровой трансформации 2026», спецтрек ФСП. Данные хранятся на серверах в РФ (152-ФЗ).</p>
      </div>
    </footer>
  );
}
