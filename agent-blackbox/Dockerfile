FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/upmarto /app/upmarto

ENV APP_HOST=0.0.0.0
ENV APP_PORT=8080
ENV DATABASE_PATH=/data/events.log
ENV SQLITE_PATH=/data/metadata.db

VOLUME ["/data"]
EXPOSE 8080

CMD ["/app/upmarto"]
