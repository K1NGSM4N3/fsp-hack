import type { Metadata } from "next";

/**
 * Absolute origin for canonical links, Open Graph URLs, JSON-LD and the sitemap. Public pages are prerendered, so set
 * `SITE_URL` at build time; the default is where `make up` serves the site.
 */
export const SITE_URL = new URL(process.env.SITE_URL ?? "http://localhost:8080");

export const SITE_NAME = "ФСП Таланты";

export const absoluteUrl = (path: string) => new URL(path, SITE_URL).href;

export interface PublicPage {
  path: string;
  /** Short name: navigation, breadcrumbs, link lists in `llms.txt`. */
  name: string;
  /** Full `<title>`, used as is. */
  title: string;
  /** Headline for link previews. */
  shareTitle: string;
  description: string;
  /** Who the page is for, one line for `llms.txt`. */
  audience: string;
  changeFrequency: "daily" | "weekly" | "monthly";
  priority: number;
}

/**
 * Every indexable page, once: page metadata, the sitemap, breadcrumbs, `llms.txt` and `llms-full.txt` read it from
 * here, so the descriptions machines see never drift from the pages.
 */
export const PUBLIC_PAGES = {
  home: {
    path: "/",
    name: "Главная",
    title: "ФСП Таланты · компании пишут первыми и сразу называют зарплату",
    shareTitle: "Компании пишут вам первыми и сразу называют зарплату",
    description:
      "Подбор ИТ-специалистов Федерации спортивного программирования России. Соискатель один раз проходит тест, компании пишут ему сами и в первом же сообщении называют зарплату от и до.",
    audience: "всем: как устроен подбор на примере одного приглашения",
    changeFrequency: "weekly",
    priority: 1,
  },
  seekers: {
    path: "/seekers",
    name: "Соискателям",
    title: "Соискателям · ФСП Таланты",
    shareTitle: "Один тест, и приглашения приходят к вам сами",
    description: "Короткий профиль, один тест на 10–25 минут и приглашения от компаний, где зарплата от и до указана в первом сообщении.",
    audience: "ИТ-специалистам: путь от профиля и теста до приглашения с вилкой, приватность, что подготовить",
    changeFrequency: "weekly",
    priority: 0.9,
  },
  business: {
    path: "/business",
    name: "Компаниям",
    title: "Компаниям · ФСП Таланты",
    shareTitle: "Специалисты с проверенным уровнем по тексту вашей вакансии",
    description:
      "Вставьте текст вакансии и получите подборку ИТ-специалистов, чей грейд подтверждён тестом. У каждого написано, чем подходит и чего не хватает. Зарплата в первом сообщении, контакты после согласия.",
    audience: "компаниям и рекрутерам: разбор текста вакансии, подборка с объяснениями, правила и квота приглашений",
    changeFrequency: "weekly",
    priority: 0.9,
  },
  fsp: {
    path: "/fsp",
    name: "Участникам ФСП",
    title: "Участникам ФСП · ФСП Таланты",
    shareTitle: "Результаты с соревнований теперь видят компании",
    description: "Привяжите ФСП ID, и разряды и призовые места из реестра Федерации появятся в вашем профиле со ссылками на протоколы.",
    audience: "участникам соревнований ФСП: привязка ФСП ID, разряды, дисциплины, как достижения влияют на подбор",
    changeFrequency: "monthly",
    priority: 0.8,
  },
  vacancies: {
    path: "/vacancies",
    name: "Вакансии",
    title: "Вакансии · ФСП Таланты",
    shareTitle: "Открытые ИТ-вакансии с вилкой от и до",
    description:
      "Вакансии компаний на платформе ФСП Таланты: бэкенд, фронтенд, DevOps, QA, ML и безопасность. В каждой карточке зарплата от и до, грейд, формат работы и стек.",
    audience: "соискателям: опубликованные вакансии с фильтрами по специализации, грейду, формату и зарплате",
    changeFrequency: "daily",
    priority: 0.8,
  },
  privacy: {
    path: "/privacy",
    name: "Политика обработки данных",
    title: "Политика обработки персональных данных · ФСП Таланты",
    shareTitle: "Политика обработки персональных данных",
    description: "Какие данные хранит платформа ФСП Таланты, зачем, кому их видно, сколько храним и как распоряжаться ими из личного кабинета.",
    audience: "всем: состав данных, цели, сроки, согласия по 152-ФЗ и права пользователя",
    changeFrequency: "monthly",
    priority: 0.3,
  },
} satisfies Record<string, PublicPage>;

export type PublicPageKey = keyof typeof PUBLIC_PAGES;

/**
 * Link preview text of the home page. It lives in the root layout, together with `app/opengraph-image.tsx`: a page
 * that sets its own `openGraph` drops the image inherited from a parent segment.
 */
export const HOME_SHARE = { title: PUBLIC_PAGES.home.shareTitle, description: PUBLIC_PAGES.home.description };

interface PageMeta {
  title: string;
  shareTitle?: string;
  description: string;
  path: string;
}

/**
 * Title, description, canonical URL and Open Graph text of a public page. Images come from `opengraph-image.tsx`;
 * the Twitter card reads title, description and image from Open Graph, so they are not repeated.
 */
export function pageMetadata({ title, shareTitle, description, path }: PageMeta): Metadata {
  const share = shareTitle ?? title;
  return {
    title: { absolute: title },
    description,
    alternates: { canonical: path },
    openGraph: { type: "website", locale: "ru_RU", siteName: SITE_NAME, url: path, title: share, description },
    twitter: { card: "summary_large_image" },
  };
}
