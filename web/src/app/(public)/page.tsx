import {
  ArrowRight,
  BadgeCheck,
  Banknote,
  Building2,
  Check,
  ClipboardList,
  EyeOff,
  FileSearch,
  GraduationCap,
  Layers,
  ListFilter,
  Lock,
  Send,
  ShieldCheck,
  Sparkles,
  Timer,
  Trophy,
  UserRound,
} from "lucide-react";
import type { ReactNode } from "react";
import { NodeLines } from "@/components/brand/Logo";
import { ButtonLink } from "@/components/ui";

function Feature({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) {
  return (
    <div className="rounded-card border border-line bg-white p-6">
      <div className="mb-4 flex size-10 items-center justify-center rounded-full bg-primary-50 text-primary">{icon}</div>
      <h3 className="font-mono text-base font-semibold tracking-tight text-ink">{title}</h3>
      <p className="mt-2 text-sm leading-relaxed text-ink-muted">{children}</p>
    </div>
  );
}

function StepRow({ n, title, text, last = false }: { n: number; title: string; text: string; last?: boolean }) {
  return (
    <li className="relative flex gap-4 pb-8 last:pb-0">
      {!last && <span className="absolute top-9 bottom-0 left-[17px] w-px bg-primary-200" aria-hidden />}
      <span className="relative z-10 flex size-9 shrink-0 items-center justify-center rounded-full border border-primary bg-white font-mono text-sm font-semibold text-primary">{n}</span>
      <div className="pt-1.5">
        <p className="font-medium text-ink">{title}</p>
        <p className="mt-1 text-sm text-ink-muted">{text}</p>
      </div>
    </li>
  );
}

function InvitationPreview() {
  return (
    <div className="relative mx-auto w-full max-w-md">
      <div className="absolute -inset-6 -z-10 rounded-[28px] bg-primary-50" aria-hidden />
      <div className="rounded-card border border-line bg-white p-5 shadow-pop">
        <div className="flex items-center justify-between gap-3">
          <span className="inline-flex items-center gap-2 text-xs font-medium text-primary-strong">
            <span className="size-2 rounded-full bg-accent" aria-hidden />
            Новое приглашение
          </span>
          <span className="text-xs text-ink-subtle">сегодня</span>
        </div>
        <p className="mt-4 font-mono text-lg font-semibold tracking-tight text-ink">Backend-разработчик в команду платежей</p>
        <p className="mt-1 text-sm text-ink-muted">Полярный код · Финтех · гибрид, Москва</p>
        <div className="mt-4 rounded-control bg-canvas p-4">
          <p className="text-xs text-ink-subtle">Зарплата</p>
          <p className="tabular font-mono text-2xl font-semibold tracking-tight text-ink">260 000 – 340 000 ₽</p>
        </div>
        <ul className="mt-4 space-y-1.5 text-sm">
          <li className="flex gap-2">
            <Check className="mt-0.5 size-4 text-success" aria-hidden /> Категория Backend · Middle подтверждена тестом
          </li>
          <li className="flex gap-2">
            <Check className="mt-0.5 size-4 text-success" aria-hidden /> Призёр всероссийских соревнований ФСП
          </li>
        </ul>
        <div className="mt-5 grid grid-cols-2 gap-2">
          <span className="inline-flex h-10 items-center justify-center rounded-control bg-primary text-sm font-medium text-white">Принять</span>
          <span className="inline-flex h-10 items-center justify-center rounded-control border border-line-strong text-sm font-medium text-ink">Отклонить</span>
        </div>
        <p className="mt-3 flex items-center gap-1.5 text-xs text-ink-subtle">
          <Lock className="size-3.5" aria-hidden /> Компания увидит ваши контакты только после «Принять»
        </p>
      </div>
    </div>
  );
}

export default function LandingPage() {
  return (
    <>
      <section className="relative overflow-hidden border-b border-line">
        <NodeLines className="pointer-events-none absolute -top-6 -right-24 hidden h-[420px] w-[560px] text-primary-200 lg:block" />
        <div className="mx-auto grid max-w-6xl items-center gap-12 px-4 py-14 sm:px-6 sm:py-20 lg:grid-cols-[1.15fr_1fr]">
          <div>
            <p className="font-mono text-xs font-medium tracking-[0.12em] text-primary uppercase">Федерация спортивного программирования</p>
            <h1 className="mt-4 font-mono text-[32px] leading-[1.1] font-semibold tracking-tight text-ink sm:text-5xl">
              Работодатель сам приходит к&nbsp;вам — <span className="text-primary">с&nbsp;вилкой в&nbsp;первом сообщении</span>
            </h1>
            <p className="mt-5 max-w-xl text-base leading-relaxed text-ink-muted sm:text-lg">
              Пройдите адаптивный тест и получите категорию «специализация × грейд». Компании ищут в подтверждённых категориях, а достижения ФСП поднимают вас выше.
            </p>
            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <ButtonLink href="/register?role=candidate" size="lg" icon={<UserRound className="size-5" aria-hidden />}>
                Я ищу работу
              </ButtonLink>
              <ButtonLink href="/register?role=employer" size="lg" variant="secondary" icon={<Building2 className="size-5" aria-hidden />}>
                Я ищу специалиста
              </ButtonLink>
            </div>
            <ul className="mt-8 flex flex-wrap gap-x-6 gap-y-2 text-sm text-ink-muted">
              <li className="inline-flex items-center gap-2">
                <Banknote className="size-4 text-primary" aria-hidden /> Зарплата видна до общения
              </li>
              <li className="inline-flex items-center gap-2">
                <BadgeCheck className="size-4 text-primary" aria-hidden /> Категория подтверждена тестом
              </li>
              <li className="inline-flex items-center gap-2">
                <EyeOff className="size-4 text-primary" aria-hidden /> Анонимно до вашего согласия
              </li>
            </ul>
          </div>
          <InvitationPreview />
        </div>
      </section>

      <section id="candidates" className="scroll-mt-20 py-16 sm:py-20">
        <div className="mx-auto max-w-6xl px-4 sm:px-6">
          <p className="font-mono text-xs font-medium tracking-[0.12em] text-primary uppercase">Кандидатам</p>
          <h2 className="mt-3 max-w-2xl font-mono text-2xl font-semibold tracking-tight text-ink sm:text-3xl">Не отправляйте сотни откликов — пусть компании выходят на вас</h2>
          <div className="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Feature icon={<Banknote className="size-5" aria-hidden />} title="Вилка сразу">
              В каждом приглашении — зарплата «от–до», описание задачи и способ связи. Решаете, тратить ли время, ещё до разговора.
            </Feature>
            <Feature icon={<GraduationCap className="size-5" aria-hidden />} title="Категория по тесту">
              Резюме не определяет грейд. Адаптивный тест подбирает задания под ваш уровень и подтверждает специализацию и грейд.
            </Feature>
            <Feature icon={<Trophy className="size-5" aria-hidden />} title="Достижения ФСП в плюс">
              Привяжите ФСП ID: призовые места и разряды поднимают вас внутри категории. Нет истории ФСП — это не минус.
            </Feature>
            <Feature icon={<ShieldCheck className="size-5" aria-hidden />} title="Контакты под контролем">
              Работодатель видит анонимную карточку. Контакты открываются только после вашего согласия, каждый доступ в журнале.
            </Feature>
          </div>
        </div>
      </section>

      <section id="how" className="scroll-mt-20 border-y border-line bg-canvas py-16 sm:py-20">
        <div className="mx-auto grid max-w-6xl gap-12 px-4 sm:px-6 lg:grid-cols-2">
          <div>
            <h2 className="font-mono text-2xl font-semibold tracking-tight text-ink">Путь кандидата</h2>
            <p className="mt-2 text-sm text-ink-muted">Около 30 минут — и профиль готов к предложениям.</p>
            <ol className="mt-8">
              <StepRow n={1} title="Профиль" text="Заполните форму или загрузите PDF-резюме — поля распознаются автоматически." />
              <StepRow n={2} title="Опрос" text="Направление, специализация, стек и отрасли. Заявите грейд, который хотите подтвердить." />
              <StepRow n={3} title="Тест" text="10–20 заданий с таймером. Задания генерируются из шаблонов — у каждого свои, сложность сопоставима." />
              <StepRow last n={4} title="Категория" text="Например, Backend · Middle. Работодатели находят вас и приглашают с вилкой." />
            </ol>
          </div>
          <div>
            <h2 className="font-mono text-2xl font-semibold tracking-tight text-ink">Путь работодателя</h2>
            <p className="mt-2 text-sm text-ink-muted">От описания задачи до контакта — без разбора сотен откликов.</p>
            <ol className="mt-8">
              <StepRow n={1} title="Опишите потребность" text="Вставьте текст вакансии — система распознает стек, грейд, формат и вилку." />
              <StepRow n={2} title="Получите подборку" text="Рекомендованные категории и кандидаты с баллом 0–100 и объяснением: что совпало и чего не хватает." />
              <StepRow n={3} title="Уточните фильтры" text="Закреплённые кандидаты остаются, новые помечаются — подборка не теряется." />
              <StepRow last n={4} title="Пригласите точечно" text="Приглашение с вилкой и способом связи. Контакты — после согласия кандидата." />
            </ol>
          </div>
        </div>
      </section>

      <section id="employers" className="scroll-mt-20 py-16 sm:py-20">
        <div className="mx-auto max-w-6xl px-4 sm:px-6">
          <p className="font-mono text-xs font-medium tracking-[0.12em] text-primary uppercase">Работодателям</p>
          <h2 className="mt-3 max-w-2xl font-mono text-2xl font-semibold tracking-tight text-ink sm:text-3xl">Ищите по подтверждённым навыкам, а не по ключевым словам</h2>
          <div className="mt-10 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Feature icon={<FileSearch className="size-5" aria-hidden />} title="Текст → требования">
              Свободное описание превращается в специализацию, грейды, обязательные и желательные навыки.
            </Feature>
            <Feature icon={<Sparkles className="size-5" aria-hidden />} title="Объяснимый рейтинг">
              Балл раскладывается на признаки: навыки, сила теста, грейд, ФСП, зарплата, актуальность.
            </Feature>
            <Feature icon={<ListFilter className="size-5" aria-hidden />} title="Уточнение без потерь">
              Меняйте фильтры — закреплённые кандидаты сохраняются, новые отмечаются.
            </Feature>
            <Feature icon={<Send className="size-5" aria-hidden />} title="Точечные приглашения">
              До 30 приглашений в неделю и одно активное на кандидата: кандидаты отвечают чаще, чем на рассылки.
            </Feature>
          </div>
        </div>
      </section>

      <section className="border-t border-line bg-white py-16 sm:py-20">
        <div className="mx-auto max-w-6xl space-y-10 px-4 sm:px-6">
          <div className="max-w-2xl">
            <h2 className="font-mono text-2xl font-semibold tracking-tight text-ink sm:text-3xl">Тест, который бесполезно сливать</h2>
            <p className="mt-3 text-ink-muted">Задания не хранятся готовыми: генератор собирает каждое по шаблону со своим кодом, числами и вариантами, а правильный ответ вычисляется.</p>
          </div>
          <ul className="grid gap-3 sm:grid-cols-3">
            <li className="rounded-card border border-line p-5">
              <Layers className="size-5 text-primary" aria-hidden />
              <p className="mt-3 text-sm font-medium text-ink">Шаблоны + IRT</p>
              <p className="mt-1 text-sm text-ink-muted">Сложность откалибрована, поэтому разные задания сопоставимы.</p>
            </li>
            <li className="rounded-card border border-line p-5">
              <ClipboardList className="size-5 text-primary" aria-hidden />
              <p className="mt-3 text-sm font-medium text-ink">Адаптивность</p>
              <p className="mt-1 text-sm text-ink-muted">Следующее задание выбирается под текущую оценку уровня.</p>
            </li>
            <li className="rounded-card border border-line p-5">
              <Timer className="size-5 text-primary" aria-hidden />
              <p className="mt-3 text-sm font-medium text-ink">Таймер</p>
              <p className="mt-1 text-sm text-ink-muted">Ограничение времени на каждое задание — гуглить некогда.</p>
            </li>
          </ul>
        </div>
      </section>

      <section className="bg-primary">
        <div className="mx-auto flex max-w-6xl flex-col items-start justify-between gap-6 px-4 py-12 sm:px-6 md:flex-row md:items-center">
          <div>
            <h2 className="font-mono text-2xl font-semibold tracking-tight text-white">Начните с категории</h2>
            <p className="mt-2 text-primary-100">Регистрация, опрос и тест — и вы в выдаче работодателей.</p>
          </div>
          <div className="flex flex-col gap-3 sm:flex-row">
            <ButtonLink href="/register?role=candidate" size="lg" variant="secondary" className="border-white" icon={<ArrowRight className="size-5" aria-hidden />}>
              Получить категорию
            </ButtonLink>
            <ButtonLink href="/vacancies" size="lg" className="border border-white/40 bg-transparent hover:bg-white/10">
              Смотреть вакансии
            </ButtonLink>
          </div>
        </div>
      </section>
    </>
  );
}
