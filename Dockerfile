FROM rust:1.85-slim AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM ubuntu:22.04

WORKDIR /app
COPY --from=builder /app/target/release/MongoToolsCLI /app/MongoToolsCLI

ENTRYPOINT ["/app/MongoToolsCLI"]
