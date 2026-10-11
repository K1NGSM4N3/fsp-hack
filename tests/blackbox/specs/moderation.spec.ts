import { expect, test } from "../lib/session";
import { DEMO, EMPLOYERS } from "../lib/accounts";
import { Api } from "../lib/api";
import { loginUI } from "../lib/flows";
import { DEMO_PASSWORD } from "../lib/env";

interface QueueItem {
  session_id: string;
  candidate_code: string;
  status: string;
  result_in_force: boolean;
  category: { specialization: string; grade: string };
}

interface Notification {
  kind: string;
  title: string;
  body: string;
  link: string | null;
}

interface AntifakeItem {
  kind: string;
  id: string;
  title: string;
  company: { name: string } | null;
  complaint: { reason: string; reason_ru: string; comment: string | null; target_kind: string } | null;
}

const ANNUL = "Верные ответы на трудных заданиях пришли сразу после уходов со вкладки, на лёгких — ошибки. Результат аннулирован.";
const CONFIRM = "Запись и журнал без нарушений: вкладку открывали на пару секунд, ответы ровные по уровню.";
const DISMISS = "Вилка в письме совпадает с вакансией: в вакансии сумма до вычета НДФЛ.";

/**
 * The moderator works through the seeded queues. Decisions are final and consume seeded items,
 * so this file runs serially and each test takes a different item.
 */
test.describe.configure({ mode: "serial" });

test.describe("кабинет модератора", () => {
  let reviewCode = "";
  let sessionId = "";

  test("модератор входит в /admin и видит засеянную очередь спорных тестов", async ({ anonymous, login }) => {
    const review = await login(DEMO.review);
    reviewCode = review.me.candidate?.public_code ?? "";
    await review.page.goto("/candidate/assessment");
    await expect(review.page.getByText("требует подтверждения").first()).toBeVisible();

    const { page } = await anonymous();
    await loginUI(page, DEMO.moderator, DEMO_PASSWORD);
    await expect(page).toHaveURL(/\/admin$/);
    await expect(page.getByText("Кабинет модератора").first()).toBeVisible();
    await expect(page.getByText("Требуют подтверждения").first()).toBeVisible();

    await page.goto("/admin/proctoring");
    await expect(page.getByRole("link", { name: reviewCode }).first()).toBeVisible();
    expect(await page.locator("tbody tr").count(), "the seed puts several disputed tests in the queue").toBeGreaterThanOrEqual(5);
  });

  test("аннулирование с комментарием: соискатель видит решение в истории и получает уведомление", async ({ login }) => {
    const review = await login(DEMO.review);
    const unread = (await review.api.get<{ count: number }>("/me/notifications/unread-count")).count;

    const { page } = await login(DEMO.moderator);
    await page.goto("/admin/proctoring");
    await page.getByRole("link", { name: reviewCode }).first().click();
    await expect(page).toHaveURL(/\/admin\/proctoring\/[0-9a-f-]{36}$/);
    await expect(page.getByRole("heading", { name: `Тест ${reviewCode}` })).toBeVisible();
    await expect(page.getByText("Согласованность ответов").first()).toBeVisible();
    await expect(page.getByText("Лента: задания и события")).toBeVisible();
    sessionId = page.url().split("/").pop() ?? "";
    expect(await page.locator("main").innerText(), "the moderator does not see who the candidate is").not.toContain(DEMO.review);
    await page.locator("input[name=decision][value=annul]").check({ force: true });
    await page.getByRole("textbox", { name: "Комментарий" }).fill(ANNUL);
    await page.locator("aside").getByRole("button", { name: "Аннулировать результат" }).click();
    const confirm = page.getByRole("dialog", { name: "Аннулировать результат?" });
    await confirm.getByRole("button", { name: "Аннулировать результат" }).click();
    await expect(page.getByText("Решение принято")).toBeVisible();
    await expect(page.getByText(ANNUL.slice(0, 40))).toBeVisible();

    await expect.poll(async () => (await review.api.get<{ count: number }>("/me/notifications/unread-count")).count).toBe(unread + 1);
    const feed = await review.api.get<{ items: Notification[] }>("/me/notifications");
    const notice = feed.items.find((n) => n.kind === "result_annulled");
    expect(notice?.link).toBe(`/candidate/assessment/${sessionId}`);
    expect(notice?.body).toContain(ANNUL.slice(0, 40));
    expect(JSON.stringify(feed), "the moderator is not named").not.toContain(DEMO.moderator);

    await review.page.goto("/candidate");
    await review.page.getByTestId("notification-bell").filter({ visible: true }).click();
    await expect(review.page.getByRole("dialog", { name: "Уведомления" }).getByText(/Результат теста аннулирован/)).toBeVisible();
    await review.page.keyboard.press("Escape");
    await review.page.goto("/candidate/assessment");
    await expect(review.page.getByText("аннулирован модератором").first()).toBeVisible();
    await review.page.goto(`/candidate/assessment/${sessionId}`);
    await expect(review.page.getByText("Модератор аннулировал результат")).toBeVisible();
    await expect(review.page.getByText(ANNUL.slice(0, 40))).toBeVisible();
    const me = await review.api.get<{ candidate: { category: unknown } }>("/me");
    expect(me.candidate.category, "the only test that gave the category was annulled").toBeNull();
    const session = await review.api.get<{ result: { assigned_grade: string | null; needs_review: boolean } }>(`/candidate/assessments/${sessionId}`);
    expect(session.result).toMatchObject({ assigned_grade: null, needs_review: false });
  });

  test("подтверждение результата: работодатель видит «Результат подтверждён» в карточке", async ({ login }) => {
    const moderator = await login(DEMO.moderator);
    const employer = await login(EMPLOYERS.search);
    const queue = await moderator.api.get<{ items: QueueItem[] }>("/admin/proctoring?filter=needs_review&limit=50");
    let confirmed: { id: string; code: string } | undefined;
    for (const item of queue.items.filter((i) => i.status === "passed" && i.result_in_force && i.candidate_code !== reviewCode)) {
      const search = await employer.api.get<{ items: { candidate: { id: string; code: string; result_confirmed?: boolean } }[] }>(
        `/employer/candidates?specialization=${item.category.specialization}&grade=${item.category.grade}&limit=100`,
      );
      const card = search.items.find((i) => i.candidate.code === item.candidate_code);
      if (!card) continue;
      expect(card.candidate.result_confirmed).toBe(false);
      await moderator.api.post(`/admin/proctoring/${item.session_id}/decision`, { decision: "confirm", comment: CONFIRM }, 200);
      confirmed = { id: card.candidate.id, code: item.candidate_code };
      break;
    }
    expect(confirmed, "a searchable disputed test is left in the seeded queue").toBeTruthy();
    const card = await employer.api.get<{ candidate: { result_confirmed: boolean } }>(`/employer/candidates/${confirmed?.id}`);
    expect(card.candidate.result_confirmed).toBe(true);
    expect(JSON.stringify(card), "the comment is for the candidate only").not.toContain(CONFIRM.slice(0, 30));
    await employer.page.goto(`/employer/candidates/${confirmed?.id}`);
    await expect(employer.page.getByText("Результат подтверждён").first()).toBeVisible();
  });

  test("журнал: оба решения с комментариями и модератором, фильтр по коду", async ({ login }) => {
    const { page } = await login(DEMO.moderator);
    await page.goto("/admin/log");
    const log = page.locator("table tbody");
    await expect(log.getByText("Аннулировал результат").first()).toBeVisible();
    await expect(log.getByText("Подтвердил результат").first()).toBeVisible();
    await expect(log.getByText(ANNUL.slice(0, 30)).first()).toBeVisible();
    await expect(log.getByText(DEMO.moderator).first()).toBeVisible();
    const subject = page.getByPlaceholder("K-7Q3M2X");
    await subject.fill(` ${reviewCode.toLowerCase()} `);
    await expect(async () => {
      const rows = await page.locator("table tbody tr").allInnerTexts();
      expect(rows.length).toBeGreaterThan(0);
      for (const row of rows) expect(row).toContain(reviewCode);
    }).toPass();
    await subject.fill("K-ZZZZZZ");
    await expect(page.getByText("Записей нет")).toBeVisible();
    await subject.fill("nobody");
    await expect(page.getByText("Нужен код вида K-7Q3M2X или ID")).toBeVisible();
  });

  test("каждая операция модерации из OpenAPI отвечает соискателю и работодателю 403, гостю 401; /admin уводит в свой кабинет", async ({ login, request }) => {
    const spec = (await (await request.get("/api/v1/openapi.json")).json()) as { paths: Record<string, Record<string, unknown>> };
    const id = "01900000-0000-7000-8000-000000000000";
    const operations = Object.entries(spec.paths)
      .filter(([path]) => path.startsWith("/api/v1/admin"))
      .flatMap(([path, ops]) => Object.keys(ops).map((method) => ({ method: method.toUpperCase(), path: path.replace(/\{[^}]+\}/g, id) })));
    expect(operations.length, "the moderator's API is documented").toBeGreaterThanOrEqual(9);
    const guest = new Api(request);
    for (const email of [DEMO.review, EMPLOYERS.search]) {
      const s = await login(email);
      for (const op of operations) {
        const data = op.method === "GET" || op.method === "HEAD" ? undefined : {};
        await s.api.problem(op.method, op.path, 403, "wrong_role", { data });
        if (email === DEMO.review) {
          const res = await guest.call(op.method, op.path, { data });
          expect(res.status(), `${op.method} ${op.path} without a session`).toBe(401);
        }
      }
    }
    const review = await login(DEMO.review);
    await review.page.goto("/admin");
    await expect(review.page).toHaveURL(/\/candidate/);
  });

  test("антифейк: отклонённая жалоба — компания видит решение на странице доверия и получает уведомление, но не автора", async ({ login }) => {
    const moderator = await login(DEMO.moderator);
    const company = await login(EMPLOYERS.complaints);
    const companyName = (await company.api.get<{ name: string }>("/employer/company")).name;
    const open = await moderator.api.get<{ items: AntifakeItem[] }>("/admin/antifake?kind=complaint&limit=50");
    const complaint = open.items.find((i) => i.complaint && i.company?.name === companyName);
    expect(complaint, `the seed has an open complaint about ${companyName}`).toBeTruthy();

    const page = moderator.page;
    await page.goto("/admin/antifake");
    const item = page
      .locator("li")
      .filter({ hasText: complaint?.complaint?.reason_ru ?? "" })
      .filter({ hasText: /жалоба на/ })
      .filter({ has: page.getByRole("button", { name: "Отклонить: нарушений нет" }) });
    await item.getByRole("button", { name: "Отклонить: нарушений нет" }).first().click();
    const dialog = page.getByRole("dialog", { name: "Отклонить: нарушений нет" });
    await expect(dialog).toContainText(complaint?.title ?? "");
    await dialog.getByRole("textbox", { name: "Комментарий" }).fill(DISMISS);
    await dialog.getByRole("button", { name: "Отклонить: нарушений нет" }).click();
    await expect(dialog).toBeHidden();
    await expect
      .poll(async () => (await moderator.api.get<{ items: AntifakeItem[] }>("/admin/antifake?kind=complaint&limit=50")).items.some((i) => i.id === complaint?.id))
      .toBe(false);

    await company.page.goto("/employer/trust");
    await expect(company.page.getByText("Решения модератора")).toBeVisible();
    await expect(company.page.getByText(/не подтвердилась/).first()).toBeVisible();
    await expect(company.page.getByText(DISMISS)).toBeVisible();
    const feed = await company.api.get<{ items: Notification[] }>("/me/notifications");
    const resolved = feed.items.find((n) => n.kind === "complaint_resolved");
    expect(resolved?.link).toBe("/employer/trust");
    const said = JSON.stringify(feed);
    for (const leak of [complaint?.complaint?.comment, DEMO.moderator].filter((x): x is string => Boolean(x))) expect(said).not.toContain(leak);
  });
});
