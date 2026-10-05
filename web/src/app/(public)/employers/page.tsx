import { Ban, Banknote, BellOff, Building2, Clock, FileSearch, Handshake, ListChecks, Megaphone, Search, Send, ShieldCheck, Target, Users } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, Note, PageHero, SplitHeading, WeightBars } from "@/components/site/blocks";
import { Faq } from "@/components/site/Faq";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section, TextLink } from "@/components/site/primitives";
import { ShortlistMock } from "@/components/site/visuals";

export const metadata: Metadata = {
  title: "Работодателям",
  description: "Подборка из кандидатов с подтверждённой тестом категорией, объяснение каждого места, приглашения с обязательной вилкой и статистика отказов.",
};

const WEIGHTS = [
  { label: "Обязательные навыки", value: 41, note: "близость навыка × доверие к источнику: тест и ФСП — 1,0, резюме — 0,7, заявлен — 0,5" },
  { label: "Грейд", value: 15, note: "1,0 — точное совпадение, 0,6 — на грейд выше, 0,5 — на грейд ниже" },
  { label: "Результат теста", value: 10, note: "положение уровня кандидата внутри диапазона грейда" },
  { label: "Достижения ФСП", value: 10, note: "ФСП-скор; отсутствие истории — ноль, не штраф" },
  { label: "Желательные навыки", value: 8, note: "считаются так же, как обязательные" },
  { label: "Зарплата", value: 7, note: "1,0, если ожидания в вилке; линейно до 0 при превышении на 30 %" },
  { label: "Актуальность", value: 5, note: "свежесть активности: тест, задание, ответ на приглашение" },
  { label: "Близость текстов", value: 4, note: "полнотекстовая релевантность «о себе» вашему описанию" },
];

const FUNNEL = [
  { label: "Отправлено", value: 30, display: "30" },
  { label: "Просмотрено", value: 24, display: "24" },
  { label: "Принято", value: 11, display: "11" },
  { label: "Отклонено", value: 9, display: "9", muted: true },
  { label: "Истекло", value: 4, display: "4", muted: true },
];

const REASONS = [
  { label: "Зарплата", value: 44, display: "44" },
  { label: "Стек", value: 22, display: "22" },
  { label: "Формат работы", value: 17, display: "17" },
  { label: "Не ищу работу", value: 11, display: "11" },
  { label: "Компания", value: 6, display: "6" },
];

export default function EmployersPage() {
  return (
    <MotionRoot>
      <PageHero
        crumb="Работодателям"
        title={
          <>
            Нанимайте из <span className="text-gradient">подтверждённых,</span> а не из заявленных
          </>
        }
        lead="Опишите задачу обычным текстом — получите рекомендованные категории и подборку анонимных кандидатов, где каждый балл разложен по признакам. Пишете первыми, с вилкой — и видите, почему вам отвечают или отказывают."
        actions={
          <>
            <Cta href="/register?role=employer">Начать подбор</Cta>
            <Cta href="/login" variant="outline">
              Войти в кабинет
            </Cta>
          </>
        }
        visual={<ShortlistMock />}
        facts={[
          { value: "0,94", label: "доля релевантных в топ-10 подборки (симуляция)" },
          { value: "×2,2", label: "лучше поиска по ключевым словам резюме (0,43)" },
          { value: "0,3 мс", label: "подбор по 10 000 кандидатов с объяснением топ-20" },
          { value: "100 %", label: "кандидатов в поиске — с категорией, подтверждённой тестом" },
        ]}
      />
      <PageTabs
        items={[
          { id: "flow", label: "Как это работает" },
          { id: "parse", label: "Распознавание" },
          { id: "shortlist", label: "Подборка" },
          { id: "rules", label: "Правила" },
          { id: "analytics", label: "Аналитика" },
          { id: "test-plan", label: "План теста" },
          { id: "faq", label: "Вопросы" },
        ]}
      />

      <Section id="flow" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Процесс" title="От описания задачи до контактов — шесть шагов">
            Никаких откликов на разбор. Вы приходите в категорию, где уровень каждого уже подтверждён, и выбираете точечно.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: FileSearch, title: "1. Потребность", text: "Форма или просто текст вакансии. Можно без публикации вакансии — приглашению она не нужна." },
              { icon: Target, title: "2. Рекомендованные категории", text: "Сколько кандидатов, сколько с ФСП, медианные ожидания и насколько категория покрывает ваш стек." },
              { icon: Users, title: "3. Подборка с объяснением", text: "Анонимные карточки по баллу 0–100. У каждого — причины и пробелы, понятные без психометрики." },
              { icon: Send, title: "4. Приглашение с вилкой", text: "Что предлагаете, сообщение, вилка «от–до», формат, способ связи. Кандидат видит условия до разговора." },
              { icon: Handshake, title: "5. Ответ кандидата", text: "Принял — контакты открываются. Отклонил — причина попадает в вашу статистику отказов." },
              { icon: ShieldCheck, title: "6. Контакты по основанию", text: "Доступ к контактам выдаётся только по принятому приглашению или отклику и записывается в журнал." },
            ]}
          />
        </Container>
      </Section>

      <Section id="parse" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Распознавание" title="Вставьте вакансию — остальное разберём сами">
            Словарь на 83 навыка и 491 синоним на русском и английском, включая сленг. «Постгрес», «кубер» и «джава» — тоже навыки.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-2">
            <div data-reveal="left" className="rounded-[24px] border border-line bg-white p-7">
              <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Текст вакансии</p>
              <p className="mt-5 text-lg leading-relaxed text-ink-muted">
                Ищем <mark className="rounded bg-primary-100 px-1 text-ink">мидла</mark> на <mark className="rounded bg-primary-100 px-1 text-ink">Go</mark> в платёжную команду. Работа с <mark className="rounded bg-primary-100 px-1 text-ink">постгресом</mark> и{" "}
                <mark className="rounded bg-primary-100 px-1 text-ink">Kafka</mark> обязательна, опыт с <mark className="rounded bg-canvas px-1 text-ink">кубером</mark> — плюс. <mark className="rounded bg-primary-100 px-1 text-ink">Гибрид</mark>, Москва.
                Вилка <mark className="rounded bg-primary-100 px-1 text-ink">250–320 тыс.</mark>
              </p>
            </div>
            <dl data-stagger="0.07" className="rounded-[24px] border border-line bg-white p-7">
              <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Распознано</p>
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
            <Note>Всё распознанное можно поправить вручную перед подбором. Специализацию и грейд система подсказывает, но решение — за вами.</Note>
          </div>
        </Container>
      </Section>

      <Section id="shortlist" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Подборка" title="Каждое место в подборке можно объяснить">
            Сначала — жёсткие фильтры: кандидат открыт к предложениям, нужная специализация, грейд из запрошенных (или соседний, если разрешили), совместимые формат и город. Затем — балл из восьми признаков.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-12 lg:grid-cols-[1.1fr_0.9fr] lg:gap-16">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <div className="mb-7 flex items-baseline justify-between">
                <p className="text-lg font-semibold tracking-tight text-ink">Веса признаков</p>
                <p className="font-mono text-xs text-ink-subtle">из 100 баллов</p>
              </div>
              <WeightBars rows={WEIGHTS} max={41} />
            </div>
            <div className="space-y-8">
              {[
                { t: "Уточнение без потери", d: "Подборка хранится. При смене фильтров она пересчитывается, но закреплённые кандидаты остаются (с пометкой, если не проходят новые фильтры), скрытые не возвращаются, а новые помечаются." },
                { t: "Пробелы — честно", d: "«Нет обязательного навыка: Kubernetes», «Вместо PostgreSQL — близкий навык MySQL», «Навык Docker только заявлен». Отсутствие ФСП в пробелы не попадает никогда." },
                { t: "Без потребности — сила профиля", d: "При просмотре категории кандидаты упорядочены по силе профиля: 0,6 · тест + 0,25 · ФСП + 0,15 · актуальность." },
                { t: "Веса подобраны на валидации", d: "Исходные веса проигрывали: на трёх случайных сидах устойчиво лучшим оказался вариант с большим весом стека и грейда — он и принят." },
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
          <SplitHeading eyebrow="Правила контакта" title="Честные правила, которые нельзя обойти" tone="night">
            Ограничения работают на вас: кандидаты отвечают, потому что не тонут в спаме и сразу видят условия.
          </SplitHeading>
          <div data-stagger="0.07" className="mt-14 grid grid-cols-1 gap-px overflow-hidden rounded-[24px] border border-night-line bg-night-line sm:grid-cols-2 lg:grid-cols-3">
            {[
              { icon: Banknote, v: "от–до", t: "Вилка обязательна", d: "В каждом приглашении. Проверяется в API и ограничениями базы данных." },
              { icon: Megaphone, v: "30", t: "приглашений за 7 дней", d: "Квота на компанию. Остаток и дата обновления — на главной кабинета." },
              { icon: BellOff, v: "1", t: "действующее на пару", d: "Одной компании — одно живое приглашение одному кандидату." },
              { icon: Clock, v: "14", t: "дней жизни приглашения", d: "Дальше статус «истекло» — и это видно в вашей воронке." },
              { icon: ShieldCheck, v: "0", t: "контактов до согласия", d: "Карточка для работодателя не содержит полей контактов на уровне типов." },
              { icon: Ban, v: "!", t: "Жалобы", d: "Кандидат может пожаловаться на приглашение — это сигнал для модерации." },
            ].map(({ icon: Icon, v, t, d }) => (
              <div key={t} className="bg-night-raised p-7 sm:p-8">
                <div className="flex items-center justify-between">
                  <span className="tabular font-display text-5xl font-semibold tracking-[-0.04em] text-white">{v}</span>
                  <Icon className="size-6 text-primary-300" aria-hidden />
                </div>
                <h3 className="mt-6 text-lg font-semibold text-white">{t}</h3>
                <p className="mt-2 text-[15px] leading-relaxed text-night-muted">{d}</p>
              </div>
            ))}
          </div>
        </Container>
      </Section>

      <Section id="analytics" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Аналитика" title="Воронка и причины отказов — не догадки">
            Кандидат отклоняет приглашение с причиной. Вы видите воронку по всем приглашениям и агрегированные причины — анонимно, без привязки к людям.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-2">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <div className="mb-7 flex items-baseline justify-between">
                <p className="text-lg font-semibold tracking-tight text-ink">Воронка приглашений</p>
                <span className="rounded-full bg-canvas px-2.5 py-1 font-mono text-[11px] text-ink-subtle">пример</span>
              </div>
              <WeightBars rows={FUNNEL} max={30} />
            </div>
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <div className="mb-7 flex items-baseline justify-between">
                <p className="text-lg font-semibold tracking-tight text-ink">Почему отказывают</p>
                <span className="rounded-full bg-canvas px-2.5 py-1 font-mono text-[11px] text-ink-subtle">пример, %</span>
              </div>
              <WeightBars rows={REASONS} max={44} unit=" %" />
              <p className="mt-6 rounded-2xl bg-warning-50 p-4 text-sm text-ink">Если 40 % и больше отказов — из-за денег, кабинет подскажет сверить вилку с медианными ожиданиями категории.</p>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="test-plan" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="План теста" title="Как проверяется категория под вашу вакансию">
            По описанию потребности видно, что именно проверит тест: целевой порог уровня, темы, шаблоны с приоритетом из-за вашего стека и примеры заданий для двух разных кандидатов.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 lg:grid-cols-[0.9fr_1.1fr]">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7">
              <p className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">Middle Go-разработчик: PostgreSQL, Kafka</p>
              <p className="mt-5 font-medium text-ink">Приоритетные шаблоны</p>
              <ul className="mt-3 flex flex-wrap gap-2">
                {["sql.group_by_having", "sql.left_join_rows", "go.defer_order", "db.index_choice", "db.isolation_anomaly"].map((t) => (
                  <li key={t} className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1.5 font-mono text-xs text-primary-strong">
                    {t}
                  </li>
                ))}
              </ul>
              <p className="mt-6 text-[15px] leading-relaxed text-ink-muted">Порог Middle на общей шкале уровня — 0. Тест подтверждает категорию, когда 95-процентный интервал оценки кандидата целиком выше порога.</p>
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
                  <p className="mt-4 text-xs text-night-faint">тот же шаблон и уровень, другие данные</p>
                </div>
              ))}
            </div>
          </div>
          <div data-reveal className="mt-10">
            <TextLink href="/how-it-works#bank">Банк заданий и защита от утечек</TextLink>
          </div>
        </Container>
      </Section>

      <Section tone="canvas">
        <Container>
          <SplitHeading eyebrow="В кабинете" title="Всё для найма — в одном месте" />
          <FeatureGrid
            className="mt-14"
            columns={4}
            items={[
              { icon: Search, title: "Поиск по банку", text: "Весь банк кандидатов с фильтрами — когда хочется посмотреть рынок шире конкретной потребности." },
              { icon: Megaphone, title: "Вакансии и отклики", text: "Классические публикации остаются: отклики приходят в кабинет, контакты — по отклику." },
              { icon: ListChecks, title: "Регулярные задания", text: "Короткие задачи кандидатам: держат профили «живыми» и показывают, как человек думает." },
              { icon: Building2, title: "Профиль компании", text: "Отрасль, город, описание и способ связи — кандидат видит, кто ему пишет." },
            ]}
          />
          <div className="mt-8">
            <Note>
              В планах: подтверждение корпоративного домена и ИНН/ОГРН по ЕГРЮЛ, рейтинг работодателя по доле ответов и жалобам, сравнение вилки с рынком категории, интеграция с ATS через вебхуки и API-ключи.
            </Note>
          </div>
        </Container>
      </Section>

      <Section id="faq" className="scroll-mt-16">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <SplitHeading eyebrow="FAQ" title="Вопросы работодателей" />
          <div data-reveal>
            <Faq
              items={[
                { q: "Почему я не вижу имя и контакты кандидата?", a: "Карточка анонимна до согласия. Решение о приглашении принимается по категории, результату теста, навыкам и опыту; контакты откроются после «Принять»." },
                { q: "Можно ли пригласить без опубликованной вакансии?", a: "Да. Приглашение не привязано к вакансии: достаточно описать, что предлагаете, указать вилку, формат и способ связи." },
                { q: "Почему вилку нельзя скрыть?", a: "Потому что это главная причина отказов в конце процесса. Вилка в первом сообщении экономит время обеим сторонам — и повышает долю ответов." },
                { q: "Что если квоты не хватает?", a: "Квота — 30 приглашений за 7 дней. Подборка с объяснениями помогает выбирать точечно; остаток и дата обновления квоты видны на главной кабинета." },
                { q: "Насколько можно доверять категории?", a: "В симуляции итоговый грейд совпадает с истинным в 87 % случаев, ошибки только у границы грейда и только «вниз». При пересдаче другими заданиями κ = 0,74." },
                { q: "Кандидат из соседнего грейда попадёт в подборку?", a: "Только если вы это разрешили. Грейд выше даёт 0,6 балла признака, ниже — 0,5, и это видно в разложении балла." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Опишите задачу — подборка соберётся из подтверждённых категорий"
        lead="Регистрация компании, профиль и первая потребность — дальше система покажет, где искать и кого приглашать."
        primary={{ href: "/register?role=employer", label: "Начать подбор" }}
        secondary={{ href: "/how-it-works", label: "Методика" }}
      />
    </MotionRoot>
  );
}
