import type { PublicVacancy } from "@/api/types";
import { GRADE_LABEL, SPEC_LONG_LABEL } from "@/lib/labels";
import { skillName } from "@/lib/skills";
import { PUBLIC_PAGES, SITE_NAME, absoluteUrl } from "@/lib/site";

type Thing = Record<string, unknown>;

const ORG_ID = absoluteUrl("/#organization");
const SITE_ID = absoluteUrl("/#website");

/** The platform and the site, on every public page; other nodes point at them by `@id`. */
export function siteGraph(): Thing {
  return {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "Organization",
        "@id": ORG_ID,
        name: SITE_NAME,
        url: absoluteUrl("/"),
        logo: absoluteUrl("/icons/icon-512.png"),
        description: PUBLIC_PAGES.home.description,
        parentOrganization: { "@type": "Organization", name: "Федерация спортивного программирования России", url: "https://fsp-russia.com" },
      },
      {
        "@type": "WebSite",
        "@id": SITE_ID,
        url: absoluteUrl("/"),
        name: SITE_NAME,
        inLanguage: "ru-RU",
        publisher: { "@id": ORG_ID },
      },
    ],
  };
}

/** Home → … → the page itself; `trail` lists the crumbs after the home page. */
export function breadcrumbs(trail: { name: string; path: string }[]): Thing {
  const items = [{ name: PUBLIC_PAGES.home.name, path: "/" }, ...trail];
  return {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: items.map((c, i) => ({ "@type": "ListItem", position: i + 1, name: c.name, item: absoluteUrl(c.path) })),
  };
}

/** The vacancy cards rendered on `/vacancies`, in the same order. */
export function vacancyList(items: PublicVacancy[]): Thing {
  return {
    "@context": "https://schema.org",
    "@type": "ItemList",
    name: PUBLIC_PAGES.vacancies.shareTitle,
    numberOfItems: items.length,
    itemListElement: items.map((v, i) => ({ "@type": "ListItem", position: i + 1, url: absoluteUrl(`/vacancies/${v.id}`), name: v.title })),
  };
}

/**
 * Only what the vacancy page shows. Our vacancies carry a work format (office, hybrid, remote) but no employment
 * type, so `employmentType` is left out rather than guessed; remote work is `jobLocationType: TELECOMMUTE`.
 */
export function jobPosting(v: PublicVacancy): Thing {
  const city = v.city ?? v.company_city ?? null;
  const remote = v.work_format === "remote";
  const place = city ? { "@type": "Place", address: { "@type": "PostalAddress", addressLocality: city, addressCountry: "RU" } } : null;
  return {
    "@context": "https://schema.org",
    "@type": "JobPosting",
    title: v.title,
    description: v.description,
    url: absoluteUrl(`/vacancies/${v.id}`),
    identifier: { "@type": "PropertyValue", name: SITE_NAME, value: v.id },
    ...(v.published_at ? { datePosted: v.published_at } : {}),
    hiringOrganization: { "@type": "Organization", name: v.company_name, ...(v.company_description ? { description: v.company_description } : {}) },
    ...(remote ? { jobLocationType: "TELECOMMUTE", applicantLocationRequirements: { "@type": "Country", name: "Россия" } } : {}),
    ...(place && !remote ? { jobLocation: place } : {}),
    baseSalary: { "@type": "MonetaryAmount", currency: "RUB", value: { "@type": "QuantitativeValue", minValue: v.salary.min, maxValue: v.salary.max, unitText: "MONTH" } },
    occupationalCategory: `${SPEC_LONG_LABEL[v.specialization]} · ${GRADE_LABEL[v.grade]}`,
    ...(v.skills.length ? { skills: v.skills.map(skillName).join(", ") } : {}),
    directApply: true,
  };
}
