import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { ImageResponse } from "next/og";

export const OG_SIZE = { width: 1200, height: 630 };
export const OG_CONTENT_TYPE = "image/png";

/**
 * Brand palette for link previews. ImageResponse renders outside the CSS pipeline, so these mirror the tokens in
 * `app/globals.css` (night, night-muted, primary, primary-300, accent) instead of reading them.
 */
const C = {
  night: "#0b0b12",
  raised: "#14141d",
  line: "#2a2a3a",
  muted: "#a5a6b8",
  faint: "#8e8fa6",
  primary: "#402fff",
  primary300: "#a9a3ff",
  accent: "#ec1d35",
  white: "#ffffff",
};

type Font = { name: string; data: Buffer; weight: 400 | 500 | 600; style: "normal" };

let assets: Promise<{ fonts: Font[]; logo: string }> | null = null;

/** Static TTFs: Satori cannot read the WOFF2 files that next/font serves, nor pick an instance of a variable font. */
function loadAssets() {
  assets ??= (async () => {
    const font = (file: string) => readFile(join(process.cwd(), "assets/og", file));
    const [regular, semibold, mono, logo] = await Promise.all([
      font("Onest-Regular.ttf"),
      font("Onest-SemiBold.ttf"),
      font("JetBrainsMono-Medium.ttf"),
      readFile(join(process.cwd(), "public/brand/fsp-logo-dark.svg"), "base64"),
    ]);
    return {
      fonts: [
        { name: "Onest", data: regular, weight: 400, style: "normal" },
        { name: "Onest", data: semibold, weight: 600, style: "normal" },
        { name: "JetBrains Mono", data: mono, weight: 500, style: "normal" },
      ],
      logo: `data:image/svg+xml;base64,${logo}`,
    };
  })();
  return assets;
}

/** Intl group separators are narrow no-break spaces, which the bundled fonts have no glyph for. */
export function ogText(s: string): string {
  return s.replace(/[\u202f\u00a0]/g, " ");
}

interface OgCardProps {
  eyebrow: string;
  title: string;
  /** Part of `title` set in the accent color, like `text-gradient-night` on the site. */
  accent?: string;
  lead?: string;
  chips?: string[];
  /** Monospace figure on the right of the footer, e.g. a salary range. */
  figure?: string;
}

function Headline({ title, accent }: { title: string; accent?: string }) {
  const at = accent ? title.indexOf(accent) : -1;
  const parts = at < 0 || !accent ? [{ text: title, on: false }] : [
    { text: title.slice(0, at), on: false },
    { text: accent, on: true },
    { text: title.slice(at + accent.length), on: false },
  ];
  const tokens = parts.flatMap((p) => p.text.split(/\s+/).filter(Boolean).map((w) => ({ w, on: p.on })));
  // Russian typesetting: a one- or two-letter word ("с", "и", "в") never ends a line, so it travels with the next one.
  const words = tokens.reduce<{ w: string; on: boolean }[]>((acc, t) => {
    const prev = acc.at(-1);
    if (prev && prev.on === t.on && prev.w.length <= 2 && !prev.w.includes(" ")) prev.w = `${prev.w} ${t.w}`;
    else acc.push({ ...t });
    return acc;
  }, []);
  const size = title.length > 70 ? 60 : title.length > 48 ? 68 : 78;
  return (
    <div style={{ display: "flex", flexWrap: "wrap", columnGap: size * 0.24, rowGap: 0, fontSize: size, lineHeight: 1.04, letterSpacing: "-0.04em", fontWeight: 600, maxWidth: 1000 }}>
      {words.map(({ w, on }, i) => (
        <span key={i} style={{ color: on ? C.primary300 : C.white, whiteSpace: "nowrap" }}>
          {w}
        </span>
      ))}
    </div>
  );
}

export async function ogImage({ eyebrow, title, accent, lead, chips = [], figure }: OgCardProps) {
  const { fonts, logo } = await loadAssets();
  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          padding: "64px 72px 60px",
          background: C.night,
          backgroundImage: `radial-gradient(circle at 92% -10%, rgba(64,47,255,0.62), rgba(64,47,255,0) 55%), radial-gradient(circle at -5% 115%, rgba(123,111,255,0.28), rgba(123,111,255,0) 45%)`,
          color: C.white,
          fontFamily: "Onest",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 22 }}>
            {/* eslint-disable-next-line @next/next/no-img-element -- Satori renders plain <img> only */}
            <img src={logo} width={155} height={52} alt="" />
            <div style={{ width: 2, height: 34, background: C.line }} />
            <div style={{ fontFamily: "JetBrains Mono", fontSize: 28, color: C.white, letterSpacing: "-0.01em" }}>Таланты</div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 12, fontFamily: "JetBrains Mono", fontSize: 20, color: C.primary300, letterSpacing: "0.14em", textTransform: "uppercase" }}>
            <div style={{ width: 10, height: 10, borderRadius: 10, background: C.accent }} />
            {eyebrow}
          </div>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 26 }}>
          <Headline title={ogText(title)} accent={accent ? ogText(accent) : undefined} />
          {lead && <div style={{ display: "flex", fontSize: 28, lineHeight: 1.4, color: C.muted, maxWidth: 940 }}>{ogText(lead)}</div>}
        </div>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 24, borderTop: `1px solid ${C.line}`, paddingTop: 26 }}>
          <div style={{ display: "flex", gap: 12, flexWrap: "wrap" }}>
            {chips.map((c) => (
              <div key={c} style={{ display: "flex", alignItems: "center", gap: 10, height: 46, padding: "0 20px", borderRadius: 999, border: "1px solid rgba(255,255,255,0.16)", background: "rgba(255,255,255,0.05)", fontSize: 22, color: C.white }}>
                <div style={{ width: 8, height: 8, borderRadius: 8, background: C.primary300 }} />
                {ogText(c)}
              </div>
            ))}
          </div>
          {figure ? (
            <div style={{ display: "flex", flexShrink: 0, fontFamily: "JetBrains Mono", fontSize: 34, color: C.white, letterSpacing: "-0.02em" }}>{ogText(figure)}</div>
          ) : (
            <div style={{ display: "flex", flexShrink: 0, fontFamily: "JetBrains Mono", fontSize: 20, color: C.faint, letterSpacing: "0.08em", textTransform: "uppercase" }}>ФСП России</div>
          )}
        </div>
      </div>
    ),
    { ...OG_SIZE, fonts },
  );
}
