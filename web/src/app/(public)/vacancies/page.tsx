import type { Metadata } from "next";
import Link from "next/link";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { Container } from "@/components/site/primitives";
import { PublicVacancyBrowser } from "./PublicVacancyBrowser";

export const metadata: Metadata = { title: "Вакансии" };

export default function VacanciesPage() {
  return (
    <MotionRoot className="flex flex-1 flex-col bg-canvas">
      <section className="relative isolate overflow-hidden border-b border-line bg-white pt-32 pb-12 sm:pt-40 sm:pb-16">
        <div aria-hidden className="bg-grid mask-radial pointer-events-none absolute inset-0 -z-10 opacity-70" />
        <div aria-hidden className="pointer-events-none absolute -top-56 right-[-10rem] -z-10 size-[40rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.18),transparent_64%)]" />
        <Container>
          <nav data-hero aria-label="Хлебные крошки" className="mb-6 flex items-center gap-2 font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">
            <Link href="/" className="transition-colors hover:text-primary">
              ФСП Таланты
            </Link>
            <span aria-hidden>/</span>
            <span className="text-primary">Вакансии</span>
          </nav>
          <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1.3fr_0.7fr] lg:items-end">
            <h1 data-hero data-split="hero" className="display-2 text-balance text-ink">
              Открытые вакансии
            </h1>
            <p data-hero className="text-lg leading-relaxed text-ink-muted">
              Классический отклик — дополнительный путь. Основной: пройти тест и получать приглашения с вилкой.{" "}
              <Link href="/candidates" className="font-medium text-primary underline-offset-4 hover:underline">
                Как это работает
              </Link>
            </p>
          </div>
        </Container>
      </section>
      <Container className="py-10 sm:py-14">
        <PublicVacancyBrowser />
      </Container>
    </MotionRoot>
  );
}
