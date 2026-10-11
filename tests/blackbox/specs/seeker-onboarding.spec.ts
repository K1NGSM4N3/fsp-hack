import { expect, test } from "../lib/session";
import { DEMO, PASSWORD, uniqueEmail } from "../lib/accounts";
import { acceptConsents, choose, confirmEmail, loginUI } from "../lib/flows";

/**
 * ТЗ S-1…S-3: a new seeker registers, confirms the email, fills the survey, takes the adaptive
 * test with the camera on and gets a result. A black-box test cannot know the right answers, so
 * it answers whatever is on screen and checks that the outcome is consistent: either a category
 * is assigned, or the result says the grade is not confirmed and offers the next step.
 */
test.describe.configure({ mode: "serial" });

test.describe("соискатель: регистрация → опрос → адаптивный тест с видеозаписью → результат", () => {
  let email = "";
  let sessionId = "";
  let passed = false;

  test("S-1: регистрация, подтверждение email и первый вход", async ({ anonymous }, testInfo) => {
    email = uniqueEmail(testInfo, "seeker");
    const { page } = await anonymous();
    const since = Date.now();
    await page.goto("/register?role=candidate");
    await expect(page.getByRole("radio", { name: /Ищу работу/ })).toBeChecked();
    await choose(page, "Мне есть 18 лет");
    await page.getByRole("textbox", { name: "Email", exact: true }).fill(email);
    await page.getByRole("textbox", { name: "Пароль", exact: true }).fill(PASSWORD);
    await acceptConsents(page);
    await page.getByRole("button", { name: "Создать аккаунт" }).click();
    await expect(page.getByRole("heading", { name: "Подтвердите email" })).toBeVisible();
    await expect(page.getByText(email)).toBeVisible();

    await confirmEmail(page, email, since);
    await loginUI(page, email, PASSWORD);
    await expect(page).toHaveURL(/\/candidate$/);
    await expect(page.locator("#next-step-title")).toHaveText("Заполните профиль");
  });

  test("S-2: профиль и опрос: специализация, стек, грейд", async ({ login }) => {
    const { page } = await login(email, PASSWORD);
    await page.goto("/candidate/profile");
    await page.locator("input[autocomplete=name]").fill("Анна Проверочная");
    await page.locator("input[type=tel]").first().fill("+79991112233");
    await page.getByRole("button", { name: "Сохранить профиль" }).click();
    await expect(page.getByText("Профиль сохранён")).toBeVisible();

    await page.goto("/candidate/survey");
    await expect(page.getByText("В каком направлении вы работаете?")).toBeVisible();
    await choose(page, "Разработка");
    await page.getByRole("button", { name: "Далее" }).click();
    await choose(page, "Бэкенд-разработка");
    await page.getByRole("button", { name: "Далее" }).click();
    for (const skill of ["Go", "PostgreSQL"]) await page.locator("button[aria-pressed]").filter({ hasText: new RegExp(`^${skill}$`) }).first().click();
    await page.getByRole("button", { name: "Далее" }).click();
    await page.getByRole("button", { name: "Далее" }).click();
    await choose(page, /^Middle/);
    await page.getByRole("button", { name: "Сохранить и перейти к тесту" }).click();
    await expect(page).toHaveURL(/\/candidate\/assessment(\?|$)/);
    await expect(page.getByRole("button", { name: "Начать тест на Middle" })).toBeVisible();
  });

  test("S-2: шаг видеозаписи: камера, согласие, затем адаптивный тест до результата", async ({ login }) => {
    test.setTimeout(300_000);
    const { page } = await login(email, PASSWORD);
    await page.goto("/candidate/assessment");
    await page.getByRole("button", { name: "Начать тест на Middle" }).click();
    await expect(page).toHaveURL(/\/candidate\/assessment\/start/);
    await expect(page.getByRole("heading", { name: "Видеозапись во время теста" })).toBeVisible();
    await expect(page.getByText("Кто увидит")).toBeVisible();
    const start = page.getByRole("button", { name: "Начать с видеозаписью" });
    await expect(start).toBeDisabled();
    await page.getByRole("button", { name: "Включить камеру" }).click();
    await expect(page.getByText("Так вас будет видно на записи")).toBeVisible();
    await expect.poll(() => page.locator("video").first().evaluate((v: HTMLVideoElement) => v.readyState >= 2 && v.videoWidth > 0)).toBe(true);
    await choose(page, "Согласен(на) на видеозапись во время теста");
    await expect(start).toBeEnabled();
    await start.click();

    await expect(page).toHaveURL(/\/candidate\/assessment\/[0-9a-f-]{36}$/);
    sessionId = page.url().split("/").pop() ?? "";
    await expect(page.getByText("Идёт запись")).toBeVisible();

    const result = page.getByRole("heading", { name: "Результат теста" });
    const task = page.getByRole("heading", { name: /^Задание \d+$/ });
    let answered = 0;
    while (!(await result.isVisible())) {
      await expect(task.or(result)).toBeVisible({ timeout: 30_000 });
      if (await result.isVisible()) break;
      const heading = await task.textContent();
      const form = page.locator("form[data-answer]");
      await expect(form.getByRole("button", { name: "Ответить" })).toBeVisible();
      await expect(form.locator("input").first()).toBeAttached();
      if (answered === 0) await form.evaluate((f) => f.dispatchEvent(new ClipboardEvent("paste", { bubbles: true, cancelable: true })));
      const text = form.locator("input[id^=ans-]");
      if (await text.count()) await text.fill("1");
      else if (await form.locator("input[type=checkbox]").count()) await form.locator("input[type=checkbox]").first().check();
      else await form.locator("label").first().click();
      await form.getByRole("button", { name: "Ответить" }).click();
      answered += 1;
      await expect(async () => {
        expect((await result.isVisible()) || (await task.textContent()) !== heading).toBe(true);
      }).toPass({ timeout: 30_000 });
      expect(answered, "the test stops by 20 items").toBeLessThanOrEqual(20);
    }
    expect(answered).toBeGreaterThanOrEqual(8);
    await expect(page.getByText(/заданий: \d+/)).toBeVisible();
  });

  test("S-3: результат согласован: категория присвоена или понятное «не подтверждён» и следующий шаг", async ({ login }) => {
    const { page, api } = await login(email, PASSWORD);
    await page.goto(`/candidate/assessment/${sessionId}`);
    const session = await api.get<{ status: string; result: { decision: { passed: boolean }; assigned_grade: string | null; offers: { grade: string; kind: string }[] } }>(`/candidate/assessments/${sessionId}`);
    expect(["passed", "failed"]).toContain(session.status);
    passed = session.status === "passed";
    expect(session.result.decision.passed).toBe(passed);
    const me = await api.get<{ candidate: { category: { specialization: string; grade: string } | null } }>("/me");
    const category = await api.get<{ available: { grade: string; kind: string }[] }>("/candidate/category");

    if (passed) {
      await expect(page.getByText("Грейд подтверждён")).toBeVisible();
      await expect(page.getByText("Категория присвоена — компании уже могут найти вас в поиске.")).toBeVisible();
      expect(me.candidate.category?.specialization).toBe("backend");
      expect(session.result.assigned_grade).toBe(me.candidate.category?.grade);
      await page.goto("/candidate");
      await expect(page.locator("#next-step-title")).toHaveText("Ждите приглашений");
    } else {
      await expect(page.getByText("Грейд Middle пока не подтверждён")).toBeVisible();
      expect(me.candidate.category).toBeNull();
      expect(session.result.assigned_grade).toBeNull();
      expect(category.available, "a failed placement opens the grade below at once").toContainEqual(expect.objectContaining({ grade: "junior", kind: "placement_step_down" }));
      await expect(page.getByText(/Junior/).first()).toBeVisible();
    }
  });

  test("видеозапись: соискатель смотрит свою запись, компании видна только отметка, просмотр модератором виден соискателю", async ({ login }) => {
    const seeker = await login(email, PASSWORD);
    const { page, api } = seeker;
    await page.goto(`/candidate/assessment/${sessionId}`);
    await expect(page.getByText("Тест с видеозаписью").first()).toBeVisible({ timeout: 30_000 });
    await expect(page.getByText("Есть замечания: 1 вставка текста")).toBeVisible();
    await expect(page.getByText("Что видят компании")).toBeVisible();
    await page.getByRole("button", { name: "Посмотреть запись" }).click();
    const player = page.locator('video[src*="/proctoring/recording"]');
    await expect.poll(() => player.evaluate((v: HTMLVideoElement) => v.readyState >= 1 && v.videoWidth > 0), { timeout: 20_000 }).toBe(true);

    const recording = await api.call("GET", `/candidate/assessments/${sessionId}/proctoring/recording`);
    expect(recording.status()).toBe(200);
    expect(recording.headers()["content-type"]).toMatch(/^video\//);
    expect((await recording.body()).byteLength).toBeGreaterThan(1000);

    const employer = await login(DEMO.employer);
    for (const path of [`/candidate/assessments/${sessionId}/proctoring/recording`, `/admin/proctoring/${sessionId}/recording`, `/admin/proctoring/${sessionId}`]) {
      await employer.api.problem("GET", path, 403, "wrong_role");
    }
    const card = await employer.api.call("GET", `/employer/candidates/${seeker.me.id}`);
    if (card.ok()) expect(await card.text(), "the company never gets the session or a link to it").not.toContain(sessionId);

    const moderator = await login(DEMO.moderator);
    const watched = await moderator.api.call("GET", `/admin/proctoring/${sessionId}/recording`);
    expect(watched.status()).toBe(200);
    expect((await watched.body()).byteLength).toBeGreaterThan(1000);
    await page.goto(`/candidate/assessment/${sessionId}`);
    await expect(page.getByText(/Запись теста просмотрена модератором/), "every view by a moderator is shown to the seeker").toBeVisible();
  });
});
