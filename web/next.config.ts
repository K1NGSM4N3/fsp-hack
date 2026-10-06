import type { NextConfig } from "next";

// In Docker, Caddy serves /api/* directly from the Rust API. In local dev this
// rewrite keeps the browser on a single origin so the API's httpOnly cookies work.
const apiProxyUrl = process.env.API_PROXY_URL ?? "http://localhost:8000";

const nextConfig: NextConfig = {
  output: "standalone",
  poweredByHeader: false,
  // Caddy compresses every response (zstd, then gzip); compressing here as well would hand it gzip it cannot improve.
  compress: false,
  // Link previews of vacancies are rendered on request and read these fonts from disk.
  outputFileTracingIncludes: { "/vacancies/*": ["./assets/og/*.ttf"] },
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: `${apiProxyUrl}/api/:path*`,
      },
    ];
  },
};

export default nextConfig;
