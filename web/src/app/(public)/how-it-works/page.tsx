import { Bot, Calculator, Dices, Gauge, Layers, Shuffle } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, Note, PageHero, SpecList, SplitHeading, WeightBars } from "@/components/site/blocks";
import { LeakChart } from "@/components/site/LeakDemo";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section } from "@/components/site/primitives";
import { ThetaConvergence } from "@/components/site/ThetaConvergence";

export const metadata: Metadata = {
  title: "Как устроены тест и подбор",
  description: "Категории, банк из 62 параметрических шаблонов, адаптивный тест на IRT, устойчивость к утечкам, объяснимый подбор и процедура валидации.",
};

const SPECS = [
  { name: "Бэкенд", dir: "Разработка", templates: 24, topics: 10 },
  { name: "Фронтенд", dir: "Разработка", templates: 15, topics: 6 },
  { name: "DevOps / SRE", dir: "Инфраструктура", templates: 18, topics: 10 },
  { name: "QA", dir: "Качество", templates: 18, topics: 8 },
  { name: "ML / DS", dir: "Данные и ИИ", templates: 18, topics: 8 },
  { name: "Безопасность", dir: "Безопасность", templates: 15, topics: 8 },
];

const ACCURACY = [
  ["Честная заявка, точная калибровка", "87,7 %", "87,0 %", "100 %", "0,74", "13,6"],
  ["20 % завышают, 10 % занижают грейд", "87,7 %", "80,2 %", "99,2 %", "0,72", "13,4"],
  ["Ошибка калибровки заданий", "87,1 %", "86,2 %", "100 %", "0,72", "13,6"],
];

export default function HowItWorksPage() {
  return (
    <MotionRoot>
      <PageHero
        crumb="Как устроено"
        title={
          <>
            Под капотом — <span className="text-gradient">психометрия</span> и честная валидация
          </>
        }
        lead="Здесь — всё, что обычно прячут за словами «умный подбор». Как задаётся категория, откуда берутся задания, почему тест нельзя слить, как считается балл и чем мы это проверили."
        actions={
          <>
            <Cta href="#validation">К результатам валидации</Cta>
            <Cta href="/api/docs" variant="outline" external>
              API · Swagger
            </Cta>
          </>
        }
        visual={<ThetaConvergence />}
      />
      <PageTabs
        items={[
          { id: "categories", label: "Категории" },
          { id: "bank", label: "Банк заданий" },
          { id: "cat", label: "Адаптивный тест" },
          { id: "leaks", label: "Утечки" },
          { id: "matching", label: "Подбор" },
          { id: "validation", label: "Валидация" },
          { id: "limits", label: "Ограничения" },
        ]}
      />

      <Section id="categories" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Категории" title="Шесть специализаций × три грейда">
            Категорию присваивает только сданный тест. Резюме и опрос влияют лишь на вторичные признаки — стек и ожидания. Кандидат без категории в поиске не участвует.
          </SplitHeading>
          <div data-reveal className="mt-14 overflow-x-auto rounded-[24px] border border-line">
            <table className="w-full min-w-[40rem] border-collapse text-left">
              <thead className="bg-canvas">
                <tr className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">
                  <th className="px-6 py-4 font-medium">Специализация</th>
                  <th className="px-6 py-4 font-medium">Направление</th>
                  <th className="px-6 py-4 font-medium">Junior · θ ≥ −1</th>
                  <th className="px-6 py-4 font-medium">Middle · θ ≥ 0</th>
                  <th className="px-6 py-4 font-medium">Senior · θ ≥ +1</th>
                </tr>
              </thead>
              <tbody>
                {SPECS.map((s) => (
                  <tr key={s.name} className="border-t border-line">
                    <td className="px-6 py-4 font-semibold text-ink">{s.name}</td>
                    <td className="px-6 py-4 text-ink-muted">{s.dir}</td>
                    {["Junior", "Middle", "Senior"].map((g) => (
                      <td key={g} className="px-6 py-4">
                        <span className="inline-flex h-7 items-center rounded-md border border-primary-200 bg-primary-50 px-2 font-mono text-xs text-primary-strong">
                          {s.name.split(" ")[0]} · {g}
                        </span>
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="mt-8">
            <Note>θ — способность кандидата на общей шкале модели IRT. Пороги грейдов: −1, 0 и +1. Общая шкала позволяет сравнивать результаты кандидатов, получивших разные задания.</Note>
          </div>
        </Container>
      </Section>

      <Section id="bank" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Банк заданий" title="62 шаблона, у каждого — тысячи экземпляров">
            Неповторимость даёт генератор, сопоставимость — калибровка. Мы разделили эти задачи, а не надеялись, что нейросеть «под каждого» сгенерирует задания одинаковой сложности.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: Shuffle, title: "52 генератора + 10 пулов", text: "Вычислительные генераторы и вариативные пулы: 5–7 формулировок, 3–4 равноправных правильных ответа и 7–10 дистракторов на вопрос." },
              { icon: Calculator, title: "Ответ вычисляется", text: "Мини-симулятор event loop JavaScript, расчёт подсети, выполнение SQL на сгенерированной таблице. Правильный ответ не хранится — его считает генератор." },
              { icon: Layers, title: "Радикалы и инциденталы", text: "Сложность задают «радикалы» — например, число асинхронных задач. Числа, имена и порядок меняются, не влияя на сложность." },
              { icon: Dices, title: "≥ 1961 из 2000", text: "Каждый уровень каждого шаблона даёт не менее 1961 различимого экземпляра на 2000 случайных seed. Это закреплено тестом." },
              { icon: Gauge, title: "1–3 уровня сложности", text: "Параметры IRT у каждого уровня: дискриминативность a, сложность b ≈ −1,4…+1,4, угадывание c." },
              { icon: Bot, title: "Нейросети — только офлайн", text: "Могут предложить новый шаблон, но в рабочий банк он попадает после ревью и пробной калибровки без влияния на оценку." },
            ]}
          />
          <div data-reveal className="mt-14 overflow-x-auto rounded-[24px] border border-line bg-white">
            <table className="w-full min-w-[44rem] border-collapse text-left">
              <thead>
                <tr className="font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">
                  <th className="px-6 py-4 font-medium">Специализация</th>
                  <th className="px-6 py-4 font-medium">Шаблонов</th>
                  <th className="px-6 py-4 font-medium">Тем</th>
                  <th className="px-6 py-4 font-medium">Примеры</th>
                </tr>
              </thead>
              <tbody className="text-[15px]">
                {[
                  ["Бэкенд", "24", "10", "изменяемый аргумент по умолчанию в Python, порядок defer в Go, выбор индекса, token bucket, LRU, аномалии изоляции, GROUP BY / HAVING"],
                  ["Фронтенд", "15", "6", "порядок вывода в event loop, замыкания в цикле, приведение типов, специфичность CSS, box model, батчинг состояния React"],
                  ["DevOps", "18", "10", "кэш слоёв Docker, ресурсы Kubernetes, CIDR, location в nginx, cron, права Linux, increase() в Prometheus, бюджет ошибок"],
                  ["QA", "18", "8", "граничные значения, классы эквивалентности, pairwise, диаграммы состояний, таблицы решений, проверки API"],
                  ["ML", "18", "8", "метрики по матрице ошибок, теорема Байеса, градиентный шаг, k-means, groupby в pandas, формы массивов NumPy"],
                  ["Безопасность", "15", "8", "класс уязвимости во фрагменте кода, энтропия пароля, XOR-шифр, Base64, обход фильтров, правила межсетевого экрана"],
                ].map(([s, t, n, ex]) => (
                  <tr key={s} className="border-t border-line align-top">
                    <td className="px-6 py-4 font-semibold text-ink">{s}</td>
                    <td className="tabular px-6 py-4 font-mono text-ink">{t}</td>
                    <td className="tabular px-6 py-4 font-mono text-ink">{n}</td>
                    <td className="px-6 py-4 leading-relaxed text-ink-muted">{ex}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Container>
      </Section>

      <Section id="cat" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Адаптивный тест" title="Каждое задание — там, где оно информативнее всего">
            Тест подстраивается под кандидата и останавливается, когда уровень определён с нужной уверенностью. Так хватает в среднем 13 заданий вместо фиксированной анкеты.
          </SplitHeading>
          <div className="mt-14">
            <SpecList
              rows={[
                ["Модель", "IRT 3PL: c = 1/k для выбора из k вариантов, 0 для числового и текстового ответа"],
                ["Оценка уровня", "Апостериорное среднее (EAP) на сетке из 81 точки в диапазоне [−4; 4], априорное N(порог заявленного грейда; 1)"],
                ["Выбор задания", "Максимум информации Фишера в текущей оценке, случайный выбор среди пяти лучших (контроль экспозиции), квоты по темам, без повтора шаблона"],
                ["Остановка", "Не меньше 8 и не больше 20 заданий; стоп, когда 95-процентный интервал оценки не содержит порог заявленного грейда"],
                ["Решение", "Оценка не ниже порога — тест сдан"],
                ["«Уверенно сдал»", "Измеренный грейд выше заявленного или P(θ ≥ следующего порога) ≥ 0,9 — предлагается дотест на грейд выше"],
                ["Время", "Лимит на задание 60–180 секунд; просроченный ответ засчитывается как неверный"],
                ["Аномалии", "Статистика person-fit l_z: при l_z < −2 («верно на трудных, неверно на лёгких») результат помечается для проверки"],
                ["Стек кандидата", "Повышает приоритет шаблонов с совпадающими навыками; шкала и пороги остаются общими. Навыки из верно решённых заданий получают отметку «подтверждён тестом»"],
              ]}
            />
          </div>
        </Container>
      </Section>

      <Section id="leaks" tone="night" className="scroll-mt-16 overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute -top-40 -left-40 size-[36rem] rounded-full bg-primary/25 blur-[150px]" />
        <Container className="relative">
          <SplitHeading eyebrow="Устойчивость к утечкам" title="Что даёт утечка? Почти ничего." tone="night">
            Даже при базе из ~42 тыс. слитых экземпляров совпадения встречаются в 0,02–0,05 % заданий — и доля прошедших слабых кандидатов не растёт.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-8 lg:grid-cols-[0.9fr_1.1fr] lg:items-start">
            <dl data-stagger="0.08" className="space-y-px overflow-hidden rounded-[24px] border border-night-line bg-night-line">
              {[
                ["Утёк конкретный экземпляр с ответом", "Практически ничего: следующий кандидат получит другой seed, а значит другой код и другой ответ."],
                ["Утёк шаблон", "«Тут спрашивают порядок вывода event loop» — понимание, как решать такой класс задач, ровно тот навык, который мы измеряем."],
                ["Утёк весь банк шаблонов", "Ответы всё равно нужно вычислить, а на каждое задание действует лимит времени."],
                ["Слили ответы фиксированного теста", "Person-fit ловит 11–12 % «сливщиков» фиксированного теста при 0,9 % ложных срабатываний."],
              ].map(([t, d]) => (
                <div key={t} className="bg-night-raised p-6">
                  <dt className="font-semibold text-white">{t}</dt>
                  <dd className="mt-2 text-[15px] leading-relaxed text-night-muted">{d}</dd>
                </div>
              ))}
            </dl>
            <div data-reveal="scale">
              <LeakChart />
            </div>
          </div>
        </Container>
      </Section>

      <Section id="matching" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Подбор" title="Балл, который раскладывается на части">
            Жёсткие фильтры отсекают неподходящих, затем балл 0–100 — взвешенная сумма восьми признаков. Признаки без данных выпадают, веса перенормируются.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-10 lg:grid-cols-[1.1fr_0.9fr] lg:gap-16">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <WeightBars
                max={41}
                rows={[
                  { label: "Обязательные навыки", value: 41, note: "близость × доверие к источнику: тест/ФСП 1,0 · резюме 0,7 · заявлен 0,5" },
                  { label: "Грейд", value: 15, note: "1,0 точно · 0,6 на грейд выше · 0,5 на грейд ниже" },
                  { label: "Результат теста", value: 10, note: "положение θ внутри диапазона грейда" },
                  { label: "Достижения ФСП", value: 10, note: "ФСП-скор, шумное ИЛИ с затуханием" },
                  { label: "Желательные навыки", value: 8 },
                  { label: "Зарплата", value: 7, note: "1,0 в вилке; линейно до 0 при превышении на 30 %" },
                  { label: "Актуальность", value: 5, note: "exp(−дней с активности / 60)" },
                  { label: "Близость текстов", value: 4, note: "полнотекстовая релевантность, русская морфология" },
                ]}
              />
            </div>
            <div>
              <SpecList
                rows={[
                  ["Жёсткие фильтры", "Видим и открыт к предложениям, нужная специализация, грейд из запрошенных или соседний, совместимые формат и город"],
                  ["Навыки", "83 навыка, 491 синоним, 77 рёбер близости: PostgreSQL ~ MySQL, React ~ Vue"],
                  ["Без потребности", "Сила профиля = 0,6 · тест + 0,25 · ФСП + 0,15 · актуальность"],
                  ["Скорость", "~0,3 мс на 10 000 кандидатов с объяснением топ-20; тексты объяснений собираются только для итоговых кандидатов"],
                ]}
              />
            </div>
          </div>
        </Container>
      </Section>

      <Section id="validation" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Валидация" title="Проверено тем же кодом, что работает в продукте">
            Процедура валидации вызывает те же модули, что и API, и пишет отчёты в репозиторий. Данных и разметки организаторы не дают, поэтому используется синтетика со скрытой «истиной».
          </SplitHeading>
          <div data-reveal className="mt-14 overflow-x-auto rounded-[24px] border border-line bg-white">
            <table className="w-full min-w-[52rem] border-collapse text-left text-[15px]">
              <caption className="px-6 pt-6 text-left text-lg font-semibold tracking-tight text-ink">Тестирование: точность и стабильность · 400 человек на специализацию</caption>
              <thead>
                <tr className="font-mono text-[11px] tracking-[0.08em] text-ink-subtle uppercase">
                  <th className="px-6 py-4 font-medium">Сценарий</th>
                  <th className="px-4 py-4 font-medium">Выше/ниже порога верно</th>
                  <th className="px-4 py-4 font-medium">Грейд = истинный</th>
                  <th className="px-4 py-4 font-medium">Отклонение ≤ 1</th>
                  <th className="px-4 py-4 font-medium">κ пересдачи</th>
                  <th className="px-4 py-4 font-medium">Заданий</th>
                </tr>
              </thead>
              <tbody>
                {ACCURACY.map(([s, ...v]) => (
                  <tr key={s} className="border-t border-line">
                    <td className="px-6 py-4 text-ink">{s}</td>
                    {v.map((x, i) => (
                      <td key={i} className="tabular px-4 py-4 font-mono text-ink">
                        {x}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="mt-5 grid grid-cols-1 gap-5 lg:grid-cols-2">
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <p className="text-lg font-semibold tracking-tight text-ink">Подбор: доля релевантных в топ-10</p>
              <p className="mt-1 text-sm text-ink-muted">2 400 синтетических кандидатов, 120 отложенных потребностей</p>
              <div className="mt-7">
                <WeightBars
                  max={1}
                  rows={[
                    { label: "Наш подбор, веса после валидации", value: 0.941, display: "0,941" },
                    { label: "Наш подбор, исходные веса", value: 0.888, display: "0,888" },
                    { label: "Ключевые слова резюме + стаж", value: 0.425, display: "0,425", muted: true },
                    { label: "Случайный порядок в специализации", value: 0.223, display: "0,223", muted: true },
                  ]}
                />
              </div>
            </div>
            <div data-reveal className="rounded-[24px] border border-line bg-white p-7 sm:p-9">
              <p className="text-lg font-semibold tracking-tight text-ink">nDCG@10 и абляции</p>
              <p className="mt-1 text-sm text-ink-muted">Что будет, если убрать часть сигналов</p>
              <div className="mt-7">
                <WeightBars
                  max={1}
                  rows={[
                    { label: "Полная модель", value: 0.835, display: "0,835" },
                    { label: "Без графа близости навыков", value: 0.829, display: "0,829", muted: true },
                    { label: "Без результата теста", value: 0.819, display: "0,819", muted: true },
                    { label: "Только по навыкам", value: 0.739, display: "0,739", muted: true },
                    { label: "Ключевые слова резюме", value: 0.356, display: "0,356", muted: true },
                  ]}
                />
              </div>
            </div>
          </div>
          <div className="mt-8">
            <Note>
              Ошибки определения грейда — только «вниз» и только у границы: тест на заявленный грейд не повышает принудительно, а пограничный кандидат может пересдать на грейд выше. Дискриминативность заданий — в среднем 0,30, у 94 % шаблонов ≥ 0,2; самые слабые выводятся в отчёт на перекалибровку.
            </Note>
          </div>
        </Container>
      </Section>

      <Section id="limits" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Ограничения" title="Что мы знаем о границах этих цифр">
            Синтетическая «истина» задаётся командой, поэтому абсолютные значения описывают поведение системы в заданных предпосылках, а не рынок. Сравнение с базовыми линиями и абляции от этого не зависят.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-5 md:grid-cols-2">
            {[
              { t: "Экспертная разметка подбора", d: "10–20 реальных вакансий партнёров ФСП, топ-20 выдачи каждой оценивают два эксперта по шкале 0–3. Согласованность экспертов (κ) и Precision@10 считаются тем же модулем валидации." },
              { t: "Пилот тестирования на участниках ФСП", d: "Сравнение грейда по тесту с разрядом и результатами соревнований, повторная сдача через неделю для проверки стабильности." },
            ].map((x, i) => (
              <div key={x.t} data-reveal className="rounded-[24px] border border-line bg-white p-8">
                <span className="font-mono text-sm text-primary">Следующий шаг 0{i + 1}</span>
                <h3 className="mt-6 text-2xl font-semibold tracking-tight text-ink">{x.t}</h3>
                <p className="mt-3 text-[15px] leading-relaxed text-ink-muted">{x.d}</p>
              </div>
            ))}
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Проверьте методику на себе"
        lead="Самый быстрый способ понять адаптивный тест — пройти его. Самый быстрый способ понять подборку — описать реальную вакансию."
        primary={{ href: "/register?role=candidate", label: "Пройти тест" }}
        secondary={{ href: "/register?role=employer", label: "Описать вакансию" }}
      />
    </MotionRoot>
  );
}
