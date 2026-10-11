import { formatSalaryRange } from "@/lib/format";
import { GRADE_LABEL, SPEC_LONG_LABEL, WORK_FORMAT_LABEL } from "@/lib/labels";
import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";
import { fetchPublicVacancy } from "@/lib/public-api";

export const alt = "Вакансия на ФСП Таланты: зарплата от и до, грейд и формат работы";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default async function Image({ params }: { params: Promise<{ id: string }> }) {
  const v = (await fetchPublicVacancy((await params).id)).vacancy?.data;
  if (!v) {
    return ogImage({
      eyebrow: "Вакансии",
      title: "Открытые ИТ-вакансии с вилкой от и до",
      accent: "с вилкой от и до",
      chips: ["Junior", "Middle", "Senior"],
    });
  }
  const place = [v.company_name, v.city ?? v.company_city].filter(Boolean).join(" · ");
  return ogImage({
    eyebrow: "Вакансия",
    title: v.title,
    lead: place,
    chips: [`${SPEC_LONG_LABEL[v.specialization]} · ${GRADE_LABEL[v.grade]}`, WORK_FORMAT_LABEL[v.work_format]],
    figure: formatSalaryRange(v.salary.min, v.salary.max),
  });
}
