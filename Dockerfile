FROM rust:1.97.1-slim AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM ubuntu:26.10

WORKDIR /app
COPY --from=builder /app/target/release/MongoToolsCLI /app/MongoToolsCLI

ENTRYPOINT ["/app/MongoToolsCLI"]
