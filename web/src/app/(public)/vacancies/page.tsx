import type { Metadata } from "next";
import Link from "next/link";
import { MotionRoot } from "@/components/motion/MotionRoot";
import { Container } from "@/components/site/primitives";
import { JsonLd } from "@/components/seo/JsonLd";
import { VACANCY_PAGE_SIZE } from "@/features/keys";
import { breadcrumbs, vacancyList } from "@/lib/jsonld";
import { fetchVacancyPage } from "@/lib/public-api";
import { PUBLIC_PAGES, pageMetadata } from "@/lib/site";
import { PublicVacancyBrowser } from "./PublicVacancyBrowser";

export const metadata: Metadata = pageMetadata(PUBLIC_PAGES.vacancies);

/**
 * Regenerated every minute with the first page of vacancies in the HTML, so crawlers see the list without JavaScript.
 * While the image is built the API is unreachable: that render has no list (the browser fetches it), and the short
 * interval replaces it soon after the first visits.
 */
export const revalidate = 60;

export default async function VacanciesPage() {
  const page = await fetchVacancyPage(VACANCY_PAGE_SIZE);
  // Cards never show the long texts or the company card; leaving them out keeps the hydration payload small.
  const seed = page && { ...page, data: { ...page.data, items: page.data.items.map((v) => ({ ...v, description: "", company_description: null, company_city: null })) } };
  return (
    <MotionRoot variant="wipe" className="flex flex-1 flex-col bg-canvas">
      <JsonLd data={breadcrumbs([PUBLIC_PAGES.vacancies])} />
      {seed && seed.data.items.length > 0 && <JsonLd data={vacancyList(seed.data.items)} />}
      <section className="relative isolate overflow-hidden border-b border-line bg-white pt-32 pb-12 sm:pt-40 sm:pb-16 short:pt-24">
        <div aria-hidden className="bg-grid mask-radial pointer-events-none absolute inset-0 -z-10 opacity-70" />
        <div aria-hidden className="pointer-events-none absolute -top-56 right-[-10rem] -z-10 size-[40rem] rounded-full bg-[radial-gradient(circle,rgb(64_47_255/0.18),transparent_64%)]" />
        <Container>
          <nav data-hero aria-label="Хлебные крошки" className="mb-6 flex items-center gap-2 font-mono text-xs tracking-[0.1em] text-ink-subtle uppercase">
            <Link href="/" className="tap transition-colors hover:text-primary">
              ФСП Таланты
            </Link>
            <span aria-hidden>/</span>
            <span className="text-primary">Вакансии</span>
          </nav>
          <div className="grid grid-cols-1 gap-8 lg:grid-cols-[1.3fr_0.7fr] lg:items-end">
            <h1 data-hero data-split="hero" className="display-2 text-balance text-ink">
              Открытые вакансии
            </h1>
            <p data-hero="lead" className="text-lg leading-relaxed text-ink-muted">
              Здесь можно откликнуться по-старому. Но после теста компании пишут сами, и откликаться становится необязательно.{" "}
              <Link href="/seekers" className="link-inline font-medium">
                Про тест
              </Link>
            </p>
          </div>
        </Container>
      </section>
      <Container className="py-10 sm:py-14">
        <PublicVacancyBrowser seed={seed} />
      </Container>
    </MotionRoot>
  );
}
