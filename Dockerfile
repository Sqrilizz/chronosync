FROM rust:bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin chronosync-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/chronosync-server /usr/local/bin/chronosync-server

RUN mkdir -p /data

ENV HOST=0.0.0.0
ENV PORT=8080
ENV DATABASE_URL="sqlite:///data/chronosync.db?mode=rwc"

EXPOSE 8080
CMD ["chronosync-server"]
