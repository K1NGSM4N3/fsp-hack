import { test, expect, type APIRequestContext } from "@playwright/test";
import { BASE } from "../lib/env";

/** Public pages as a crawler sees them: server HTML without JavaScript (lib/site.ts PUBLIC_PAGES). */
const PAGES = ["/", "/seekers", "/business", "/fsp", "/vacancies", "/privacy"];
const AUTH_PAGES = ["/login", "/register"];
const PRIVATE = ["/candidate", "/employer", "/admin", "/guardian", "/api/", "/login", "/register"];
const AI_BOTS = ["GPTBot", "OAI-SearchBot", "ChatGPT-User", "ClaudeBot", "Claude-Web", "PerplexityBot", "Google-Extended", "YandexAdditional", "Googlebot", "YandexBot"];
const GPTBOT = "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; GPTBot/1.2; +https://openai.com/gptbot)";

type Node = Record<string, unknown> & { "@type"?: string; "@graph"?: Node[] };

async function crawl(request: APIRequestContext, path: string) {
  const res = await request.get(path, { headers: { "User-Agent": GPTBOT }, maxRedirects: 0 });
  return { status: res.status(), type: res.headers()["content-type"] ?? "", text: await res.text() };
}

function jsonLd(html: string): Node[] {
  const blocks = [...html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)].map((m) => JSON.parse(m[1] ?? "null") as Node);
  return blocks.flatMap((b) => b["@graph"] ?? [b]);
}

const isAbsolute = (u: unknown) => typeof u === "string" && /^https?:\/\//.test(u);
const visibleText = (html: string) =>
  html
    .replace(/<script[\s\S]*?<\/script>/g, " ")
    .replace(/<style[\s\S]*?<\/style>/g, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/\s+/g, " ");

async function firstVacancyPath(request: APIRequestContext): Promise<string> {
  const res = await request.get("/api/v1/vacancies?limit=1");
  expect(res.status()).toBe(200);
  const body = (await res.json()) as { items: { id: string }[] };
  const id = body.items[0]?.id;
  expect(id, "the seed publishes vacancies").toBeTruthy();
  return `/vacancies/${id}`;
}

test.describe("публичный сайт для поисковиков и ИИ-ассистентов", () => {
  test("сервер отдаёт каждую публичную страницу целиком: 200, lang=ru, один h1, main, канонический адрес", async ({ request }) => {
    for (const path of [...PAGES, await firstVacancyPath(request)]) {
      const page = await crawl(request, path);
      expect(page.status, path).toBe(200);
      expect(page.text, path).toMatch(/<html lang="ru"/);
      expect(page.text.match(/<h1[\s>]/g) ?? [], `${path}: exactly one h1`).toHaveLength(1);
      expect(page.text, path).toMatch(/<main[\s>]/);
      expect(page.text, path).toMatch(/<link rel="canonical" href="https?:\/\//);
    }
  });

  test("в браузере у каждой публичной страницы и страниц входа ровно один h1", async ({ page, request }) => {
    for (const path of [...PAGES, ...AUTH_PAGES, await firstVacancyPath(request)]) {
      const res = await page.goto(path);
      expect(res?.status(), path).toBe(200);
      await expect(page.locator("h1"), path).toHaveCount(1);
    }
  });

  test("robots.txt называет ИИ-ботов, закрывает кабинеты и ссылается на sitemap", async ({ request }) => {
    const robots = await crawl(request, "/robots.txt");
    expect(robots.status).toBe(200);
    expect(robots.type).toContain("text/plain");
    for (const bot of AI_BOTS) expect(robots.text).toMatch(new RegExp(`^User-agent: ${bot}$`, "m"));
    for (const p of PRIVATE) expect(robots.text).toContain(`Disallow: ${p}`);
    expect(robots.text).toMatch(/^Sitemap: https?:\/\/\S+\/sitemap\.xml$/m);
  });

  test("sitemap.xml перечисляет публичные страницы и вакансии, но не кабинеты", async ({ request }) => {
    const sitemap = await crawl(request, "/sitemap.xml");
    expect(sitemap.status).toBe(200);
    const locs = [...sitemap.text.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => new URL(m[1] ?? "").pathname);
    for (const p of PAGES) expect(locs, p).toContain(p);
    expect(locs.some((l) => l.startsWith("/vacancies/"))).toBe(true);
    expect(locs.filter((l) => PRIVATE.some((p) => l.startsWith(p.replace(/\/$/, ""))))).toEqual([]);
  });

  test("llms.txt и llms-full.txt: краткая карта и полный текст сайта", async ({ request }) => {
    const llms = await crawl(request, "/llms.txt");
    expect(llms.status).toBe(200);
    expect(llms.text).toMatch(/^# .+\n\n> .+/);
    const site = new URL((await crawl(request, "/sitemap.xml")).text.match(/<loc>([^<]+)<\/loc>/)?.[1] ?? BASE).origin;
    for (const p of PAGES) expect(llms.text, p).toContain(`](${new URL(p, site).href})`);
    const full = await crawl(request, "/llms-full.txt");
    expect(full.status).toBe(200);
    expect(full.text).not.toContain("временно недоступен");
    expect(full.text.length).toBeGreaterThan(20_000);
    expect(full.text).toMatch(/## Соискателям — /);
    expect(full.text).toMatch(/8–20 заданий/);
  });

  test("JSON-LD: Organization и WebSite везде, хлебные крошки, ItemList вакансий и JobPosting", async ({ request }) => {
    const vacancy = await firstVacancyPath(request);
    for (const path of [...PAGES, vacancy]) {
      const html = (await crawl(request, path)).text;
      const nodes = jsonLd(html);
      const ofType = (t: string) => nodes.filter((n) => n["@type"] === t);
      expect(ofType("Organization").some((o) => o.name && isAbsolute(o.url) && isAbsolute(o.logo)), `${path}: Organization`).toBe(true);
      expect(ofType("WebSite").some((w) => w.name && isAbsolute(w.url)), `${path}: WebSite`).toBe(true);
      if (path !== "/") {
        const items = (ofType("BreadcrumbList")[0]?.itemListElement ?? []) as { position: number; item: string; name: string }[];
        expect(items.length, `${path}: breadcrumbs`).toBeGreaterThanOrEqual(2);
        items.forEach((it, i) => {
          expect(it.position).toBe(i + 1);
          expect(isAbsolute(it.item)).toBe(true);
          expect(it.name).toBeTruthy();
        });
        expect(new URL(items.at(-1)?.item ?? BASE).pathname, `${path}: last crumb`).toBe(path);
      }
      const text = visibleText(html);
      if (path === "/vacancies") {
        const list = ofType("ItemList")[0] as { numberOfItems: number; itemListElement: { position: number; url: string; name: string }[] } | undefined;
        expect(list, "/vacancies: ItemList").toBeTruthy();
        expect(list?.numberOfItems).toBe(list?.itemListElement.length);
        for (const [i, it] of (list?.itemListElement ?? []).entries()) {
          expect(it.position).toBe(i + 1);
          expect(isAbsolute(it.url)).toBe(true);
          expect(text).toContain(it.name);
        }
      }
      if (path === vacancy) {
        const job = ofType("JobPosting")[0] as
          | {
              title: string;
              description: string;
              datePosted?: string;
              hiringOrganization: { name: string };
              baseSalary: { "@type": string; currency: string; value: { unitText: string; minValue: number; maxValue: number } };
              jobLocationType?: string;
              jobLocation?: { address?: { addressLocality?: string } };
              directApply: boolean;
              url: string;
            }
          | undefined;
        expect(job, "JobPosting").toBeTruthy();
        if (!job) continue;
        expect(job.title && job.description && job.hiringOrganization.name).toBeTruthy();
        if (job.datePosted) expect(Number.isNaN(Date.parse(job.datePosted))).toBe(false);
        expect(job.baseSalary["@type"]).toBe("MonetaryAmount");
        expect(job.baseSalary.currency).toBe("RUB");
        expect(job.baseSalary.value.unitText).toBe("MONTH");
        expect(job.baseSalary.value.minValue).toBeLessThanOrEqual(job.baseSalary.value.maxValue);
        expect(job.jobLocationType === "TELECOMMUTE" || Boolean(job.jobLocation?.address?.addressLocality)).toBe(true);
        expect(job.directApply).toBe(true);
        expect(isAbsolute(job.url)).toBe(true);
        expect(text).toContain(job.title);
        expect(text).toContain(job.hiringOrganization.name);
      }
    }
  });
});
