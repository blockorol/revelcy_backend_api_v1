# ===== Base stage =====
FROM rust:1.87 as base
WORKDIR /app
ENV SQLX_OFFLINE=true

# Установим нужные утилиты
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

# Добавляем musl target
RUN rustup target add x86_64-unknown-linux-musl

# Соберём и установим OpenSSL вручную
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

# Прописать переменные для cargo
ENV OPENSSL_LIB_DIR=$OPENSSL_DIR/lib
ENV OPENSSL_INCLUDE_DIR=$OPENSSL_DIR/include
ENV OPENSSL_STATIC=1

# Копируем исходники
COPY . .

# ===== Stage 1: Build goose =====
FROM golang:1.23-alpine AS goose
RUN go install github.com/pressly/goose/v3/cmd/goose@latest

# ===== Stage 2: Build main binary =====
FROM base as builder
RUN cargo build --release --target x86_64-unknown-linux-musl --bin revelcy-backend-api

# ===== Final minimal image =====
FROM alpine:3.20

# нужен клиент для pg_isready
RUN apk add --no-cache postgresql-client

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/revelcy-backend-api /usr/local/bin/app
COPY --from=goose /go/bin/goose /usr/local/bin/goose
COPY migrations /migrations
COPY --chmod=755 entrypoint.sh /entrypoint.sh

ENV PORT=8080
ENV RUST_LOG=info

ENTRYPOINT ["/entrypoint.sh"]
