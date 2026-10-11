import { randomBytes } from "node:crypto";
import type { TestInfo } from "@playwright/test";

/** Demo accounts the seed creates on an empty database (docs/guide.md, section 2.2). */
export const DEMO = {
  employer: "employer@demo.fsp",
  moderator: "moderator@demo.fsp",
  candidate: "candidate@demo.fsp",
  minor: "minor@demo.fsp",
  review: "review@demo.fsp",
} as const;

/** A unique address per test and call, so letters and accounts never collide between workers or runs. */
export function uniqueEmail(testInfo: TestInfo, tag: string): string {
  return `bb.${tag}.${testInfo.parallelIndex}.${Date.now().toString(36)}${randomBytes(3).toString("hex")}@example.ru`;
}

export const PASSWORD = "blackbox-secret-1";

/**
 * Seeded seekers with a category from the deterministic demo seed, one per spec, so specs that
 * invite, revoke or apply never touch each other's seeker. All are adult, published, searching
 * backend developers (Middle) without a ФСП link; each spec checks that before relying on it.
 */
export const SEEKERS = {
  invitation: "cand10@demo.fsp",
  notifications: "cand8@demo.fsp",
  ats: "cand21@demo.fsp",
  privacy: "cand32@demo.fsp",
  application: "cand5@demo.fsp",
} as const;

/** A seeded seeker whose refresh token the session spec spends on purpose; no other spec signs in as it. */
export const SESSION_SEEKER = "cand44@demo.fsp";

/** Seeded companies, one per spec that reads its own letters or quota. */
export const EMPLOYERS = {
  demo: "employer@demo.fsp",
  notifications: "hr@oblakostroy.example",
  application: "hr@medsignal.example",
  search: "career@shield-key.example",
  complaints: "hr@pixel-manufactura.example",
} as const;
