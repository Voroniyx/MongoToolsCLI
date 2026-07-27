FROM rust:1.97.1-slim AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/MongoToolsCLI /app/MongoToolsCLI

ENTRYPOINT ["/app/MongoToolsCLI"]