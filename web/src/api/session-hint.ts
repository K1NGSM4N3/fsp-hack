/**
 * The access token is an httpOnly cookie, so the page cannot see when it runs out. The API sets `fsp_at_exp` next to it:
 * the expiry in unix seconds and nothing else. A request that would carry an expired token refreshes first instead of
 * collecting a 401 and retrying.
 */
const COOKIE = /(?:^|;\s*)fsp_at_exp=(\d+)/;
/** Refresh this many seconds early: the request still has to reach the server. */
const MARGIN_SECS = 10;

let clockAhead = false;

function expiry(): number | null {
  if (typeof document === "undefined") return null;
  const m = COOKIE.exec(document.cookie);
  return m ? Number(m[1]) : null;
}

export function accessExpired(): boolean {
  if (clockAhead) return false;
  const exp = expiry();
  return exp !== null && exp - MARGIN_SECS <= Date.now() / 1000;
}

/** Called after a successful refresh: if the new token still looks expired, the device clock is off and the hint is ignored from then on. */
export function checkClockAfterRefresh() {
  if (accessExpired()) clockAhead = true;
}
