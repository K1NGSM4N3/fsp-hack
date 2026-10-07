import { expect, test } from "../lib/session";
import { ATS, BASE } from "../lib/env";
import { DEMO, PASSWORD, SEEKERS, uniqueEmail } from "../lib/accounts";
import { confirmEmail, invitationBody, register } from "../lib/flows";
import { Api } from "../lib/api";
import { expectSearchableSeeker } from "../lib/seeker";

interface AtsEntry {
  kind: "webhook" | "outcome";
  event_id?: string | null;
  event_type?: string | null;
  verdict?: string;
  duplicate?: boolean;
  body?: { id?: string; type?: string; data?: { invitation?: { id: string; status: string }; candidate?: { code: string; id?: string; disclosed?: { full_name?: string | null } } } };
  subject?: string;
  stage?: string;
  status?: number | null;
}

async function atsEvents(): Promise<AtsEntry[]> {
  const res = await fetch(`${ATS}/events`);
  expect(res.status).toBe(200);
  return (await res.json()) as AtsEntry[];
}

const forInvitation = (id: string) => (e: AtsEntry) => e.kind === "webhook" && e.body?.data?.invitation?.id === id;

test.describe("интеграция с ATS работодателя", () => {
  test("«Интеграции»: компания подключает вебхук к своей ATS и выпускает API-ключ", async ({ anonymous, login }, testInfo) => {
    const email = uniqueEmail(testInfo, "hr");
    const guest = await anonymous();
    const since = Date.now();
    await register(guest.api, { email, password: PASSWORD, role: "employer", company_name: `Проверка ATS ${Date.now().toString(36)}` });
    await confirmEmail(guest.page, email, since);
    const { page, api } = await login(email, PASSWORD);

    await page.goto("/employer/integrations");
    await page.getByRole("textbox", { name: "Адрес для событий" }).fill("http://ats-mock:8082/webhooks");
    await page.getByRole("button", { name: "Подключить" }).click();
    const secret = page.getByRole("dialog", { name: "Секрет подписи" });
    await expect(secret.locator("code").first()).toHaveText(/^whsec_/);
    await secret.getByRole("button", { name: "Я сохранил" }).click();
    const overview = await api.get<{ webhook: { url: string; secret_hint: string } }>("/employer/integrations");
    expect(overview.webhook.url).toBe("http://ats-mock:8082/webhooks");
    expect(overview.webhook.secret_hint).toMatch(/^whsec_.{4}…$/);

    const sent = page.waitForResponse((r) => r.url().endsWith("/integrations/webhook/test"));
    await page.getByRole("button", { name: "Тестовое событие" }).click();
    const test = (await (await sent).json()) as { event_id: string; status_code: number };
    await expect.poll(async () => (await atsEvents()).some((e) => e.event_id === test.event_id), { message: "the ATS received the test event" }).toBe(true);
    const received = (await atsEvents()).find((e) => e.event_id === test.event_id);
    expect(received?.event_type).toBe("integration.test");
    expect(received?.verdict, "signed: the demo ATS holds only the demo company's secret, so it cannot verify a new one").toBe("mismatch");
    expect(test.status_code).toBe(401);

    await page.getByRole("textbox", { name: "Название" }).fill("Huntflow");
    await page.getByRole("button", { name: "Создать ключ" }).click();
    const reveal = page.getByRole("dialog", { name: "API-ключ" });
    const key = (await reveal.locator("code").first().textContent()) ?? "";
    expect(key).toMatch(/^fsp_live_[A-Za-z0-9_-]{43}$/);
    await reveal.getByRole("button", { name: "Я сохранил" }).click();
    const ats = new Api(api.request).withBearer(key);
    const whoami = await ats.get<{ company_name: string; scopes: string[] }>("/integrations/whoami");
    expect(whoami.company_name).toMatch(/^Проверка ATS/);
    const keys = await api.get<{ prefix: string; last_used_at: string | null }[]>("/employer/integrations/keys");
    expect(key.startsWith(keys[0]?.prefix ?? "-")).toBe(true);
    expect(JSON.stringify(keys), "the key itself is shown once").not.toContain(key);
    await new Api(guest.context.request).withBearer(key).problem("GET", "/employer/integrations", 401, "unauthorized");
  });

  test("приглашение и принятие доходят до ATS подписанными; контакты — только в invitation.accepted; «Нанят» из ATS видит соискатель", async ({ login }) => {
    const seeker = await login(SEEKERS.ats);
    await expectSearchableSeeker(seeker);
    const profile = await seeker.api.get<{ full_name: string }>("/candidate/profile");
    const employer = await login(DEMO.employer);
    const title = `Backend-разработчик в ATS ${Date.now().toString(36)}`;
    const inv = await employer.api.post<{ id: string; candidate_code: string }>("/employer/invitations", invitationBody(seeker.me.id, title), 201);

    await expect.poll(async () => (await atsEvents()).filter(forInvitation(inv.id)).map((e) => e.event_type)).toContain("invitation.sent");
    await seeker.api.get(`/candidate/invitations/${inv.id}`);
    await seeker.api.post(`/candidate/invitations/${inv.id}/accept`, {}, 200);
    await expect.poll(async () => (await atsEvents()).filter(forInvitation(inv.id)).map((e) => e.event_type), { timeout: 30_000 }).toContain("invitation.accepted");

    const events = (await atsEvents()).filter(forInvitation(inv.id));
    for (const e of events) {
      expect(e.verdict, `${e.event_type}: the ATS verified our signature`).toBe("valid");
      expect(e.duplicate).toBe(false);
      expect(e.body?.id).toBe(e.event_id);
      expect(e.body?.data?.candidate?.code).toBe(inv.candidate_code);
      if (e.event_type === "invitation.accepted") {
        expect(e.body?.data?.candidate?.id).toBe(seeker.me.id);
        expect(e.body?.data?.candidate?.disclosed?.full_name).toBe(profile.full_name);
      } else {
        expect(e.body?.data?.candidate?.disclosed, `${e.event_type} carries no contacts`).toBeUndefined();
        expect(JSON.stringify(e.body)).not.toContain(profile.full_name);
      }
    }

    const mock = employer.page;
    await mock.goto(ATS);
    const form = mock.locator("form.stages").filter({ has: mock.locator(`input[name=id][value="${inv.id}"]`) }).first();
    await form.locator("input[name=reason]").fill("Вышел на работу");
    await form.getByRole("button", { name: "Нанят" }).click();
    await expect.poll(async () => (await atsEvents()).find((e) => e.kind === "outcome" && e.subject?.includes(inv.id))?.status).toBe(201);

    await seeker.page.goto(`/candidate/invitations/${inv.id}`);
    await expect(seeker.page.getByText("Компания отметила: вы наняты")).toBeVisible();
    await expect(seeker.page.getByText("«Вышел на работу»")).toBeVisible();
  });

  test("API-ключ: опрос по курсору отдаёт изменения, импорт вакансии с external_id идемпотентен", async ({ login }) => {
    const employer = await login(DEMO.employer);
    const created = await employer.api.post<{ key: { id: string }; secret: string }>("/employer/integrations/keys", { name: `blackbox ${Date.now().toString(36)}`, scopes: ["invitations:read", "needs:write"] }, 201);
    const ats = new Api(employer.api.request).withBearer(created.secret);
    try {
      let cursor: string | null = null;
      const seen: string[] = [];
      for (;;) {
        const page: { items: { invitation: { id: string; status: string } }[]; next_cursor: string | null } = await ats.get(`/integrations/invitations?limit=200${cursor ? `&cursor=${cursor}` : ""}`);
        seen.push(...page.items.map((i) => i.invitation.id));
        if (page.items.length === 0) {
          expect(page.next_cursor, "an empty page hands the cursor back").toBe(cursor);
          break;
        }
        expect(page.next_cursor).toBeTruthy();
        cursor = page.next_cursor;
      }
      expect(seen.length, "the demo company's invitation history").toBeGreaterThan(0);
      await ats.problem("GET", "/integrations/invitations?cursor=nope", 400, "invalid_query");
      await ats.problem("POST", "/integrations/outcomes", 403, "insufficient_scope", { data: { invitation_id: seen[0], stage: "interview", at: new Date().toISOString() } });

      const body = {
        external_id: `hh-${Date.now().toString(36)}`,
        text: "Senior Backend-разработчик (Go)\nПлатёжный сервис, от 350 000 до 450 000 ₽.\nGo, PostgreSQL, Kafka; опыт от 5 лет.",
      };
      const first = await ats.call("POST", "/integrations/needs", { data: body });
      expect(first.status()).toBe(201);
      const need = (await first.json()) as { need_id: string; specialization: string; shortlist_url: string };
      expect(first.headers()["location"]).toBe(`/api/v1/employer/needs/${need.need_id}`);
      expect(need.specialization).toBe("backend");
      expect(need.shortlist_url).toBe(`${BASE}/employer/needs/${need.need_id}`);
      const retry = await ats.call("POST", "/integrations/needs", { data: { ...body, text: `${body.text}\nОбновили текст.` } });
      expect(retry.status(), "a retry returns the need it made").toBe(200);
      expect(((await retry.json()) as { need_id: string }).need_id).toBe(need.need_id);
      await employer.api.delete(`/employer/needs/${need.need_id}`, 204);
    } finally {
      await employer.api.delete(`/employer/integrations/keys/${created.key.id}`, 204);
    }
    await ats.problem("GET", "/integrations/whoami", 401, "api_key_invalid");
  });
});
