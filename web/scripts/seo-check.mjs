// Crawler's-eye check of the public site against a running server: no JavaScript, GPTBot user agent.
// Usage: node scripts/seo-check.mjs [base-url]   (default http://localhost:8080)
const BASE = (process.argv[2] ?? process.env.BASE ?? "http://localhost:8080").replace(/\/$/, "");
const UA = "Mozilla/5.0 AppleWebKit/537.36 (KHTML, like Gecko; compatible; GPTBot/1.2; +https://openai.com/gptbot)";
const PAGES = ["/", "/seekers", "/business", "/fsp", "/vacancies", "/privacy"];
const PRIVATE = ["/candidate", "/employer", "/admin", "/guardian", "/api/", "/login", "/register"];
const AI_BOTS = ["GPTBot", "OAI-SearchBot", "ChatGPT-User", "ClaudeBot", "Claude-Web", "PerplexityBot", "Google-Extended", "YandexAdditional", "Googlebot", "YandexBot"];

let failures = 0;
const ok = (cond, msg) => {
  if (!cond) failures++;
  console.log(`${cond ? "ok  " : "FAIL"} ${msg}`);
};
const get = async (path) => {
  const res = await fetch(BASE + path, { headers: { "User-Agent": UA }, redirect: "manual" });
  return { status: res.status, type: res.headers.get("content-type") ?? "", text: await res.text() };
};
const ldBlocks = (html) => [...html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)].map((m) => JSON.parse(m[1]));
const nodes = (blocks) => blocks.flatMap((b) => (b["@graph"] ? b["@graph"] : [b]));
const isAbs = (u) => typeof u === "string" && /^https?:\/\//.test(u);
const visibleText = (html) => html.replace(/<script[\s\S]*?<\/script>/g, " ").replace(/<style[\s\S]*?<\/style>/g, " ").replace(/<[^>]+>/g, " ").replace(/\s+/g, " ");

function checkBreadcrumbs(list, path) {
  const items = list?.itemListElement ?? [];
  ok(items.length >= 2 && items.every((it, i) => it.position === i + 1 && isAbs(it.item) && it.name), `${path}: BreadcrumbList positions, names, absolute items`);
  ok(new URL(items.at(-1)?.item ?? "http://x/").pathname === path, `${path}: last crumb is the page itself`);
}

function checkJobPosting(j, path) {
  ok(j && j.title && j.description && j.hiringOrganization?.name, `${path}: JobPosting title, description, hiringOrganization`);
  ok(!j?.datePosted || !Number.isNaN(Date.parse(j.datePosted)), `${path}: JobPosting datePosted is a date`);
  const v = j?.baseSalary?.value;
  ok(j?.baseSalary?.["@type"] === "MonetaryAmount" && j.baseSalary.currency === "RUB" && v?.unitText === "MONTH" && Number.isFinite(v?.minValue) && v.minValue <= v.maxValue, `${path}: JobPosting baseSalary RUB/MONTH min ≤ max`);
  ok(j?.jobLocationType === "TELECOMMUTE" || j?.jobLocation?.address?.addressLocality, `${path}: JobPosting jobLocation or TELECOMMUTE`);
  ok(j?.directApply === true && isAbs(j?.url), `${path}: JobPosting directApply and absolute url`);
}

const robots = await get("/robots.txt");
ok(robots.status === 200 && robots.type.startsWith("text/plain"), "robots.txt served as text/plain");
for (const bot of AI_BOTS) ok(new RegExp(`^User-agent: ${bot}$`, "m").test(robots.text), `robots.txt names ${bot}`);
for (const p of PRIVATE) ok(robots.text.includes(`Disallow: ${p}`), `robots.txt disallows ${p}`);
ok(/^Sitemap: https?:\/\/\S+\/sitemap\.xml$/m.test(robots.text), "robots.txt references the sitemap");

const sitemap = await get("/sitemap.xml");
const locs = [...sitemap.text.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => new URL(m[1]).pathname);
for (const p of PAGES) ok(locs.includes(p), `sitemap lists ${p}`);
ok(!locs.some((l) => PRIVATE.some((p) => l.startsWith(p.replace(/\/$/, "")))), "sitemap has no private paths");
const vacancy = locs.find((l) => l.startsWith("/vacancies/"));

const llms = await get("/llms.txt");
ok(llms.status === 200 && /^# .+\n\n> .+/.test(llms.text), "llms.txt: title and summary blockquote");
for (const p of PAGES) ok(llms.text.includes(`](${new URL(p, sitemap.text.match(/<loc>([^<]+)<\/loc>/)?.[1] ?? BASE).href})`), `llms.txt links ${p}`);
const full = await get("/llms-full.txt");
ok(full.status === 200 && !full.text.includes("временно недоступен"), "llms-full.txt has every page");
ok(full.text.length > 20000 && /## Соискателям — /.test(full.text) && /8–20 заданий/.test(full.text), "llms-full.txt carries page text and product facts");

for (const path of [...PAGES, ...(vacancy ? [vacancy] : [])]) {
  const page = await get(path);
  ok(page.status === 200, `${path}: 200 for a crawler`);
  const html = page.text;
  ok(/<html lang="ru"/.test(html), `${path}: lang="ru"`);
  ok((html.match(/<h1[\s>]/g) ?? []).length === 1, `${path}: exactly one h1`);
  ok(/<main[\s>]/.test(html), `${path}: <main> landmark`);
  ok(/<link rel="canonical" href="https?:\/\//.test(html), `${path}: absolute canonical link`);
  const all = nodes(ldBlocks(html));
  const byType = (t) => all.filter((n) => n["@type"] === t);
  ok(byType("Organization").some((o) => o.name && isAbs(o.url) && isAbs(o.logo)), `${path}: Organization`);
  ok(byType("WebSite").some((w) => w.name && isAbs(w.url)), `${path}: WebSite`);
  if (path !== "/") checkBreadcrumbs(byType("BreadcrumbList")[0], path);
  const text = visibleText(html);
  if (path === "/vacancies") {
    const list = byType("ItemList")[0];
    ok(list && list.numberOfItems === list.itemListElement.length && list.itemListElement.every((it, i) => it.position === i + 1 && isAbs(it.url)), "/vacancies: ItemList matches its items");
    ok(list?.itemListElement.every((it) => text.includes(it.name.replace(/&/g, "&amp;")) || text.includes(it.name)), "/vacancies: listed vacancies are in the server HTML");
  }
  if (path.startsWith("/vacancies/")) {
    const job = byType("JobPosting")[0];
    checkJobPosting(job, path);
    ok(job && text.includes(job.title) && text.includes(job.hiringOrganization.name), `${path}: title and company are in the server HTML`);
  }
}

console.log(failures ? `\n${failures} check(s) failed` : "\nall checks passed");
process.exit(failures ? 1 : 0);
