export const MOTION_OK = "(prefers-reduced-motion: no-preference)";

/** Full choreography: pinned and scrubbed scenes, line splits, smooth scroll, pointer effects. */
export const MOTION_DESKTOP = "(prefers-reduced-motion: no-preference) and (pointer: fine) and (min-width: 1024px)";

/**
 * Exact complement of MOTION_DESKTOP among motion-allowed devices: one-shot reveals, transforms and opacity only,
 * nothing pinned or scrubbed. Written as a comma list (OR) instead of Media Queries 4 `or` for older iOS Safari.
 */
export const MOTION_MOBILE = [
  "(prefers-reduced-motion: no-preference) and (pointer: coarse)",
  "(prefers-reduced-motion: no-preference) and (pointer: none)",
  "(prefers-reduced-motion: no-preference) and (max-width: 1023.98px)",
].join(", ");

export const FINE_POINTER = "(prefers-reduced-motion: no-preference) and (pointer: fine)";

export function matches(query: string): boolean {
  return typeof window !== "undefined" && window.matchMedia(query).matches;
}

const numberFormats = new Map<number, Intl.NumberFormat>();

export function formatCounter(value: number, decimals: number): string {
  let f = numberFormats.get(decimals);
  if (!f) {
    f = new Intl.NumberFormat("ru-RU", { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
    numberFormats.set(decimals, f);
  }
  return f.format(value);
}
