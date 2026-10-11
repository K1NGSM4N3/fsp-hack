import { expect, type Page } from "@playwright/test";
import type { Api } from "./api";
import { linkIn, waitForLetter } from "./mail";

export interface ConsentDocument {
  id: string;
  kind: string;
  version: number;
  title: string;
  required_for: string[];
  applies_to: string[];
}

async function currentConsents(api: Api): Promise<ConsentDocument[]> {
  return api.get<ConsentDocument[]>("/consents/current");
}

export interface Registration {
  email: string;
  password: string;
  role: "candidate" | "employer";
  company_name?: string;
  age?: "adult" | "minor";
  minor?: { birth_date: string; guardian: { full_name: string; email: string } };
}

/** The registration form's request, with every consent the role must give. */
export async function registrationBody(api: Api, r: Registration): Promise<Record<string, unknown>> {
  const docs = await currentConsents(api);
  return { ...r, consents: docs.filter((d) => d.required_for.includes(r.role)).map((d) => d.id) };
}

export async function register(api: Api, r: Registration): Promise<string> {
  const created = await api.post<{ user_id: string; verification_required: boolean }>("/auth/register", await registrationBody(api, r), 201);
  expect(created.verification_required).toBe(true);
  return created.user_id;
}

/** Opens the confirmation link from the letter, as the person would. */
export async function confirmEmail(page: Page, email: string, since?: number): Promise<void> {
  const letter = await waitForLetter(email, { subject: /Подтверждение email/, since });
  await page.goto(linkIn(letter, /\/verify-email\?token=/));
  await expect(page.getByRole("heading", { name: "Email подтверждён" })).toBeVisible();
}

export async function loginUI(page: Page, email: string, password: string): Promise<void> {
  await page.goto("/login");
  await page.getByRole("textbox", { name: "Email", exact: true }).fill(email);
  await page.getByRole("textbox", { name: "Пароль", exact: true }).fill(password);
  await page.locator("form").getByRole("button", { name: "Войти" }).click();
  await expect(page).not.toHaveURL(/\/login/);
}

/** A date `years` years and `days` days before today, YYYY-MM-DD. */
export function dateYearsAgo(years: number, days = 0): string {
  const d = new Date();
  d.setUTCFullYear(d.getUTCFullYear() - years);
  d.setUTCDate(d.getUTCDate() - days);
  return d.toISOString().slice(0, 10);
}

/** Ticks every consent the registration form asks for, once the list of documents has loaded. */
export async function acceptConsents(page: Page): Promise<void> {
  const boxes = page.getByRole("group", { name: "Согласия" }).getByRole("checkbox");
  await expect(boxes.first()).toBeVisible();
  for (const box of await boxes.all()) await box.check({ force: true });
}

/** Clicks the visible label of a radio card or checkbox: the inputs themselves are visually hidden. */
export async function choose(page: Page, text: string | RegExp): Promise<void> {
  await page.locator("label").filter({ hasText: text }).first().click();
}

export function invitationBody(candidateId: string, title: string, extra: Record<string, unknown> = {}) {
  return {
    candidate_id: candidateId,
    title,
    message: "Здравствуйте! Посмотрели ваш результат теста — давайте обсудим задачу и команду.",
    salary: { min: 280_000, max: 340_000 },
    work_format: "remote",
    contact: "@blackbox_hr",
    ...extra,
  };
}
