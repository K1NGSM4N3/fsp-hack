import { mainText } from "@/lib/html-text";
import { llmsFullTxt } from "@/lib/llms";
import { fetchVacancyPage } from "@/lib/public-api";
import { PUBLIC_PAGES } from "@/lib/site";

/**
 * Built on request from the pages this server renders, so the text always matches the site; kept in memory for ten
 * minutes. The pages are fetched from this process on the loopback interface, not through the public origin.
 */
export const dynamic = "force-dynamic";

const TTL = 10 * 60_000;
const SELF = `http://127.0.0.1:${process.env.PORT ?? 3000}`;

let cached: { at: number; body: string } | null = null;

async function pageText(path: string): Promise<string | null> {
  try {
    const res = await fetch(SELF + path, { cache: "no-store", headers: { Accept: "text/html" }, signal: AbortSignal.timeout(8000) });
    if (!res.ok) {
      await res.body?.cancel();
      return null;
    }
    return mainText(await res.text());
  } catch {
    return null;
  }
}

export async function GET() {
  if (!cached || Date.now() - cached.at > TTL) {
    const pages = Object.values(PUBLIC_PAGES);
    const [texts, list] = await Promise.all([Promise.all(pages.map((p) => pageText(p.path))), fetchVacancyPage(100)]);
    const body = llmsFullTxt(
      pages.map((page, i) => ({ page, text: texts[i] ?? null })),
      list?.data.items ?? [],
      new Date(),
    );
    if (texts.every(Boolean)) cached = { at: Date.now(), body };
    else return new Response(body, { headers: { "Content-Type": "text/markdown; charset=utf-8", "Cache-Control": "no-store" } });
  }
  return new Response(cached.body, { headers: { "Content-Type": "text/markdown; charset=utf-8", "Cache-Control": "public, max-age=600" } });
}
