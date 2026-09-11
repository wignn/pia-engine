# -- Stage 1: Install dependencies & build --
FROM oven/bun:1-alpine AS builder
WORKDIR /app

# Copy admin package files as standalone application
COPY apps/admin/package.json ./package.json

# Install dependencies standalone
RUN bun install

# Copy admin app sources
COPY apps/admin/app ./app
COPY apps/admin/components ./components
COPY apps/admin/lib ./lib
COPY apps/admin/public ./public
COPY apps/admin/tsconfig.json ./tsconfig.json
COPY apps/admin/next.config.ts ./next.config.ts
COPY apps/admin/postcss.config.mjs ./postcss.config.mjs

# Build Next.js
ENV NEXT_TELEMETRY_DISABLED=1
RUN bun run build

# -- Stage 2: Production runtime --
FROM oven/bun:1-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000

# Copy built artifacts
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/public ./public

EXPOSE 3000
CMD ["bun", "run", "start"]
