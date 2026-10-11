import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { VacancyDetail } from "@/components/domain/VacancyDetail";
import { JsonLd } from "@/components/seo/JsonLd";
import { Container } from "@/components/site/primitives";
import { formatSalaryRange } from "@/lib/format";
import { breadcrumbs, jobPosting } from "@/lib/jsonld";
import { GRADE_LABEL, SPEC_LONG_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { fetchPublicVacancy } from "@/lib/public-api";
import { PUBLIC_PAGES, pageMetadata } from "@/lib/site";

type Props = { params: Promise<{ id: string }> };

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { id } = await params;
  const v = (await fetchPublicVacancy(id)).vacancy?.data;
  if (!v) return { title: "Вакансия", robots: { index: false } };
  const facts = [formatSalaryRange(v.salary.min, v.salary.max), `${SPEC_LONG_LABEL[v.specialization]} · ${GRADE_LABEL[v.grade]}`, WORK_FORMAT_LABEL[v.work_format]];
  const summary = v.description.replace(/\s+/g, " ").trim();
  return pageMetadata({
    title: `${v.title} · ${v.company_name} · ФСП Таланты`,
    shareTitle: `${v.title} · ${v.company_name}`,
    description: `${facts.join(", ")}. ${summary.length > 150 ? `${summary.slice(0, 150).trimEnd()}…` : summary}`,
    path: `/vacancies/${v.id}`,
  });
}

/** Rendered on the server with the vacancy already in place, so crawlers without JavaScript read the full posting. */
export default async function VacancyPage({ params }: Props) {
  const { id } = await params;
  const { vacancy, missing } = await fetchPublicVacancy(id);
  if (missing) notFound();
  const v = vacancy?.data;
  return (
    <div className="flex flex-1 flex-col bg-canvas">
      {v && (
        <>
          <JsonLd data={jobPosting(v)} />
          <JsonLd data={breadcrumbs([PUBLIC_PAGES.vacancies, { name: v.title, path: `/vacancies/${v.id}` }])} />
        </>
      )}
      <Container className="pt-28 pb-14 sm:pt-32">
        <VacancyDetail id={id} backHref="/vacancies" backLabel="Все вакансии" seed={vacancy} />
      </Container>
    </div>
  );
}
