import {
  BarChart3,
  Briefcase,
  CalendarClock,
  Code2,
  EyeOff,
  FileText,
  Gauge,
  Layers,
  ListChecks,
  ScanText,
  Timer,
  Trophy,
  UserX,
} from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, Note, NumberedSteps, PageHero, SplitHeading } from "@/components/site/blocks";
import { Faq } from "@/components/site/Faq";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section, TextLink } from "@/components/site/primitives";
import { InviteStack } from "@/components/site/visuals";

export const metadata: Metadata = {
  title: "Кандидатам",
  description: "Пройдите адаптивный тест один раз, получите категорию «специализация × грейд» и получайте приглашения с зарплатной вилкой в первом сообщении.",
};

const TOPICS: [string, string][] = [
  ["Бэкенд", "изменяемый аргумент по умолчанию в Python, порядок defer в Go, выбор индекса, token bucket, LRU, аномалии изоляции, GROUP BY / HAVING"],
  ["Фронтенд", "порядок вывода в event loop, замыкания в цикле, приведение типов, специфичность CSS, box model, батчинг состояния React"],
  ["DevOps / SRE", "кэш слоёв Docker, ресурсы Kubernetes, CIDR, location в nginx, cron, права Linux, increase() в Prometheus, бюджет ошибок"],
  ["QA", "граничные значения, классы эквивалентности, pairwise, диаграммы состояний, таблицы решений, проверки API"],
  ["ML / DS", "метрики по матрице ошибок, теорема Байеса, градиентный шаг, k-means, groupby в pandas, формы массивов NumPy"],
  ["Безопасность", "класс уязвимости во фрагменте кода, энтропия пароля, XOR-шифр, Base64, обход фильтров, правила межсетевого экрана"],
];

const INVITE_PARTS = [
  ["01", "Что предлагают", "Роль и задача своими словами — не «у нас печеньки», а с чем предстоит работать."],
  ["02", "Вилка «от–до»", "Обязательна в каждом приглашении. Без неё отправить приглашение нельзя — это проверяют и API, и база данных."],
  ["03", "Формат и город", "Офис, гибрид или удалённо. Совместимость с вашими настройками проверяется ещё на этапе подбора."],
  ["04", "Как связаться с компанией", "Компания сама указывает удобный канал — вы знаете, что произойдёт после «Принять»."],
  ["05", "Срок — 14 дней", "Приглашение не висит вечно. Не ответили — оно истечёт, и компания увидит это в своей воронке."],
  ["06", "Принять или отклонить с причиной", "Зарплата, стек, формат, компания, не ищу работу. Причина уходит компании в агрегированной статистике."],
];

const GRADE_RULES = [
  { title: "Первичное размещение", text: "Вы выбираете грейд и проходите тест. Не сдали — сразу можно пройти тест на грейд ниже. Повтор того же или более высокого грейда — через 30 дней, иначе тест можно было бы перебирать.", meta: "старт" },
  { title: "Уверенно сдали — растите сразу", text: "Если измеренный уровень выше заявленного или вероятность следующего грейда не меньше 0,9, система один раз сразу предложит дотест на грейд выше.", meta: "дотест" },
  { title: "Смена грейда — раз в 90 дней", text: "После присвоения грейд меняется вверх или вниз не чаще раза в 90 дней. Неудачная попытка повышения текущий грейд не понижает; следующая — через 30 дней.", meta: "90 дней" },
  { title: "Всё прозрачно", text: "В кабинете видны текущая категория, история опросов, тестов и присвоений, доступные попытки и дата, когда откроется смена грейда.", meta: "история" },
];

export default function CandidatesPage() {
  return (
    <MotionRoot>
      <PageHero
        crumb="Кандидатам"
        title={
          <>
            Один тест — и <span className="text-gradient">предложения приходят сами</span>
          </>
        }
        lead="Пройдите адаптивный тест, получите категорию «специализация × грейд» и перестаньте рассылать резюме. Компании видят анонимную карточку и пишут первыми — сразу с задачей, форматом и вилкой «от–до»."
        actions={
          <>
            <Cta href="/register?role=candidate">Пройти тест</Cta>
            <Cta href="/how-it-works" variant="outline">
              Как устроен тест
            </Cta>
          </>
        }
        visual={<InviteStack />}
        facts={[
          { value: "8–20", label: "заданий в тесте, в среднем около 13" },
          { value: "60–180 с", label: "лимит на одно задание" },
          { value: "14 дней", label: "живёт приглашение — без вечных «мы вам перезвоним»" },
          { value: "1 раз", label: "проходите тест — категория работает на все приглашения" },
        ]}
      />
      <PageTabs
        items={[
          { id: "path", label: "Путь" },
          { id: "test", label: "Тест" },
          { id: "invites", label: "Приглашения" },
          { id: "grade", label: "Грейд" },
          { id: "privacy", label: "Приватность" },
          { id: "more", label: "Ещё" },
          { id: "faq", label: "Вопросы" },
        ]}
      />

      <Section id="path" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Путь кандидата" title="Пять шагов до первого приглашения">
            Регистрация занимает пару минут, тест — столько, сколько нужно, чтобы уверенно определить уровень. Дальше вы только отвечаете на предложения.
          </SplitHeading>
          <div className="mt-14">
            <NumberedSteps
              steps={[
                { title: "Регистрация", meta: "email + согласия", text: "Email с подтверждением по ссылке и согласия на обработку и публикацию данных. Согласия версионируются — вы всегда видите, на что согласились." },
                { title: "Профиль", meta: "форма или PDF", text: "Заполните форму или загрузите PDF-резюме: текст распознаётся, опыт и навыки подставляются автоматически. Название прошлых компаний — по вашему желанию." },
                { title: "Опрос", meta: "направление, стек, грейд", text: "Выберите направление, специализацию, стек и грейд, на который претендуете. Стек повышает приоритет тем с вашими навыками, но шкала и пороги общие для всех." },
                { title: "Адаптивный тест", meta: "~13 заданий", text: "Каждое следующее задание подбирается под ваш текущий уровень. Тест останавливается, как только уровень определён уверенно: минимум 8, максимум 20 заданий." },
                { title: "Категория и приглашения", meta: "например, Бэкенд · Middle", text: "Сдали — профиль появляется в поиске работодателей. Навыки из решённых заданий получают отметку «подтверждён тестом» и весят в подборе больше заявленных." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <Section id="test" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Тест" title="Никаких анкет на час и вопросов «на память»">
            Задания проверяют, как вы рассуждаете: что выведет код, какой индекс выбрать, где уязвимость. У каждого кандидата свои числа и код — списать у коллеги не получится, но и подвоха нет.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: Gauge, title: "Адаптивно", text: "Верный ответ — следующее задание сложнее, ошибка — проще. Так уровень определяется за 8–20 заданий, а не за сотню вопросов." },
              { icon: Layers, title: "Под ваш стек", text: "Темы с навыками из опроса получают приоритет. Измерение при этом общее — Middle у всех означает одно и то же." },
              { icon: Timer, title: "Лимит на задание", text: "От 60 до 180 секунд в зависимости от задания. Просроченный ответ засчитывается как неверный — гуглить бессмысленно." },
              { icon: ListChecks, title: "Четыре типа ответа", text: "Один вариант, несколько вариантов, число или короткий текст. Интерфейс и задания — на русском." },
              { icon: Code2, title: "Код на вашем языке", text: "Go, Python, JavaScript, SQL, конфиги nginx и Kubernetes — фрагменты на том языке, на котором вы работаете." },
              { icon: Trophy, title: "Навыки подтверждаются", text: "Решили задание по PostgreSQL — навык получает отметку «подтверждён тестом» и полный вес в подборе." },
            ]}
          />
          <div className="mt-16">
            <p data-reveal className="eyebrow text-ink-subtle">
              Примеры тем по специализациям
            </p>
            <dl data-stagger="0.05" className="mt-6 grid grid-cols-1 gap-px overflow-hidden rounded-[24px] border border-line bg-line md:grid-cols-2">
              {TOPICS.map(([spec, topics]) => (
                <div key={spec} className="bg-white p-6">
                  <dt className="font-semibold text-ink">{spec}</dt>
                  <dd className="mt-2 text-[15px] leading-relaxed text-ink-muted">{topics}</dd>
                </div>
              ))}
            </dl>
          </div>
        </Container>
      </Section>

      <Section id="invites" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Приглашения" title="Приглашение, в котором уже есть ответы">
            Вы решаете, тратить ли время на разговор, ещё до его начала. Всё, что обычно выясняется на третьем созвоне, — в первом сообщении.
          </SplitHeading>
          <ol data-stagger="0.06" className="mt-14 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {INVITE_PARTS.map(([n, title, text]) => (
              <li key={n} className="rounded-[22px] border border-line bg-white p-7">
                <span className="font-mono text-sm text-primary">{n}</span>
                <h3 className="mt-6 text-xl font-semibold tracking-tight text-ink">{title}</h3>
                <p className="mt-2 text-[15px] leading-relaxed text-ink-muted">{text}</p>
              </li>
            ))}
          </ol>
          <div className="mt-16 grid grid-cols-1 gap-8 rounded-[28px] bg-night p-8 text-white sm:p-12 lg:grid-cols-[1fr_1.2fr] lg:items-center">
            <div>
              <BarChart3 className="size-7 text-primary-300" aria-hidden />
              <h3 data-split className="display-3 mt-6">
                Видно, как вас ищут
              </h3>
              <p data-reveal className="mt-4 leading-relaxed text-night-muted">
                В кабинете — анонимная статистика спроса в вашей категории за 30 дней. Не гадаете, сколько стоите, а видите рынок.
              </p>
            </div>
            <dl data-stagger className="grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-night-line bg-night-line">
              {[
                ["Компаний смотрели профиль", "сколько работодателей открывали вашу карточку"],
                ["Медианная вилка", "типичные предложения в вашей категории"],
                ["Приглашений в категории", "насколько активен спрос прямо сейчас"],
                ["Открытых запросов", "сколько компаний ищут ваш профиль"],
              ].map(([t, d]) => (
                <div key={t} className="bg-night-raised p-5">
                  <dt className="font-medium text-white">{t}</dt>
                  <dd className="mt-1.5 text-sm text-night-muted">{d}</dd>
                </div>
              ))}
            </dl>
          </div>
        </Container>
      </Section>

      <Section id="grade" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Правила грейда" title="Грейд — не приговор, а текущая точка">
            Система не понижает грейд принудительно и не даёт перебирать тест. Правила одни для всех и видны в кабинете заранее.
          </SplitHeading>
          <div className="relative mt-16">
            <div aria-hidden className="absolute top-[22px] right-0 left-0 hidden h-px bg-line-strong lg:block" />
            <ol data-stagger="0.1" className="grid grid-cols-1 gap-8 lg:grid-cols-4 lg:gap-6">
              {GRADE_RULES.map((r, i) => (
                <li key={r.title} className="relative">
                  <span className="relative z-10 flex size-11 items-center justify-center rounded-full border border-primary bg-white font-mono text-sm font-semibold text-primary">{i + 1}</span>
                  <p className="mt-6 font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">{r.meta}</p>
                  <h3 className="mt-2 text-xl font-semibold tracking-tight text-ink">{r.title}</h3>
                  <p className="mt-2 text-[15px] leading-relaxed text-ink-muted">{r.text}</p>
                </li>
              ))}
            </ol>
          </div>
          <div className="mt-14 flex flex-wrap items-center gap-3 rounded-2xl border border-line bg-white p-5 text-sm text-ink-muted">
            <CalendarClock className="size-5 text-primary" aria-hidden />
            Не сдали Junior? Пересдача через 30 дней — а пока можно прокачаться на регулярных заданиях от компаний.
          </div>
        </Container>
      </Section>

      <Section id="privacy" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Приватность" title="Видимость — под вашим контролем">
            Профиль анонимен по умолчанию. Вы решаете, искать ли работу активно, что показывать и когда открыть контакты.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: EyeOff, title: "Анонимная карточка", text: "Компании видят код вроде K-5F8A1F, категорию, навыки и результат — без имени и контактов." },
              { icon: Gauge, title: "Статус поиска", text: "«Активно ищу работу», «Открыт к предложениям» или «Не ищу работу» — статус влияет на то, попадаете ли вы в подборки." },
              { icon: Trophy, title: "ФСП можно скрыть", text: "Скрытые ФСП-данные не показываются в карточке и не участвуют в ранжировании." },
              { icon: Briefcase, title: "Компании в опыте", text: "Названия прошлых работодателей видны только если вы это разрешили." },
              { icon: UserX, title: "Отзыв и удаление", text: "Отзыв согласия на публикацию скрывает профиль, удаление аккаунта стирает персональные данные." },
              { icon: FileText, title: "Журнал доступа", text: "Каждое обращение компании к вашим контактам записывается: кто, когда, на каком основании." },
            ]}
          />
          <div data-reveal className="mt-10">
            <TextLink href="/security">Как устроена защита данных</TextLink>
          </div>
        </Container>
      </Section>

      <Section id="more" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Дополнительно" title="Не только приглашения">
            Основной путь — приглашения от работодателей. Но всё, что нужно для поиска, тоже под рукой.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            columns={4}
            items={[
              { icon: Briefcase, title: "Вакансии и отклики", text: "Классический отклик на опубликованные вакансии. Контакты в этом случае открываются по вашему отклику." },
              { icon: ListChecks, title: "Регулярные задания", text: "Короткие задачи от компаний. Ответ обновляет актуальность профиля — а она влияет на позицию в подборках." },
              { icon: ScanText, title: "Распознавание резюме", text: "Загрузите PDF — опыт и навыки подставятся сами. Понимаем сленг: «кубер», «постгрес», «джава»." },
              { icon: FileText, title: "PDF-профиль", text: "Стандартизированный профиль в фирменном стиле ФСП: категория, подтверждённые навыки, достижения." },
            ]}
          />
        </Container>
      </Section>

      <Section id="faq" className="scroll-mt-16">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <div>
            <SplitHeading eyebrow="FAQ" title="Вопросы кандидатов" />
            <div className="mt-8">
              <Note>Остались вопросы про задания и оценку? Подробная методика — в разделе «Как устроено».</Note>
            </div>
          </div>
          <div data-reveal>
            <Faq
              items={[
                { q: "Можно ли подготовиться к тесту?", a: "Да — так же, как к работе: разбираться в своём стеке. Задания генерируются заново для каждого кандидата, поэтому заучивать ответы бесполезно, а понимать — полезно." },
                { q: "Что если я заявлю грейд выше реального?", a: "Тест просто не подтвердит его — и сразу предложит попробовать грейд ниже. Завышение не даёт преимуществ: категорию присваивает только сданный тест." },
                { q: "Сколько приглашений я буду получать?", a: "Зависит от спроса в вашей категории. Спама не будет: у каждой компании не больше 30 приглашений за 7 дней и только одно действующее приглашение вам." },
                { q: "Работодатель увидит мои ответы в тесте?", a: "Нет. Компания видит итог: категорию, силу результата внутри грейда и навыки, подтверждённые тестом." },
                { q: "Как отказаться от приглашения?", a: "Нажмите «Отклонить» и выберите причину: зарплата, стек, формат, компания, не ищу работу или другое. Компания увидит причины только в агрегированной статистике." },
                { q: "Можно ли удалить аккаунт?", a: "Да, в настройках. Удаление стирает персональные данные; отзыв согласия на публикацию скрывает профиль из поиска." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Подтвердите уровень один раз — дальше пусть пишут вам"
        lead="Регистрация, опрос и адаптивный тест. Категория появится сразу после успешной сдачи."
        primary={{ href: "/register?role=candidate", label: "Пройти тест" }}
        secondary={{ href: "/fsp", label: "Я участник ФСП" }}
      />
    </MotionRoot>
  );
}
