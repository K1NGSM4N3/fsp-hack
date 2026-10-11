import type { BrowserContextOptions } from "@playwright/test";

const trim = (url: string) => url.replace(/\/$/, "");

/** The stack under test, reached only through its public ports (deploy/compose.blackbox.yml). */
export const BASE = trim(process.env.BLACKBOX_BASE_URL ?? "http://localhost:8080");
export const MAILPIT = trim(process.env.BLACKBOX_MAILPIT_URL ?? "http://localhost:8025");
export const ATS = trim(process.env.BLACKBOX_ATS_URL ?? "http://localhost:8082");
export const FSP_ADMIN = trim(process.env.BLACKBOX_FSP_ADMIN_URL ?? "http://127.0.0.1:8091");
export const FSP_ADMIN_TOKEN = process.env.FSP_REGISTRY_TOKEN ?? "dev-registry-token";

export const DEMO_PASSWORD = "demo12345";

export const DESKTOP = { width: 1440, height: 900 };
export const PHONE = { width: 375, height: 812 };

/** Options for every browser context, including the ones the session fixture opens by hand. */
export const CONTEXT: BrowserContextOptions = {
  baseURL: BASE,
  locale: "ru-RU",
  timezoneId: "Europe/Moscow",
  viewport: DESKTOP,
  permissions: ["camera"],
  acceptDownloads: true,
};
