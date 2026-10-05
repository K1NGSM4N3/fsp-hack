/**
 * Auth is cookie-only (httpOnly `fsp_at`/`fsp_rt` set by the API), so every request
 * is same-origin with `credentials: "include"`. The access token lives 15 minutes:
 * a 401 triggers one refresh and a retry before giving up.
 * When the generated client lands, `openapi-fetch` can replace `http()`; keep the
 * refresh step and the problem+json → ApiError mapping.
 */

export const API_BASE = "/api/v1";

/** Inlined at build time, so switching requires a rebuild. */
export const USE_MOCKS = process.env.NEXT_PUBLIC_USE_MOCKS === "1";

export type FieldErrors = Record<string, string[]>;

export interface ProblemDetails {
  type?: string;
  title?: string;
  status?: number;
  code?: string;
  detail?: string;
  errors?: FieldErrors;
}

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly title: string;
  readonly detail: string | null;
  readonly errors: FieldErrors | null;

  constructor(init: { status: number; code: string; title: string; detail?: string | null; errors?: FieldErrors | null }) {
    super(init.detail ? `${init.title}: ${init.detail}` : init.title);
    this.name = "ApiError";
    this.status = init.status;
    this.code = init.code;
    this.title = init.title;
    this.detail = init.detail ?? null;
    this.errors = init.errors ?? null;
  }

  fieldError(field: string): string | undefined {
    return this.errors?.[field]?.[0];
  }
}

export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

const FALLBACK_TITLES: Record<number, string> = {
  0: "Нет связи с сервером",
  400: "Некорректный запрос",
  401: "Нужно войти в аккаунт",
  403: "Недостаточно прав",
  404: "Не найдено",
  409: "Конфликт состояния",
  413: "Файл слишком большой",
  422: "Проверьте введённые данные",
  429: "Слишком много запросов, попробуйте позже",
  500: "Ошибка на сервере",
  502: "Сервер недоступен",
  503: "Сервис временно недоступен",
};

function fallbackTitle(status: number): string {
  return FALLBACK_TITLES[status] ?? (status >= 500 ? FALLBACK_TITLES[500] : "Что-то пошло не так");
}

async function toApiError(res: Response): Promise<ApiError> {
  let body: ProblemDetails = {};
  const type = res.headers.get("content-type") ?? "";
  if (type.includes("json")) {
    try {
      body = (await res.json()) as ProblemDetails;
    } catch {
      body = {};
    }
  }
  if (res.status === 429) {
    const retry = res.headers.get("retry-after");
    if (retry && !body.detail) body.detail = `Повторите через ${retry} с`;
  }
  return new ApiError({
    status: res.status,
    code: body.code ?? `http_${res.status}`,
    title: body.title ?? fallbackTitle(res.status),
    detail: body.detail ?? null,
    errors: body.errors ?? null,
  });
}

type QueryValue = string | number | boolean | null | undefined | ReadonlyArray<string | number>;

export function toQueryString(query?: Record<string, QueryValue>): string {
  if (!query) return "";
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null || value === "") continue;
    if (Array.isArray(value)) {
      // axum `Query` cannot decode repeated keys into a Vec, hence comma-separated.
      if (value.length) params.set(key, value.join(","));
    } else {
      params.set(key, String(value));
    }
  }
  const qs = params.toString();
  return qs ? `?${qs}` : "";
}

let refreshInFlight: Promise<boolean> | null = null;

/** Single-flight refresh: concurrent 401s share one `POST /auth/refresh`. */
function refreshSession(): Promise<boolean> {
  refreshInFlight ??= fetch(`${API_BASE}/auth/refresh`, {
    method: "POST",
    credentials: "include",
    headers: { Accept: "application/json" },
  })
    .then((res) => res.ok)
    .catch(() => false)
    .finally(() => {
      refreshInFlight = null;
    });
  return refreshInFlight;
}

function isCabinetPath(pathname: string): boolean {
  return pathname.startsWith("/candidate") || pathname.startsWith("/employer");
}

function redirectToLogin() {
  if (typeof window === "undefined") return;
  const { pathname, search } = window.location;
  if (!isCabinetPath(pathname)) return;
  const next = encodeURIComponent(pathname + search);
  // eslint-disable-next-line @next/next/no-location-assign-relative-destination -- runs outside React; a full reload also drops cached private data
  window.location.href = `/login?next=${next}`;
}

export interface HttpOptions {
  query?: Record<string, QueryValue>;
  body?: unknown;
  headers?: Record<string, string>;
  blob?: boolean;
  noRefresh?: boolean;
}

export async function http<T>(method: string, path: string, options: HttpOptions = {}): Promise<T> {
  const url = `${API_BASE}${path}${toQueryString(options.query)}`;
  const headers: Record<string, string> = { Accept: "application/json", ...options.headers };
  let body: BodyInit | undefined;
  if (options.body instanceof FormData) {
    body = options.body;
  } else if (options.body !== undefined) {
    headers["Content-Type"] = "application/json";
    body = JSON.stringify(options.body);
  }
  const init: RequestInit = { method, headers, body, credentials: "include" };

  const send = async () => {
    try {
      return await fetch(url, init);
    } catch {
      throw new ApiError({ status: 0, code: "network", title: fallbackTitle(0), detail: "Проверьте подключение к интернету" });
    }
  };

  let res = await send();
  const canRefresh = !options.noRefresh && !path.startsWith("/auth/");
  if (res.status === 401 && canRefresh) {
    if (await refreshSession()) {
      res = await send();
    }
    if (res.status === 401) redirectToLogin();
  }

  if (!res.ok) throw await toApiError(res);
  if (options.blob) return (await res.blob()) as T;
  if (res.status === 204 || res.headers.get("content-length") === "0") return undefined as T;
  const type = res.headers.get("content-type") ?? "";
  if (!type.includes("json")) return undefined as T;
  return (await res.json()) as T;
}
