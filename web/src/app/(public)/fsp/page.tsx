import { CloudOff, Link2Off, Trophy, UserRound } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CtaBand, FeatureGrid, PageHero, SpecList, SplitHeading } from "@/components/site/blocks";
import { Faq } from "@/components/site/Faq";
import { AchievementPath } from "@/components/site/fsp/AchievementPath";
import { DecayCurve } from "@/components/site/fsp/DecayCurve";
import { MedalDrop } from "@/components/site/fsp/MedalDrop";
import { RankClimb } from "@/components/site/fsp/RankClimb";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section } from "@/components/site/primitives";

export const metadata: Metadata = {
  title: "Участникам ФСП",
  description: "Привяжите ФСП ID: разряды и призовые места из реестра Федерации появятся в профиле и поднимут вас в подборке.",
};

const DISCIPLINES = ["Алгоритмическое программирование", "Продуктовое программирование", "Программирование систем ИБ", "Программирование систем ИИ", "Программирование робототехники", "Программирование БАС"];

export default function FspPage() {
  return (
    <MotionRoot variant="blur">
      <PageHero
        crumb="Участникам ФСП"
        title={
          <>
            Разряд ФСП теперь видят <span className="text-gradient">и работодатели</span>
          </>
        }
        lead="Привяжите ФСП ID, и разряды, призовые места и командные результаты из реестра Федерации появятся в профиле. Они поднимают вас среди кандидатов с похожим результатом теста. Если истории ФСП нет, вы ничего не теряете."
        actions={
          <>
            <Cta href="/register?role=candidate">Создать профиль</Cta>
            <Cta href="/candidate/settings" variant="outline">
              Привязать ФСП ID
            </Cta>
          </>
        }
        visual={<MedalDrop />}
        facts={[
          { value: "6", label: "дисциплин спортивного программирования" },
          { value: "до 10", label: "баллов из 100 в подборке дают достижения" },
          { value: "2 года", label: "и вклад достижения уменьшается вдвое" },
          { value: "0", label: "штрафа, если истории ФСП нет" },
        ]}
      />
      <PageTabs
        items={[
          { id: "link", label: "Привязка" },
          { id: "data", label: "Что учитываем" },
          { id: "rank", label: "Место в подборке" },
          { id: "decay", label: "Свежесть" },
          { id: "neutral", label: "Без истории" },
          { id: "faq", label: "Вопросы" },
        ]}
      />

      <Section id="link" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Привязка" title="Войдите через ФСП ID, достижения подтянутся сами">
            Кнопка в настройках кабинета открывает вход ФСП ID. Пароль вы вводите на стороне ФСП, мы его не видим.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: UserRound, title: "Один ФСП ID на профиль", text: "Одолжить достижения у друга не получится: ФСП ID привязывается только к одному кандидату." },
              { icon: CloudOff, title: "Если реестр молчит", text: "Привязка всё равно сохранится, а достижения подтянутся, когда реестр ответит." },
              { icon: Link2Off, title: "Отвязать можно в любой момент", text: "Вместе с отвязкой из профиля пропадают и достижения." },
            ]}
          />
        </Container>
      </Section>

      <Section id="data" tone="canvas" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Что учитываем" title="В профиль попадают спортивные достижения">
            Разряды и результаты соревнований, у каждого есть ссылка на протокол. Больше ничего из реестра мы не берём.
          </SplitHeading>
          <div className="mt-14 grid grid-cols-1 gap-12 lg:grid-cols-[1fr_1fr] lg:gap-16">
            <AchievementPath />
            <div>
              <SpecList
                rows={[
                  ["Разряд", "Юношеский, III, II, I, КМС или МС. В каждой дисциплине считается только самый высокий"],
                  ["Соревнования", "Региональные, межрегиональные, всероссийские, международные"],
                  ["Место", "1–3, финалист или участник"],
                  ["Команда", "Название команды и ваша роль в ней"],
                ]}
              />
              <p data-reveal className="mt-10 eyebrow text-ink-subtle">
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
                Важна связь дисциплины со специализацией. Победа в программировании систем ИБ поднимет кандидата в безопасность сильнее, чем фронтенд-разработчика.
              </p>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="rank" className="scroll-mt-16">
        <Container>
          <div className="grid grid-cols-1 items-center gap-14 lg:grid-cols-2 lg:gap-20">
            <div>
              <SplitHeading eyebrow="Место в подборке" title="ФСП поднимает среди равных" />
              <div data-reveal className="mt-8 space-y-5 text-lg leading-relaxed text-ink-muted">
                <p>Компания видит кандидатов одной категории, отсортированных по баллу. Достижения ФСП дают до 10 баллов из 100.</p>
                <p>Этого хватает, чтобы обойти соседей с таким же результатом теста. Кандидата с заметно более сильным тестом достижения не обгонят.</p>
              </div>
            </div>
            <div data-reveal>
              <RankClimb />
            </div>
          </div>
        </Container>
      </Section>

      <Section id="decay" tone="canvas" className="scroll-mt-16">
        <Container>
          <div className="grid grid-cols-1 items-center gap-14 lg:grid-cols-[1.1fr_0.9fr] lg:gap-20">
            <div data-reveal className="order-2 lg:order-1">
              <DecayCurve />
            </div>
            <div className="order-1 lg:order-2">
              <SplitHeading eyebrow="Свежесть" title="Вчерашняя победа весит больше давней" />
              <div data-reveal className="mt-8 space-y-5 text-lg leading-relaxed text-ink-muted">
                <p>Каждые два года вклад достижения уменьшается вдвое. Победа шестилетней давности ещё считается, но уже на восьмую часть.</p>
                <p>Каждое следующее достижение тоже добавляет меньше предыдущего. Поэтому десять дипломов результат теста не перебьют.</p>
              </div>
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
                Нет достижений? Ноль баллов, и всё.
              </h2>
              <p data-reveal className="mt-6 max-w-xl text-lg leading-relaxed text-night-muted">
                Кандидат без истории ФСП получает ноль баллов за этот признак. В карточке будет нейтральное «Нет данных ФСП», а в список пробелов отсутствие ФСП не попадает никогда. Категорию по-прежнему определяет только тест.
              </p>
            </div>
            <div data-reveal className="rounded-[24px] border border-night-line bg-night-raised p-6">
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
                  <span className="size-1.5 rounded-full bg-night-faint" aria-hidden /> Пробелы: только навыки, грейд и зарплата
                </li>
              </ul>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="faq" className="scroll-mt-16">
        <Container className="grid grid-cols-1 gap-12 lg:grid-cols-[0.75fr_1.25fr] lg:gap-20">
          <SplitHeading eyebrow="FAQ" title="Вопросы участников" />
          <div data-reveal>
            <Faq
              items={[
                { q: "Достижения заменяют тест?", a: "Нет. Категорию даёт только тест. Достижения поднимают вас внутри категории и видны компании в карточке." },
                { q: "Старые победы считаются?", a: "Считаются, но их вклад уменьшается вдвое каждые два года." },
                { q: "А командная победа?", a: "Тоже. В профиле сохраняются название команды и ваша роль." },
                { q: "Компания увидит мой ФСП ID?", a: "Компания видит сами достижения, например «Мастер спорта по продуктовому программированию, 2023». Контакты закрыты до вашего согласия." },
                { q: "Можно не показывать ФСП?", a: "Скройте достижения в настройках. Тогда они не видны компаниям и не влияют на подборку." },
              ]}
            />
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Привяжите ФСП ID после регистрации"
        lead="Создайте профиль, сдайте тест и нажмите «Привязать ФСП ID» в настройках кабинета."
        primary={{ href: "/register?role=candidate", label: "Создать профиль" }}
        secondary={{ href: "/candidates", label: "Путь кандидата" }}
      />
    </MotionRoot>
  );
}
