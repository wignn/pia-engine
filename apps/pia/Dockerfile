# -- Stage 1: Install dependencies & build --
FROM oven/bun:1-alpine AS builder
WORKDIR /app

# Copy workspace root files needed for install
COPY package.json ./
COPY apps/pia/package.json apps/pia/package.json
COPY apps/pia/bun.lock apps/pia/bun.lock

# Install deps
WORKDIR /app/apps/pia
RUN bun install --frozen-lockfile || bun install

# Copy PIA app source
COPY apps/pia /app/apps/pia

# Build Next.js
WORKDIR /app/apps/pia
ENV NEXT_TELEMETRY_DISABLED=1
RUN bun run build

# -- Stage 2: Production runtime --
FROM oven/bun:1-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000

# Copy built artifacts
COPY --from=builder /app/apps/pia/.next ./.next
COPY --from=builder /app/apps/pia/node_modules ./node_modules
COPY --from=builder /app/apps/pia/package.json ./package.json
COPY --from=builder /app/apps/pia/public ./public

EXPOSE 3000
CMD ["bun", "run", "start"]
