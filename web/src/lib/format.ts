const rubFormatter = new Intl.NumberFormat("ru-RU", { maximumFractionDigits: 0 });

export function formatRub(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—";
  return `${rubFormatter.format(value)} ₽`;
}

export function formatSalaryRange(min: number, max: number): string {
  if (min === max) return formatRub(min);
  return `${rubFormatter.format(min)} – ${rubFormatter.format(max)} ₽`;
}

export function formatRubShort(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—";
  if (value >= 1000) return `${rubFormatter.format(Math.round(value / 1000))} тыс. ₽`;
  return formatRub(value);
}

export function formatSalaryRangeShort(min: number, max: number): string {
  const k = (v: number) => rubFormatter.format(Math.round(v / 1000));
  return `${k(min)}–${k(max)} тыс. ₽`;
}

export function formatNumber(value: number): string {
  return rubFormatter.format(value);
}

export function formatPercent(value01: number, digits = 0): string {
  return `${(value01 * 100).toFixed(digits)} %`;
}

const dateFormatter = new Intl.DateTimeFormat("ru-RU", { day: "numeric", month: "long", year: "numeric" });
const dateShortFormatter = new Intl.DateTimeFormat("ru-RU", { day: "numeric", month: "short" });
const dateTimeFormatter = new Intl.DateTimeFormat("ru-RU", {
  day: "numeric",
  month: "short",
  hour: "2-digit",
  minute: "2-digit",
});

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  return dateFormatter.format(new Date(iso)).replace(/\s?г\.$/, "");
}

export function formatDateShort(iso: string | null | undefined): string {
  if (!iso) return "—";
  return dateShortFormatter.format(new Date(iso));
}

export function formatDateTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  return dateTimeFormatter.format(new Date(iso));
}

export function plural(n: number, forms: [string, string, string]): string {
  const abs = Math.abs(n) % 100;
  const last = abs % 10;
  if (abs > 10 && abs < 20) return forms[2];
  if (last > 1 && last < 5) return forms[1];
  if (last === 1) return forms[0];
  return forms[2];
}

export function pluralize(n: number, forms: [string, string, string]): string {
  return `${formatNumber(n)} ${plural(n, forms)}`;
}

const DAY = 24 * 60 * 60 * 1000;

export function daysBetween(fromIso: string, to: Date = new Date()): number {
  return Math.round((to.getTime() - new Date(fromIso).getTime()) / DAY);
}

export function formatRelative(iso: string | null | undefined, now: Date = new Date()): string {
  if (!iso) return "—";
  const diff = Math.round((new Date(iso).getTime() - now.getTime()) / DAY);
  if (diff === 0) return "сегодня";
  if (diff === -1) return "вчера";
  if (diff === 1) return "завтра";
  if (diff < 0) {
    const d = -diff;
    if (d > 60) return formatDate(iso);
    return `${pluralize(d, ["день", "дня", "дней"])} назад`;
  }
  return `через ${pluralize(diff, ["день", "дня", "дней"])}`;
}

export function formatDuration(seconds: number): string {
  const s = Math.max(0, Math.round(seconds));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${m}:${String(r).padStart(2, "0")}`;
}

export function initials(name: string): string {
  return name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((p) => p[0]?.toUpperCase() ?? "")
    .join("");
}
