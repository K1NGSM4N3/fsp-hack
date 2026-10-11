import { OG_CONTENT_TYPE, OG_SIZE, ogImage } from "@/lib/og";

export const alt = "Участникам ФСП: результаты с соревнований видят компании";
export const size = OG_SIZE;
export const contentType = OG_CONTENT_TYPE;

export default function Image() {
  return ogImage({
    eyebrow: "Участникам ФСП",
    title: "Результаты с соревнований теперь видят компании",
    accent: "компании",
    lead: "Привяжите ФСП ID, и разряды и призовые места из реестра Федерации появятся в профиле со ссылками на протоколы.",
    chips: ["ФСП ID", "Разряды и призовые места", "Протоколы"],
  });
}
