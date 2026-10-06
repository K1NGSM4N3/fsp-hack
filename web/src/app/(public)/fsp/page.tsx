import type { Metadata } from "next";
import { DisciplineTrack } from "@/components/site/fsp/DisciplineTrack";
import { FspHero } from "@/components/site/fsp/FspHero";
import { LinkDemo } from "@/components/site/fsp/LinkDemo";
import { RankLadder } from "@/components/site/fsp/RankLadder";
import { SeasonTimeline } from "@/components/site/fsp/SeasonTimeline";
import { StartLine } from "@/components/site/fsp/StartLine";
import { JsonLd } from "@/components/seo/JsonLd";
import { breadcrumbs } from "@/lib/jsonld";
import { PUBLIC_PAGES, pageMetadata } from "@/lib/site";

export const metadata: Metadata = pageMetadata(PUBLIC_PAGES.fsp);

export default function FspPage() {
  return (
    <>
      <JsonLd data={breadcrumbs([PUBLIC_PAGES.fsp])} />
      <FspHero />
      <LinkDemo />
      <RankLadder />
      <DisciplineTrack />
      <SeasonTimeline />
      <StartLine />
    </>
  );
}
