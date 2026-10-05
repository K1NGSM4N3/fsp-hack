import {
  ArrowDownRight,
  Banknote,
  BellOff,
  Check,
  EyeOff,
  FileLock,
  KeyRound,
  LockKeyhole,
  ScrollText,
  Server,
  Shuffle,
  UserCheck,
} from "lucide-react";
import Link from "next/link";
import type { ReactNode } from "react";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { Faq, type FaqItem } from "@/components/site/Faq";
import { FlipTable } from "@/components/site/FlipTable";
import { FspScore } from "@/components/site/FspScore";
import { HeroGraph } from "@/components/site/HeroGraph";
import { Journey } from "@/components/site/Journey";
import { LeakChart, LeakDemo } from "@/components/site/LeakDemo";
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
              Платформа подбора ИТ-специалистов
            </p>
            <h1 data-hero data-split="hero" className="display-1 max-w-[15ch] text-balance">
              Найм наоборот: <span className="text-gradient-night">первым пишет работодатель</span>
            </h1>
            <p data-hero className="mt-8 max-w-xl text-lg leading-relaxed text-night-muted sm:text-xl">
              Подтвердите уровень один раз — адаптивным тестом, а не резюме. Дальше компании приходят сами: с задачей, форматом и зарплатной вилкой «от–до» в первом же сообщении.
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
                Войти в кабинет
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
                <p className="mt-1 text-sm text-night-muted">Финтех · гибрид, Москва · через платформу</p>
                <div className="mt-5 rounded-2xl border border-white/10 bg-white/5 p-4">
                  <p className="text-xs text-night-muted">Зарплатная вилка</p>
                  <p className="tabular mt-1 font-mono text-2xl font-semibold tracking-tight">280 000 – 350 000 ₽</p>
                </div>
                <ul className="mt-5 space-y-2 text-sm">
                  <li className="flex items-center gap-2.5">
                    <Check className="size-4 text-success-300" aria-hidden /> Категория Бэкенд · Middle подтверждена тестом
                  </li>
                  <li className="flex items-center gap-2.5">
                    <Check className="size-4 text-success-300" aria-hidden /> Go и PostgreSQL — подтверждены тестом
                  </li>
                </ul>
                <div className="mt-6 grid grid-cols-2 gap-2">
                  <span className="inline-flex h-11 items-center justify-center rounded-xl bg-white text-sm font-medium text-ink">Принять</span>
                  <span className="inline-flex h-11 items-center justify-center rounded-xl border border-white/15 text-sm font-medium text-white">Отклонить</span>
                </div>
                <p className="mt-4 flex items-center gap-2 text-xs text-night-faint">
                  <LockKeyhole className="size-3.5" aria-hidden /> Компания увидит контакты только после «Принять»
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
                  <span className="block text-sm font-medium text-white">Балл соответствия</span>с расшифровкой по 8 признакам
                </span>
              </div>
            </div>
          </div>
        </div>

        <dl data-hero className="mt-20 grid grid-cols-2 gap-x-6 gap-y-8 border-t border-white/10 pt-8 lg:mt-24 lg:grid-cols-4">
          {[
            ["0,94", "доля релевантных в топ-10 подборки — против 0,43 у поиска по ключевым словам резюме"],
            ["87 %", "итоговый грейд совпадает с истинным в симуляции, κ при пересдаче — 0,74"],
            ["~13", "заданий в среднем: тест адаптивный и заканчивается, когда уровень ясен"],
            ["9–10 %", "слабых кандидатов проходят тест — даже когда ответы слиты"],
          ].map(([v, l]) => (
            <div key={v}>
              <dt className="sr-only">{l}</dt>
              <dd>
                <span className="tabular block font-display text-3xl font-semibold tracking-[-0.03em] sm:text-4xl">{v}</span>
                <span className="mt-2 block max-w-[17rem] text-sm leading-relaxed text-night-muted">{l}</span>
              </dd>
            </div>
          ))}
        </dl>
      </Container>
      <a href="#problem" className="group absolute right-6 bottom-6 hidden items-center gap-2 font-mono text-xs tracking-[0.12em] text-night-faint uppercase transition-colors hover:text-white lg:inline-flex">
        Листайте
        <ArrowDownRight className="size-4 transition-transform group-hover:translate-y-0.5" aria-hidden />
      </a>
    </section>
  );
}

const PAINS = [
  {
    title: "Сотни откликов — ноль сигнала",
    text: "Резюме пишут по одним шаблонам, рекрутер ищет по ключевым словам. В нашей симуляции поиск по ключевым словам резюме даёт лишь 43 % релевантных кандидатов в первой десятке.",
  },
  {
    title: "Зарплата — на третьем этапе",
    text: "Вилку часто узнают после пары собеседований. Отказ «не сошлись по деньгам» случается в самом конце, когда время обеих сторон уже потрачено.",
  },
  {
    title: "Тесты сливаются за неделю",
    text: "Фиксированный тест живёт до первой утечки: стоит слить половину ответов — и его проходят 99,9 % слабых кандидатов.",
  },
  {
    title: "Рассылки вместо предложений",
    text: "«Посмотрите нашу вакансию» без условий и без понимания, кто перед тобой. Сильные специалисты просто перестают отвечать.",
  },
];

const TRUST = [
  { icon: EyeOff, title: "Анонимно до вашего «да»", text: "Компания видит категорию, навыки и результат теста — без имени, телефона и почты. Названия прошлых работодателей — только если вы разрешили." },
  { icon: FileLock, title: "Контакты закрыты в коде", text: "Карточка для работодателя физически не содержит полей контактов. Открыть их может одна функция — и только при принятом приглашении или вашем отклике." },
  { icon: ScrollText, title: "Журнал каждого доступа", text: "Любое обращение к контактам записывается: кто, когда и на каком основании. Не «доверьтесь нам», а проверяемый след." },
  { icon: KeyRound, title: "Шифрование персональных данных", text: "Email, ФИО и контакты хранятся зашифрованными (XChaCha20-Poly1305), поиск по email — через keyed-хеш, без открытого текста." },
  { icon: UserCheck, title: "Согласия и удаление", text: "Версионированные согласия с фиксацией времени. Отзыв согласия на публикацию скрывает профиль, удаление аккаунта стирает персональные данные." },
  { icon: Server, title: "Хранение в России", text: "Серверы в РФ, требования 152-ФЗ. Криптография вынесена за интерфейсы — переход на ГОСТ не потребует менять схему данных." },
];

const SCENARIOS = [
  {
    tag: "Кандидат",
    title: "Middle-бэкендер, который устал откликаться",
    steps: [
      "Отмечает в опросе Go, PostgreSQL и Kafka, заявляет Middle",
      "Проходит адаптивный тест; сдал уверенно — система сразу предлагает дотест на Senior",
      "Получает приглашения с вилкой и форматом, неподходящие отклоняет в один клик — причина уходит компании анонимно",
      "Принимает одно — и только эта компания видит его контакты",
    ],
  },
  {
    tag: "Работодатель",
    title: "Тимлид, которому нужен Go-разработчик к релизу",
    steps: [
      "Вставляет текст вакансии — система распознаёт Middle, Go, PostgreSQL, Kafka и вилку",
      "Смотрит рекомендованные категории: сколько кандидатов, медианные ожидания, покрытие стека",
      "Уточняет фильтры — подборка не теряется, закреплённые кандидаты остаются на месте",
      "Отправляет пять приглашений с вилкой и видит воронку и причины отказов",
    ],
  },
  {
    tag: "Участник ФСП",
    title: "Студент — призёр соревнований по программированию",
    steps: [
      "Привязывает ФСП ID: разряд и призовые места подтягиваются из реестра Федерации",
      "Сдаёт тест на Junior — навыки из решённых заданий отмечены «подтверждён тестом»",
      "Внутри категории стоит выше сверстников с тем же результатом, но без достижений",
      "Решает регулярные задания от компаний — профиль остаётся «свежим» в подборках",
    ],
  },
];

const FAQ: FaqItem[] = [
  { q: "Почему категорию даёт тест, а не резюме?", a: "Резюме описывает, как человек себя видит; тест измеряет, что он умеет. Резюме и опрос влияют только на вторичные признаки — стек и ожидания, — но никогда не определяют категорию «специализация × грейд»." },
  { q: "Сколько длится тест?", a: "От 8 до 20 заданий, в среднем около 13. На каждое задание — лимит от 60 до 180 секунд; просроченный ответ засчитывается как неверный. Тест останавливается, как только уровень определён с нужной уверенностью." },
  { q: "Что будет, если я не сдам?", a: "Можно сразу пройти тест на грейд ниже. Повтор того же или более высокого грейда — через 30 дней, чтобы тест нельзя было перебирать. Не сдали Junior — пересдача через 30 дней." },
  { q: "Как поднять грейд?", a: "Если сдали уверенно, система сразу один раз предложит дотест на грейд выше. После присвоения грейд меняется не чаще раза в 90 дней, а неудачная попытка повышения текущий грейд не понижает." },
  { q: "А если задания утекут в интернет?", a: "Ничего не сломается: следующий кандидат получит другие числа, код и ответы. Слитый шаблон учит решать класс задач — то есть ровно тому навыку, который мы и измеряем." },
  { q: "Кто увидит мои контакты?", a: "Только компания, чьё приглашение вы приняли, или та, на чью вакансию вы откликнулись сами. Каждый доступ к контактам записывается в журнал." },
  { q: "У меня нет достижений ФСП. Это минус?", a: "Нет. Без истории ФСП скор равен нулю, а не штрафу: в карточке нейтральное «Нет данных ФСП», и это никогда не попадает в список пробелов." },
  { q: "Почему компания не может написать всем сразу?", a: "Квота — не больше 30 приглашений за 7 дней и одно действующее приглашение на пару «компания–кандидат». Приглашение живёт 14 дней. Так предложения остаются точечными, а кандидаты не тонут в спаме." },
  { q: "Классические вакансии остались?", a: "Да. Можно откликнуться на опубликованную вакансию — в этом случае контакты открываются по вашему отклику. Но основной путь — приглашения от работодателей." },
];

export default function HomePage() {
  return (
    <MotionRoot>
      <Hero />

      <div className="border-b border-line bg-canvas py-10 sm:py-14">
        <p className="eyebrow mb-7 text-center text-ink-subtle">6 специализаций · 83 навыка · 491 синоним — «кубер» и «постгрес» тоже понимаем</p>
        <Marquee items={STACK_A.map((s) => <StackChip key={s}>{s}</StackChip>)} label="Технологии в тестах и подборе" />
        <Marquee items={STACK_B.map((s, i) => <StackChip key={s} accent={i === 5}>{s}</StackChip>)} reverse duration={52} className="mt-4" />
      </div>

      <Section id="problem">
        <Container>
          <SectionHead
            eyebrow="Проблема"
            title={
              <>
                Обычный найм — лотерея <span className="text-gradient">с обеих сторон</span>
              </>
            }
            lead="Кандидат отправляет отклики в пустоту, компания разбирает сотни одинаковых резюме, а самое важное — уровень и деньги — выясняется последним."
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
                <Kicker index="→">Решение</Kicker>
                <h2 data-split className="display-2c mt-5 text-balance text-ink">
                  Мы перевернули схему. Буквально.
                </h2>
              </div>
              <div data-reveal>
                <p className="text-lg leading-relaxed text-ink-muted">
                  Уровень подтверждается до того, как кто-то кому-то напишет. Поэтому разговор начинается не с «расскажите о себе», а с конкретной задачи и конкретных денег.
                </p>
                <div className="mt-6">
                  <TextLink href="/how-it-works">Как устроены тест и подбор</TextLink>
                </div>
              </div>
            </div>
            <div className="mt-12">
              <FlipTable />
            </div>
          </div>
        </Container>
      </Section>

      <Journey />

      <Section id="card" tone="canvas" className="overflow-hidden">
        <BrandLine className="pointer-events-none absolute top-16 left-0 w-full text-primary-200" variant={2} />
        <Container className="relative">
          <div className="grid grid-cols-1 items-start gap-14 lg:grid-cols-[0.85fr_1.15fr] lg:gap-16">
            <div className="lg:sticky lg:top-32">
              <Eyebrow>Для работодателя</Eyebrow>
              <h2 data-split className="display-2c mt-5 text-balance text-ink">
                Карточка без имени. Балл — с расшифровкой.
              </h2>
              <p data-reveal className="mt-6 text-lg leading-relaxed text-ink-muted">
                Компания видит то, что важно для решения: категорию, силу результата теста, навыки с источником подтверждения и достижения ФСП. Каждый балл 0–100 раскладывается на восемь признаков — веса подобраны на валидации.
              </p>
              <ul data-stagger className="mt-8 space-y-4">
                {[
                  "Источник навыка имеет вес: тест и ФСП — 1,0, резюме — 0,7, «просто заявлен» — 0,5",
                  "Пробелы показываются честно, близкие навыки тоже: «вместо PostgreSQL — MySQL»",
                  "Отсутствие ФСП никогда не попадает в пробелы — это ноль, а не минус",
                  "Подбор 10 000 кандидатов с объяснением топ-20 — около 0,3 мс",
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

      <Section id="test" tone="night" className="overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute top-1/4 -right-48 size-[40rem] rounded-full bg-primary/25 blur-[150px]" />
        <Container className="relative">
          <SectionHead
            tone="night"
            eyebrow="Тестирование"
            title="Тест, который бесполезно сливать"
            lead="Каждое задание — параметрический шаблон. Кандидаты получают одну тему и одну сложность, но разные числа, код и данные. Правильный ответ не хранится: его вычисляет генератор — мини-симулятор event loop, расчёт подсети, SQL на сгенерированной таблице."
          />
          <div className="mt-14 grid grid-cols-1 gap-8 lg:grid-cols-[1.05fr_0.95fr] lg:items-start">
            <div data-reveal>
              <LeakDemo />
            </div>
            <div data-reveal="scale">
              <LeakChart />
              <p className="mt-4 text-sm leading-relaxed text-night-muted">
                Симуляция: слабые кандидаты (θ от −0,9 до −0,1) сдают тест на Middle. Для нашего теста «утечка» — база из ~42 тыс. экземпляров, показанных в 600 предыдущих сессиях; совпадения случаются в 0,02–0,05 % заданий.
              </p>
            </div>
          </div>
          <div data-stagger className="mt-16 grid grid-cols-1 gap-px overflow-hidden rounded-[24px] border border-night-line bg-night-line sm:grid-cols-3">
            {[
              { icon: Shuffle, title: "62 шаблона", text: "52 вычислительных генератора и 10 вариативных пулов. Каждый уровень даёт ≥ 1961 различимый экземпляр на 2000 seed." },
              { icon: Banknote, title: "Одна шкала", text: "Шаблоны откалиброваны по IRT (3PL), поэтому разные задания дают сравнимый результат у всех кандидатов." },
              { icon: BellOff, title: "Защита от перебора", text: "Лимит времени на задание и контроль person-fit: «верно на трудных, неверно на лёгких» помечается для проверки." },
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

      <Section id="numbers">
        <Container>
          <SectionHead
            eyebrow="Метрики"
            title="Цифры, за которые мы отвечаем"
            lead="Не «AI-подбор», а измеримое качество: собственная процедура валидации вызывает те же модули, что и боевой API, и пишет отчёты в репозиторий."
            aside={<TextLink href="/how-it-works#validation">Методика валидации</TextLink>}
          />
          <div data-stagger="0.06" className="mt-16 grid grid-cols-1 gap-x-8 gap-y-12 sm:grid-cols-2 lg:grid-cols-4">
            <Fact value="62" counter={62} label="параметрических шаблона заданий в шести специализациях" />
            <Fact value="0,94" counter={0.94} decimals={2} label="доля релевантных в топ-10 подборки против 0,43 у поиска по ключевым словам" />
            <Fact value="87" counter={87} suffix="%" label="итоговый грейд совпадает с истинным в симуляции на 2 400 профилях" />
            <Fact value="0,74" counter={0.74} decimals={2} label="κ Коэна при пересдаче другими заданиями — результат воспроизводим" />
            <Fact value="13" counter={13} prefix="~" label="заданий в среднем; минимум 8, максимум 20" />
            <Fact value="30" counter={30} suffix="/ нед" label="приглашений на компанию — потолок, чтобы предложения оставались точечными" />
            <Fact value="14" counter={14} suffix="дней" label="живёт приглашение; одно действующее на пару «компания–кандидат»" />
            <Fact value="0,3" counter={0.3} decimals={1} suffix="мс" label="на подбор по 10 000 кандидатов с объяснением топ-20" />
          </div>
          <p data-reveal className="mt-14 max-w-3xl text-sm leading-relaxed text-ink-subtle">
            Метрики подбора и тестирования получены на синтетических данных со скрытой «истиной» — абсолютные значения описывают поведение системы в заданных предпосылках. Следующий шаг — экспертная разметка реальных вакансий партнёров ФСП и пилот на участниках соревнований.
          </p>
        </Container>
      </Section>

      <Section id="fsp" tone="canvas" className="overflow-hidden">
        <Container>
          <div className="grid grid-cols-1 items-center gap-14 lg:grid-cols-2 lg:gap-20">
            <div>
              <Eyebrow>Федерация спортивного программирования</Eyebrow>
              <h2 data-split className="display-2c mt-5 text-balance text-ink">
                Разряд и призовые места — в плюс. Их отсутствие — не в минус.
              </h2>
              <p data-reveal className="mt-6 text-lg leading-relaxed text-ink-muted">
                Привяжите ФСП ID — и достижения из реестра Федерации подтянутся в профиль сами. Они поднимают вас внутри категории, но не заменяют тест: вес ФСП в подборе — не больше 10 баллов из 100.
              </p>
              <dl data-stagger className="mt-10 grid grid-cols-1 gap-6 sm:grid-cols-2">
                {[
                  ["Вход через ФСП ID", "OpenID Connect с PKCE на Keycloak — пароль от ФСП мы не видим"],
                  ["Шесть дисциплин", "алгоритмическое, продуктовое, ИБ, ИИ, робототехника, БАС"],
                  ["Свежесть важна", "вклад достижения полураспадается за 2 года"],
                  ["Под вашим контролем", "ФСП-данные можно скрыть — тогда они не участвуют и в ранжировании"],
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

      <Section id="trust">
        <Container>
          <SectionHead
            eyebrow="Доверие и данные"
            title="Контакты — только после вашего «да»"
            lead="Приватность здесь — не галочка в настройках, а свойство системы: то, чего компания не должна видеть, отсутствует в её данных."
            aside={<TextLink href="/security">Безопасность и 152-ФЗ</TextLink>}
          />
          <div data-stagger="0.07" className="mt-16 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {TRUST.map(({ icon: Icon, title, text }) => (
              <div key={title} className="group relative overflow-hidden rounded-[22px] border border-line bg-white p-7 transition-colors hover:border-primary-200">
                <div aria-hidden className="absolute -top-16 -right-16 size-40 rounded-full bg-primary-50 opacity-0 transition-opacity duration-500 group-hover:opacity-100" />
                <span className="relative flex size-12 items-center justify-center rounded-2xl bg-ink text-white">
                  <Icon className="size-5" aria-hidden />
                </span>
                <h3 className="relative mt-8 text-xl font-semibold tracking-tight text-ink">{title}</h3>
                <p className="relative mt-2.5 text-[15px] leading-relaxed text-ink-muted">{text}</p>
              </div>
            ))}
          </div>
        </Container>
      </Section>

      <Section id="scenarios" tone="canvas">
        <Container>
          <SectionHead eyebrow="Сценарии" title="Как это выглядит в жизни" lead="Три типичных пути через платформу — без выдуманных отзывов, только то, что происходит на каждом шаге." />
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
              Частые вопросы
            </h2>
            <p data-reveal className="mt-6 text-lg leading-relaxed text-ink-muted">
              Не нашли ответ? Подробности о тесте, подборе и данных — в разделах для кандидатов, работодателей и в описании устройства платформы.
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
          Хватит откликаться. <span className="text-gradient-night">Пусть откликаются на вас.</span>
        </h2>
        <div data-stagger="0.12" className="mx-auto mt-16 grid grid-cols-1 max-w-5xl gap-5 md:grid-cols-2">
          {[
            {
              title: "Я ищу работу",
              items: ["Тест один раз — категория на все приглашения", "Вилка видна до начала разговора", "Контакты — только после вашего согласия"],
              href: "/register?role=candidate",
              cta: "Пройти тест",
            },
            {
              title: "Я нанимаю",
              items: ["Потребность текстом — система распознает стек и вилку", "Подборка из подтверждённых категорий с объяснением", "Воронка и причины отказов по каждому приглашению"],
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
