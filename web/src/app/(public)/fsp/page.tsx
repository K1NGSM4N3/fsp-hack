import { CloudOff, EyeOff, Link2, Link2Off, Scale, Trophy, UserRound } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, Note, PageHero, SpecList, SplitHeading } from "@/components/site/blocks";
import { Faq } from "@/components/site/Faq";
import { FspScore } from "@/components/site/FspScore";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section } from "@/components/site/primitives";
import { FspFlow } from "@/components/site/visuals";

export const metadata: Metadata = {
  title: "Участникам ФСП",
  description: "Привязка ФСП ID, учёт разрядов и призовых мест в подборе и нейтральное отношение к кандидатам без истории ФСП.",
};

const DISCIPLINES = ["Алгоритмическое программирование", "Продуктовое программирование", "Программирование систем информационной безопасности", "Программирование систем ИИ", "Программирование робототехники", "Программирование БАС"];

export default function FspPage() {
  return (
    <MotionRoot>
      <PageHero
        crumb="Участникам ФСП"
        title={
          <>
            Ваши разряды <span className="text-gradient">наконец-то работают</span> на карьеру
          </>
        }
        lead="Спортивное программирование — это дедлайны, команда и задачи без готового ответа. Привяжите ФСП ID, и достижения из реестра Федерации поднимут вас внутри категории. А если истории ФСП нет — вы ничего не теряете."
        actions={
          <>
            <Cta href="/register?role=candidate">Создать профиль</Cta>
            <Cta href="/login?next=/candidate/settings" variant="outline">
              Привязать ФСП ID
            </Cta>
          </>
        }
        visual={<FspFlow />}
        facts={[
          { value: "6", label: "дисциплин спортивного программирования учитываются" },
          { value: "≤ 10", label: "баллов из 100 — вес ФСП в подборе, тест важнее" },
          { value: "2 года", label: "период полураспада вклада достижения" },
          { value: "0", label: "штрафа за отсутствие истории ФСП" },
        ]}
      />
      <PageTabs
        items={[
          { id: "link", label: "Привязка" },
          { id: "data", label: "Какие данные" },
          { id: "score", label: "ФСП-скор" },
          { id: "neutral", label: "Без истории" },
          { id: "control", label: "Контроль" },
          { id: "faq", label: "Вопросы" },
        ]}
      />

      <Section id="link" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Привязка" title="Вход через ФСП ID — и достижения уже в профиле">
            Привязка работает по стандартному OpenID Connect: ваш пароль от ФСП ID остаётся у ФСП, мы получаем только подписанный идентификатор участника.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: Link2, title: "OIDC + PKCE", text: "Клиент с обязательным PKCE S256, проверкой state и nonce. Обмен кода на токен — по обратному каналу, подпись проверяется по JWKS." },
              { icon: UserRound, title: "Один ФСП ID — один профиль", text: "Достижения нельзя «одолжить»: ФСП ID привязывается только к одному профилю кандидата." },
              { icon: CloudOff, title: "Реестр недоступен?", text: "Привязка сохраняется, синхронизация достижений повторится позже автоматически." },
            ]}
          />
        </Container>
      </Section>

      <Section id="data" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Данные из реестра" title="Что именно мы получаем">
            Только спортивные достижения — то, что относится к профессиональному уровню. Каждое попадает в снимок профиля со ссылкой на протокол.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-10 lg:grid-cols-[1.2fr_0.8fr] lg:gap-16">
            <SpecList
              rows={[
                ["Тип", "Разряд или соревнование"],
                ["Разряд", "Юношеский, III, II, I, КМС, МС — учитывается только высший в дисциплине"],
                ["Уровень соревнования", "Региональное, межрегиональное, всероссийское, международное"],
                ["Место", "1–3, финалист, участник"],
                ["Команда и роль", "Название команды и ваша роль в ней"],
                ["Дата и протокол", "Когда получено и ссылка на протокол соревнования"],
              ]}
            />
            <div>
              <p data-reveal className="eyebrow text-ink-subtle">
                Дисциплины
              </p>
              <ul data-stagger="0.05" className="mt-5 flex flex-wrap gap-2">
                {DISCIPLINES.map((d) => (
                  <li key={d} className="rounded-full border border-line bg-white px-4 py-2 text-sm text-ink">
                    {d}
                  </li>
                ))}
              </ul>
              <p data-reveal className="mt-8 text-[15px] leading-relaxed text-ink-muted">
                Релевантность дисциплины специализации учитывается: победа в программировании систем ИБ весит больше для кандидата в безопасность, чем для фронтенд-разработчика.
              </p>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="score" className="scroll-mt-16">
        <Container>
          <div className="grid grid-cols-1 items-center gap-14 lg:grid-cols-2 lg:gap-20">
            <div>
              <SplitHeading eyebrow="ФСП-скор" title="Растёт с каждым достижением, но не перевешивает тест" />
              <div data-reveal className="mt-8 space-y-5 text-lg leading-relaxed text-ink-muted">
                <p>Вклад достижения = вес уровня соревнования × вес места (или вес разряда) × релевантность дисциплины специализации × затухание с периодом полураспада 2 года.</p>
                <p>Итог — «шумное ИЛИ»: 1 − Π(1 − vᵢ). Каждое новое достижение добавляет, но скор насыщается. В подборе его вес — 10 из 100, в силе профиля — 0,25 против 0,6 у теста.</p>
              </div>
            </div>
            <div data-reveal="scale">
              <FspScore />
            </div>
          </div>
        </Container>
      </Section>

      <Section id="neutral" tone="night" className="scroll-mt-16 overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute -top-40 left-1/3 size-[40rem] rounded-full bg-primary/30 blur-[160px]" />
        <Container className="relative">
          <div className="grid grid-cols-1 items-center gap-12 lg:grid-cols-[1.2fr_0.8fr]">
            <div>
              <p data-reveal="fade" className="eyebrow text-primary-300">
                Без истории ФСП
              </p>
              <h2 data-split className="display-2 mt-5 text-balance">
                Нет достижений — это ноль. Не минус.
              </h2>
              <p data-reveal className="mt-6 max-w-xl text-lg leading-relaxed text-night-muted">
                Кандидат без истории ФСП получает нулевой ФСП-скор без штрафа. В карточке — нейтральное «Нет данных ФСП», и отсутствие ФСП никогда не попадает в список пробелов. Категорию по-прежнему определяет только тест.
              </p>
            </div>
            <div data-reveal="scale" className="rounded-[24px] border border-night-line bg-night-raised p-6">
              <p className="font-mono text-xs tracking-[0.1em] text-night-faint uppercase">Так это выглядит в карточке</p>
              <div className="mt-5 flex items-center gap-3 rounded-2xl border border-dashed border-night-line px-4 py-3 text-sm text-night-muted">
                <Trophy className="size-4 text-night-faint" aria-hidden />
                Нет данных ФСП
              </div>
              <ul className="mt-5 space-y-2 text-sm">
                <li className="flex items-center gap-2 text-white">
                  <span className="size-1.5 rounded-full bg-success-300" aria-hidden /> Причины: навыки, грейд, результат теста
                </li>
                <li className="flex items-center gap-2 text-night-muted">
                  <span className="size-1.5 rounded-full bg-night-faint" aria-hidden /> Пробелы: только про навыки, грейд и зарплату
                </li>
              </ul>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="control" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Контроль" title="Ваши достижения — ваши правила" />
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: EyeOff, title: "Скрыть ФСП-данные", text: "Скрытые достижения не показываются работодателям и не участвуют в ранжировании." },
              { icon: Link2Off, title: "Отвязать ФСП ID", text: "Отвязка удаляет снимок достижений из профиля. Привязать снова можно в любой момент." },
              { icon: Scale, title: "Только высший разряд", text: "В каждой дисциплине учитывается один — высший — разряд, а не все промежуточные." },
            ]}
          />
          <div className="mt-8">
            <Note>Для подключения к боевому реестру ФСП достаточно заменить адрес реестра и выдать платформе клиентские учётные данные Keycloak — схема привязки уже работает на Keycloak 26.</Note>
          </div>
        </Container>
      </Section>

      <Section id="faq" className="scroll-mt-16">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <SplitHeading eyebrow="FAQ" title="Вопросы участников" />
          <div data-reveal>
            <Faq
              items={[
                { q: "Достижения заменяют тест?", a: "Нет. Категорию даёт только тест. Достижения поднимают вас внутри категории и показываются работодателю в карточке." },
                { q: "Старые победы тоже учитываются?", a: "Да, но с затуханием: вклад достижения уменьшается вдвое каждые 2 года. Свежие результаты важнее." },
                { q: "Командная победа считается?", a: "Да. В снимке хранятся команда и ваша роль, а вклад определяется уровнем соревнования, местом и релевантностью дисциплины." },
                { q: "Работодатель увидит мой ФСП ID?", a: "Работодатель видит достижения в карточке — например, «Мастер спорта по продуктовому программированию, 2023». Контакты по-прежнему закрыты до вашего согласия." },
                { q: "Что если я не хочу показывать ФСП?", a: "Скройте ФСП-данные в настройках: они перестанут показываться и участвовать в ранжировании." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Соревнования — уже опыт. Пусть его увидят."
        lead="Создайте профиль, пройдите тест и привяжите ФСП ID в настройках кабинета."
        primary={{ href: "/register?role=candidate", label: "Создать профиль" }}
        secondary={{ href: "/candidates", label: "Путь кандидата" }}
      />
    </MotionRoot>
  );
}
