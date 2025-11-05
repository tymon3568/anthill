# Use Node.js 20 Alpine for smaller image size
FROM node:20-alpine AS base

# Install dependencies only when needed
FROM base AS deps
# Check https://github.com/nodejs/docker-node/tree/b4117f9333da4138b03a546ec926ef50a31506c3#nodealpine to understand why libc6-compat might be needed.
RUN apk add --no-cache libc6-compat
WORKDIR /app

# Install dependencies based on the preferred package manager
COPY package.json bun.lock ./
RUN \
  if [ -f bun.lock ]; then bun install --frozen-lockfile; \
  else echo "Lockfile not found." && exit 1; \
  fi

# Rebuild the source code only when needed
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# SvelteKit telemetry can be disabled if needed
# Learn more: https://kit.svelte.dev/docs/cli#svelte-kit-sync
# Uncomment the following line in case you want to disable telemetry during the build.
# ENV SVELTEKIT_TELEMETRY_DISABLED 1

RUN bun run build

# Production image, copy all the files and run next
FROM base AS runner
WORKDIR /app

ENV NODE_ENV production
# Uncomment the following line in case you want to disable telemetry during runtime.
# ENV SVELTEKIT_TELEMETRY_DISABLED 1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 sveltekit

COPY --from=builder /app/public ./public

# Set the correct permission for prerender cache
RUN mkdir .svelte-kit
RUN chown sveltekit:nodejs .svelte-kit

# Copy SvelteKit build output
COPY --from=builder --chown=sveltekit:nodejs /app/.svelte-kit/output ./.svelte-kit/output
COPY --from=builder --chown=sveltekit:nodejs /app/package.json ./package.json
COPY --from=builder --chown=sveltekit:nodejs /app/bun.lock ./bun.lock

USER sveltekit

EXPOSE 3000

ENV PORT 3000
# set hostname to localhost
ENV HOSTNAME "0.0.0.0"

# server.js is created by sveltekit automatically
CMD ["bun", "run", ".svelte-kit/output/server/index.js"]
