import { expect, test } from "../lib/session";
import { DEMO, EMPLOYERS } from "../lib/accounts";
import { PHONE } from "../lib/env";

interface Need {
  id: string;
  title: string;
  specialization: string;
  created_at: string;
}

interface ShortlistItem {
  candidate: { id: string; code: string; proctoring?: { status: string } | null };
  explanation?: { reasons: unknown[] };
}

test.describe("работодатель: потребность → категории → подборка с объяснением → план теста", () => {
  test("S-4: потребность из текста вакансии сразу даёт категории, подборку с баллом и план теста", async ({ login }) => {
    const { page } = await login(EMPLOYERS.search);
    await page.goto("/employer/needs/new");
    await page.getByRole("button", { name: "Вставить пример" }).click();
    await page.getByRole("button", { name: "Распознать", exact: true }).click();
    await expect(page.getByText("Распознано", { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Сохранить и показать подборку" }).click();
    await expect(page).toHaveURL(/\/employer\/needs\/[0-9a-f-]{36}$/);

    await expect(page.getByText("Рекомендованные категории")).toBeVisible();
    const explain = page.getByRole("button", { name: "Как посчитан балл" }).first();
    await expect(explain).toBeVisible({ timeout: 30_000 });
    await explain.click();
    await expect(page.getByText(/Итог — сумма баллов по признакам \(\d+ из 100\)/).first()).toBeVisible();

    await page.getByRole("button", { name: /Как проверяется категория под эту потребность/ }).click();
    await expect(page.getByText(/\d+–\d+ заданий, тест подстраивается под ответы кандидата/)).toBeVisible();
  });

  test("демо-компания: шесть потребностей, у каждой план теста и подборка с объяснениями", async ({ login }) => {
    const { api, page } = await login(DEMO.employer);
    const needs = await api.get<Need[]>("/employer/needs");
    const seeded = needs.filter((n) => ["backend", "frontend", "devops", "qa", "ml", "security"].includes(n.specialization));
    expect(new Set(seeded.map((n) => n.specialization)).size).toBe(6);
    expect(needs.map((n) => n.title)).toContain("Go-разработчик в команду платежей");
    for (const need of seeded.slice(0, 6)) {
      const plan = await api.get<{ plan: { topics: unknown[] }; samples: unknown[] }>(`/employer/needs/${need.id}/test-plan`);
      expect(plan.plan.topics.length, need.title).toBeGreaterThanOrEqual(3);
      expect(plan.samples, `${need.title}: two candidates get different variants`).toHaveLength(2);
      const shortlist = await api.post<{ items: ShortlistItem[] }>(`/employer/needs/${need.id}/shortlist`, {}, 200);
      expect(shortlist.items.length, need.title).toBeGreaterThanOrEqual(3);
    }
    const go = needs.find((n) => n.title === "Go-разработчик в команду платежей");
    await page.goto(`/employer/needs/${go?.id}`);
    await expect(page.getByText("Рекомендованные категории")).toBeVisible();
    await expect(page.getByRole("button", { name: "Как посчитан балл" }).first()).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText(/Тест (с|без) видеозаписи/).first()).toBeVisible();
  });

  test("банк кандидатов: отметки видеозаписи, а на 375 px фильтры свёрнуты под одной кнопкой", async ({ login }) => {
    const { page } = await login(EMPLOYERS.search);
    await page.goto("/employer/candidates?specialization=backend");
    await expect(page.getByText(/Найдено \d+ кандидат/)).toBeVisible();
    await expect(page.getByText("Тест с видеозаписью").first()).toBeVisible();

    await page.setViewportSize(PHONE);
    const toggle = page.locator("button[aria-controls=search-filters]").filter({ visible: true });
    await expect(toggle).toHaveAttribute("aria-expanded", "false");
    await expect(page.locator("#search-filters")).toBeHidden();
    await toggle.click();
    await expect(page.locator("#search-filters")).toBeVisible();
    await page.locator("#search-filters input[placeholder='Москва']").fill("Москва");
    await expect(toggle).toContainText("1 фильтр");
    await page.keyboard.press("Escape");
    await expect(page.locator("#search-filters")).toBeHidden();
    await expect(toggle).toBeFocused();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)).toBeLessThanOrEqual(0);
  });
});
