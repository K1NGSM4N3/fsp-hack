/**
 * Readable text of a server-rendered page for `llms-full.txt`: the content of `<main>` as light Markdown. Skips what a
 * person never reads as text — scripts, SVG, buttons, form controls and anything aria-hidden, hidden or inert (the
 * decorative mockups) — and keeps headings, list items and term/definition pairs on their own lines. Blocks with the
 * `hidden` utility are skipped too: they are the desktop-only twins of sections that also render in a plain layout.
 */

const VOID = new Set(["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"]);
const SKIP = new Set(["script", "style", "svg", "template", "noscript", "button", "select", "textarea", "video", "audio", "canvas", "iframe", "dialog"]);
/** Inline elements that sit side by side as separate pieces (buttons in a row, a number before a title). */
const SPACED = new Set(["a", "span", "label", "time", "strong", "b", "em", "small", "code", "mark", "abbr"]);
const BLOCK = new Set([
  "p", "div", "section", "article", "header", "footer", "nav", "aside", "ul", "ol", "dl", "blockquote", "figure", "figcaption",
  "table", "tr", "form", "fieldset", "legend", "details", "summary", "pre", "main",
]);

const TOKEN = /<!--[\s\S]*?-->|<(\/)?([a-zA-Z][\w-]*)((?:\s+[^\s"'>/=]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?)*)\s*(\/)?>|([^<]+)/g;
const ATTR = /([^\s"'>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;

const ENTITIES: Record<string, string> = { amp: "&", lt: "<", gt: ">", quot: '"', apos: "'", nbsp: " " };

function decode(s: string): string {
  return s.replace(/&(#x[0-9a-f]+|#\d+|[a-z]+);/gi, (m, e: string) => {
    if (e[0] === "#") {
      const code = e[1] === "x" || e[1] === "X" ? parseInt(e.slice(2), 16) : parseInt(e.slice(1), 10);
      return Number.isFinite(code) ? String.fromCodePoint(code) : m;
    }
    return ENTITIES[e.toLowerCase()] ?? m;
  });
}

function hiddenByAttrs(attrs: string): boolean {
  for (const m of attrs.matchAll(ATTR)) {
    const name = (m[1] ?? "").toLowerCase();
    const value = m[2] ?? m[3] ?? m[4] ?? "";
    if (name === "hidden" || name === "inert") return true;
    if (name === "aria-hidden" && value === "true") return true;
    if (name === "class" && value.split(/\s+/).includes("hidden")) return true;
  }
  return false;
}

export function mainText(html: string): string {
  const start = html.indexOf("<main");
  const end = html.lastIndexOf("</main>");
  const body = start >= 0 && end > start ? html.slice(start, end + 7) : html;
  const out: string[] = [];
  const stack: { tag: string; skip: boolean }[] = [];
  let skipping = 0;

  for (const m of body.matchAll(TOKEN)) {
    const [, closing, rawTag, attrs = "", selfClosing, text] = m;
    if (text !== undefined) {
      if (!skipping) out.push(decode(text));
      continue;
    }
    if (!rawTag) continue;
    const tag = rawTag.toLowerCase();
    if (closing) {
      const at = stack.map((s) => s.tag).lastIndexOf(tag);
      if (at < 0) continue;
      for (const s of stack.splice(at)) if (s.skip) skipping--;
      if (skipping) continue;
      if (/^h[1-6]$/.test(tag) || BLOCK.has(tag) || tag === "li" || tag === "dd") out.push("\n");
      else if (SPACED.has(tag)) out.push(" ");
      continue;
    }
    if (VOID.has(tag) || selfClosing) {
      if (!skipping && tag === "br") out.push("\n");
      continue;
    }
    const skip = SKIP.has(tag) || hiddenByAttrs(attrs);
    stack.push({ tag, skip });
    if (skip) {
      skipping++;
      continue;
    }
    if (skipping) continue;
    if (/^h[1-6]$/.test(tag)) out.push(`\n\n${"#".repeat(Math.min(6, Number(tag[1]) + 2))} `);
    else if (tag === "li") out.push("\n- ");
    else if (tag === "dt") out.push("\n");
    else if (tag === "dd") out.push(": ");
    else if (BLOCK.has(tag)) out.push("\n");
  }

  const lines = out
    .join("")
    .split("\n")
    .map((l) =>
      l
        .replace(/[ \t\u00a0\u202f]+/g, " ")
        .replace(/ ([,.;:!?»)])/g, "$1")
        .replace(/([«(]) /g, "$1")
        .trim(),
    )
    .filter((l) => l && l !== "-" && !/^[·•/|]+$/.test(l));
  const result: string[] = [];
  for (const l of lines) {
    if (l.startsWith("#") && result.length) result.push("");
    if (result.at(-1) === l) continue;
    result.push(l);
  }
  return result.join("\n");
}
