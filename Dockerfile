FROM rust:1-bookworm AS backend-build
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release

FROM node:20-bookworm-slim AS frontend-build
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend ./
RUN npm run build

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-build /app/backend/target/release/backend /usr/local/bin/atlas-backend
COPY --from=frontend-build /app/frontend/build /app/public
ENV ATLAS_BIND_ADDR=0.0.0.0:3000
ENV ATLAS_STATIC_DIR=/app/public
EXPOSE 3000
CMD ["atlas-backend"]
