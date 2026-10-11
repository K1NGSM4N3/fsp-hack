import { checkClockAfterRefresh, expiry } from "./session-hint";

const URL = "/api/v1/auth/refresh";
const LOCK = "fsp-auth-refresh";

let inFlight: Promise<boolean> | null = null;

/**
 * The one way the page refreshes its session. The API rotates the refresh token on every use and treats a rotated
 * token presented again as stolen: it revokes the whole session family, so the user is logged out at the next refresh.
 * Two refreshes holding the same cookie — the public header and the API client on one page, or two open tabs — are
 * exactly that. Callers on a page share one request; tabs queue on a Web Lock, and a tab that waited while another
 * one refreshed (the `fsp_at_exp` hint moved) takes that result instead of spending the cookie again.
 */
export function refreshSession(): Promise<boolean> {
  inFlight ??= exclusive().finally(() => {
    inFlight = null;
  });
  return inFlight;
}

async function exclusive(): Promise<boolean> {
  const seen = expiry();
  const locks = typeof navigator === "undefined" ? undefined : navigator.locks;
  // Web Locks exist only in secure contexts; plain http on a LAN address falls back to the per-page guard.
  if (!locks) return post();
  return await locks.request(LOCK, async () => refreshedElsewhere(seen) || (await post()));
}

/** Another tab refreshed while this one waited for the lock: the cookies already hold a new session. */
function refreshedElsewhere(seen: number | null): boolean {
  const now = expiry();
  return now !== null && now !== seen;
}

async function post(): Promise<boolean> {
  try {
    const res = await fetch(URL, { method: "POST", credentials: "include", headers: { Accept: "application/json" } });
    // A `no-store` response must be read or cancelled to free its connection.
    if (res.body && !res.bodyUsed) res.body.cancel().catch(() => {});
    if (res.ok) checkClockAfterRefresh();
    return res.ok;
  } catch {
    return false;
  }
}
