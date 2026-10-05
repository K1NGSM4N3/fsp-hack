import { BarChart3, Briefcase, CalendarClock, Code2, FileText, Gauge, ListChecks, ScanText, Shuffle, Timer, Trophy } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, Note, NumberedSteps, PageHero, SplitHeading } from "@/components/site/blocks";
import { ContactReveal } from "@/components/site/candidates/ContactReveal";
import { InviteDeck } from "@/components/site/candidates/InviteDeck";
import { TaskFlip } from "@/components/site/candidates/TaskFlip";
import { Faq } from "@/components/site/Faq";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section } from "@/components/site/primitives";

export const metadata: Metadata = {
  title: "Кандидатам",
  description: "Один адаптивный тест, категория «специализация × грейд» и приглашения, в которых зарплата написана сразу.",
};

const TOPICS: [string, string][] = [
  ["Бэкенд", "аргумент по умолчанию в Python, порядок defer в Go, выбор индекса, ограничение частоты запросов, LRU-кэш, аномалии транзакций, GROUP BY и HAVING"],
  ["Фронтенд", "порядок вывода в event loop, замыкания в цикле, приведение типов, специфичность CSS, box model, обновления состояния в React"],
  ["DevOps / SRE", "слои Docker-образа, ресурсы в Kubernetes, подсети, location в nginx, cron, права в Linux, метрики Prometheus, бюджет ошибок"],
  ["QA", "граничные значения, классы эквивалентности, попарное тестирование, диаграммы состояний, таблицы решений, проверки HTTP-запросов"],
  ["ML / DS", "метрики по матрице ошибок, теорема Байеса, шаг градиентного спуска, k-means, groupby в pandas, формы массивов NumPy"],
  ["Безопасность", "уязвимость во фрагменте кода, стойкость пароля, XOR, Base64, обход фильтров, правила межсетевого экрана"],
];

const INVITE_PARTS = [
  ["01", "Что за работа", "Роль и задача, описанные самой компанией."],
  ["02", "Зарплата от и до", "Обязательное поле. Без него приглашение не отправить."],
  ["03", "Формат и город", "Офис, гибрид или удалённо, с городом."],
  ["04", "Как связаться", "Куда и кому писать после того, как вы нажмёте «Принять»."],
  ["05", "14 дней на ответ", "Потом приглашение истекает, и компания это видит."],
  ["06", "Отказ с причиной", "Зарплата, стек, формат, компания или «не ищу работу». Компания видит причины только в общей статистике."],
];

const GRADE_RULES = [
  { title: "Первая попытка", text: "Выбираете грейд и сдаёте тест. Не вышло, можно сразу попробовать грейд ниже. Тот же грейд или выше снова через 30 дней.", meta: "старт" },
  { title: "Сдали с запасом", text: "Если результат явно выше заявленного, сервис один раз сразу предложит тест на грейд выше.", meta: "бонус" },
  { title: "Раз в 90 дней", text: "После присвоения грейд меняется не чаще раза в 90 дней. Неудачная попытка повышения текущий грейд не снижает.", meta: "90 дней" },
  { title: "Даты видны заранее", text: "В кабинете есть категория, история тестов и дата, когда откроется следующая попытка.", meta: "кабинет" },
];

const PRIVACY = [
  "Компания не видит ваше имя и контакты, пока вы не приняли её приглашение.",
  "Каждый просмотр ваших контактов записывается: какая компания и когда.",
  "Откликнулись на вакансию сами? Тогда контакты откроются этой компании по вашему отклику.",
  "Статус поиска решает, попадаете ли вы в подборки: «Активно ищу работу», «Открыт к предложениям» или «Не ищу работу».",
  "Достижения ФСП и названия прошлых работодателей можно скрыть.",
  "Отзовите согласие на публикацию, и профиль пропадёт из поиска. Удалите аккаунт, и персональные данные сотрутся.",
];

export default function CandidatesPage() {
  return (
    <MotionRoot variant="wipe">
      <PageHero
        crumb="Кандидатам"
        title={
          <>
            Один тест <span className="text-gradient">вместо сотни откликов</span>
          </>
        }
        lead="Сдайте адаптивный тест и получите категорию, например «Фронтенд · Junior». Дальше компании находят вас сами и пишут с готовыми условиями: задача, формат и зарплата от и до."
        actions={
          <>
            <Cta href="/register?role=candidate">Пройти тест</Cta>
            <Cta href="#test" variant="outline">
              Как устроен тест
            </Cta>
          </>
        }
        visual={<InviteDeck />}
        facts={[
          { value: "8–20", label: "заданий, обычно 10–25 минут" },
          { value: "1–3 мин", label: "на одно задание" },
          { value: "14 дней", label: "на ответ компании" },
          { value: "30", label: "приглашений в неделю максимум у одной компании" },
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
            Регистрация и профиль занимают несколько минут, тест обычно 10–25. Потом остаётся отвечать на приглашения.
          </SplitHeading>
          <div className="mt-14">
            <NumberedSteps
              steps={[
                { title: "Регистрация", meta: "почта и согласия", text: "Подтверждаете почту по ссылке и соглашаетесь на обработку данных. Согласия сохраняются с датой, отозвать их можно в настройках." },
                { title: "Профиль", meta: "вручную или из PDF", text: "Заполните форму или загрузите резюме в PDF: опыт и навыки подтянутся сами, останется проверить. Названия прошлых работодателей компании увидят, только если вы разрешите." },
                { title: "Опрос", meta: "специализация, стек, грейд", text: "Выбираете специализацию, отмечаете стек и грейд, на который идёте. От стека зависят темы заданий." },
                { title: "Тест", meta: "в среднем 13 заданий", text: "Задания подстраиваются под ответы. Тест заканчивается сам, когда уровень понятен, но не раньше 8-го задания и не позже 20-го." },
                { title: "Категория и приглашения", meta: "например, Бэкенд · Middle", text: "Сдали, и профиль попадает в поиск. Навыки из решённых заданий отмечены как подтверждённые и в подборке весят больше заявленных." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <Section id="test" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Тест" title="Задания про то, чем вы занимаетесь на работе">
            Что выведет код, какой индекс выбрать, сколько адресов в подсети. Просить перечислить принципы ООП никто не будет.
          </SplitHeading>
          <div className="mt-14">
            <TaskFlip />
          </div>
          <FeatureGrid
            className="mt-16"
            items={[
              { icon: Gauge, title: "Подстраивается под вас", text: "Ответили верно, следующее задание сложнее. Ошиблись, проще." },
              { icon: Shuffle, title: "У каждого свой вариант", text: "Числа, код и данные у всех разные, так что ответы коллеги вам не помогут. Ваши ему тоже." },
              { icon: Timer, title: "Лимит на задание", text: "От одной до трёх минут. Не успели, ответ засчитан как неверный." },
              { icon: ListChecks, title: "Четыре вида ответа", text: "Один вариант, несколько вариантов, число или короткий текст." },
              { icon: Code2, title: "Код на вашем языке", text: "Go, Python, JavaScript, SQL, конфиги nginx и Kubernetes." },
              { icon: Trophy, title: "Навыки подтверждаются", text: "Решили задание по PostgreSQL, и в профиле появляется отметка «подтверждён тестом»." },
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
          <div className="mt-10">
            <Note>Грейд определяется верно в 87 % случаев. Если тест ошибается, то максимум на один грейд. Пересдача другими заданиями даёт тот же результат в 81 % случаев.</Note>
          </div>
        </Container>
      </Section>

      <Section id="invites" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Приглашения" title="В приглашении уже есть ответы на первые вопросы">
            Всё, что обычно выясняют на третьем созвоне, написано в первом сообщении. Остаётся решить, интересно ли.
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
                В кабинете есть анонимная статистика по вашей категории за 30 дней. Можно не гадать, сколько вы стоите на рынке.
              </p>
            </div>
            <dl data-stagger className="grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-night-line bg-night-line">
              {[
                ["Компании смотрели профиль", "сколько работодателей открывали вашу карточку"],
                ["Типичная вилка", "медиана предложений в вашей категории"],
                ["Приглашений в категории", "насколько активен спрос сейчас"],
                ["Открытых запросов", "сколько компаний ищут людей вашего профиля"],
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
          <SplitHeading eyebrow="Правила грейда" title="Грейд можно менять, но не перебирать">
            Правила одни для всех, а даты следующих попыток видны в кабинете заранее.
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
          <div data-reveal className="mt-14 flex flex-wrap items-center gap-3 rounded-2xl border border-line bg-white p-5 text-sm text-ink-muted">
            <CalendarClock className="size-5 text-primary" aria-hidden />
            Не сдали Junior? Следующая попытка через 30 дней. Пока можно решать короткие задания от компаний.
          </div>
        </Container>
      </Section>

      <Section id="privacy" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Приватность" title="Контакты видит только компания, которой вы нажали «Принять»">
            До этого компания видит карточку с кодом вроде K-5F8A1F: категорию, навыки, результат теста и ожидания по зарплате.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-10 lg:grid-cols-[1fr_1fr] lg:items-start lg:gap-16">
            <div data-reveal>
              <ContactReveal />
            </div>
            <ul data-stagger className="space-y-px overflow-hidden rounded-[24px] border border-line bg-line">
              {PRIVACY.map((t) => (
                <li key={t} className="bg-white px-6 py-5 text-[15px] leading-relaxed text-ink">
                  {t}
                </li>
              ))}
            </ul>
          </div>
          <div className="mt-10">
            <Note>Персональные данные обрабатываются по 152-ФЗ. Согласия хранятся с датой, и их можно отозвать в настройках.</Note>
          </div>
        </Container>
      </Section>

      <Section id="more" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Дополнительно" title="Что ещё есть в кабинете" />
          <FeatureGrid
            className="mt-14"
            columns={4}
            items={[
              { icon: Briefcase, title: "Вакансии и отклики", text: "Откликнуться можно и по-старому. Контакты тогда откроются по вашему отклику." },
              { icon: ListChecks, title: "Короткие задания", text: "Компании присылают небольшие задачи. Ответ держит профиль в числе свежих." },
              { icon: ScanText, title: "Резюме из PDF", text: "Загрузили файл, получили заполненный профиль. «Кубер» и «джава» тоже распознаются." },
              { icon: FileText, title: "PDF-профиль", text: "Выгрузка с категорией и подтверждёнными навыками в стиле ФСП." },
            ]}
          />
        </Container>
      </Section>

      <Section id="faq" className="scroll-mt-16">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <SplitHeading eyebrow="FAQ" title="Вопросы кандидатов" />
          <div data-reveal>
            <Faq
              items={[
                { q: "Можно подготовиться к тесту?", a: "Да, так же, как к работе: разобраться в своём стеке. Вариант собирается для каждого заново, поэтому заучивать ответы бессмысленно." },
                { q: "Что будет, если заявить грейд выше реального?", a: "Тест его не подтвердит и сразу предложит попробовать грейд ниже. Категорию даёт только сданный тест, так что завышать нет смысла." },
                { q: "Сколько приглашений я буду получать?", a: "Зависит от спроса в вашей категории. У каждой компании 30 приглашений в неделю и только одно действующее для вас, поэтому рассылок не будет." },
                { q: "Компания увидит мои ответы в тесте?", a: "Нет. Она видит категорию, результат внутри грейда и навыки, которые тест подтвердил." },
                { q: "Как отказаться от приглашения?", a: "Нажмите «Отклонить» и выберите причину. Компания увидит её только в общей статистике, без вашего имени." },
                { q: "Кто смотрит мои контакты?", a: "Компании, чьё приглашение вы приняли, и те, кому вы откликнулись сами. Каждый просмотр записывается." },
                { q: "Как удалить аккаунт?", a: "В настройках. Персональные данные сотрутся. Если нужно только пропасть из поиска, отзовите согласие на публикацию." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Регистрация занимает пару минут"
        lead="Тест можно пройти сразу или вернуться к нему позже. Категория появится сразу после успешной сдачи."
        primary={{ href: "/register?role=candidate", label: "Пройти тест" }}
        secondary={{ href: "/fsp", label: "Я участник ФСП" }}
      />
    </MotionRoot>
  );
}
