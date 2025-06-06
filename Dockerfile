FROM rust:alpine AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    ca-certificates

RUN rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
COPY config.yaml ./
COPY migrations ./migrations
COPY src src

ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

RUN apk --no-cache add ca-certificates tzdata

WORKDIR /app

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/utotool-rust /usr/local/bin/utotool-rust
COPY --from=builder /usr/src/app/config.yaml ./config.yaml

CMD ["utotool-rust"]
