import { ArrowUpRight } from "lucide-react";
import Image from "next/image";
import Link from "next/link";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { FOOTER_COLUMNS } from "./nav";
import { Container } from "./primitives";

export function SiteFooter() {
  return (
    <footer className="relative isolate mt-auto overflow-hidden bg-night text-white print:hidden">
      <div aria-hidden className="pointer-events-none absolute -bottom-64 left-1/2 size-[44rem] -translate-x-1/2 rounded-full bg-primary/25 blur-[160px]" />
      <MotionRoot>
        <Container className="relative pt-20 pb-10 sm:pt-24">
          <div className="grid grid-cols-1 gap-14 lg:grid-cols-[1.2fr_2fr]">
            <div data-reveal>
              <Image src="/brand/fsp-logo-full-dark.svg" alt="Федерация спортивного программирования России" width={320} height={40} unoptimized className="h-9 w-auto" />
              <p className="mt-6 max-w-sm leading-relaxed text-night-muted">
                Подбор ИТ-специалистов от Федерации спортивного программирования России. Соискатель один раз проходит тест, и компании пишут ему сами, называя зарплату в первом же сообщении.
              </p>
              <div className="mt-8 flex flex-wrap gap-3">
                <Link href="/seekers" className="inline-flex h-11 items-center rounded-full bg-white px-5 text-sm font-medium text-ink transition-colors hover:bg-primary-100">
                  Ищу работу
                </Link>
                <Link href="/business" className="inline-flex h-11 items-center rounded-full border border-white/25 px-5 text-sm font-medium text-white transition-colors hover:border-white">
                  Ищу специалистов
                </Link>
              </div>
            </div>
            <div data-stagger className="grid grid-cols-1 gap-10 sm:grid-cols-3">
              {FOOTER_COLUMNS.map((col) => (
                <nav key={col.title} aria-label={col.title}>
                  <p className="eyebrow text-night-faint">{col.title}</p>
                  <ul className="mt-5 space-y-3">
                    {col.links.map((l) => (
                      <li key={l.href}>
                        <Link href={l.href} className="group inline-flex items-center gap-1 text-[15px] text-night-muted transition-colors hover:text-white">
                          {l.label}
                          <ArrowUpRight className="size-3.5 opacity-0 transition-opacity group-hover:opacity-100" aria-hidden />
                        </Link>
                      </li>
                    ))}
                  </ul>
                </nav>
              ))}
            </div>
          </div>

          <div className="relative mt-20 select-none" aria-hidden>
            <p data-reveal="scale" className="font-display text-[clamp(3.2rem,15.2vw,13.5rem)] leading-[0.82] font-semibold tracking-[-0.06em] text-transparent [-webkit-text-stroke:1px_rgb(169_163_255/0.45)]">
              ФСП Таланты
            </p>
          </div>

          <div className="mt-10 flex flex-col gap-4 border-t border-night-line pt-6 text-sm text-night-faint md:flex-row md:items-center md:justify-between">
            <p>© 2026 ФСП Таланты · Хакатон «Лидеры цифровой трансформации 2026», спецтрек ФСП</p>
            <div className="flex flex-wrap gap-x-6 gap-y-2">
              <span>Персональные данные храним и обрабатываем по 152-ФЗ</span>
              <a href="https://fsp-russia.com" target="_blank" rel="noreferrer" className="transition-colors hover:text-white">
                fsp-russia.com
              </a>
            </div>
          </div>
        </Container>
      </MotionRoot>
    </footer>
  );
}
