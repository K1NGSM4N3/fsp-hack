import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";

export const alt = "Открытые ИТ-вакансии с вилкой от и до";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default function Image() {
  return ogImage({
    eyebrow: "Вакансии",
    title: "Открытые ИТ-вакансии с вилкой от и до",
    accent: "с вилкой от и до",
    lead: "Бэкенд, фронтенд, DevOps, QA, ML и безопасность. В каждой карточке зарплата, формат работы и стек.",
    chips: ["Junior", "Middle", "Senior"],
  });
}
