FROM oven/bun:1-alpine AS builder

WORKDIR /app

# Install dependencies first (for better caching)
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Copy source and build
COPY . .
RUN bun run build

# ---
FROM oven/bun:1-alpine

WORKDIR /app

# Copy built assets and dependencies
COPY --from=builder /app/build ./build
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000
ENV NODE_ENV=production

CMD ["bun", "run", "build/index.js"]
