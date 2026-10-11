import fs from "node:fs";
import path from "node:path";
import { expect, test } from "../lib/session";
import { Api } from "../lib/api";
import { EMPLOYERS } from "../lib/accounts";

const PAIRS = path.join(__dirname, "..", "..", "..", "docs", "validation", "pairs-example");

test.describe("соглашения API: problem+json, Location, кеширование", () => {
  test("ошибки разбора запроса — problem+json со стабильным code", async ({ request }) => {
    const api = new Api(request);
    const json = { "Content-Type": "application/json" };
    await api.problem("POST", "/auth/login", 400, "invalid_json", { data: '{"email": "x@example.ru",', headers: json });
    await api.problem("POST", "/auth/login", 422, "invalid_json", { data: '{"email": 1, "password": "x"}', headers: json });
    await api.problem("POST", "/auth/login", 415, "unsupported_media_type", { data: "{}", headers: { "Content-Type": "text/plain" } });
    await api.problem("GET", "/vacancies/not-a-uuid", 400, "invalid_path");
    await api.problem("GET", "/catalog/test-plan?specialization=wizard&grade=middle", 400, "invalid_query");
    const unauthorized = await api.call("GET", "/me");
    expect(unauthorized.status()).toBe(401);
    expect(unauthorized.headers()["content-type"]).toContain("application/problem+json");
    await api.problem("GET", `/vacancies/${"0".repeat(8)}-0000-7000-8000-${"0".repeat(12)}`, 404, "not_found");
  });

  test("201 отдаёт Location, по которому созданное читается", async ({ login }) => {
    const { api } = await login(EMPLOYERS.search);
    const res = await api.call("POST", "/employer/needs", {
      data: { title: `Проверка Location ${Date.now().toString(36)}`, specialization: "backend", grades: ["middle"], must_skills: ["go"], salary: { min: 250_000, max: 320_000 } },
    });
    expect(res.status()).toBe(201);
    const created = (await res.json()) as { id: string };
    const location = res.headers()["location"];
    expect(location).toBe(`/api/v1/employer/needs/${created.id}`);
    expect((await api.get<{ id: string }>(location ?? "")).id).toBe(created.id);
    await api.delete(location ?? "", 204);
  });

  test("справочники и вакансии кешируются публично с ETag и 304, личные ответы — никогда", async ({ request, login }) => {
    const identity = { "Accept-Encoding": "identity" };
    for (const [url, policy] of [
      ["/api/v1/catalog", "public, max-age=300, stale-while-revalidate=86400"],
      ["/api/v1/consents/current", "public, max-age=300, stale-while-revalidate=86400"],
      ["/api/v1/vacancies?limit=5", "public, max-age=30, stale-while-revalidate=120"],
    ] as const) {
      const first = await request.get(url, { headers: identity });
      expect(first.status(), url).toBe(200);
      expect(first.headers()["cache-control"], url).toBe(policy);
      const etag = first.headers()["etag"];
      expect(etag, `${url}: ETag`).toBeTruthy();
      const again = await request.get(url, { headers: { ...identity, "If-None-Match": etag ?? "" } });
      expect(again.status(), `${url}: revalidation`).toBe(304);
      expect(again.headers()["etag"]).toBe(etag);
      expect((await again.body()).byteLength).toBe(0);
    }
    expect((await request.get("/api/v1/me")).headers()["cache-control"]).toBe("private, no-cache");
    const { api } = await login(EMPLOYERS.search);
    for (const url of ["/me", "/employer/company", "/employer/candidates?specialization=backend", "/me/notifications"]) {
      const res = await api.call("GET", url);
      expect(res.status(), url).toBe(200);
      expect(res.headers()["cache-control"], url).toBe("private, no-store");
    }
    const missing = await request.get(`/api/v1/vacancies/${"0".repeat(8)}-0000-7000-8000-${"0".repeat(12)}`);
    expect(missing.status()).toBe(404);
    expect(missing.headers()["cache-control"], "errors are never stored").toBe("no-store");
  });

  test("POST /eval/pairs на примере из репозитория воспроизводит метрики закоммиченного отчёта", async ({ request }) => {
    const input = fs.readFileSync(path.join(PAIRS, "input.json"), "utf8");
    const committed = JSON.parse(fs.readFileSync(path.join(PAIRS, "report.json"), "utf8")) as { metrics: unknown; coverage: unknown; k: number };
    const res = await request.post("/api/v1/eval/pairs", { data: input, headers: { "Content-Type": "application/json" } });
    expect(res.status()).toBe(200);
    expect(res.headers()["cache-control"]).toBe("no-store");
    const report = (await res.json()) as { metrics: unknown; coverage: unknown; k: number };
    expect(report.k).toBe(committed.k);
    expect(report.coverage).toEqual(committed.coverage);
    expect(report.metrics).toEqual(committed.metrics);
  });
});
