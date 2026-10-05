export interface SiteLink {
  href: string;
  label: string;
}

export const SITE_NAV: SiteLink[] = [
  { href: "/seekers", label: "Соискателям" },
  { href: "/business", label: "Компаниям" },
  { href: "/fsp", label: "Участникам ФСП" },
  { href: "/vacancies", label: "Вакансии" },
];

export const FOOTER_COLUMNS: { title: string; links: SiteLink[] }[] = [
  {
    title: "Соискателям",
    links: [
      { href: "/seekers", label: "Как найти работу" },
      { href: "/vacancies", label: "Открытые вакансии" },
      { href: "/register?role=candidate", label: "Пройти тест" },
    ],
  },
  {
    title: "Компаниям",
    links: [
      { href: "/business", label: "Как нанимать" },
      { href: "/register?role=employer", label: "Начать подбор" },
    ],
  },
  {
    title: "Ещё",
    links: [
      { href: "/fsp", label: "Участникам ФСП" },
      { href: "/login", label: "Войти" },
    ],
  },
];
