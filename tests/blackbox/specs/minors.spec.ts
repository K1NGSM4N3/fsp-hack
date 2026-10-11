import { expect, test } from "../lib/session";
import { DEMO, EMPLOYERS, PASSWORD, uniqueEmail } from "../lib/accounts";
import { acceptConsents, choose, confirmEmail, dateYearsAgo } from "../lib/flows";
import { linksIn, lettersTo, waitForLetter } from "../lib/mail";

const GUARDIAN_LETTER = /Согласие на участие в «ФСП Таланты»/;

interface SearchPage {
  items: { candidate: { id: string; code: string; minor?: boolean } }[];
}

/** Our addition to 152-ФЗ: seekers aged 14–17 are published only with a legal representative's consent. */
test.describe("соискатели 14–17 лет", () => {
  test("шаг возраста: младше 14 — отказ без отправки данных", async ({ page }) => {
    await page.goto("/register?role=candidate");
    await expect(page.getByRole("group", { name: /Сколько вам лет\?/ })).toBeVisible();
    await choose(page, "Мне меньше 14");
    await expect(page.getByText("Регистрация откроется с 14 лет")).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Email", exact: true })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Создать аккаунт" })).toHaveCount(0);
  });

  test("регистрация 14–17: письмо представителю уходит после подтверждения почты, ссылка из него даёт согласие", async ({ anonymous }, testInfo) => {
    const teen = uniqueEmail(testInfo, "teen");
    const parent = uniqueEmail(testInfo, "parent");
    const { page } = await anonymous();
    const since = Date.now();
    await page.goto("/register?role=candidate");
    await choose(page, "Мне 14–17 лет");
    await page.getByLabel("Дата рождения").fill(dateYearsAgo(15, 40));
    await page.getByRole("textbox", { name: "ФИО законного представителя" }).fill("Петрова Мария Ивановна");
    await page.getByRole("textbox", { name: "Email законного представителя" }).fill(parent);
    await page.getByRole("textbox", { name: "Email", exact: true }).fill(teen);
    await page.getByRole("textbox", { name: "Пароль", exact: true }).fill(PASSWORD);
    await acceptConsents(page);
    await page.getByRole("button", { name: "Создать аккаунт" }).click();
    await expect(page.getByRole("heading", { name: "Подтвердите email" })).toBeVisible();
    await expect(page.getByText(parent)).toBeVisible();

    await waitForLetter(teen, { subject: /Подтверждение email/, since });
    expect(await lettersTo(parent), "the representative hears from us only after the teen confirms the address").toHaveLength(0);
    await confirmEmail(page, teen, since);

    const letter = await waitForLetter(parent, { subject: GUARDIAN_LETTER, since });
    expect(letter.text, "the letter masks the teen's address").not.toContain(teen);
    const [confirm, revoke] = linksIn(letter, /\/guardian\/confirm\?token=/);
    expect(confirm && revoke && confirm !== revoke, "one link to answer, one to revoke").toBe(true);

    const guardian = await anonymous();
    await guardian.page.goto(confirm ?? "");
    await expect(guardian.page.getByRole("heading", { name: "Согласие на участие в «ФСП Таланты»" })).toBeVisible();
    await guardian.page.getByRole("button", { name: "Подтверждаю согласие" }).click();
    await expect(guardian.page.getByText(/Согласие подтверждено/).first()).toBeVisible();
    const reused = await guardian.api.call("POST", "/guardian/consent", { data: { token: new URL(confirm ?? "").searchParams.get("token"), action: "grant" } });
    expect(reused.status(), "the answer link works once").toBe(409);
  });

  test("демо-подросток: не виден до согласия → «Отправить письмо ещё раз» → представитель подтверждает → виден с отметкой «14–17 лет», «Только 18+» скрывает", async ({ login, anonymous }) => {
    const minor = await login(DEMO.minor);
    expect(minor.me.candidate?.age.bracket).toBe("minor");
    const category = minor.me.candidate?.category;
    expect(category, "the seeded teen has a category").toBeTruthy();
    const query = `specialization=${category?.specialization}&grade=${category?.grade}&limit=100`;

    const employer = await login(EMPLOYERS.search);
    const before = await employer.api.get<SearchPage>(`/employer/candidates?${query}`);
    expect(before.items.map((i) => i.candidate.id), "hidden until the representative consents").not.toContain(minor.me.id);
    await employer.api.problem("GET", `/employer/candidates/${minor.me.id}`, 404, "not_found");
    await minor.api.problem("PUT", "/candidate/privacy", 409, "guardian_consent_required", {
      data: { ...(await minor.api.get<Record<string, unknown>>("/candidate/privacy")), visible: true },
    });

    const guardianEmail = (await minor.api.get<{ guardian_email: string }>("/candidate/guardian")).guardian_email;
    const since = Date.now();
    await minor.page.goto("/candidate");
    await expect(minor.page.getByText("Ждём подтверждения от законного представителя")).toBeVisible();
    await minor.page.getByRole("button", { name: "Отправить письмо ещё раз" }).click();
    await expect(minor.page.getByText("Письмо отправлено").first()).toBeVisible();

    const letter = await waitForLetter(guardianEmail, { subject: GUARDIAN_LETTER, since });
    const [confirm] = linksIn(letter, /\/guardian\/confirm\?token=/);
    const guardian = await anonymous();
    await guardian.page.goto(confirm ?? "");
    await guardian.page.getByRole("button", { name: "Подтверждаю согласие" }).click();
    await expect(guardian.page.getByText(/Согласие подтверждено/).first()).toBeVisible();

    const after = await employer.api.get<SearchPage>(`/employer/candidates?${query}`);
    const card = after.items.find((i) => i.candidate.id === minor.me.id);
    expect(card?.candidate.minor, "searchable, marked 14–17").toBe(true);
    const adults = await employer.api.get<SearchPage>(`/employer/candidates?${query}&adults_only=true`);
    expect(adults.items.map((i) => i.candidate.id)).not.toContain(minor.me.id);

    const page = employer.page;
    await page.goto(`/employer/candidates?specialization=${category?.specialization}&grade=${category?.grade}`);
    const article = page.locator("article").filter({ has: page.locator(`a[href^="/employer/candidates/${minor.me.id}"]`) });
    await expect(page.getByText(/Найдено \d+ кандидат/)).toBeVisible();
    await expect(page.locator("article").first()).toBeVisible();
    const more = page.getByRole("button", { name: "Показать ещё" });
    const cards = page.locator("article");
    for (let i = 0; i < 10 && !(await article.count()) && (await more.isVisible()); i++) {
      const shown = await cards.count();
      await more.click();
      await expect.poll(() => cards.count(), { message: "the next page of the bank loads" }).toBeGreaterThan(shown);
    }
    await expect(article.getByText("14–17 лет")).toBeVisible();
    await page.getByRole("switch", { name: "Только 18+" }).click();
    await expect(article).toHaveCount(0);
    await expect(page.locator("article").first()).toBeVisible();
  });
});
