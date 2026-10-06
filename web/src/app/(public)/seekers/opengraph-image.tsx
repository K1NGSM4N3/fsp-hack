import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";

export const alt = "Соискателям: один тест, и приглашения приходят к вам сами";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default function Image() {
  return ogImage({
    eyebrow: "Соискателям",
    title: "Один тест, и приглашения приходят к вам сами",
    accent: "приходят к вам сами",
    lead: "Короткий профиль, тест на 10–25 минут и приглашения, где зарплата от и до стоит в первом сообщении.",
    chips: ["8–20 заданий", "Категория по итогам теста", "Контакты после «Принять»"],
  });
}
