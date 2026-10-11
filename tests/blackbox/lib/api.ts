import { expect, type APIRequestContext, type APIResponse } from "@playwright/test";
import { BASE } from "./env";

/** RFC 9457 problem with the API's own `code`. */
export interface Problem {
  type?: string;
  title: string;
  status: number;
  code: string;
  detail?: string | null;
  errors?: Record<string, string[]>;
  [extra: string]: unknown;
}

type Body = Record<string, unknown> | unknown[];

export interface CallOptions {
  data?: Body | string;
  headers?: Record<string, string>;
  form?: Record<string, string>;
}

/**
 * The public HTTP API, as a browser of this origin would call it: the session lives in the
 * context's cookies, and every request carries our Origin (the API's CSRF check wants it on writes).
 */
export class Api {
  constructor(
    readonly request: APIRequestContext,
    private readonly bearer?: string,
  ) {}

  withBearer(token: string): Api {
    return new Api(this.request, token);
  }

  /** `/me` means `/api/v1/me`; absolute URLs and `/api/...` paths go as they are. */
  private url(path: string): string {
    if (path.startsWith("http")) return path;
    return path.startsWith("/api/") ? `${BASE}${path}` : `${BASE}/api/v1${path}`;
  }

  async call(method: string, path: string, opts: CallOptions = {}): Promise<APIResponse> {
    const headers: Record<string, string> = { Origin: BASE, ...(this.bearer ? { Authorization: `Bearer ${this.bearer}` } : {}), ...opts.headers };
    return this.request.fetch(this.url(path), {
      method,
      headers,
      // A string body goes byte for byte: Playwright would JSON-encode a string that is not valid JSON.
      data: typeof opts.data === "string" ? Buffer.from(opts.data) : opts.data,
      form: opts.form,
      failOnStatusCode: false,
      maxRedirects: 0,
    });
  }

  /** Calls and checks the status; the body of an unexpected answer goes into the failure message. */
  async expect<T = unknown>(method: string, path: string, status: number | number[], opts: CallOptions = {}): Promise<T> {
    const res = await this.call(method, path, opts);
    const want = Array.isArray(status) ? status : [status];
    if (!want.includes(res.status())) {
      throw new Error(`${method} ${path}: ${res.status()} (want ${want.join("/")}) ${await res.text()}`);
    }
    const text = await res.text();
    return (text ? JSON.parse(text) : null) as T;
  }

  get<T = unknown>(path: string, status: number | number[] = 200): Promise<T> {
    return this.expect<T>("GET", path, status);
  }

  post<T = unknown>(path: string, data?: Body, status: number | number[] = [200, 201]): Promise<T> {
    return this.expect<T>("POST", path, status, { data: data ?? {} });
  }

  put<T = unknown>(path: string, data: Body, status: number | number[] = [200, 201]): Promise<T> {
    return this.expect<T>("PUT", path, status, { data });
  }

  patch<T = unknown>(path: string, data: Body, status: number | number[] = 200): Promise<T> {
    return this.expect<T>("PATCH", path, status, { data });
  }

  delete<T = unknown>(path: string, status: number | number[] = [200, 204]): Promise<T> {
    return this.expect<T>("DELETE", path, status);
  }

  /** Expects a problem+json error with this status and code. */
  async problem(method: string, path: string, status: number, code: string, opts: CallOptions = {}): Promise<Problem> {
    const res = await this.call(method, path, opts);
    const text = await res.text();
    expect(res.status(), `${method} ${path}: ${text}`).toBe(status);
    expect(res.headers()["content-type"]).toContain("application/problem+json");
    const problem = JSON.parse(text) as Problem;
    expect(problem.code, `${method} ${path}: ${text}`).toBe(code);
    expect(problem.status).toBe(status);
    expect(typeof problem.title).toBe("string");
    return problem;
  }
}
