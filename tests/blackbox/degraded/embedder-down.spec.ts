import fs from "node:fs";
import path from "node:path";
import { expect, test } from "../lib/session";
import { DEMO, EMPLOYERS } from "../lib/accounts";
import { BASE } from "../lib/env";

/**
 * Opt-in (`make blackbox-degraded`): runs after the main suite with the embedder container stopped
 * by the Makefile, never by the test — the test still sees only the public site and API. Everything
 * keeps working on keywords; stored vectors keep serving, new texts wait in the queue.
 */
interface OwnVacancy {
  id: string;
}

interface Related {
  method: "meaning" | "skills";
  items: { vacancy: { id: string; specialization: string } }[];
}

interface Feature {
  key: string;
  explanation_ru: string;
}

const PAIRS = path.join(__dirname, "..", "..", "..", "docs", "validation", "pairs-example");

test("эмбеддер остановлен: публикация, похожие вакансии, подборка и проверка на своих данных работают на ключевых словах", async ({ anonymous, login }) => {
  const guest = await anonymous();
  for (const path of ["/readyz", "/", "/vacancies"]) {
    expect((await guest.api.call("GET", `${BASE}${path}`)).status(), path).toBe(200);
  }

  const company = await login(EMPLOYERS.search);
  const run = Date.now().toString(36);
  const vacancy = await company.api.post<OwnVacancy>(
    "/employer/vacancies",
    {
      title: `Python-разработчик биллинга ${run}`,
      description: `Биллинг оператора связи, ${run}: сервисы на Python и FastAPI, PostgreSQL, Redis, очереди на Kafka.`,
      specialization: "backend",
      grade: "middle",
      skills: ["python", "postgresql", "redis"],
      salary: { min: 230_000, max: 300_000 },
      work_format: "remote",
      city: null,
      status: "published",
    },
    201,
  );
  const related = await guest.api.get<Related>(`/vacancies/${vacancy.id}/similar`);
  expect(related.method, "no vector for a text written while the embedder is down").toBe("skills");
  expect(related.items.every((i) => i.vacancy.specialization === "backend")).toBe(true);

  const employer = await login(DEMO.employer);
  const need = await employer.api.post<{ id: string }>(
    "/employer/needs",
    {
      title: `Python-разработчик биллинга ${run}`,
      description: "Сервисы биллинга на Python: тарификация, PostgreSQL, Kafka.",
      specialization: "backend",
      grades: ["middle"],
      must_skills: ["python", "postgresql"],
      nice_skills: [],
      salary: { min: 230_000, max: 300_000 },
      work_formats: ["remote"],
      city: null,
      industries: [],
    },
    201,
  );
  const shortlist = await employer.api.post<{ items: { breakdown: { features: Feature[] } }[] }>(`/employer/needs/${need.id}/shortlist`, {}, 200);
  expect(shortlist.items.length).toBeGreaterThan(0);
  const text = shortlist.items[0]?.breakdown.features.find((f) => f.key === "text");
  expect(text?.explanation_ru).toMatch(/^Близость описаний по ключевым словам/);

  const input = fs.readFileSync(path.join(PAIRS, "input.json"), "utf8");
  const res = await guest.api.call("POST", "/eval/pairs", { data: input, headers: { "Content-Type": "application/json" } });
  expect(res.status()).toBe(200);
  const report = (await res.json()) as { text: { by_meaning: boolean; fallback: string | null } };
  expect(report.text.by_meaning).toBe(false);
  expect(report.text.fallback).toMatch(/эмбеддер/);
});
