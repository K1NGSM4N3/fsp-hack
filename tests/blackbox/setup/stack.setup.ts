import { test, expect } from "@playwright/test";
import { ATS, BASE, FSP_ADMIN, MAILPIT } from "../lib/env";
import { openSession, saveSession } from "../lib/session";
import { DEMO } from "../lib/accounts";

const ready = async (url: string) => {
  try {
    return (await fetch(url, { redirect: "manual" })).status === 200;
  } catch {
    return false;
  }
};

test("the stack answers on its public ports", async () => {
  test.setTimeout(300_000);
  for (const url of [`${BASE}/readyz`, `${BASE}/`, `${BASE}/api/v1/catalog`, `${MAILPIT}/api/v1/info`, `${ATS}/healthz`, `${FSP_ADMIN}/healthz`]) {
    await expect.poll(() => ready(url), { message: url, timeout: 240_000, intervals: [1000, 2000] }).toBe(true);
  }
});

/** Demo accounts many specs share: one session per worker slot, saved for the session fixture. */
test("demo sessions for every worker slot", async ({ browser }, testInfo) => {
  for (let slot = 0; slot < testInfo.config.workers; slot++) {
    for (const email of [DEMO.employer, DEMO.moderator]) {
      await saveSession(await openSession(browser, email, slot), slot);
    }
  }
});
