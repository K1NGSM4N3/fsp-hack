import { expect } from "@playwright/test";
import type { Session } from "./session";

/** The seed promise a spec relies on: an adult, categorized, published seeker who is looking for work. */
export async function expectSearchableSeeker(s: Session, category = { specialization: "backend", grade: "middle" }): Promise<void> {
  expect(s.me.role, s.email).toBe("candidate");
  expect(s.me.candidate?.category, `${s.email}: seeded category`).toEqual(category);
  expect(s.me.candidate?.age.bracket, `${s.email}: adult`).toBe("adult");
  const privacy = await s.api.get<{ visible: boolean; search_status: string }>("/candidate/privacy");
  expect(privacy.visible, `${s.email}: published`).toBe(true);
  expect(privacy.search_status, `${s.email}: looking for work`).not.toBe("closed");
}
