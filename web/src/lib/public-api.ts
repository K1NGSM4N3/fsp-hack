import type { Page, PublicVacancy } from "@/api/types";
import type { Seed } from "@/features/vacancies/hooks";

const API = process.env.API_PROXY_URL ?? "http://localhost:8000";
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

type Result<T> = { ok: true; data: T } | { ok: false; missing: boolean };

async function getJson<T>(path: string, revalidate: number): Promise<Result<T>> {
  try {
    const res = await fetch(`${API}/api/v1${path}`, { next: { revalidate }, signal: AbortSignal.timeout(3000) });
    if (!res.ok) {
      await res.body?.cancel();
      return { ok: false, missing: res.status === 404 };
    }
    return { ok: true, data: (await res.json()) as T };
  } catch {
    return { ok: false, missing: false };
  }
}

/**
 * A published vacancy for server rendering, metadata and link previews. `missing` is a definite 404 from the API;
 * when the API is merely unreachable both fields are empty and the page falls back to fetching in the browser.
 */
export async function fetchPublicVacancy(id: string): Promise<{ vacancy: Seed<PublicVacancy> | null; missing: boolean }> {
  if (!UUID.test(id)) return { vacancy: null, missing: true };
  const r = await getJson<PublicVacancy>(`/vacancies/${id}`, 300);
  return r.ok ? { vacancy: { data: r.data, at: Date.now() }, missing: false } : { vacancy: null, missing: r.missing };
}

/** First page of the public list with no filters, as the browser on `/vacancies` requests it. */
export async function fetchVacancyPage(limit: number): Promise<Seed<Page<PublicVacancy>> | null> {
  const r = await getJson<Page<PublicVacancy>>(`/vacancies?limit=${limit}&offset=0`, 60);
  return r.ok ? { data: r.data, at: Date.now() } : null;
}
