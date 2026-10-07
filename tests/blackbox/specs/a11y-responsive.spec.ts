import AxeBuilder from "@axe-core/playwright";
import type { Page } from "@playwright/test";
import { expect, test } from "../lib/session";
import { DEMO } from "../lib/accounts";
import { DESKTOP, PHONE } from "../lib/env";

const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"];

/** Lets scroll-triggered sections of the public site settle into their final state first. */
async function settle(page: Page): Promise<void> {
  await page.waitForLoadState("networkidle");
  await page.evaluate(async () => {
    for (let y = 0; y < document.body.scrollHeight; y += 600) {
      window.scrollTo(0, y);
      await new Promise((r) => setTimeout(r, 40));
    }
    window.scrollTo(0, 0);
  });
  await page.evaluate(() => document.fonts.ready);
}

/**
 * Audited as a visitor who asked for reduced motion sees the site: the same content, already in
 * its final state, so contrast is measured on settled colors and not mid-animation.
 */
async function audit(page: Page, paths: string[]): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
  for (const path of paths) {
    await test.step(`axe ${path}`, async () => {
      await page.setViewportSize(DESKTOP);
      await page.goto(path);
      await settle(page);
      const { violations } = await new AxeBuilder({ page }).withTags(TAGS).analyze();
      const summary = violations.map((v) => `${v.id} ×${v.nodes.length}: ${v.nodes.map((n) => n.target.join(" ")).slice(0, 3).join(" | ")}`);
      expect.soft(summary, `${path}: axe violations`).toEqual([]);
    });
    await test.step(`375 px ${path}`, async () => {
      await page.setViewportSize(PHONE);
      await page.goto(path);
      await settle(page);
      const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
      expect.soft(overflow, `${path}: horizontal overflow at 375 px`).toBeLessThanOrEqual(0);
    });
  }
}

async function firstId(page: Page, url: string): Promise<string> {
  const res = await page.request.get(url);
  expect(res.status(), url).toBe(200);
  const body = (await res.json()) as { items?: { id?: string; candidate?: { id: string } }[] } | { id: string }[];
  const items = Array.isArray(body) ? body : (body.items ?? []);
  const first = items[0] as { id?: string; candidate?: { id: string } } | undefined;
  const id = first?.candidate?.id ?? first?.id;
  expect(id, url).toBeTruthy();
  return id ?? "";
}

test.describe("доступность (axe) и вёрстка на 375 px", () => {
  test("публичные страницы", async ({ page }) => {
    test.setTimeout(240_000);
    const vacancy = await firstId(page, "/api/v1/vacancies?limit=1");
    await audit(page, ["/", "/seekers", "/business", "/fsp", "/vacancies", `/vacancies/${vacancy}`, "/privacy", "/login", "/register", "/register?role=employer"]);
  });

  test("кабинет соискателя", async ({ login }) => {
    test.setTimeout(240_000);
    const { page } = await login(DEMO.candidate);
    await audit(page, ["/candidate", "/candidate/profile", "/candidate/survey", "/candidate/assessment", "/candidate/invitations", "/candidate/vacancies", "/candidate/tasks", "/candidate/notifications", "/candidate/settings"]);
  });

  test("кабинет работодателя", async ({ login }) => {
    test.setTimeout(240_000);
    const { page } = await login(DEMO.employer);
    const need = await firstId(page, "/api/v1/employer/needs");
    const candidate = await firstId(page, "/api/v1/employer/candidates?specialization=backend&limit=1");
    await audit(page, [
      "/employer",
      "/employer/needs",
      `/employer/needs/${need}`,
      "/employer/needs/new",
      "/employer/candidates",
      `/employer/candidates/${candidate}`,
      "/employer/invitations",
      "/employer/vacancies",
      "/employer/vacancies/new",
      "/employer/integrations",
      "/employer/company",
      "/employer/trust",
    ]);
  });

  test("кабинет модератора", async ({ login }) => {
    test.setTimeout(240_000);
    const { page } = await login(DEMO.moderator);
    await audit(page, ["/admin", "/admin/proctoring", "/admin/antifake", "/admin/log"]);
  });
});
