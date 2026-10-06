# syntax=docker/dockerfile:1
# Next.js standalone build; needs `output: "standalone"` in web/next.config.*.
# Context: repo root (docker compose -f deploy/compose.yml build).

FROM node:24-alpine AS deps
WORKDIR /app
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund

FROM node:24-alpine AS builder
WORKDIR /app
# next.config rewrites are resolved at build time, so the API address is needed here too.
ARG API_PROXY_URL=http://api:8000
# Public pages are prerendered, so canonical links, sitemap and OG image URLs take the site address at build time.
ARG SITE_URL=http://localhost:8080
ENV API_PROXY_URL=${API_PROXY_URL} \
    SITE_URL=${SITE_URL} \
    NEXT_TELEMETRY_DISABLED=1
COPY --from=deps /app/node_modules ./node_modules
COPY web/ ./
RUN mkdir -p public && npm run build

FROM node:24-alpine AS runner
WORKDIR /app
ARG SITE_URL=http://localhost:8080
ENV SITE_URL=${SITE_URL} \
    NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    HOSTNAME=0.0.0.0 \
    PORT=3000 \
    API_PROXY_URL=http://api:8000
RUN addgroup -S -g 1001 nodejs && adduser -S -u 1001 -G nodejs nextjs
COPY --from=builder --chown=nextjs:nodejs /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static
USER nextjs
EXPOSE 3000
CMD ["node", "server.js"]
