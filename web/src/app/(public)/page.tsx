import { ArrowDownRight, Check, EyeOff, LockKeyhole, ScrollText, ShieldAlert, Shuffle, Scale, UserX } from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { Faq, type FaqItem } from "@/components/site/Faq";
import { FlipTable } from "@/components/site/FlipTable";
import { FspScore } from "@/components/site/FspScore";
import { HeroGraph } from "@/components/site/HeroGraph";
import { Journey } from "@/components/site/Journey";
import { LeakChart, LeakDemo } from "@/components/site/LeakDemo";
import { LevelFinder } from "@/components/site/LevelFinder";
import { ManifestoScrub } from "@/components/site/ManifestoScrub";
import { Marquee } from "@/components/site/Marquee";
import { BrandLine, Container, Cta, Eyebrow, Fact, Kicker, Section, SectionHead, TextLink } from "@/components/site/primitives";
import { ScoreDemo } from "@/components/site/ScoreDemo";

const STACK_A = ["Go", "PostgreSQL", "Kafka", "Python", "Java", "Redis", "gRPC", "SQL", "Kotlin", "C++", "Node.js", "ClickHouse"];
const STACK_B = ["React", "TypeScript", "Docker", "Kubernetes", "nginx", "Prometheus", "Linux", "pandas", "NumPy", "PyTorch", "OWASP", "Playwright"];

function StackChip({ children, accent = false }: { children: ReactNode; accent?: boolean }) {
  return (
    <span className="mx-2 inline-flex items-center gap-3 rounded-full border border-line bg-white px-5 py-2.5 font-mono text-sm text-ink shadow-card sm:text-base">
      <span aria-hidden className={accent ? "size-1.5 rounded-full bg-accent" : "size-1.5 rounded-full bg-primary"} />
      {children}
    </span>
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
      <HeroGraph />

      <Container className="relative flex flex-1 flex-col justify-center pt-32 pb-14 sm:pt-36 lg:pt-40">
        <div className="grid grid-cols-1 items-center gap-14 lg:grid-cols-[1.25fr_0.75fr]">
          <div>
            <p data-hero className="mb-7 inline-flex items-center gap-3 rounded-full border border-white/15 bg-white/5 py-1.5 pr-4 pl-1.5 text-sm text-night-muted backdrop-blur">
              <span className="rounded-full bg-primary px-2.5 py-0.5 font-mono text-[11px] tracking-[0.1em] text-white uppercase">ФСП</span>
              Подбор ИТ-специалистов
            </p>
            <h1 data-hero data-split="hero" className="display-1 max-w-[15ch] text-balance">
              Компании пишут первыми. <span className="text-gradient-night">Сразу с вилкой.</span>
            </h1>
            <p data-hero className="mt-8 max-w-xl text-lg leading-relaxed text-night-muted sm:text-xl">
              Вы один раз сдаёте адаптивный тест и получаете категорию, например «Бэкенд · Middle». Работодатели видят анонимную карточку и присылают приглашение, в котором уже есть задача, формат работы и зарплата от и до.
            </p>
            <div data-hero className="mt-10 flex flex-wrap items-center gap-3">
              <Cta href="/register?role=candidate">Я ищу работу</Cta>
              <Cta href="/register?role=employer" variant="outline-night">
                Я нанимаю
              </Cta>
            </div>
            <p data-hero className="mt-6 text-sm text-night-faint">
              Уже есть аккаунт?{" "}
              <Link href="/login" className="text-white underline decoration-white/30 underline-offset-4 hover:decoration-white">
                Войти
              </Link>
            </p>
          </div>

          <div data-hero className="relative mx-auto mb-8 w-full max-w-[26rem] lg:mx-0 lg:justify-self-end">
            <div data-tilt="7" className="relative">
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
                <ul className="mt-5 space-y-2 text-sm">
                  <li className="flex items-center gap-2.5">
                    <Check className="size-4 text-success-300" aria-hidden /> Категория Бэкенд · Middle подтверждена тестом
                  </li>
                  <li className="flex items-center gap-2.5">
                    <Check className="size-4 text-success-300" aria-hidden /> Go и PostgreSQL подтверждены тестом
                  </li>
                </ul>
                <div className="mt-6 grid grid-cols-2 gap-2">
                  <span className="inline-flex h-11 items-center justify-center rounded-xl bg-white text-sm font-medium text-ink">Принять</span>
                  <span className="inline-flex h-11 items-center justify-center rounded-xl border border-white/15 text-sm font-medium text-white">Отклонить</span>
                </div>
                <p className="mt-4 flex items-center gap-2 text-xs text-night-faint">
                  <LockKeyhole className="size-3.5" aria-hidden /> Ваши контакты компания увидит после «Принять»
                </p>
              </div>
              <div className="glass-night absolute -bottom-12 -left-8 hidden items-center gap-3 rounded-2xl border border-white/12 px-4 py-3 shadow-2xl sm:flex">
                <span className="relative flex size-12 items-center justify-center">
                  <svg viewBox="0 0 48 48" className="absolute inset-0 -rotate-90" aria-hidden>
                    <circle cx="24" cy="24" r="20" fill="none" strokeWidth="4" className="stroke-white/10" />
                    <circle cx="24" cy="24" r="20" fill="none" strokeWidth="4" strokeLinecap="round" strokeDasharray="125.66" strokeDashoffset="15.1" className="stroke-primary-400" />
                  </svg>
                  <span className="font-mono text-sm font-semibold">88</span>
                </span>
                <span className="text-xs leading-tight text-night-muted">
                  <span className="block text-sm font-medium text-white">Балл соответствия</span>расписан по восьми признакам
                </span>
              </div>
            </div>
          </div>
        </div>

        <dl data-hero className="mt-20 grid grid-cols-2 gap-x-6 gap-y-8 border-t border-white/10 pt-8 lg:mt-24 lg:grid-cols-4">
          {[
            ["94 %", "подходящих кандидатов в первой десятке подборки. Поиск по словам из резюме даёт 43 %"],
            ["87 %", "тестов определяют грейд верно. Промах бывает максимум на один грейд"],
            ["~13", "заданий в среднем. Тест заканчивается, когда уровень понятен"],
            ["9–10 %", "слабых кандидатов сдают тест. Со слитыми ответами столько же"],
          ].map(([v, l]) => (
            <div key={v}>
              <dt className="sr-only">{l}</dt>
              <dd>
                <span className="tabular block font-display text-3xl font-semibold tracking-[-0.03em] sm:text-4xl">{v}</span>
                <span aria-hidden className="mt-2 block max-w-[17rem] text-sm leading-relaxed text-night-muted">{l}</span>
              </dd>
            </div>
          ))}
        </dl>
      </Container>
      <a href="#problem" className="group absolute right-6 bottom-6 hidden items-center gap-2 font-mono text-xs tracking-[0.12em] text-night-faint uppercase transition-colors hover:text-white lg:inline-flex">
        Дальше
        <ArrowDownRight className="size-4 transition-transform group-hover:translate-y-0.5" aria-hidden />
      </a>
    </section>
  );
}

const PAINS = [
  { title: "Резюме похожи друг на друга", text: "У всех «уверенное знание» и одинаковые списки технологий. Поиск по ключевым словам ставит в первую десятку меньше половины подходящих людей: в нашей проверке 43 %." },
  { title: "Про деньги говорят в конце", text: "Вилку часто называют после двух созвонов. Не сошлись, и время ушло у обеих сторон." },
  { title: "Тесты живут до первой утечки", text: "Слейте половину ответов к тесту с одинаковыми вопросами, и его сдадут 99,9 % тех, кто до уровня не дотягивает." },
  { title: "Письма без условий", text: "«Посмотрите нашу вакансию», и больше ничего. После десятка таких писем сильный кандидат перестаёт их открывать." },
];

const SCENARIOS = [
  {
    tag: "Кандидат",
    title: "Middle-бэкендер, которому надоело откликаться",
    steps: [
      "Отмечает в опросе Go, PostgreSQL и Kafka и выбирает Middle.",
      "Сдаёт тест с запасом. Сервис сразу предлагает попробовать Senior.",
      "Получает приглашения с вилкой. Два отклоняет с причиной «зарплата», компании это увидят.",
      "Принимает третье. Контакты получает только эта компания.",
    ],
  },
  {
    tag: "Работодатель",
    title: "Тимлид ищет Go-разработчика к релизу",
    steps: [
      "Вставляет текст вакансии. Сервис находит в нём Middle, Go, PostgreSQL, Kafka и вилку.",
      "Смотрит категории: сколько там людей и сколько они просят.",
      "Сужает фильтры. Закреплённые кандидаты остаются в подборке.",
      "Отправляет пять приглашений и через пару дней видит, кто ответил и почему отказал.",
    ],
  },
  {
    tag: "Участник ФСП",
    title: "Студент, призёр соревнований по программированию",
    steps: [
      "Привязывает ФСП ID. Разряд и призовые места появляются в профиле.",
      "Сдаёт тест на Junior, навыки из решённых заданий помечены как подтверждённые.",
      "В подборке стоит выше тех, у кого такой же результат теста, но нет достижений.",
      "Решает короткие задания от компаний, чтобы профиль не выпадал из свежих.",
    ],
  },
];

const FAQ: FaqItem[] = [
  { q: "Почему категорию даёт тест, а резюме не учитывается?", a: "Резюме учитывается, но для другого: из него берутся стек и ожидания. Категорию «специализация × грейд» присваивает только сданный тест, потому что написать «Senior» в резюме может каждый." },
  { q: "Сколько длится тест?", a: "8–20 заданий, обычно 10–25 минут. На одно задание отводится от одной до трёх минут. Если время вышло, ответ считается неверным." },
  { q: "Что будет, если не сдать?", a: "Можно сразу пройти тест на грейд ниже. Тот же или более высокий грейд можно пересдать через 30 дней. Если не получился Junior, следующая попытка тоже через 30 дней." },
  { q: "Как поднять грейд?", a: "Если сдали с запасом, сервис сразу предложит один дополнительный тест на грейд выше. Потом грейд меняется не чаще раза в 90 дней. Неудачная попытка повышения текущий грейд не снижает." },
  { q: "Задания же утекут в интернет?", a: "Пусть. Следующий кандидат получит другие числа, другой код и другой правильный ответ. Если кто-то разобрался, как решать такой тип задач, значит, он стал сильнее." },
  { q: "Кто увидит мои контакты?", a: "Компания, чьё приглашение вы приняли, или та, на чью вакансию вы откликнулись сами. Каждый просмотр контактов записывается." },
  { q: "У меня нет достижений ФСП, я в минусе?", a: "Нет. Без истории ФСП вы получаете ноль баллов за этот признак, штрафа нет. В карточке будет нейтральное «Нет данных ФСП»." },
  { q: "Почему компания не может написать всем сразу?", a: "У каждой компании 30 приглашений в неделю и только одно действующее приглашение одному человеку. Приходится выбирать, поэтому до вас доходят письма, которые кто-то действительно отобрал." },
  { q: "Вакансии с откликами остались?", a: "Остались. На опубликованную вакансию можно откликнуться, тогда контакты откроются по вашему отклику. Но основная механика здесь — приглашения." },
];

export default function HomePage() {
  return (
    <MotionRoot variant="rise">
      <Hero />

      <div className="border-b border-line bg-canvas py-10 sm:py-14">
        <p className="eyebrow mb-7 px-4 text-center text-ink-subtle">6 специализаций, 83 навыка и 491 синоним. «Кубер» и «постгрес» тоже узнаём</p>
        <Marquee items={STACK_A.map((s) => <StackChip key={s}>{s}</StackChip>)} label="Технологии в тестах и подборе" />
        <Marquee items={STACK_B.map((s, i) => <StackChip key={s} accent={i === 5}>{s}</StackChip>)} reverse duration={52} className="mt-4" />
      </div>

      <Section id="problem">
        <Container>
          <SectionHead
            eyebrow="Как обычно"
            title={
              <>
                Сто откликов, два ответа и <span className="text-gradient">вилка на третьем собеседовании</span>
              </>
            }
            lead="Кандидат рассылает резюме, рекрутер листает похожие. Уровень и деньги обсуждают в самом конце, хотя именно из-за них разговор чаще всего и срывается."
          />
          <ol data-stagger className="mt-16 grid grid-cols-1 gap-px overflow-hidden rounded-panel border border-line bg-line sm:grid-cols-2 lg:grid-cols-4">
            {PAINS.map((p, i) => (
              <li key={p.title} className="flex flex-col bg-white p-7 sm:p-8">
                <span className="font-mono text-sm text-accent-strong">0{i + 1}</span>
                <h3 className="mt-10 text-xl leading-snug font-semibold tracking-tight text-ink">{p.title}</h3>
                <p className="mt-3 text-[15px] leading-relaxed text-ink-muted">{p.text}</p>
              </li>
            ))}
          </ol>

          <div className="mt-28 lg:mt-36">
            <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1fr_1fr] lg:items-end lg:gap-16">
              <div>
                <Kicker index="→">У нас</Kicker>
                <h2 data-split className="display-2c mt-5 text-balance text-ink">
                  Мы поменяли порядок шагов
                </h2>
              </div>
              <div data-reveal>
                <p className="text-lg leading-relaxed text-ink-muted">
                  Сначала тест, потом знакомство. Компания пишет, когда уже знает ваш уровень, поэтому первое сообщение начинается с задачи и зарплаты.
                </p>
                <div className="mt-6">
                  <TextLink href="#how">Как устроен тест</TextLink>
                </div>
              </div>
            </div>
            <div className="mt-12">
              <FlipTable />
            </div>
          </div>
        </Container>
      </Section>

      <Section tone="canvas" className="overflow-hidden">
        <Container>
          <ManifestoScrub eyebrow="Зачем всё это">
            Хороший разработчик не должен доказывать свой уровень в каждом отклике заново. Он <span className="text-primary">сдаёт один тест</span>, а дальше компании приходят сами, с задачей и суммой. Неподходящее предложение отклоняется одной кнопкой, и компания узнаёт причину.
          </ManifestoScrub>
        </Container>
      </Section>

      <Journey />

      <Section id="how" className="scroll-mt-20">
        <Container>
          <SectionHead
            eyebrow="Как устроен тест"
            title="Тест подстраивается под ответы"
            lead="После верного ответа следующее задание сложнее, после ошибки проще. Поэтому хватает 8–20 заданий, обычно это 10–25 минут."
          />
          <div className="mt-14 grid grid-cols-1 gap-10 lg:grid-cols-[1.05fr_0.95fr] lg:items-center lg:gap-16">
            <div data-reveal="scale">
              <LevelFinder />
            </div>
            <ul data-stagger className="space-y-6">
              {[
                ["На задание от одной до трёх минут", "Загуглить ответ не успеете, подумать успеете."],
                ["Стек влияет на темы", "Пишете на Go, значит, будет больше заданий про Go. Планка для Middle при этом у всех одинаковая."],
                ["Грейд определяется верно в 87 % случаев", "Если тест ошибается, то максимум на один грейд, и обычно у самой границы."],
                ["Результат повторяется", "Пересдача другими заданиями дала тот же грейд в 81 % случаев."],
                ["Есть второй шанс", "Не сдали, сразу пробуйте грейд ниже. Сдали с запасом, предложим грейд выше."],
              ].map(([t, d]) => (
                <li key={t} className="border-t border-line pt-5">
                  <p className="text-lg font-semibold tracking-tight text-ink">{t}</p>
                  <p className="mt-1.5 text-[15px] leading-relaxed text-ink-muted">{d}</p>
                </li>
              ))}
            </ul>
          </div>
        </Container>
      </Section>

      <Section id="leaks" tone="night" className="overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute top-1/4 -right-48 size-[40rem] rounded-full bg-primary/25 blur-[150px]" />
        <Container className="relative">
          <SectionHead
            tone="night"
            eyebrow="Утечки"
            title="Слитые ответы здесь не помогут"
            lead="Тема и сложность у всех одинаковые, а числа, код и данные у каждого свои. Правильный ответ считается отдельно для каждого варианта, так что выучить его заранее не выйдет."
          />
          <div className="mt-14 grid grid-cols-1 gap-8 lg:grid-cols-[1.05fr_0.95fr] lg:items-start">
            <div data-reveal>
              <LeakDemo />
            </div>
            <div data-reveal="scale">
              <LeakChart />
              <p className="mt-4 text-sm leading-relaxed text-night-muted">
                Моделирование: кандидаты слабее Middle сдают тест на Middle. У них на руках все варианты из 600 предыдущих попыток, около 42 тысяч. Совпадает примерно одно задание из двух-пяти тысяч.
              </p>
            </div>
          </div>
          <div data-stagger className="mt-16 grid grid-cols-1 gap-px overflow-hidden rounded-[24px] border border-night-line bg-night-line sm:grid-cols-3">
            {[
              { icon: Shuffle, title: "62 типа заданий", text: "В шести специализациях. Из 2000 случайных вариантов одного задания не меньше 1961 отличаются друг от друга." },
              { icon: Scale, title: "Сложность выровнена", text: "Варианты одного задания одинаково трудные, поэтому результаты разных людей можно сравнивать." },
              { icon: ShieldAlert, title: "Странные ответы видно", text: "Решил трудные и провалил лёгкие? Такой результат уходит на перепроверку." },
            ].map(({ icon: Icon, title, text }) => (
              <div key={title} className="bg-night-raised p-7">
                <Icon className="size-6 text-primary-300" aria-hidden />
                <p className="mt-6 text-xl font-semibold tracking-tight">{title}</p>
                <p className="mt-2 text-[15px] leading-relaxed text-night-muted">{text}</p>
              </div>
            ))}
          </div>
        </Container>
      </Section>

      <Section id="card" tone="canvas" className="overflow-hidden">
        <BrandLine className="pointer-events-none absolute top-16 left-0 w-full text-primary-200" variant={2} />
        <Container className="relative">
          <div className="grid grid-cols-1 items-start gap-14 lg:grid-cols-[0.85fr_1.15fr] lg:gap-16">
            <div className="lg:sticky lg:top-32">
              <Eyebrow>Что видит работодатель</Eyebrow>
              <h2 data-split className="display-2c mt-5 text-balance text-ink">
                Балл и то, из чего он сложился
              </h2>
              <p data-reveal className="mt-6 text-lg leading-relaxed text-ink-muted">
                Карточка без имени и контактов: категория, результат теста, навыки с пометкой, откуда о них известно, и достижения ФСП. Балл от 0 до 100 раскладывается на восемь признаков.
              </p>
              <ul data-stagger className="mt-8 space-y-4">
                {[
                  "Навык из теста или ФСП весит вдвое больше заявленного на словах. Навык из резюме где-то посередине.",
                  "Пробелы видны сразу, вместе с близкими заменами вроде «вместо PostgreSQL есть MySQL».",
                  "Нет истории ФСП? Ноль баллов за этот признак, и в пробелы он не попадает.",
                  "Подборку можно уточнять, закреплённые кандидаты из неё не пропадут.",
                ].map((t) => (
                  <li key={t} className="flex gap-3 text-ink">
                    <span className="mt-1 flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-white" aria-hidden>
                      <Check className="size-3" strokeWidth={3} />
                    </span>
                    {t}
                  </li>
                ))}
              </ul>
            </div>
            <div data-reveal="scale">
              <ScoreDemo />
            </div>
          </div>
        </Container>
      </Section>

      <Section id="numbers">
        <Container>
          <SectionHead
            eyebrow="Проверка"
            title="Что показала проверка на 2 400 профилях"
            lead="Мы собрали профили, у которых заранее известен настоящий уровень, и прогнали их через те же тест и подбор, что работают в кабинетах."
          />
          <div data-stagger="0.06" className="mt-16 grid grid-cols-1 gap-x-8 gap-y-12 sm:grid-cols-2 lg:grid-cols-4">
            <Fact value="94" counter={94} suffix="%" label="подходящих кандидатов в первой десятке подборки. У поиска по словам из резюме 43 %" />
            <Fact value="87" counter={87} suffix="%" label="тестов определили грейд верно. Остальные ошиблись не больше чем на один грейд" />
            <Fact value="81" counter={81} suffix="%" label="пересдач другими заданиями дали тот же грейд" />
            <Fact value="9–10" suffix="%" label="кандидатов слабее уровня сдают тест. Слитые ответы эту долю не меняют" />
            <Fact value="13" counter={13} prefix="~" label="заданий в среднем, от 8 до 20" />
            <Fact value="62" counter={62} label="типа заданий в шести специализациях" />
            <Fact value="30" counter={30} suffix="в неделю" label="приглашений может отправить одна компания" />
            <Fact value="14" counter={14} suffix="дней" label="есть у кандидата, чтобы ответить на приглашение" />
          </div>
          <p data-reveal className="mt-14 max-w-3xl text-sm leading-relaxed text-ink-subtle">
            Профили синтетические, поэтому цифры описывают поведение сервиса, а не рынок. Следующий шаг: проверить подборку на реальных вакансиях партнёров ФСП, а тест на участниках соревнований.
          </p>
        </Container>
      </Section>

      <Section id="fsp" tone="canvas" className="overflow-hidden">
        <Container>
          <div className="grid grid-cols-1 items-center gap-14 lg:grid-cols-2 lg:gap-20">
            <div>
              <Eyebrow>Федерация спортивного программирования</Eyebrow>
              <h2 data-split className="display-2c mt-5 text-balance text-ink">
                Разряды и призовые места поднимают вас в подборке
              </h2>
              <p data-reveal className="mt-6 text-lg leading-relaxed text-ink-muted">
                Привяжите ФСП ID, и достижения из реестра Федерации появятся в профиле. Тест они не заменяют: за ФСП можно получить до 10 баллов из 100. Если истории ФСП нет, это ноль баллов без штрафа.
              </p>
              <dl data-stagger className="mt-10 grid grid-cols-1 gap-6 sm:grid-cols-2">
                {[
                  ["Вход через ФСП ID", "Пароль остаётся у ФСП, мы его не видим."],
                  ["Шесть дисциплин", "Алгоритмы, продуктовое программирование, ИБ, ИИ, робототехника и БАС."],
                  ["Свежие весят больше", "Вклад достижения уменьшается вдвое каждые два года."],
                  ["Можно скрыть", "Скрытые достижения не видны компаниям и не влияют на подборку."],
                ].map(([t, d]) => (
                  <div key={t} className="border-t border-line-strong pt-4">
                    <dt className="font-semibold text-ink">{t}</dt>
                    <dd className="mt-1.5 text-[15px] leading-relaxed text-ink-muted">{d}</dd>
                  </div>
                ))}
              </dl>
              <div data-reveal className="mt-10">
                <Cta href="/fsp" variant="ink" size="md">
                  Участникам ФСП
                </Cta>
              </div>
            </div>
            <div data-reveal="scale">
              <FspScore />
            </div>
          </div>
        </Container>
      </Section>

      <Section id="privacy" tone="night" className="overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute -bottom-40 left-1/4 size-[36rem] rounded-full bg-primary/25 blur-[150px]" />
        <Container className="relative">
          <SectionHead tone="night" eyebrow="Приватность" title="Контакты видны только после «Принять»" aside={<TextLink href="/candidates#privacy" tone="night">Подробнее о приватности</TextLink>} />
          <div data-stagger className="mt-14 grid grid-cols-1 gap-4 md:grid-cols-3">
            {[
              { icon: EyeOff, text: "Компания не видит ваше имя и контакты, пока вы не приняли её приглашение." },
              { icon: ScrollText, text: "Каждый просмотр ваших контактов записывается: какая компания и когда." },
              { icon: UserX, text: "Удалите аккаунт, и персональные данные сотрутся." },
            ].map(({ icon: Icon, text }) => (
              <div key={text} className="rounded-[22px] border border-night-line bg-night-raised p-7">
                <Icon className="size-6 text-primary-300" aria-hidden />
                <p className="mt-8 text-lg leading-snug font-medium">{text}</p>
              </div>
            ))}
          </div>
        </Container>
      </Section>

      <Section id="scenarios" tone="canvas">
        <Container>
          <SectionHead eyebrow="Сценарии" title="Три типичных пути" lead="Люди собирательные, шаги настоящие." />
          <div data-stagger="0.1" className="mt-16 grid grid-cols-1 gap-5 lg:grid-cols-3">
            {SCENARIOS.map((s) => (
              <article key={s.title} className="flex flex-col rounded-[24px] border border-line bg-white p-7 sm:p-8">
                <span className="w-fit rounded-full bg-primary-50 px-3 py-1 font-mono text-[11px] tracking-[0.1em] text-primary-strong uppercase">{s.tag}</span>
                <h3 className="mt-6 text-2xl leading-tight font-semibold tracking-tight text-ink">{s.title}</h3>
                <ol className="relative mt-8 space-y-6 before:absolute before:top-2 before:bottom-2 before:left-[11px] before:w-px before:bg-line-strong">
                  {s.steps.map((step, i) => (
                    <li key={step} className="relative flex gap-4">
                      <span className="relative z-10 flex size-6 shrink-0 items-center justify-center rounded-full border border-primary bg-white font-mono text-[11px] font-semibold text-primary">{i + 1}</span>
                      <p className="text-[15px] leading-relaxed text-ink-muted">{step}</p>
                    </li>
                  ))}
                </ol>
              </article>
            ))}
          </div>
        </Container>
      </Section>

      <Section id="faq">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <div className="lg:sticky lg:top-32 lg:self-start">
            <Eyebrow>FAQ</Eyebrow>
            <h2 data-split className="display-2 mt-5 text-ink">
              Вопросы и ответы
            </h2>
            <p data-reveal className="mt-6 text-lg leading-relaxed text-ink-muted">
              Подробнее про тест, приглашения и контакты рассказываем в разделах для кандидатов и работодателей.
            </p>
          </div>
          <div data-reveal>
            <Faq items={FAQ} />
          </div>
        </Container>
      </Section>

      <FinalCta />
    </MotionRoot>
  );
}

function FinalCta() {
  return (
    <Section tone="night" className="overflow-hidden">
      <div aria-hidden className="pointer-events-none absolute -top-40 left-1/2 size-[50rem] -translate-x-1/2 rounded-full bg-primary/35 blur-[160px]" />
      <BrandLine className="pointer-events-none absolute bottom-10 left-0 w-full text-primary-400/40" variant={3} />
      <Container className="relative">
        <h2 data-split className="display-1 mx-auto max-w-[16ch] text-center text-balance">
          Тест ждёт. <span className="text-gradient-night">Компании тоже.</span>
        </h2>
        <div data-stagger="0.12" className="mx-auto mt-16 grid max-w-5xl grid-cols-1 gap-5 md:grid-cols-2">
          {[
            {
              title: "Я ищу работу",
              items: ["Тест сдаётся один раз и работает на все приглашения", "Зарплату видно до разговора", "Контакты открываете только вы, кнопкой «Принять»"],
              href: "/register?role=candidate",
              cta: "Пройти тест",
            },
            {
              title: "Я нанимаю",
              items: ["Вставьте текст вакансии, стек и вилку сервис найдёт сам", "В подборке только те, кто сдал тест", "Причины отказов видны по каждому приглашению"],
              href: "/register?role=employer",
              cta: "Начать подбор",
            },
          ].map((c, i) => (
            <div key={c.title} className={i === 0 ? "rounded-[28px] bg-white p-8 text-ink sm:p-10" : "rounded-[28px] border border-white/15 bg-white/5 p-8 backdrop-blur sm:p-10"}>
              <p className="text-3xl font-semibold tracking-tight">{c.title}</p>
              <ul className="mt-6 space-y-3">
                {c.items.map((t) => (
                  <li key={t} className={i === 0 ? "flex gap-3 text-ink-muted" : "flex gap-3 text-night-muted"}>
                    <Check className={i === 0 ? "mt-0.5 size-5 shrink-0 text-primary" : "mt-0.5 size-5 shrink-0 text-primary-300"} aria-hidden />
                    {t}
                  </li>
                ))}
              </ul>
              <div className="mt-10">
                <Cta href={c.href} variant={i === 0 ? "primary" : "light"}>
                  {c.cta}
                </Cta>
              </div>
            </div>
          ))}
        </div>
      </Container>
    </Section>
  );
}
