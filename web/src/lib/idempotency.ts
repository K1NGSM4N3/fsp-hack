/**
 * An `Idempotency-Key`: made once per form, so a second click or a retry after a lost response returns what the first
 * one created. `randomUUID` exists only in secure contexts; `getRandomValues` works everywhere.
 */
export function newIdempotencyKey(): string {
  const c: Crypto | undefined = globalThis.crypto;
  if (!c) return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  if (typeof c.randomUUID === "function") return c.randomUUID();
  return Array.from(c.getRandomValues(new Uint8Array(16)), (b) => b.toString(16).padStart(2, "0")).join("");
}
