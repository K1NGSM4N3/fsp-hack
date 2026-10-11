import fs from "node:fs";
import { expect, test } from "../lib/session";
import { Api } from "../lib/api";
import { EMPLOYERS, PASSWORD, SEEKERS, uniqueEmail } from "../lib/accounts";
import { invitationBody, registrationBody } from "../lib/flows";
import { expectSearchableSeeker } from "../lib/seeker";
import type { Session } from "../lib/session";

interface SearchPage {
  items: { candidate: { id: string } }[];
  total: number;
}

async function found(searcher: Session, candidateId: string): Promise<boolean> {
  const page = await searcher.api.get<SearchPage>("/employer/candidates?specialization=backend&grade=middle&limit=100");
  return page.items.some((i) => i.candidate.id === candidateId);
}

/** 152-ФЗ: publication by consent only, who opened the contacts, a copy of one's data, the policy. */
test.describe("152-ФЗ: согласия, журнал доступа к контактам, выгрузка данных", () => {
  test.describe.configure({ mode: "serial" });

  test("компания открыла контакты по приглашению — соискатель видит это в «Кто открывал ваши контакты»", async ({ login }) => {
    const seeker = await login(SEEKERS.privacy);
    await expectSearchableSeeker(seeker);
    const inviter = await login(EMPLOYERS.search);
    const company = await inviter.api.get<{ name: string }>("/employer/company");
    const sent = await inviter.api.post<{ id: string }>("/employer/invitations", invitationBody(seeker.me.id, `Backend-разработчик ${Date.now().toString(36)}`), 201);
    await seeker.api.post(`/candidate/invitations/${sent.id}/accept`, {}, 200);
    await inviter.api.get(`/employer/candidates/${seeker.me.id}/contacts`);

    await seeker.page.goto("/candidate/settings#contact-access");
    const log = seeker.page.locator("#contact-access");
    await expect(log.getByText(company.name).first()).toBeVisible();
    await expect(log.getByText("Вы приняли приглашение").first()).toBeVisible();
    const entries = await seeker.api.get<Record<string, unknown>[]>("/candidate/contact-access");
    const entry = entries.find((e) => e.company_name === company.name);
    expect(entry && Object.keys(entry).sort(), "the log names the company, not the person who looked").toEqual(["accessed_at", "basis", "channel", "company_id", "company_name"]);
  });

  test("отзыв согласия на публикацию скрывает из поиска; без согласия видимость не включить (409 и диалог); повторное согласие возвращает в поиск", async ({ login }) => {
    const seeker = await login(SEEKERS.privacy);
    const searcher = await login(EMPLOYERS.application);
    const id = seeker.me.id;
    expect(await found(searcher, id), "searchable before the revocation").toBe(true);

    const page = seeker.page;
    await page.goto("/candidate/settings#consents");
    const row = page.locator("li").filter({ hasText: "Согласие на публикацию профиля" });
    await row.getByRole("button", { name: "Отозвать" }).click();
    const confirm = page.getByRole("dialog", { name: "Отозвать согласие?" });
    await confirm.getByRole("button", { name: "Отозвать", exact: true }).click();
    await expect(page.getByText("Согласие отозвано").first()).toBeVisible();
    await expect(row.getByRole("button", { name: "Дать согласие снова" })).toBeVisible();

    expect((await seeker.api.get<{ visible: boolean }>("/candidate/privacy")).visible).toBe(false);
    expect(await found(searcher, id), "the search no longer shows the seeker").toBe(false);
    await searcher.api.problem("GET", `/employer/candidates/${id}`, 404, "not_found");

    await page.goto("/candidate/settings#visibility");
    const answer = page.waitForResponse((r) => r.url().endsWith("/api/v1/candidate/privacy") && r.request().method() === "PUT");
    await page.getByRole("switch", { name: "Показывать профиль в поиске" }).click();
    const refused = await answer;
    expect(refused.status()).toBe(409);
    const problem = (await refused.json()) as { code: string; document_kind: string };
    expect(problem).toMatchObject({ code: "consent_required", document_kind: "pd_publication" });
    const offer = page.getByRole("dialog", { name: "Чтобы показать профиль, нужно согласие" });
    await expect(offer).toBeVisible();
    await offer.getByRole("button", { name: "Дать согласие и показать профиль" }).click();
    await expect(page.getByText("Профиль снова виден компаниям")).toBeVisible();

    expect((await seeker.api.get<{ visible: boolean }>("/candidate/privacy")).visible).toBe(true);
    const consents = await seeker.api.get<{ kind: string; revoked_at: string | null }[]>("/me/consents");
    expect(consents.filter((c) => c.kind === "pd_publication").map((c) => c.revoked_at === null).sort()).toEqual([false, true]);
    expect(await found(searcher, id), "visible again").toBe(true);

    const current = new Set((await seeker.api.get<{ id: string }[]>("/consents/current")).map((d) => d.id));
    const granted = await seeker.api.get<{ document_id: string; kind: string }[]>("/me/consents");
    const superseded = granted.find((c) => !current.has(c.document_id));
    expect(superseded, "the seed granted an earlier wording of the processing consent").toBeTruthy();
    const outdated = await seeker.api.problem("POST", "/me/consents", 422, "validation_failed", { data: { document_id: superseded?.document_id ?? "" } });
    expect(outdated.errors?.document_id?.[0]).toBe("Это не действующая редакция документа — обновите страницу");
  });

  test("«Скачать мои данные» отдаёт корректный JSON со всем, что о соискателе хранится", async ({ login }) => {
    const seeker = await login(SEEKERS.privacy);
    const page = seeker.page;
    await page.goto("/candidate/settings#data");
    const download = page.waitForEvent("download");
    await page.getByRole("button", { name: "Скачать мои данные" }).click();
    const file = await download;
    expect(file.suggestedFilename()).toMatch(/^fsp-talents-\d{4}-\d{2}-\d{2}\.json$/);
    await expect(page.getByText("Файл с вашими данными сохранён")).toBeVisible();
    const data = JSON.parse(fs.readFileSync((await file.path()) ?? "", "utf8")) as {
      format: string;
      account: { email: string; role: string };
      consents: { kind: string; ip: string | null }[];
      candidate: { contact_access: unknown[]; invitations: { status: string }[]; assessments: unknown[] };
      employer: unknown;
    };
    expect(data.format).toBe("fsp-talents-export/1");
    expect(data.account).toMatchObject({ email: SEEKERS.privacy, role: "candidate" });
    expect(data.employer).toBeNull();
    expect(data.consents.map((c) => c.kind)).toEqual(expect.arrayContaining(["pd_processing", "pd_publication", "terms"]));
    expect(data.candidate.contact_access.length).toBeGreaterThan(0);
    expect(data.candidate.invitations.some((i) => i.status === "accepted")).toBe(true);
    const text = JSON.stringify(data);
    for (const internal of ["template_id", "current_item"]) expect(text, "no test internals").not.toContain(internal);

    const res = await seeker.api.call("GET", "/me/export");
    expect(res.headers()["content-disposition"]).toMatch(/^attachment; filename="fsp-talents-\d{4}-\d{2}-\d{2}\.json"$/);
    expect(res.headers()["cache-control"]).toBe("private, no-store");
  });
});

test.describe("политика и регистрация", () => {
  test("/privacy: разделы для каждого документа согласия, ссылки из формы регистрации и подвала", async ({ page }) => {
    await page.goto("/privacy");
    await expect(page.getByRole("heading", { level: 1, name: "Политика обработки персональных данных" })).toBeVisible();
    for (const id of ["pd_processing", "pd_publication", "terms", "proctoring_video", "guardian_consent", "minors"]) {
      const text = (await page.locator(`[id="${id}"]`).innerText()).trim();
      expect(text.length, `#${id}`).toBeGreaterThan(200);
    }
    await expect(page.locator('footer a[href="/privacy"]').first()).toBeVisible();
    await page.goto("/register?role=candidate");
    for (const anchor of ["pd_processing", "pd_publication"]) await expect(page.locator(`form a[href="/privacy#${anchor}"]`).first()).toBeVisible();
  });

  test("две одновременные регистрации на один email: один аккаунт и 409 email_taken", async ({ request }, testInfo) => {
    const api = new Api(request);
    const body = await registrationBody(api, { email: uniqueEmail(testInfo, "twice"), password: PASSWORD, role: "candidate", age: "adult" });
    const [a, b] = await Promise.all([api.call("POST", "/auth/register", { data: body }), api.call("POST", "/auth/register", { data: body })]);
    expect([a.status(), b.status()].sort()).toEqual([201, 409]);
    const conflict = a.status() === 409 ? a : b;
    expect(conflict.headers()["content-type"]).toContain("application/problem+json");
    expect(((await conflict.json()) as { code: string }).code).toBe("email_taken");
  });
});
