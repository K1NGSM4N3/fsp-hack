import { Ban, Banknote, Clock, FileLock, FileSearch, Gauge, KeyRound, LockKeyhole, ScrollText, Server, ShieldCheck, UserCheck } from "lucide-react";
import type { Metadata } from "next";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { CipherReveal } from "@/components/site/CipherReveal";
import { CtaBand, FeatureGrid, Note, PageHero, SpecList, SplitHeading } from "@/components/site/blocks";
import { PageTabs } from "@/components/site/PageTabs";
import { Container, Cta, Section } from "@/components/site/primitives";

export const metadata: Metadata = {
  title: "Безопасность и данные",
  description: "Шифрование персональных данных, контакты только по основанию доступа, журнал обращений, согласия, удаление и хранение в РФ по 152-ФЗ.",
};

export default function SecurityPage() {
  return (
    <MotionRoot>
      <PageHero
        crumb="Безопасность"
        title={
          <>
            Приватность <span className="text-gradient">по умолчанию,</span> а не по запросу
          </>
        }
        lead="Контакты кандидата хранятся зашифрованными и не существуют в данных, которые видит работодатель. Открыть их можно только по основанию — принятому приглашению или отклику, — и каждое такое обращение остаётся в журнале."
        actions={
          <>
            <Cta href="/register?role=candidate">Создать профиль</Cta>
            <Cta href="#pd" variant="outline">
              Как храним данные
            </Cta>
          </>
        }
        visual={<CipherReveal />}
        facts={[
          { value: "XChaCha20", label: "Poly1305 — шифрование email, ФИО и контактов" },
          { value: "Argon2id", label: "хеширование паролей: 19 МиБ, t = 2, p = 1" },
          { value: "15 мин", label: "живёт access-токен; refresh ротируется" },
          { value: "РФ", label: "серверы хранения, требования 152-ФЗ" },
        ]}
      />
      <PageTabs
        items={[
          { id: "pd", label: "Персональные данные" },
          { id: "access", label: "Доступ к контактам" },
          { id: "auth", label: "Аутентификация" },
          { id: "crypto", label: "Криптография и ГОСТ" },
          { id: "antifraud", label: "Антифрод" },
        ]}
      />

      <Section id="pd" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Персональные данные" title="Храним минимум и только зашифрованным">
            Всё, что идентифицирует человека, лежит в базе в виде шифротекста. Поиск по email работает без расшифровки — через keyed-хеш.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            items={[
              { icon: KeyRound, title: "Шифрование полей", text: "Email, ФИО и контакты кандидата — XChaCha20-Poly1305. Версия шифротекста хранится вместе с данными." },
              { icon: FileSearch, title: "Blind index", text: "Поиск по email — через keyed-хеш: по базе нельзя восстановить адрес, но можно найти аккаунт при входе." },
              { icon: UserCheck, title: "Версионированные согласия", text: "Согласия на обработку и публикацию фиксируются со временем, IP и User-Agent. Отзыв согласия на публикацию скрывает профиль." },
              { icon: Ban, title: "Удаление аккаунта", text: "Стирает персональные данные. Не «деактивация до лучших времён», а удаление." },
              { icon: ScrollText, title: "Журнал доступа", text: "Кто, когда и на каком основании обращался к контактам кандидата." },
              { icon: Server, title: "Хранение в РФ", text: "Серверы в России, обработка в соответствии со 152-ФЗ." },
            ]}
          />
        </Container>
      </Section>

      <Section id="access" tone="night" className="scroll-mt-16 overflow-hidden">
        <div aria-hidden className="pointer-events-none absolute -right-40 -bottom-40 size-[40rem] rounded-full bg-primary/30 blur-[160px]" />
        <Container className="relative">
          <SplitHeading eyebrow="Разграничение доступа" title="Контакты закрыты не настройкой, а типами" tone="night">
            Мы не полагаемся на то, что фронтенд «не покажет» лишнее. Данные, которые получает работодатель, физически не содержат контактов.
          </SplitHeading>
          <div data-stagger="0.12" className="mt-14 grid grid-cols-1 gap-4 lg:grid-cols-3">
            {[
              { k: "Карточка кандидата", v: "Категория, навыки, результат, опыт, ожидания. Полей контактов в этом типе данных нет вообще.", tag: "работодатель видит" },
              { k: "Основание доступа", v: "Выдаёт единственная функция: проверяет принятое приглашение или собственный отклик кандидата и пишет журнал.", tag: "ContactGrant" },
              { k: "Контакты", v: "Строятся только из основания доступа. Нет основания — нет способа получить контакты ни через интерфейс, ни через API.", tag: "после согласия" },
            ].map((x, i) => (
              <div key={x.k} className="relative rounded-[24px] border border-night-line bg-night-raised p-7">
                <span className="font-mono text-xs text-primary-300">0{i + 1} · {x.tag}</span>
                <h3 className="mt-6 text-2xl font-semibold tracking-tight">{x.k}</h3>
                <p className="mt-3 text-[15px] leading-relaxed text-night-muted">{x.v}</p>
              </div>
            ))}
          </div>
          <div className="mt-12">
            <SpecList
              tone="night"
              rows={[
                ["Роли", "Права кандидата и работодателя проверяются на уровне маршрутов API; интерфейс ничего не решает"],
                ["Чужие ресурсы", "Кандидат видит только свои приглашения, отклики и тесты; чужие ресурсы возвращают 404, а не 403"],
                ["Ограничение частоты", "Вход, регистрация, повторные письма, приглашения и жалобы ограничены по частоте"],
              ]}
            />
          </div>
        </Container>
      </Section>

      <Section id="auth" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Аутентификация" title="Сессии, которые сложно украсть и бесполезно переиспользовать" />
          <div className="mt-14">
            <SpecList
              rows={[
                ["Пароли", "Argon2id (19 МиБ, t = 2, p = 1). Для неизвестного email выполняется та же проверка — время ответа не выдаёт наличие аккаунта"],
                ["Access-токен", "JWT на EdDSA, живёт 15 минут, в httpOnly-cookie"],
                ["Refresh-токен", "Ротируется при каждом обновлении, живёт 30 дней. Повторное использование старого токена отзывает всю цепочку"],
                ["Выход", "Access-токен заносится в deny-list — разлогин действует сразу, а не через 15 минут"],
                ["CSRF", "Изменяющие запросы с cookie принимаются только с собственного Origin"],
                ["Email", "Подтверждение по ссылке обязательно до входа в кабинет"],
              ]}
            />
          </div>
        </Container>
      </Section>

      <Section id="crypto" tone="canvas" className="scroll-mt-16">
        <Container>
          <div className="grid grid-cols-1 gap-12 lg:grid-cols-2 lg:gap-20">
            <SplitHeading eyebrow="Криптография" title="Готовность к ГОСТ — без переделки схемы данных" />
            <div data-reveal className="space-y-5 text-lg leading-relaxed text-ink-muted">
              <p>Алгоритм хранится вместе с данными: строка хеша в формате PHC, версия у каждого шифротекста. Хеширование и шифрование вынесены за интерфейсы.</p>
              <p>
                Переход на PBKDF2-HMAC-Стрибог (Р 50.1.111-2016), «Кузнечик» и сертифицированное СКЗИ — это новая реализация интерфейса, а не миграция базы. В MVP ГОСТ не используется: соответствие обеспечивает сертификация СКЗИ, а не сам алгоритм.
              </p>
            </div>
          </div>
        </Container>
      </Section>

      <Section id="antifraud" className="scroll-mt-16">
        <Container>
          <SplitHeading eyebrow="Защита от недобросовестных" title="Фиктивным вакансиям здесь неуютно">
            Часть механизмов уже работает, часть — следующий шаг. Мы честно разделяем одно и другое.
          </SplitHeading>
          <FeatureGrid
            className="mt-14"
            columns={4}
            items={[
              { icon: Banknote, title: "Обязательная вилка", text: "Без зарплаты «от–до» приглашение не отправить.", badge: "работает" },
              { icon: Gauge, title: "Квота приглашений", text: "30 за 7 дней на компанию и одно действующее на пару.", badge: "работает" },
              { icon: Clock, title: "Срок жизни", text: "Приглашение истекает через 14 дней.", badge: "работает" },
              { icon: ShieldCheck, title: "Жалобы", text: "На приглашение или вакансию — в один клик.", badge: "работает" },
              { icon: FileLock, title: "Проверка компании", text: "Корпоративный домен email, ИНН/ОГРН по ЕГРЮЛ.", badge: "в планах" },
              { icon: LockKeyhole, title: "Рейтинг работодателя", text: "По доле ответов кандидатам и жалобам.", badge: "в планах" },
              { icon: Banknote, title: "Вилка против рынка", text: "Слишком высокая для категории — повод для проверки.", badge: "в планах" },
              { icon: FileSearch, title: "Дубли вакансий", text: "Поиск почти одинаковых текстов по эмбеддингам и ручная модерация по сигналам.", badge: "в планах" },
            ]}
          />
          <div className="mt-8">
            <Note>Подробное описание архитектуры безопасности — в документации проекта и в спецификации API (Swagger, /api/docs).</Note>
          </div>
        </Container>
      </Section>

      <CtaBand
        title="Ваши контакты увидит только тот, кому вы сказали «да»"
        primary={{ href: "/register?role=candidate", label: "Создать профиль" }}
        secondary={{ href: "/candidates", label: "Путь кандидата" }}
      />
    </MotionRoot>
  );
}
