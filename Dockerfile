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
FROM golang:1.25-alpine AS goose
# ---- Install Goose CLI --------------------------------------------------------------
RUN go install github.com/pressly/goose/v3/cmd/goose@latest



# =====================================================================================
# ========== Stage 2: Build Rust project with caches ==================================
# =====================================================================================
FROM base AS builder
WORKDIR /app

# ---- Step 1: Copy only Cargo manifests (to enable dependency caching) ---------------
# If you have a workspace, ALSO copy Cargo.toml for each crate here.
COPY Cargo.toml Cargo.lock ./

# ---- Step 2: Download all dependencies without building the project ----------------
# This step is cached until Cargo.toml/Cargo.lock changes.
RUN cargo fetch --target x86_64-unknown-linux-musl

# ---- Step 3: Copy project sources ---------------------------------------------------
COPY . .

# ---- Step 4: Build final binary (static musl build) ---------------------------------
RUN cargo build --release --target x86_64-unknown-linux-musl --bin revelcy-backend-api



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
