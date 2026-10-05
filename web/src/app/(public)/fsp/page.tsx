import type { Metadata } from "next";
import { DisciplineTrack } from "@/components/site/fsp/DisciplineTrack";
import { FspHero } from "@/components/site/fsp/FspHero";
import { LinkDemo } from "@/components/site/fsp/LinkDemo";
import { RankLadder } from "@/components/site/fsp/RankLadder";
import { SeasonTimeline } from "@/components/site/fsp/SeasonTimeline";
import { StartLine } from "@/components/site/fsp/StartLine";

export const metadata: Metadata = {
  title: { absolute: "Участникам ФСП · ФСП Таланты" },
  description: "Привяжите ФСП ID, и разряды и призовые места из реестра Федерации появятся в профиле кандидата со ссылками на протоколы.",
};

export default function FspPage() {
  return (
    <>
      <FspHero />
      <LinkDemo />
      <RankLadder />
      <DisciplineTrack />
      <SeasonTimeline />
      <StartLine />
    </>
  );
}
