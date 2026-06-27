FROM rust:1-bookworm AS backend-build
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-build /app/backend/target/release/backend /usr/local/bin/atlas-backend
ENV ATLAS_BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["atlas-backend"]
