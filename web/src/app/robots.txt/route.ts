import { absoluteUrl } from "@/lib/site";

export const dynamic = "force-static";

/** Search engines and AI assistants named explicitly: the same rules as everyone, stated so no crawler has to guess. */
const CRAWLERS = [
  "Googlebot",
  "Google-Extended",
  "Bingbot",
  "YandexBot",
  "YandexAdditional",
  "GPTBot",
  "OAI-SearchBot",
  "ChatGPT-User",
  "ClaudeBot",
  "Claude-Web",
  "Claude-SearchBot",
  "Claude-User",
  "anthropic-ai",
  "PerplexityBot",
  "Perplexity-User",
  "Applebot",
  "Applebot-Extended",
  "*",
];

/** Cabinets, auth, the guardian page, the API and the moderator cabinet: personal or session-only, never indexed. */
const PRIVATE = ["/candidate", "/employer", "/admin", "/guardian", "/login", "/register", "/verify-email", "/session", "/api/"];

/** Marketing tags never change what a page shows; Yandex folds such URLs into one. */
const CLEAN_PARAM = "utm_source&utm_medium&utm_campaign&utm_content&utm_term&yclid&gclid";

export function GET() {
  const lines = [
    ...CRAWLERS.map((ua) => `User-agent: ${ua}`),
    "Allow: /",
    ...PRIVATE.map((p) => `Disallow: ${p}`),
    `Clean-param: ${CLEAN_PARAM}`,
    "",
    `Sitemap: ${absoluteUrl("/sitemap.xml")}`,
    "",
  ];
  return new Response(lines.join("\n"), { headers: { "Content-Type": "text/plain; charset=utf-8" } });
}
