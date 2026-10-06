import type { MetadataRoute } from "next";
import { fetchVacancyPage } from "@/lib/public-api";
import { PUBLIC_PAGES, absoluteUrl } from "@/lib/site";

/** Rendered per request: published vacancies come from the API, which is not reachable while the image is built. */
export const dynamic = "force-dynamic";

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const vacancies = (await fetchVacancyPage(100))?.data.items ?? [];
  return [
    ...Object.values(PUBLIC_PAGES).map((p) => ({ url: absoluteUrl(p.path), changeFrequency: p.changeFrequency, priority: p.priority })),
    ...vacancies.map((v) => ({
      url: absoluteUrl(`/vacancies/${v.id}`),
      lastModified: v.published_at ?? undefined,
      changeFrequency: "weekly" as const,
      priority: 0.6,
    })),
  ];
}
