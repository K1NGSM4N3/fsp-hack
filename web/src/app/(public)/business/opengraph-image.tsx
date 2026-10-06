import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";

export const alt = "Компаниям: специалисты с проверенным уровнем";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default function Image() {
  return ogImage({
    eyebrow: "Компаниям",
    title: "Опишите задачу один раз и получите специалистов с проверенным уровнем",
    accent: "с проверенным уровнем",
    lead: "Вставьте текст вакансии: сервис выделит роль, грейд, навыки и вилку и соберёт подборку из тех, кто сдал тест.",
    chips: ["Грейд подтверждён тестом", "Вилка в первом сообщении", "Отказ с причиной"],
  });
}
