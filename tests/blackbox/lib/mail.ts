import { expect } from "@playwright/test";
import { MAILPIT } from "./env";

/** A letter as the recipient sees it in Mailpit, the users' inbox of the stack. */
export interface Letter {
  id: string;
  subject: string;
  to: string[];
  created: number;
  text: string;
  html: string;
  headers: Record<string, string[]>;
}

interface Summary {
  ID: string;
  Subject: string;
  Created: string;
  To: { Address: string }[] | null;
}

const nbsp = (s: string) => s.replace(/[  ]/g, " ");

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${MAILPIT}${path}`);
  if (!res.ok) throw new Error(`Mailpit ${path}: ${res.status}`);
  return (await res.json()) as T;
}

async function open(summary: Summary): Promise<Letter> {
  const full = await getJson<{ Subject: string; Text?: string; HTML?: string }>(`/api/v1/message/${summary.ID}`);
  const headers = await getJson<Record<string, string[]>>(`/api/v1/message/${summary.ID}/headers`);
  return {
    id: summary.ID,
    subject: nbsp(full.Subject),
    to: (summary.To ?? []).map((t) => t.Address.toLowerCase()),
    created: Date.parse(summary.Created),
    text: nbsp(full.Text ?? ""),
    html: nbsp(full.HTML ?? ""),
    headers,
  };
}

/** Every letter to `to`, newest first. */
export async function lettersTo(to: string): Promise<Letter[]> {
  const query = encodeURIComponent(`to:"${to}"`);
  const list = await getJson<{ messages: Summary[] | null }>(`/api/v1/search?query=${query}&limit=50`);
  return Promise.all((list.messages ?? []).map(open));
}

export interface MailQuery {
  /** Only letters created at or after this moment (ms since epoch). */
  since?: number;
  subject?: RegExp;
  /** Extra predicate over the opened letter. */
  where?: (letter: Letter) => boolean;
  timeout?: number;
}

/** Waits for a letter to `to` that matches; the stack sends mail asynchronously. */
export async function waitForLetter(to: string, query: MailQuery = {}): Promise<Letter> {
  let found: Letter | undefined;
  await expect
    .poll(
      async () => {
        const letters = await lettersTo(to);
        found = letters.find(
          (l) =>
            (query.since === undefined || l.created >= query.since - 1000) &&
            (query.subject === undefined || query.subject.test(l.subject)) &&
            (query.where === undefined || query.where(l)),
        );
        return found !== undefined;
      },
      { message: `letter to ${to} ${query.subject ?? ""}`, timeout: query.timeout ?? 30_000, intervals: [500, 1000] },
    )
    .toBe(true);
  return found as Letter;
}

/** The first link in the letter's text that matches `pattern` (absolute URL). */
export function linkIn(letter: Letter, pattern: RegExp): string {
  const links = letter.text.match(/https?:\/\/[^\s<>"')]+/g) ?? [];
  const link = links.find((l) => pattern.test(l));
  if (!link) throw new Error(`no link matching ${pattern} in «${letter.subject}»:\n${letter.text}`);
  return link;
}

/** All links in the letter's text that match `pattern`, in order. */
export function linksIn(letter: Letter, pattern: RegExp): string[] {
  return (letter.text.match(/https?:\/\/[^\s<>"')]+/g) ?? []).filter((l) => pattern.test(l));
}

export function header(letter: Letter, name: string): string | undefined {
  const key = Object.keys(letter.headers).find((k) => k.toLowerCase() === name.toLowerCase());
  return key ? letter.headers[key]?.[0] : undefined;
}
