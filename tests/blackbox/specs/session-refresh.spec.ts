import type { BrowserContext, Page } from "@playwright/test";
import { expect, test } from "../lib/session";
import { SESSION_SEEKER } from "../lib/accounts";
import type { Api } from "../lib/api";
import { BASE, DEMO_PASSWORD } from "../lib/env";

/** The page's hint that the 15-minute access token ran out: every request then refreshes the session first. */
const EXPIRED = { name: "fsp_at_exp", value: "1000", url: BASE };

function refreshStatuses(page: Page): number[] {
  const statuses: number[] = [];
  page.on("response", (r) => {
    if (r.url().endsWith("/api/v1/auth/refresh")) statuses.push(r.status());
  });
  return statuses;
}

async function refreshFromPage(page: Page): Promise<number> {
  return page.evaluate(() => fetch("/api/v1/auth/refresh", { method: "POST", credentials: "include" }).then((r) => r.status));
}

/**
 * A new session of its own, not the saved one: these tests spend refresh tokens on purpose, and a saved session whose
 * family was revoked still passes the fixture's `/me` check while its access token lives.
 */
async function freshSession(open: () => Promise<{ context: BrowserContext; page: Page; api: Api }>) {
  const session = await open();
  await session.api.post("/auth/login", { email: SESSION_SEEKER, password: DEMO_PASSWORD }, 200);
  await session.context.addCookies([EXPIRED]);
  return session;
}

/**
 * The API rotates the refresh token on every use and revokes the whole session family when a rotated token comes back,
 * so two refreshes holding the same cookie log the user out. A public page refreshes from the header and from the API
 * client; two tabs refresh on their own. Both must spend the cookie once.
 */
test.describe("Сессия: продление токена не разлогинивает", () => {
  test.describe.configure({ mode: "serial" });

  test("публичная страница с истёкшим токеном продлевает сессию один раз, и сессия остаётся живой", async ({ anonymous }) => {
    const { page } = await freshSession(anonymous);
    const refreshes = refreshStatuses(page);

    await page.goto("/vacancies");
    await expect(page.getByRole("link", { name: "Мой кабинет" }).first()).toBeVisible();
    await expect(page.getByText(/Найдено \d+/)).toBeVisible();
    expect(refreshes, "the header and the API client share one refresh").toEqual([200]);
    expect(await refreshFromPage(page), "the session family is not revoked").toBe(200);
  });

  test("две вкладки с истёкшим токеном открываются одновременно — ни одного 401, сессия остаётся живой", async ({ anonymous }) => {
    const { context, page } = await freshSession(anonymous);
    const second = await context.newPage();
    const refreshes = [page, second].map(refreshStatuses);

    await Promise.all([page.goto("/vacancies"), second.goto("/seekers")]);
    for (const tab of [page, second]) await expect(tab.getByRole("link", { name: "Мой кабинет" }).first()).toBeVisible();
    const all = refreshes.flat();
    expect(all.length, "at least one tab refreshed").toBeGreaterThan(0);
    expect(all, "no tab spent a refresh token another tab had already rotated").not.toContain(401);
    expect(await refreshFromPage(page), "the session family is not revoked").toBe(200);
  });
});
