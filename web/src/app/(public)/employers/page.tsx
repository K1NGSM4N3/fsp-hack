import { Ban, Banknote, BellOff, Building2, Clock, FileSearch, Handshake, ListChecks, Megaphone, Search, Send, ShieldCheck, Target, Trophy, Users } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, Note, PageHero, SplitHeading, WeightBars } from "@/components/site/blocks";
import { FunnelDraw } from "@/components/site/employers/FunnelDraw";
import { LiveShortlist } from "@/components/site/employers/LiveShortlist";
import { QuotaMeter } from "@/components/site/employers/QuotaMeter";
import { Faq } from "@/components/site/Faq";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section, TextLink } from "@/components/site/primitives";
import { cn } from "@/lib/cn";

export const metadata: Metadata = {
  title: "Работодателям",
  description: "Подборка из кандидатов, сдавших тест, объяснение каждого места, приглашения с обязательной вилкой и статистика отказов.",
};

const WEIGHTS = [
  { label: "Обязательные навыки", value: 41, note: "навык из теста или ФСП весит больше заявленного на словах" },
  { label: "Грейд", value: 15, note: "точное совпадение даёт полный балл, соседний грейд около половины" },
  { label: "Результат теста", value: 10, note: "насколько уверенно сдан грейд" },
  { label: "Достижения ФСП", value: 10, note: "без истории ФСП ноль, без штрафа" },
  { label: "Желательные навыки", value: 8, note: "считаются так же, как обязательные" },
  { label: "Зарплата", value: 7, note: "ожидания внутри вилки дают полный балл" },
  { label: "Актуальность", value: 5, note: "как давно кандидат был активен" },
  { label: "Совпадение описания", value: 4, note: "насколько текст «о себе» похож на вашу вакансию" },
];

const REASONS = [
  { label: "Зарплата", value: 44, display: "44" },
  { label: "Стек", value: 22, display: "22" },
  { label: "Формат работы", value: 17, display: "17" },
  { label: "Не ищу работу", value: 11, display: "11" },
  { label: "Компания", value: 6, display: "6" },
];

const CATEGORIES = [
  { grade: "Junior", people: 10, fsp: 0, median: "130 000 ₽", stack: 65, note: "Соседний грейд, можно расширить поиск" },
  { grade: "Middle", people: 5, fsp: 1, median: "280 000 ₽", stack: 75, note: "Совпадает с запросом", best: true },
  { grade: "Senior", people: 1, fsp: 0, median: "380 000 ₽", stack: 67, note: "Медиана ожиданий выше вашей вилки" },
];

const RULES = [
  { icon: Banknote, v: "от–до", t: "Зарплата обязательна", d: "Без вилки приглашение не отправится." },
  { icon: BellOff, v: "1", t: "приглашение одному человеку", d: "Пока кандидат не ответил, второе ему не отправить." },
  { icon: Clock, v: "14", t: "дней на ответ", d: "Потом приглашение истекает и попадает в воронку." },
  { icon: ShieldCheck, v: "0", t: "контактов до согласия", d: "Пока кандидат не нажал «Принять», контактов в карточке нет." },
  { icon: Ban, v: "!", t: "Жалобы", d: "Кандидат может пожаловаться на приглашение, и мы это разберём." },
];

export default function EmployersPage() {
  return (
    <MotionRoot variant="center">
      <PageHero
        crumb="Работодателям"
        title={
          <>
            Подборка из тех, <span className="text-gradient">кто уже сдал тест</span>
          </>
        }
        lead="Вставьте текст вакансии. Сервис найдёт в нём стек, грейд и вилку, покажет подходящие категории и кандидатов с баллом от 0 до 100. Вы пишете первыми, сразу с зарплатой, и видите, почему вам отвечают или отказывают."
        actions={
          <>
            <Cta href="/register?role=employer">Начать подбор</Cta>
            <Cta href="/login" variant="outline">
              Войти в кабинет
            </Cta>
          </>
        }
        visual={<LiveShortlist />}
        scramble
        facts={[
          { value: "94 %", label: "подходящих кандидатов в первой десятке подборки" },
          { value: "43 %", label: "даёт поиск по словам из резюме на тех же данных" },
          { value: "8", label: "признаков в балле, каждый расписан" },
          { value: "100 %", label: "кандидатов в поиске сдали тест" },
        ]}
      />
      <PageTabs
        items={[
          { id: "flow", label: "Как это работает" },
          { id: "parse", label: "Вакансия" },
          { id: "categories", label: "Категории" },
          { id: "shortlist", label: "Подборка" },
          { id: "rules", label: "Правила" },
          { id: "analytics", label: "Аналитика" },
          { id: "test-plan", label: "Что проверит тест" },
          { id: "faq", label: "Вопросы" },
        ]}
      />

      <Section id="flow" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Процесс" title="Шесть шагов от вакансии до контактов">
            Разбирать отклики не нужно. Вы приходите в категорию, где уровень каждого уже проверен, и выбираете сами.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: FileSearch, title: "1. Описание", text: "Форма или текст вакансии целиком. Публиковать вакансию не обязательно." },
              { icon: Target, title: "2. Категории", text: "Сколько в них людей, у скольких есть достижения ФСП, сколько они просят и насколько совпадает стек." },
              { icon: Users, title: "3. Подборка", text: "Анонимные карточки, отсортированные по баллу. Причины и пробелы написаны словами." },
              { icon: Send, title: "4. Приглашение", text: "Задача, зарплата от и до, формат и контакт для связи. Кандидат видит всё это до разговора." },
              { icon: Handshake, title: "5. Ответ", text: "Принял, и вы получаете контакты. Отклонил, и причина попадает в вашу статистику." },
              { icon: ShieldCheck, title: "6. Контакты", text: "Открываются по принятому приглашению или отклику. Каждый просмотр записывается." },
            ]}
          />
        </Container>
      </Section>

      <Section id="parse" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Описание вакансии" title="Вставьте вакансию как есть">
            Сервис знает 83 навыка и 491 вариант их написания. «Постгрес», «кубер» и «джава» тоже считаются.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-2">
            <div data-reveal="wipe" className="rounded-[24px] border border-line bg-white p-7">
              <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Текст вакансии</p>
              <p className="mt-5 text-lg leading-relaxed text-ink-muted">
                Ищем <mark className="rounded bg-primary-100 px-1 text-ink">мидла</mark> на <mark className="rounded bg-primary-100 px-1 text-ink">Go</mark> в платёжную команду. Работа с <mark className="rounded bg-primary-100 px-1 text-ink">постгресом</mark> и{" "}
                <mark className="rounded bg-primary-100 px-1 text-ink">Kafka</mark> обязательна, опыт с <mark className="rounded bg-canvas px-1 text-ink">кубером</mark> будет плюсом. <mark className="rounded bg-primary-100 px-1 text-ink">Гибрид</mark>, Москва.
                Вилка <mark className="rounded bg-primary-100 px-1 text-ink">250–320 тыс.</mark>
              </p>
            </div>
            <dl data-stagger="0.07" className="rounded-[24px] border border-line bg-white p-7">
              <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Что нашёл сервис</p>
              {[
                ["Специализация", "Бэкенд-разработка"],
                ["Грейд", "Middle"],
                ["Обязательные навыки", "Go · PostgreSQL · Kafka"],
                ["Желательные навыки", "Kubernetes"],
                ["Вилка", "250 000 – 320 000 ₽"],
                ["Формат", "Гибрид, Москва"],
              ].map(([k, v]) => (
                <div key={k} className="flex items-baseline justify-between gap-4 border-b border-line py-3 last:border-0">
                  <dt className="text-sm text-ink-subtle">{k}</dt>
                  <dd className="text-right font-medium text-ink">{v}</dd>
                </div>
              ))}
            </dl>
          </div>
          <div className="mt-8">
            <Note>Всё найденное можно поправить вручную до подбора.</Note>
          </div>
        </Container>
      </Section>

      <Section id="categories" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Категории" title="Сначала категории, потом люди">
            Для каждой категории видно, сколько в ней кандидатов и сколько они хотят. Так быстрее понять, реальна ли ваша вилка.
          </SplitHeading>
          <div data-stagger="0.12" className="mt-14 grid grid-cols-1 gap-4 md:grid-cols-3">
            {CATEGORIES.map((c) => (
              <article key={c.grade} className={cn("rounded-[28px] border bg-white p-7", c.best ? "border-primary shadow-glow" : "border-line")}>
                <div className="flex items-center justify-between gap-3">
                  <span className="inline-flex h-7 items-center overflow-hidden rounded-md border border-primary-200 bg-primary-50 font-mono text-xs text-primary-strong">
                    <span className="px-2">Бэкенд</span>
                    <span className="h-full w-px bg-primary-200" />
                    <span className="px-2">{c.grade}</span>
                  </span>
                  {c.best && <span className="rounded-full bg-primary px-2.5 py-1 text-[11px] font-medium text-white">Рекомендуем</span>}
                </div>
                <dl className="mt-6 grid grid-cols-2 gap-4">
                  <div>
                    <dt className="text-xs text-ink-subtle">Кандидатов</dt>
                    <dd data-scramble className="tabular font-display text-3xl font-semibold text-ink">
                      {c.people}
                    </dd>
                  </div>
                  <div>
                    <dt className="flex items-center gap-1 text-xs text-ink-subtle">
                      <Trophy className="size-3.5" aria-hidden /> с ФСП
                    </dt>
                    <dd data-scramble className="tabular font-display text-3xl font-semibold text-ink">
                      {c.fsp}
                    </dd>
                  </div>
                </dl>
                <p className="mt-5 text-xs text-ink-subtle">Медиана ожиданий</p>
                <p className="tabular font-mono text-lg font-semibold text-ink">{c.median}</p>
                <div className="mt-4">
                  <p className="mb-1.5 text-xs text-ink-subtle">Совпадение со стеком: {c.stack} %</p>
                  <div className="h-1.5 overflow-hidden rounded-full bg-primary-50">
                    <div data-grow className="h-full rounded-full bg-primary" style={{ width: `${c.stack}%` }} />
                  </div>
                </div>
                <p className="mt-4 text-sm text-ink-muted">{c.note}</p>
              </article>
            ))}
          </div>
          <p data-reveal="fade" className="mt-4 text-xs text-ink-subtle">
            Пример экрана с демо-данными.
          </p>
        </Container>
      </Section>

      <Section id="shortlist" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Подборка" title="Каждое место в подборке объяснено">
            Сначала отсекаются те, кто не ищет работу или не подходит по специализации, грейду, формату и городу. Остальные получают балл из восьми признаков.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-12 lg:grid-cols-[1.1fr_0.9fr] lg:gap-16">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <div className="mb-7 flex items-baseline justify-between">
                <p className="text-lg font-semibold tracking-tight text-ink">Из чего складывается балл</p>
                <p className="font-mono text-xs text-ink-subtle">из 100</p>
              </div>
              <WeightBars rows={WEIGHTS} max={41} />
            </div>
            <div className="space-y-8">
              {[
                { t: "Уточнения не сбрасывают подборку", d: "Поменяли фильтры, и подборка пересчиталась. Закреплённые кандидаты остаются на месте, скрытые не возвращаются, новые помечены." },
                { t: "Пробелы написаны прямо", d: "«Нет обязательного навыка: Kubernetes». «Вместо PostgreSQL близкий навык MySQL». «Навык Docker только заявлен»." },
                { t: "Можно смотреть категорию целиком", d: "Без описания вакансии кандидаты отсортированы по результату теста, достижениям ФСП и свежести профиля." },
                { t: "Веса подобраны на проверке", d: "Мы сравнили несколько вариантов весов. Лучше всего сработал тот, где больше веса у стека и грейда." },
              ].map((x) => (
                <div key={x.t} data-reveal className="border-t border-line pt-6">
                  <h3 className="text-xl font-semibold tracking-tight text-ink">{x.t}</h3>
                  <p className="mt-2 text-[15px] leading-relaxed text-ink-muted">{x.d}</p>
                </div>
              ))}
            </div>
          </div>
        </Container>
      </Section>

      <Section id="rules" tone="night" className="scroll-mt-16 overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute -top-40 right-0 size-[36rem] rounded-full bg-primary/30 blur-[150px]" />
        <Container className="relative">
          <SplitHeading eyebrow="Правила контакта" title="Правила, которые нельзя обойти" tone="night">
            Кандидаты охотнее отвечают, когда не тонут в рассылках и видят условия сразу. Поэтому ограничения одинаковые для всех компаний.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-[1.1fr_0.9fr]">
            <div data-reveal>
              <QuotaMeter />
            </div>
            <div data-stagger="0.07" className="grid grid-cols-1 gap-px overflow-hidden rounded-[24px] border border-night-line bg-night-line sm:grid-cols-2">
              {RULES.map(({ icon: Icon, v, t, d }, i) => (
                <div key={t} className={cn("bg-night-raised p-6", i === RULES.length - 1 && "sm:col-span-2")}>
                  <div className="flex items-center justify-between">
                    <span data-scramble={/^\d+$/.test(v) ? "" : undefined} className="tabular font-display text-4xl font-semibold tracking-[-0.04em] text-white">
                      {v}
                    </span>
                    <Icon className="size-5 text-primary-300" aria-hidden />
                  </div>
                  <h3 className="mt-5 font-semibold text-white">{t}</h3>
                  <p className="mt-1.5 text-sm leading-relaxed text-night-muted">{d}</p>
                </div>
              ))}
            </div>
          </div>
        </Container>
      </Section>

      <Section id="analytics" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Аналитика" title="Видно, кто ответил и почему отказал">
            Кандидат отклоняет приглашение с причиной. Вы видите воронку по всем приглашениям и сводку причин без имён.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-2">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <div className="mb-6 flex items-baseline justify-between">
                <p className="text-lg font-semibold tracking-tight text-ink">Воронка приглашений</p>
                <span className="rounded-full bg-canvas px-2.5 py-1 font-mono text-[11px] text-ink-subtle">пример</span>
              </div>
              <FunnelDraw />
            </div>
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <div className="mb-7 flex items-baseline justify-between">
                <p className="text-lg font-semibold tracking-tight text-ink">Почему отказывают</p>
                <span className="rounded-full bg-canvas px-2.5 py-1 font-mono text-[11px] text-ink-subtle">пример, %</span>
              </div>
              <WeightBars rows={REASONS} max={44} unit=" %" />
              <p className="mt-6 rounded-2xl bg-warning-50 p-4 text-sm text-ink">Когда из-за денег отказывают в 40 % случаев и чаще, кабинет предложит сравнить вилку с медианой категории.</p>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="test-plan" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Что проверит тест" title="Тест под вашу вакансию можно посмотреть заранее">
            По описанию вакансии видно, какие темы получат приоритет и как выглядят задания у двух разных кандидатов.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-[0.9fr_1.1fr]">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7">
              <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Middle Go-разработчик: PostgreSQL, Kafka</p>
              <p className="mt-5 font-medium text-ink">Темы с приоритетом</p>
              <ul className="mt-3 flex flex-wrap gap-2">
                {["SQL: GROUP BY и HAVING", "SQL: LEFT JOIN", "Go: порядок defer", "Выбор индекса", "Аномалии транзакций"].map((t) => (
                  <li key={t} className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1.5 text-xs text-primary-strong">
                    {t}
                  </li>
                ))}
              </ul>
              <p className="mt-6 text-[15px] leading-relaxed text-ink-muted">Это темы, связанные с вашим стеком. Планка для грейда при этом та же, что и для остальных кандидатов.</p>
            </div>
            <div data-reveal className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              {[
                ["Кандидат А", "600"],
                ["Кандидат Б", "750"],
              ].map(([who, n]) => (
                <div key={who} className="rounded-[24px] bg-night p-6 text-white">
                  <p className="text-sm font-medium">{who}</p>
                  <pre className="mt-4 overflow-x-auto font-mono text-[13px] leading-6 text-night-muted">
                    <code>
                      <span className="text-primary-300">SELECT</span> c.city, <span className="text-primary-300">COUNT</span>(*){"\n"}
                      <span className="text-primary-300">FROM</span> orders o{"\n"}
                      <span className="text-primary-300">JOIN</span> customers c{"\n"}
                      {"  "}
                      <span className="text-primary-300">ON</span> c.id = o.customer_id{"\n"}
                      <span className="text-primary-300">WHERE</span> o.amount &gt; <span className="rounded bg-primary/30 px-1 text-white">{n}</span>
                      {"\n"}
                      <span className="text-primary-300">GROUP BY</span> c.city
                    </code>
                  </pre>
                  <p className="mt-4 text-xs text-night-faint">то же задание, другие данные</p>
                </div>
              ))}
            </div>
          </div>
          <div data-reveal className="mt-10">
            <TextLink href="/#how">Как устроен тест</TextLink>
          </div>
        </Container>
      </Section>

      <Section tone="canvas">
        <Container>
          <SplitHeading eyebrow="В кабинете" title="Остальное для найма тоже здесь" />
          <FeatureGrid
            className="mt-14"
            columns={4}
            items={[
              { icon: Search, title: "Поиск по всей базе", text: "Когда хочется посмотреть рынок шире одной вакансии." },
              { icon: Megaphone, title: "Вакансии и отклики", text: "Публикации никуда не делись. Отклики приходят в кабинет, контакты открываются по отклику." },
              { icon: ListChecks, title: "Короткие задания", text: "Небольшие задачи кандидатам. Видно, как человек думает, а его профиль остаётся свежим." },
              { icon: Building2, title: "Профиль компании", text: "Отрасль, город, описание и контакт. Кандидат видит, кто ему пишет." },
            ]}
          />
        </Container>
      </Section>

      <Section id="faq" className="scroll-mt-16">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <SplitHeading eyebrow="FAQ" title="Вопросы работодателей" />
          <div data-reveal>
            <Faq
              items={[
                { q: "Почему я не вижу имя и контакты кандидата?", a: "Карточка анонимна, пока кандидат не согласился. Решение о приглашении принимается по категории, результату теста, навыкам и опыту." },
                { q: "Можно пригласить без вакансии?", a: "Да. Достаточно описать задачу и указать зарплату, формат и контакт." },
                { q: "Почему вилку нельзя скрыть?", a: "Из-за денег чаще всего отказываются в самом конце. Когда вилка видна сразу, неподходящий кандидат откажет за минуту, а не после трёх созвонов." },
                { q: "Что делать, если не хватает квоты?", a: "Квота обновляется каждые 7 дней. Остаток и дата обновления видны на главной кабинета, а подборка с объяснениями помогает тратить приглашения точнее." },
                { q: "Насколько можно верить категории?", a: "В проверке на 2 400 профилях тест определил грейд верно в 87 % случаев. Когда ошибался, то максимум на один грейд и у самой границы." },
                { q: "Попадут ли кандидаты соседнего грейда?", a: "Только если вы это разрешите. Тогда за грейд они получат около половины балла, и это будет видно в расшифровке." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Начните с текста вакансии"
        lead="Регистрация компании занимает пару минут. Потом вставляете вакансию и смотрите, кого предлагает сервис."
        primary={{ href: "/register?role=employer", label: "Начать подбор" }}
        secondary={{ href: "/#how", label: "Как устроен тест" }}
      />
    </MotionRoot>
  );
}
