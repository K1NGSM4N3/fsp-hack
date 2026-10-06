import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";

export const alt = "ФСП Таланты: компании пишут первыми и сразу называют зарплату";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default function Image() {
  return ogImage({
    eyebrow: "Подбор ИТ-специалистов",
    title: "Компании пишут вам первыми и сразу называют зарплату",
    accent: "и сразу называют зарплату",
    lead: "Один раз пройдите тест. Компании найдут вас по категории и пришлют приглашение с вилкой от и до.",
    chips: ["Бэкенд · Middle", "Фронтенд · Senior"],
    figure: "280 000 – 350 000 ₽",
  });
}
