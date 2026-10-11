import fs from "node:fs";
import path from "node:path";
import { test as base, expect, type Browser, type BrowserContext, type Page } from "@playwright/test";
import { Api } from "./api";
import { CONTEXT, DEMO_PASSWORD } from "./env";

export interface Me {
  id: string;
  role: "candidate" | "employer" | "admin";
  email: string;
  email_verified: boolean;
  candidate?: {
    public_code: string;
    onboarding_step: string;
    category?: { specialization: string; grade: string } | null;
    active_session_id?: string | null;
    age: { bracket: "adult" | "minor"; guardian?: string | null; adult_from?: string | null };
  } | null;
  company?: { id: string; name: string } | null;
}

/** A logged-in browser context: the page for the UI, `api` for the same session over HTTP. */
export interface Session {
  email: string;
  me: Me;
  context: BrowserContext;
  page: Page;
  api: Api;
}

const STATE_DIR = path.join(__dirname, "..", ".auth");

/**
 * Login is limited per IP and email (10 per 15 minutes), so a session is kept per account and per
 * parallel worker slot and reused by every test that runs in that slot. Tests in one slot run one
 * after another, so a refresh-token rotation by one test is saved before the next test reads it.
 */
export function stateFile(email: string, slot: number): string {
  return path.join(STATE_DIR, `${email.replace(/[^a-z0-9.@_-]/gi, "_")}-${slot}.json`);
}

async function currentUser(api: Api): Promise<Me | null> {
  const res = await api.call("GET", "/me");
  return res.ok() ? ((await res.json()) as Me) : null;
}

/** Restores the saved session of `email` in this slot, refreshing or logging in only when it is gone. */
export async function openSession(browser: Browser, email: string, slot: number, password = DEMO_PASSWORD): Promise<Session> {
  fs.mkdirSync(STATE_DIR, { recursive: true });
  const file = stateFile(email, slot);
  const context = await browser.newContext({ ...CONTEXT, storageState: fs.existsSync(file) ? file : undefined });
  const api = new Api(context.request);
  let me = await currentUser(api);
  if (me?.email !== email) {
    const refreshed = me === null && (await api.call("POST", "/auth/refresh")).ok();
    me = refreshed ? await currentUser(api) : null;
  }
  if (me?.email !== email) {
    await context.clearCookies();
    await api.post("/auth/login", { email, password }, 200);
    me = await currentUser(api);
  }
  expect(me?.email, `session of ${email}`).toBe(email);
  await context.storageState({ path: file });
  const page = await context.newPage();
  return { email, me: me as Me, context, page, api };
}

export async function saveSession(session: Session, slot: number): Promise<void> {
  await session.context.storageState({ path: stateFile(session.email, slot) }).catch(() => undefined);
  await session.context.close().catch(() => undefined);
}

interface Fixtures {
  /** Opens a session of an existing, verified account (demo accounts by default password). */
  login: (email: string, password?: string) => Promise<Session>;
  /** A fresh context with no session: a visitor, a guardian opening a letter. */
  anonymous: () => Promise<{ context: BrowserContext; page: Page; api: Api }>;
}

export const test = base.extend<Fixtures>({
  login: async ({ browser }, use, testInfo) => {
    const opened: Session[] = [];
    await use(async (email, password) => {
      const session = await openSession(browser, email, testInfo.parallelIndex, password);
      opened.push(session);
      return session;
    });
    for (const session of opened) await saveSession(session, testInfo.parallelIndex);
  },
  anonymous: async ({ browser }, use) => {
    const contexts: BrowserContext[] = [];
    await use(async () => {
      const context = await browser.newContext(CONTEXT);
      contexts.push(context);
      return { context, page: await context.newPage(), api: new Api(context.request) };
    });
    for (const context of contexts) await context.close().catch(() => undefined);
  },
});

export { expect };
