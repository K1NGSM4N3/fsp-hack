import { expect, test } from "../lib/session";
import { PASSWORD, uniqueEmail } from "../lib/accounts";
import { confirmEmail, register } from "../lib/flows";
import type { Session } from "../lib/session";

interface Vacancy {
  id: string;
  title: string;
}

/** A brand-new company: its trust level, quota and anti-fake checks start from a known state. */
test.describe.configure({ mode: "serial" });

test.describe("доверие и антифейк для новой компании", () => {
  let email = "";
  const run = Date.now().toString(36);
  const title = `Go-разработчик платёжного шлюза ${run}`;
  const description = `Пишем платёжный шлюз ${run}: API на Go, PostgreSQL, Kafka, интеграции с банками-эквайерами и антифрод. Команда из пяти человек.`;

  async function company(login: (email: string, password?: string) => Promise<Session>): Promise<Session> {
    return login(email, PASSWORD);
  }

  test("страница доверия: новая компания, база квоты и как её поднять", async ({ anonymous, login }, testInfo) => {
    email = uniqueEmail(testInfo, "trust");
    const guest = await anonymous();
    const since = Date.now();
    await register(guest.api, { email, password: PASSWORD, role: "employer", company_name: `Проверка доверия ${run}` });
    await confirmEmail(guest.page, email, since);
    const { page, api } = await company(login);
    const trust = await api.get<{ level: string; quota_limit: number }>("/employer/trust");
    expect(trust).toMatchObject({ level: "new", quota_limit: 10 });
    await page.goto("/employer/trust");
    await expect(page.getByRole("heading", { level: 1, name: "Доверие и квота" })).toBeVisible();
    await expect(page.getByText("Новая компания").first()).toBeVisible();
    await expect(page.getByText("Как поднять квоту")).toBeVisible();
    await expect(page.getByText(/Подтвердите корпоративный домен — квота вырастет на 10/)).toBeVisible();
  });

  test("форма вакансии предупреждает о вилке, но публиковать не мешает", async ({ login }) => {
    const { page, api } = await company(login);
    const check = await api.get<{ warnings: { kind: string; text: string }[] }>("/employer/salary-check?specialization=backend&grade=middle&min=300000&max=900000");
    expect(check.warnings.map((w) => w.kind)).toContain("salary_wide_range");
    for (const w of check.warnings) expect(w.text).toBeTruthy();

    await page.goto("/employer/vacancies/new");
    await page.getByRole("textbox", { name: "Название" }).fill(title);
    await page.getByRole("combobox", { name: "Специализация" }).selectOption("backend");
    await page.getByRole("combobox", { name: "Грейд" }).selectOption("middle");
    await page.getByRole("textbox", { name: "Описание" }).fill(description);
    await page.getByRole("textbox", { name: "Зарплата от, ₽" }).fill("300000");
    await page.getByRole("textbox", { name: "Зарплата до, ₽" }).fill("900000");
    const hint = page.getByRole("alert").filter({ hasText: "Проверьте вилку" }).or(page.getByText("Проверьте вилку"));
    await expect(hint.first()).toBeVisible();
    await expect(page.getByText(/очень широкая/).first()).toBeVisible();

    await page.getByRole("textbox", { name: "Зарплата до, ₽" }).fill("360000");
    await page.locator("input[name=status][value=published]").check({ force: true });
    await page.getByRole("button", { name: "Создать вакансию" }).click();
    await expect(page.getByText("Вакансия создана").first()).toBeVisible();
    await expect(page).toHaveURL(/\/employer\/vacancies\/[0-9a-f-]{36}$/);
    const mine = await api.get<Vacancy[]>("/employer/vacancies");
    expect(mine.filter((v) => v.title === title)).toHaveLength(1);
  });

  test("тот же текст второй раз — форма показывает похожую вакансию до сохранения", async ({ login }) => {
    const { page } = await company(login);
    await page.goto("/employer/vacancies/new");
    await page.getByRole("textbox", { name: "Название" }).fill(title);
    await page.getByRole("combobox", { name: "Специализация" }).selectOption("backend");
    await page.getByRole("combobox", { name: "Грейд" }).selectOption("middle");
    await page.getByRole("textbox", { name: "Описание" }).fill(description);
    await page.getByRole("textbox", { name: "Зарплата от, ₽" }).fill("300000");
    await page.getByRole("textbox", { name: "Зарплата до, ₽" }).fill("360000");
    await page.getByRole("button", { name: "Создать вакансию" }).click();
    await expect(page.getByText("Похожая вакансия уже есть")).toBeVisible();
    await expect(page.getByRole("link", { name: `«${title}»` })).toBeVisible();
    await expect(page.getByRole("button", { name: "Всё равно создать" })).toBeVisible();
  });

  test("жалоба на одну вакансию от одного человека — одна, пока модератор не решил", async ({ login }) => {
    const { api } = await company(login);
    const list = await api.get<{ items: { id: string; company_name: string }[] }>("/vacancies?limit=100");
    const target = list.items.find((v) => v.company_name === "Северный Код");
    expect(target, "the demo company publishes vacancies").toBeTruthy();
    const body = { target_kind: "vacancy", target_id: target?.id, reason: "fake_vacancy", comment: `Проверка повтора жалобы ${run}` };
    const first = await api.post<{ id: string; already_open: boolean }>("/reports", body, 201);
    expect(first.already_open).toBe(false);
    const again = await api.post<{ id: string; already_open: boolean }>("/reports", { ...body, reason: "salary_mismatch" }, 200);
    expect(again).toEqual({ id: first.id, already_open: true });
  });

  test("двойная отправка с одним Idempotency-Key создаёт одну вакансию", async ({ login }) => {
    const { api } = await company(login);
    const body = {
      title: `QA-инженер ${run}`,
      description: `Автотесты на Playwright для кабинета ${run}, ревью сценариев с аналитиками, нагрузочные прогоны перед релизом.`,
      specialization: "qa",
      grade: "middle",
      skills: [],
      salary: { min: 180_000, max: 230_000 },
      work_format: "remote",
      city: null,
      status: "draft",
    };
    const key = `bb-${run}`;
    const [a, b] = await Promise.all([1, 2].map(() => api.call("POST", "/employer/vacancies", { data: body, headers: { "Idempotency-Key": key } })));
    expect([a?.status(), b?.status()].sort()).toEqual([200, 201]);
    const ids = await Promise.all([a, b].map(async (r) => ((await r?.json()) as Vacancy).id));
    expect(ids[0]).toBe(ids[1]);
    const created = a?.status() === 201 ? a : b;
    expect(created?.headers()["location"]).toBe(`/api/v1/employer/vacancies/${ids[0]}`);
    const again = await api.call("POST", "/employer/vacancies", { data: body, headers: { "Idempotency-Key": key } });
    expect(again.status(), "a retry returns the vacancy made the first time").toBe(200);
    const mine = await api.get<Vacancy[]>("/employer/vacancies");
    expect(mine.filter((v) => v.title === body.title)).toHaveLength(1);
    const long = await api.problem("POST", "/employer/vacancies", 422, "validation_failed", { data: body, headers: { "Idempotency-Key": "k".repeat(101) } });
    expect(Object.keys(long.errors ?? {})).toEqual(["idempotency_key"]);
  });
});
