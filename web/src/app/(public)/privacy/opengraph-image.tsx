import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";

export const alt = "Политика обработки персональных данных ФСП Таланты";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default function Image() {
  return ogImage({
    eyebrow: "Документы",
    title: "Политика обработки персональных данных",
    lead: "Какие данные хранит платформа, зачем, кому их видно, сколько мы их храним и как распоряжаться ими из кабинета.",
    chips: ["152-ФЗ", "Согласия", "Выгрузка данных"],
  });
}
