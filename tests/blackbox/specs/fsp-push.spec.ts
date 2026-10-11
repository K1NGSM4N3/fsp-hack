import { createHmac } from "node:crypto";
import { expect, test } from "../lib/session";
import { Api } from "../lib/api";
import { DEMO } from "../lib/accounts";
import { FSP_ADMIN, FSP_ADMIN_TOKEN } from "../lib/env";

/** The registry's push secret is shared deployment config between the registry and the platform (.env.example). */
const PUSH_SECRET = process.env.FSP_REGISTRY_WEBHOOK_SECRET ?? "whsec_demo_fsp_registry_do_not_use_in_production";

interface Achievements {
  linked: boolean;
  fsp_id: string;
  achievements: { id: string; text: string }[];
}

interface Notification {
  kind: string;
  title: string;
  link: string | null;
}

interface SkillDetail {
  slug: string;
  sources: string[];
}

async function registry(method: string, path: string, body?: unknown) {
  const res = await fetch(`${FSP_ADMIN}/admin/participants/${path}`, {
    method,
    headers: { Authorization: `Bearer ${FSP_ADMIN_TOKEN}`, ...(body ? { "Content-Type": "application/json" } : {}) },
    body: body ? JSON.stringify(body) : undefined,
  });
  return { status: res.status, body: (await res.json()) as { added?: { id: string; event_name: string }; push: { event_id: string; status: number } | null } };
}

function signature(body: string, t: number): string {
  return `t=${t},v1=${createHmac("sha256", PUSH_SECRET).update(`${t}.${body}`).digest("hex")}`;
}

test.describe.configure({ mode: "serial" });

test.describe("реестр ФСП сам сообщает о новом результате", () => {
  test("новое достижение в реестре появляется в профиле без ручной синхронизации, с уведомлением и подтверждёнными навыками", async ({ login }) => {
    const { api, page } = await login(DEMO.candidate);
    const before = await api.get<Achievements>("/fsp/achievements");
    expect(before.linked, "candidate@demo.fsp is linked to a registry participant (docs/guide.md, section 2.6)").toBe(true);

    const added = await registry("POST", `${before.fsp_id}/achievements`);
    expect(added.status).toBe(201);
    expect(added.body.push?.status, "the registry pushed achievements.changed and the platform accepted it").toBe(202);
    const event = added.body.added?.event_name ?? "";

    const after = await api.get<Achievements>("/fsp/achievements");
    expect(after.achievements).toHaveLength(before.achievements.length + 1);
    expect(after.achievements.some((a) => a.text.includes(event))).toBe(true);
    const newest = (await api.get<{ items: Notification[] }>("/me/notifications")).items[0];
    expect(newest).toMatchObject({ kind: "fsp_achievements", title: "Новые достижения ФСП в профиле", link: "/candidate/settings#fsp" });
    const skills = (await api.get<{ skill_details: SkillDetail[] }>("/candidate/profile")).skill_details;
    expect(skills.find((s) => s.slug === "algorithms")?.sources, "a podium confirms the discipline's skills").toContain("fsp");

    await page.goto("/candidate/settings#fsp");
    await expect(page.locator("#fsp").getByText(event).first()).toBeVisible();
    await page.getByTestId("notification-bell").filter({ visible: true }).click();
    await expect(page.getByRole("dialog", { name: "Уведомления" }).getByText("Новые достижения ФСП в профиле").first()).toBeVisible();

    const removed = await registry("DELETE", `${before.fsp_id}/achievements/${added.body.added?.id}`);
    expect(removed.status).toBe(200);
    expect(removed.body.push?.status).toBe(202);
    expect((await api.get<Achievements>("/fsp/achievements")).achievements).toHaveLength(before.achievements.length);
    expect((await api.get<{ items: Notification[] }>("/me/notifications")).items[0]?.title).toBe("Данные ФСП в профиле обновились");
  });

  test("приём событий реестра: без подписи, с чужой и с устаревшей подписью — 401; повтор события ничего не меняет", async ({ request, login }) => {
    const api = new Api(request);
    const { api: candidate } = await login(DEMO.candidate);
    const fspId = (await candidate.get<Achievements>("/fsp/achievements")).fsp_id;
    const body = JSON.stringify({ event_id: `evt_bb_${Date.now()}`, type: "achievements.changed", fsp_id: fspId, occurred_at: new Date().toISOString() });
    const now = Math.floor(Date.now() / 1000);
    const json = { "Content-Type": "application/json" };
    await api.problem("POST", "/fsp/registry/events", 401, "signature_invalid", { data: body, headers: json });
    await api.problem("POST", "/fsp/registry/events", 401, "signature_invalid", { data: body, headers: { ...json, "FSP-Signature": `t=${now},v1=${"0".repeat(64)}` } });
    await api.problem("POST", "/fsp/registry/events", 401, "signature_expired", { data: body, headers: { ...json, "FSP-Signature": signature(body, now - 301) } });
    await api.problem("POST", "/fsp/registry/events", 401, "signature_invalid", { data: body.replace(fspId, "FSP-999999"), headers: { ...json, "FSP-Signature": signature(body, now) } });

    const feed = async () => (await candidate.get<{ items: Notification[] }>("/me/notifications")).items.length;
    const count = await feed();
    for (let i = 0; i < 2; i++) {
      const res = await api.call("POST", "/fsp/registry/events", { data: body, headers: { ...json, "FSP-Signature": signature(body, Math.floor(Date.now() / 1000)) } });
      expect(res.status(), "accepted, and the same answer for a repeat").toBe(202);
    }
    expect(await feed(), "nothing changed in the registry, so nothing to tell the candidate").toBe(count);
    const stranger = JSON.stringify({ event_id: `evt_bb_x_${Date.now()}`, type: "achievements.changed", fsp_id: "FSP-999999", occurred_at: new Date().toISOString() });
    const unknown = await api.call("POST", "/fsp/registry/events", { data: stranger, headers: { ...json, "FSP-Signature": signature(stranger, Math.floor(Date.now() / 1000)) } });
    expect(unknown.status(), "an unknown participant gets the same answer: nothing is revealed").toBe(202);
  });
});
