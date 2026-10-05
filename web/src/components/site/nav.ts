export interface SiteLink {
  href: string;
  label: string;
  hint?: string;
}

export const SITE_NAV: SiteLink[] = [
  { href: "/candidates", label: "Кандидатам", hint: "Тест, категория, приглашения с вилкой" },
  { href: "/employers", label: "Работодателям", hint: "Подборка с объяснением и честный контакт" },
  { href: "/how-it-works", label: "Как устроено", hint: "Адаптивный тест, шаблоны, валидация" },
  { href: "/fsp", label: "Участникам ФСП", hint: "ФСП ID, разряды и призовые места" },
  { href: "/security", label: "Безопасность", hint: "152-ФЗ, шифрование, журнал доступа" },
  { href: "/vacancies", label: "Вакансии", hint: "Классический отклик" },
];

export const FOOTER_COLUMNS: { title: string; links: SiteLink[] }[] = [
  {
    title: "Кандидатам",
    links: [
      { href: "/candidates", label: "Как это работает" },
      { href: "/register?role=candidate", label: "Пройти тест и получить категорию" },
      { href: "/fsp", label: "Достижения ФСП в профиле" },
      { href: "/vacancies", label: "Открытые вакансии" },
    ],
  },
  {
    title: "Работодателям",
    links: [
      { href: "/employers", label: "Как нанимать" },
      { href: "/register?role=employer", label: "Начать подбор" },
      { href: "/employers#rules", label: "Квоты и правила контакта" },
      { href: "/employers#analytics", label: "Воронка и причины отказов" },
    ],
  },
  {
    title: "Платформа",
    links: [
      { href: "/how-it-works", label: "Как устроен тест и подбор" },
      { href: "/how-it-works#validation", label: "Валидация и метрики" },
      { href: "/security", label: "Безопасность и данные" },
      { href: "/login", label: "Вход в кабинет" },
    ],
  },
];
