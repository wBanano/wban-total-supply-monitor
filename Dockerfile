FROM rust:1.74-alpine3.18 AS builder

RUN apk add --no-cache musl-dev openssl1.1-compat-dev openssl1.1-compat-libs-static
ENV OPENSSL_STATIC=1 OPENSSL_DIR=/usr

WORKDIR /app

# Add our source code.
ADD Cargo.toml Cargo.lock ./
ADD src ./src
ADD res ./res

# Build our application.
RUN cargo build --release

# Now, we need to build our _real_ Docker container, copying in `wban-hot-wallet-monitor`.
FROM alpine:latest
RUN apk --no-cache add ca-certificates
COPY --from=builder \
    /app/target/release/wban-total-supply-monitor \
    /usr/local/bin/
CMD /usr/local/bin/wban-total-supply-monitor