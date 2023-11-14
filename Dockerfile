FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM node:20 AS node_builder
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml turbo.json ./
COPY packages ./packages
COPY examples ./examples
RUN corepack enable
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.pnpm-store \
      pnpm install --frozen-lockfile --prefer-offline
ENV NODE_ENV=production
RUN pnpm run build --scope=@pointguard/web-ui

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer! Uses Buildkit to cache dependencies
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
COPY --from=node_builder /app/packages/web-ui/dist ./packages/web-ui/dist
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin pointguard_cli && \
    cp /app/target/release/pointguard_cli /app/pointguard_cli

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/pointguard_cli.exe /usr/local/bin
ENTRYPOINT ["/usr/local/bin/pointguard_cli.exe"]
