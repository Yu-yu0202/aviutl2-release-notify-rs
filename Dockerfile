ARG RUST_VERSION=1.97
ARG APP_NAME=aviutl2-release-notify-rs

FROM rust:1.97-alpine AS build
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git pkgconfig openssl-dev openssl-libs-static

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    SQLX_OFFLINE=true \
    cargo build --locked --release && \
    cp target/release/${APP_NAME} /bin/server

FROM debian:trixie-slim AS runtime

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build --chown=65534:65534 /bin/server /bin/

USER 65534:65534

CMD ["/bin/server"]
