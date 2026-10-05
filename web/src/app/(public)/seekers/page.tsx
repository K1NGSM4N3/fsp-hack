import type { Metadata } from "next";
import type { ReactNode } from "react";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { Container, Cta, Eyebrow, TextLink } from "@/components/site/primitives";
import { FinalCta } from "@/components/site/seekers/FinalCta";
import { InviteSwipe } from "@/components/site/seekers/InviteSwipe";
import { MiniTest } from "@/components/site/seekers/MiniTest";
import { PrepChecklist } from "@/components/site/seekers/PrepChecklist";
import { PrivacyPanel } from "@/components/site/seekers/PrivacyPanel";
import { PushPhone } from "@/components/site/seekers/PushPhone";
import { WeekStrip } from "@/components/site/seekers/WeekStrip";

export const metadata: Metadata = {
  title: { absolute: "Соискателям · ФСП Таланты" },
  description: "Короткий профиль, один тест на 10–25 минут и приглашения от компаний, где зарплата от и до указана в первом сообщении.",
};

const REGISTER = "/register?role=candidate";

const INVITE_PARTS: [string, string][] = [
  ["Задача", "Что предстоит делать, словами самой компании."],
  ["Формат и город", "Офис, гибрид или удалённо, и где именно."],
  ["Зарплата от и до", "Обязательное поле: без него приглашение не отправить."],
  ["Компания", "Название и отрасль."],
  ["Срок ответа", "14 дней, потом приглашение истекает."],
];

function Hero() {
  return (
    <section className="relative isolate overflow-hidden bg-white pt-28 pb-20 sm:pt-36 lg:pt-40 lg:pb-28">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="bg-grid mask-radial absolute inset-0 opacity-70" />
        <div className="absolute top-[-14rem] right-[-10rem] size-[44rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.16),transparent_64%)]" />
      </div>
      <Container className="grid grid-cols-1 items-center gap-16 lg:grid-cols-[1.08fr_0.92fr] lg:gap-10">
        <div>
          <div data-hero>
            <Eyebrow>Соискателям</Eyebrow>
          </div>
          <h1 data-hero data-split="hero" className="display-hero mt-7 text-balance text-ink">
            Компании пишут вам первыми и <span className="text-gradient">сразу называют зарплату</span>
          </h1>
          <p data-hero className="mt-7 max-w-xl text-lg leading-relaxed text-pretty text-ink-muted sm:text-xl">
            Заполните короткий профиль и пройдите один тест на 10–25 минут. По его итогам вы получите категорию, например «Бэкенд · Middle». Компании ищут людей по
            категориям и присылают приглашения, где вилка от и до стоит в первом же сообщении.
          </p>
          <div data-hero className="mt-10 flex flex-wrap items-center gap-x-7 gap-y-5">
            <Cta href={REGISTER}>Создать профиль</Cta>
            <TextLink href="#test">Как проходит тест</TextLink>
          </div>
          <p data-hero className="mt-12 max-w-xl border-t border-line pt-6 text-sm leading-relaxed text-ink-subtle">
            Тест есть для бэкенда, фронтенда, DevOps / SRE, QA, ML / DS и безопасности. Грейды: Junior, Middle и Senior.
          </p>
        </div>
        <div data-hero className="relative">
          <PushPhone />
        </div>
      </Container>
    </section>
  );
}

function Week() {
  return (
    <section id="week" className="relative overflow-x-clip bg-canvas py-20 sm:py-28 lg:py-32">
      <Container className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_0.85fr] lg:items-end lg:gap-16">
        <div>
          <div data-reveal="fade">
            <Eyebrow>По дням</Eyebrow>
          </div>
          <h2 data-split className="display-2c mt-5 text-balance text-ink">
            Ваша первая неделя
          </h2>
        </div>
        <p data-reveal className="text-lg leading-relaxed text-pretty text-ink-muted">
          Так может пройти неделя после регистрации. Сроки у всех свои: приглашения зависят от того, сколько компаний сейчас ищет людей в вашей категории.
        </p>
      </Container>
      <div className="mt-12 sm:mt-14">
        <WeekStrip />
      </div>
    </section>
  );
}

function Para({ lead, children }: { lead: string; children: ReactNode }) {
  return (
    <p className="text-[17px] leading-relaxed text-night-muted sm:text-lg">
      <span className="font-semibold text-white">{lead}</span> {children}
    </p>
  );
}

function Test() {
  return (
    <section id="test" className="relative isolate overflow-hidden bg-night py-20 text-white sm:py-28 lg:py-36">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="bg-grid-night mask-radial absolute inset-0" />
        <div className="absolute top-1/3 right-[-12rem] size-[42rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.32),transparent_66%)]" />
      </div>
      <Container className="grid grid-cols-1 items-start gap-14 lg:grid-cols-[0.82fr_1.18fr] lg:gap-16">
        <div className="lg:sticky lg:top-32">
          <div data-reveal="fade">
            <Eyebrow tone="night">Тест</Eyebrow>
          </div>
          <h2 data-split className="display-2c mt-5 text-balance text-white">
            Задания подстраиваются под ваши ответы
          </h2>
          <div data-reveal className="mt-8 space-y-5">
            <Para lead="Как идёт тест.">
              В нём от 8 до 20 заданий, на каждое от одной до трёх минут. Если ответили верно, следующее задание будет сложнее, если ошиблись, проще. Тест заканчивается, когда
              уровень понятен, поэтому обычно занимает 10–25 минут.
            </Para>
            <Para lead="Свой вариант.">Задания собираются лично для вас: числа, условия и варианты ответа у каждого свои. Чужие ответы из интернета здесь не помогут.</Para>
            <Para lead="Вторая попытка.">
              Если не получилось, можно сразу пройти тест на грейд ниже, а на тот же грейд вернуться через 30 дней. Сдали уверенно, и мы предложим попробовать грейд выше.
            </Para>
          </div>
        </div>
        <div data-reveal="scale">
          <MiniTest />
          <p className="mt-4 text-sm text-night-faint">Ответьте на три задания: маркер справа покажет, куда сдвинулся уровень.</p>
        </div>
      </Container>
    </section>
  );
}

function Inbox() {
  return (
    <section id="inbox" className="relative overflow-x-clip bg-white py-20 sm:py-28 lg:py-36">
      <Container className="grid grid-cols-1 items-center gap-14 lg:grid-cols-[1fr_minmax(0,400px)] lg:gap-24">
        <div>
          <div data-reveal="fade">
            <Eyebrow>Приглашения</Eyebrow>
          </div>
          <h2 data-split className="display-2c mt-5 text-balance text-ink">
            Условия видны до первого разговора
          </h2>
          <p data-reveal className="mt-6 max-w-xl text-lg leading-relaxed text-pretty text-ink-muted">
            Каждое приглашение устроено одинаково. Отказ уходит компании с причиной, а имя и контакты она получает только после «Принять».
          </p>
          <ol data-stagger="0.06" className="mt-10 max-w-xl border-t border-line">
            {INVITE_PARTS.map(([title, text], i) => (
              <li key={title} className="flex items-baseline gap-4 border-b border-line py-4">
                <span className="inline-flex size-6 shrink-0 translate-y-[-1px] items-center justify-center rounded-full bg-ink font-mono text-[11px] font-semibold text-white">{i + 1}</span>
                <span className="text-[16px] leading-relaxed">
                  <span className="font-semibold text-ink">{title}.</span> <span className="text-ink-muted">{text}</span>
                </span>
              </li>
            ))}
          </ol>
          <p data-reveal className="mt-6 max-w-xl text-[15px] leading-relaxed text-ink-subtle">
            Число приглашений у каждой компании ограничено, и от одной компании одновременно приходит только одно.
          </p>
        </div>
        <InviteSwipe />
      </Container>
    </section>
  );
}

function Privacy() {
  return (
    <section id="privacy" className="relative isolate overflow-hidden bg-night py-20 text-white sm:py-28 lg:py-36">
      <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
        <div className="absolute -top-40 left-1/2 size-[50rem] -translate-x-1/2 rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.26),transparent_64%)]" />
      </div>
      <Container>
        <div className="mx-auto max-w-3xl text-center">
          <div data-reveal="fade">
            <Eyebrow tone="night">Приватность</Eyebrow>
          </div>
          <h2 data-split className="display-2 mt-5 text-balance text-white">
            Контакты под замком
          </h2>
          <p data-reveal className="mx-auto mt-6 max-w-2xl text-lg leading-relaxed text-pretty text-night-muted">
            Пока вы не нажали «Принять», компания видит карточку без имени и контактов: категорию, навыки, результат теста и ожидания по зарплате. Нажмите «Принять» ниже и
            посмотрите, что изменится.
          </p>
        </div>
        <div className="mt-14 lg:mt-20">
          <PrivacyPanel />
        </div>
      </Container>
    </section>
  );
}

function Prepare() {
  return (
    <section id="start" className="relative overflow-hidden bg-white pt-20 pb-6 sm:pt-28 lg:pt-36">
      <Container>
        <div className="grid grid-cols-1 gap-10 lg:grid-cols-[0.75fr_1.25fr] lg:gap-16">
          <div className="lg:sticky lg:top-32 lg:self-start">
            <div data-reveal="fade">
              <Eyebrow>Перед стартом</Eyebrow>
            </div>
            <h2 data-split className="display-2c mt-5 text-balance text-ink">
              Что подготовить
            </h2>
            <p data-reveal className="mt-6 max-w-md text-lg leading-relaxed text-pretty text-ink-muted">
              Это стоит держать под рукой при регистрации. Всё, кроме результата теста, можно поправить потом в профиле.
            </p>
          </div>
          <PrepChecklist />
        </div>

        <div className="relative isolate mt-20 overflow-hidden rounded-[32px] bg-primary px-6 py-14 sm:mt-28 sm:px-12 sm:py-20 lg:px-16">
          <div aria-hidden className="pointer-events-none absolute inset-0 -z-10">
            <div className="absolute -top-48 -right-40 size-[38rem] rounded-full bg-[radial-gradient(circle,var(--color-primary-400),transparent_62%)] opacity-70" />
            <div className="absolute -bottom-56 -left-32 size-[34rem] rounded-full bg-[radial-gradient(circle,var(--color-primary-800),transparent_62%)]" />
          </div>
          <FinalCta href={REGISTER} />
        </div>
      </Container>
    </section>
  );
}

export default function SeekersPage() {
  return (
    <MotionRoot variant="rise">
      <Hero />
      <Week />
      <Test />
      <Inbox />
      <Privacy />
      <Prepare />
    </MotionRoot>
  );
}
