# syntax=docker/dockerfile:1.7

# =====================================================================================
# ========== Stage 0: Base image for building Rust + musl + static OpenSSL ============
# =====================================================================================
FROM rust:1.87 AS base
WORKDIR /app
ENV SQLX_OFFLINE=true

# ---- Install required build tools (musl, gcc, cmake, git, zlib, etc.) ---------------
RUN apt-get update && \
    apt-get install -y \
      musl-tools \
      pkg-config \
      build-essential \
      curl \
      perl \
      make \
      gcc \
      cmake \
      git \
      perl \
      zlib1g-dev

# ---- Add musl target for static linking ---------------------------------------------
RUN rustup target add x86_64-unknown-linux-musl

# ---- Build and install static OpenSSL (musl-compatible) -----------------------------
ENV OPENSSL_VERSION=1.1.1u
ENV OPENSSL_DIR=/opt/openssl
ENV CFLAGS=-DOPENSSL_NO_SECURE_MEMORY

RUN curl -sSL https://www.openssl.org/source/openssl-$OPENSSL_VERSION.tar.gz | tar xz && \
    cd openssl-$OPENSSL_VERSION && \
    CC=musl-gcc ./Configure no-shared no-dso no-async no-zlib no-afalgeng \
        --prefix=$OPENSSL_DIR \
        --openssldir=$OPENSSL_DIR \
        linux-x86_64 && \
    make -j$(nproc) && make install_sw

# ---- Cargo environment variables for static OpenSSL --------------------------------
ENV OPENSSL_LIB_DIR=$OPENSSL_DIR/lib
ENV OPENSSL_INCLUDE_DIR=$OPENSSL_DIR/include
ENV OPENSSL_STATIC=1

# =====================================================================================
# ========== Stage 1: Build goose migration tool (Go) =================================
# =====================================================================================
FROM golang:1.23-alpine AS goose
# ---- Install Goose CLI --------------------------------------------------------------
RUN go install github.com/pressly/goose/v3/cmd/goose@latest



# =====================================================================================
# ========== Stage 2: Build Rust project with compiled dependency cache ===============
# =====================================================================================
FROM base AS builder
WORKDIR /app

# 1) Только манифесты
COPY Cargo.toml Cargo.lock ./

# 2) "Заглушка" для компиляции зависимостей
#    (собираем минимальный crate, чтобы rustc скомпилил deps в /app/target)
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# 3) Компилим deps (и сохраняем кэш registry/git + target между билдами)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-musl --bin revelcy-backend-api

# 4) Теперь копируем реальные исходники
#    (важно: после этого меняется слой при правках кода)
RUN rm -rf src
COPY . .

# 5) Финальная сборка — deps уже в кеше, пересоберётся в основном твой код
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --target x86_64-unknown-linux-musl --bin revelcy-backend-api



# =====================================================================================
# ========== Stage 3: Final minimal runtime image (Alpine) ============================
# =====================================================================================
FROM alpine:3.20

# ---- Install PostgreSQL client to allow pg_isready ----------------------------------
RUN apk add --no-cache postgresql-client

# ---- Copy compiled Rust binary -------------------------------------------------------
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/revelcy-backend-api /usr/local/bin/app

# ---- Copy goose migration tool -------------------------------------------------------
COPY --from=goose /go/bin/goose /usr/local/bin/goose

# ---- Copy migrations and entrypoint -------------------------------------------------
COPY migrations /migrations
COPY --chmod=755 entrypoint.sh /entrypoint.sh

# ---- Convert CRLF → LF (Windows-safe) ----------------------------------------------
RUN sed -i 's/\r$//' /entrypoint.sh

ENV PORT=8080
ENV RUST_LOG=info

ENTRYPOINT ["/entrypoint.sh"]
