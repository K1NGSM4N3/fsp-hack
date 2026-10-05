export interface SiteLink {
  href: string;
  label: string;
}

export const SITE_NAV: SiteLink[] = [
  { href: "/candidates", label: "Кандидатам" },
  { href: "/employers", label: "Работодателям" },
  { href: "/fsp", label: "Участникам ФСП" },
  { href: "/vacancies", label: "Вакансии" },
];

export const FOOTER_COLUMNS: { title: string; links: SiteLink[] }[] = [
  {
    title: "Кандидатам",
    links: [
      { href: "/candidates", label: "Как это работает" },
      { href: "/candidates#test", label: "Каким будет тест" },
      { href: "/candidates#privacy", label: "Приватность и контакты" },
      { href: "/register?role=candidate", label: "Пройти тест" },
    ],
  },
  {
    title: "Работодателям",
    links: [
      { href: "/employers", label: "Как нанимать" },
      { href: "/employers#rules", label: "Правила контакта" },
      { href: "/employers#analytics", label: "Воронка и отказы" },
      { href: "/register?role=employer", label: "Начать подбор" },
    ],
  },
  {
    title: "Ещё",
    links: [
      { href: "/fsp", label: "Участникам ФСП" },
      { href: "/vacancies", label: "Вакансии" },
      { href: "/#how", label: "Как устроен тест" },
      { href: "/login", label: "Войти" },
    ],
  },
];
