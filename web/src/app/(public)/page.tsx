import { ArrowDown, Check, Clock, LockKeyhole } from "lucide-react";
import type { Metadata } from "next";
import Link from "next/link";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { HeroGraphLazy } from "@/components/site/HeroGraphLazy";
import { Doors } from "@/components/site/home/Doors";
import { InviteScene } from "@/components/site/home/InviteScene";
import { SceneLaptop } from "@/components/site/home/SceneLaptop";
import { ScenePhone } from "@/components/site/home/ScenePhone";
import { SideLines } from "@/components/site/home/SideLines";
import { Marquee } from "@/components/site/Marquee";
import { Container, Cta } from "@/components/site/primitives";

export const metadata: Metadata = {
  title: { absolute: "ФСП Таланты · компании пишут первыми и сразу называют зарплату" },
  description:
    "Подбор ИТ-специалистов Федерации спортивного программирования России. Соискатель один раз проходит тест, компании пишут ему сами и в первом же сообщении называют зарплату от и до.",
};

const SPECS = ["Бэкенд", "Фронтенд", "DevOps", "Тестирование", "Data Science", "Безопасность"];
const GRADES = ["Junior", "Middle", "Senior"];
const CATEGORIES = GRADES.flatMap((g, gi) => SPECS.map((s, si) => ({ label: `${s} · ${g}`, accent: (gi + si) % 5 === 0 })));

function CategoryChip({ label, accent }: { label: string; accent: boolean }) {
  return (
    <span className="mx-1.5 inline-flex items-center gap-2.5 rounded-full border border-line bg-white px-4 py-2 text-sm font-medium whitespace-nowrap text-ink sm:px-5 sm:py-2.5 sm:text-[15px]">
      <span aria-hidden className={accent ? "size-1.5 rounded-full bg-accent" : "size-1.5 rounded-full bg-primary"} />
      {label}
    </span>
  );
}

function InvitationCard() {
  return (
    <div className="relative">
      <div aria-hidden className="absolute inset-x-6 -top-6 h-full rounded-[26px] border border-white/10 bg-white/[0.04] sm:inset-x-8 sm:-top-7" />
      <div aria-hidden className="absolute inset-x-3 -top-3 h-full rounded-[26px] border border-white/10 bg-white/[0.06] sm:inset-x-4 sm:-top-3.5" />
      <div className="glass-night relative rounded-[26px] border border-white/12 p-5 shadow-[0_40px_80px_-30px_rgb(0_0_0/0.8)] sm:p-6">
        <div className="flex items-center justify-between gap-3 text-xs">
          <span className="inline-flex items-center gap-2 font-medium text-primary-300">
            <span className="relative flex size-2">
              <span className="absolute inset-0 animate-ping rounded-full bg-accent opacity-60" />
              <span className="relative size-2 rounded-full bg-accent" />
            </span>
            Новое приглашение
          </span>
          <span className="font-mono text-night-faint">сегодня, 10:42</span>
        </div>
        <p className="mt-4 text-lg leading-snug font-semibold tracking-tight">Go-разработчик в команду платежей</p>
        <p className="mt-1 text-sm text-night-muted">Финтех · гибрид, Москва</p>
        <div className="mt-5 rounded-2xl border border-white/10 bg-white/5 p-4">
          <p className="text-xs text-night-muted">Зарплата</p>
          <p className="tabular mt-1 font-mono text-2xl font-semibold tracking-tight">280 000 – 350 000 ₽</p>
        </div>
        <ul className="mt-5 space-y-2 text-sm text-night-muted">
          <li className="flex items-start gap-2.5">
            <Check className="mt-0.5 size-4 shrink-0 text-success-300" aria-hidden /> Задача: сервис выплат для партнёров
          </li>
          <li className="flex items-start gap-2.5">
            <Check className="mt-0.5 size-4 shrink-0 text-success-300" aria-hidden /> Компания выбрала вас по категории «Бэкенд · Middle»
          </li>
        </ul>
        <div className="mt-6 grid grid-cols-2 gap-2">
          <span className="inline-flex h-11 items-center justify-center rounded-xl bg-white text-sm font-medium text-ink">Принять</span>
          <span className="inline-flex h-11 items-center justify-center rounded-xl border border-white/15 text-sm font-medium text-white">Отклонить</span>
        </div>
        <p className="mt-4 flex items-center gap-2 text-xs text-night-faint">
          <LockKeyhole className="size-3.5 shrink-0" aria-hidden /> Ваши контакты компания увидит после «Принять»
        </p>
      </div>
      <div className="glass-night absolute -bottom-7 -left-3 flex items-center gap-2.5 rounded-2xl border border-white/12 px-3.5 py-2.5 text-xs shadow-2xl sm:-bottom-8 sm:-left-10 sm:px-4 sm:py-3 sm:text-sm">
        <Clock className="size-4 shrink-0 text-primary-300" aria-hidden />
        Ответить можно до 19 октября
      </div>
    </div>
  );
}

function Hero() {
  return (
    <section className="relative isolate flex min-h-[100svh] flex-col overflow-hidden bg-night text-white">
      <div aria-hidden className="pointer-events-none absolute inset-0">
        <div className="absolute -top-1/3 right-[-10%] size-[60rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.55),transparent_62%)]" data-parallax="0.25" />
        <div className="absolute bottom-[-30%] left-[-15%] size-[44rem] rounded-full bg-[radial-gradient(circle,rgb(123_111_255/0.28),transparent_65%)]" />
        <div className="bg-grid-night mask-radial absolute inset-0 opacity-70" />
      </div>
      <HeroGraphLazy />

      <Container className="relative flex flex-1 flex-col justify-center pt-32 pb-24 sm:pt-36 lg:pt-40 lg:pb-28">
        <div className="grid grid-cols-1 items-center gap-16 lg:grid-cols-[1.25fr_0.75fr] lg:gap-14">
          <div>
            <p data-hero className="mb-7 inline-flex items-center gap-3 rounded-full border border-white/15 bg-white/5 py-1.5 pr-4 pl-1.5 text-sm text-night-muted backdrop-blur">
              <span className="rounded-full bg-primary px-2.5 py-0.5 font-mono text-xs tracking-[0.1em] text-white uppercase">ФСП</span>
              Подбор ИТ-специалистов
            </p>
            <h1 data-hero data-split="hero" className="display-hero max-w-[17ch] text-balance">
              Компании пишут вам первыми <span className="text-gradient-night">и сразу называют зарплату</span>
            </h1>
            <p data-hero="lead" className="mt-8 max-w-xl text-lg leading-relaxed text-night-muted sm:text-xl">
              Один раз пройдите тест по своему направлению. Компании найдут вас по категории и пришлют приглашение, в котором уже указана зарплата от и до. Имя и телефон они увидят, только если вы согласитесь.
            </p>
            <div data-hero className="mt-10 flex flex-wrap items-center gap-3">
              <Cta href="/register?role=candidate">Пройти тест</Cta>
              <Cta href="/business" variant="outline-night">
                Я ищу специалистов
              </Cta>
            </div>
            <p data-hero className="mt-6 text-sm text-night-faint">
              Уже есть аккаунт?{" "}
              <Link href="/login" className="text-white underline decoration-white/30 underline-offset-4 hover:decoration-white">
                Войти
              </Link>
            </p>
          </div>

          <div data-hero className="relative mx-auto w-full max-w-[26rem] pt-6 lg:mx-0 lg:justify-self-end">
            <div data-tilt="7">
              <InvitationCard />
            </div>
          </div>
        </div>
      </Container>
      <a
        href="#scene"
        className="group absolute inset-x-0 bottom-6 mx-auto hidden min-h-10 w-fit items-center gap-2 font-mono whitespace-nowrap text-xs tracking-[0.12em] text-night-faint uppercase transition-colors hover:text-white sm:inline-flex"
      >
        Как это выглядит
        <ArrowDown className="size-4 transition-transform group-hover:translate-y-0.5" aria-hidden />
      </a>
    </section>
  );
}

export default function HomePage() {
  return (
    <MotionRoot variant="rise">
      <Hero />

      <div className="border-b border-line bg-white py-9 sm:py-12">
        <p className="eyebrow mb-6 px-4 text-center text-ink-subtle">Категории, по которым есть тест</p>
        <Marquee items={CATEGORIES.slice(0, 9).map((c) => <CategoryChip key={c.label} {...c} />)} label="Категории, по которым есть тест" duration={60} />
        <Marquee items={CATEGORIES.slice(9).map((c) => <CategoryChip key={c.label} {...c} />)} reverse duration={66} className="mt-3" />
      </div>

      <InviteScene laptop={<SceneLaptop />} phone={<ScenePhone />} />
      <SideLines />
      <Doors />
    </MotionRoot>
  );
}
