"use client";

import { useEffect, useState } from "react";
import type { Me } from "@/api/types";

/** Same as `discard` in the API client: a `no-store` response must be read or cancelled to free its connection. */
function release(res: Response) {
  if (res.body && !res.bodyUsed) res.body.cancel().catch(() => {});
}

const ME = "/api/v1/me";
const REFRESH = "/api/v1/auth/refresh";
const init: RequestInit = { credentials: "include", headers: { Accept: "application/json" } };

/** Same contract as the API client (one refresh, then one retry) without pulling the client into marketing pages. */
async function fetchMe(): Promise<Me | null> {
  if (process.env.NEXT_PUBLIC_USE_MOCKS === "1") return (await import("@/api/endpoints")).getMe();
  let res = await fetch(ME, init);
  if (res.status === 401) {
    release(res);
    const refreshed = await fetch(REFRESH, { ...init, method: "POST" });
    release(refreshed);
    if (!refreshed.ok) return null;
    res = await fetch(ME, init);
  }
  if (res.ok) return (await res.json()) as Me;
  release(res);
  return null;
}

/** Who is signed in, for the public header only; asked again whenever the public layout mounts (e.g. after logging in). */
export function useSessionRole(): Me["role"] | null {
  const [role, setRole] = useState<Me["role"] | null>(null);
  useEffect(() => {
    let live = true;
    void fetchMe().catch(() => null).then((me) => {
      if (live) setRole(me?.role ?? null);
    });
    return () => {
      live = false;
    };
  }, []);
  return role;
}
