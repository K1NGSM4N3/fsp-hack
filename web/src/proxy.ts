import { NextResponse, type NextRequest } from "next/server";

/**
 * Only a convenience redirect for signed-out visitors; authorization is enforced by the API.
 * `fsp_session` lives as long as the refresh token, unlike the 15-minute `fsp_at`.
 */
export function proxy(request: NextRequest) {
  if (request.cookies.has("fsp_session")) return NextResponse.next();
  const url = request.nextUrl.clone();
  url.pathname = "/login";
  url.search = `?next=${encodeURIComponent(request.nextUrl.pathname + request.nextUrl.search)}`;
  return NextResponse.redirect(url);
}

export const config = {
  matcher: ["/candidate/:path*", "/employer/:path*"],
};
