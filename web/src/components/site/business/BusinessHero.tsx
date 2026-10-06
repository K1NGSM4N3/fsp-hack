import Link from "next/link";
import { Container, Cta } from "@/components/site/primitives";
import { VacancyParse } from "./VacancyParse";

export function BusinessHero() {
  return (
    <section className="relative isolate overflow-hidden bg-night text-white">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="bg-grid-night mask-radial absolute inset-0 opacity-60" />
        <div className="absolute -top-[30rem] right-[-16rem] size-[64rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.5),transparent_60%)]" data-parallax="0.18" />
        <div className="absolute bottom-[-26rem] left-[-18rem] size-[44rem] rounded-full bg-[radial-gradient(circle,rgb(123_111_255/0.2),transparent_64%)]" />
      </div>

      <Container className="pt-28 pb-20 sm:pt-36 sm:pb-24 lg:pt-36 lg:pb-28">
        <div className="grid grid-cols-1 gap-12 lg:grid-cols-[1fr_minmax(0,32rem)] lg:items-center lg:gap-12 xl:grid-cols-[1fr_minmax(0,35rem)]">
          <div>
            <p data-hero className="mb-7 inline-flex items-center gap-3 rounded-full border border-white/15 bg-white/5 py-1.5 pr-4 pl-1.5 text-sm text-night-muted">
              <span className="rounded-full bg-primary px-2.5 py-0.5 font-mono text-xs tracking-[0.1em] text-white uppercase">Компаниям</span>
              Найм ИТ-специалистов
            </p>
            <h1
              data-hero
              data-split="hero"
              className="font-display text-[clamp(2.2rem,1.1rem+2.45vw,3.6rem)] leading-[1.03] font-semibold tracking-[-0.042em] text-balance"
            >
              Опишите задачу один раз и получите специалистов <span className="text-gradient-night"><span className="whitespace-nowrap">с проверенным</span> уровнем</span>
            </h1>
            <p data-hero="lead" className="mt-7 max-w-xl text-lg leading-relaxed text-night-muted">
              Вставьте текст вакансии как есть, сервис сам выделит роль, грейд, навыки, вилку и формат работы. В подборку попадут только люди, которые сдали тест на нужную специализацию и уровень.
            </p>
            <div data-hero className="mt-9 flex flex-wrap items-center gap-x-6 gap-y-4">
              <Cta href="/register?role=employer">Зарегистрировать компанию</Cta>
              <Link href="/login" className="inline-flex min-h-10 items-center text-sm font-medium text-night-muted underline decoration-white/25 underline-offset-[6px] transition-colors hover:text-white hover:decoration-white">
                Уже есть аккаунт
              </Link>
            </div>
          </div>

          <div data-hero>
            <VacancyParse />
          </div>
        </div>
      </Container>
    </section>
  );
}
