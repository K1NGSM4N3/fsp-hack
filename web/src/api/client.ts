/**
 * Auth is cookie-only (httpOnly `fsp_at`/`fsp_rt` set by the API), so every request
 * is same-origin with `credentials: "include"`. The access token lives 15 minutes:
 * when the `fsp_at_exp` hint says it has run out, the request refreshes first;
 * a 401 anyway triggers one refresh and a retry before giving up.
 */
import { refreshSession } from "./refresh";
import { accessExpired } from "./session-hint";

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
  /** `consent_required`: the current consent document to grant first. */
  document_id?: string;
  document_kind?: string;
}

export class ApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly title: string;
  readonly detail: string | null;
  readonly errors: FieldErrors | null;
  readonly documentId: string | null;
  readonly documentKind: string | null;

  constructor(init: {
    status: number;
    code: string;
    title: string;
    detail?: string | null;
    errors?: FieldErrors | null;
    documentId?: string | null;
    documentKind?: string | null;
  }) {
    super(init.detail ? `${init.title}: ${init.detail}` : init.title);
    this.name = "ApiError";
    this.status = init.status;
    this.code = init.code;
    this.title = init.title;
    this.detail = init.detail ?? null;
    this.errors = init.errors ?? null;
    this.documentId = init.documentId ?? null;
    this.documentKind = init.documentKind ?? null;
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
  } else {
    discard(res);
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
    documentId: body.document_id ?? null,
    documentKind: body.document_kind ?? null,
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

/**
 * Releases a response whose body we do not need. The API marks authenticated responses `no-store`, and Chrome keeps
 * such a response open until its body is read or cancelled; over HTTP/1.1 that pins one of six connections per host.
 */
export function discard(res: Response) {
  if (res.body && !res.bodyUsed) res.body.cancel().catch(() => {});
}

function isCabinetPath(pathname: string): boolean {
  return /^\/(candidate|employer|admin)(\/|$)/.test(pathname);
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
  /** Lets the request outlive the page (pagehide flush). */
  keepalive?: boolean;
}

export async function http<T>(method: string, path: string, options: HttpOptions = {}): Promise<T> {
  const url = `${API_BASE}${path}${toQueryString(options.query)}`;
  const headers: Record<string, string> = { Accept: "application/json", ...options.headers };
  let body: BodyInit | undefined;
  if (options.body instanceof FormData || options.body instanceof Blob) {
    body = options.body;
  } else if (options.body !== undefined) {
    headers["Content-Type"] = "application/json";
    body = JSON.stringify(options.body);
  }
  const init: RequestInit = { method, headers, body, credentials: "include", keepalive: options.keepalive };

  const send = async () => {
    try {
      return await fetch(url, init);
    } catch {
      throw new ApiError({ status: 0, code: "network", title: fallbackTitle(0), detail: "Проверьте подключение к интернету" });
    }
  };

  const canRefresh = !options.noRefresh && !path.startsWith("/auth/");
  const refreshedFirst = canRefresh && accessExpired() ? await refreshSession() : null;
  let res = await send();
  if (res.status === 401 && canRefresh) {
    if (refreshedFirst !== false && (await refreshSession())) {
      discard(res);
      res = await send();
    }
    if (res.status === 401) redirectToLogin();
  }

  if (!res.ok) throw await toApiError(res);
  if (options.blob) return (await res.blob()) as T;
  const type = res.headers.get("content-type") ?? "";
  if (res.status === 204 || res.headers.get("content-length") === "0" || !type.includes("json")) {
    discard(res);
    return undefined as T;
  }
  return (await res.json()) as T;
}
