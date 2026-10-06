import { cookies } from "next/headers";

/**
 * Whether the browser holds a session at all, answered from the httpOnly `fsp_session` marker without touching the
 * API. The public header asks this first, so signed-out visitors never trigger a 401 from `/me` and `/auth/refresh`.
 */
export async function GET() {
  const signedIn = (await cookies()).has("fsp_session");
  return Response.json({ signedIn }, { headers: { "Cache-Control": "private, no-store" } });
}
