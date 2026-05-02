FROM rust:1.95 as builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release
RUN cp target/release/backend /app/gravity-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/gravity-server /usr/local/bin/
COPY backend/config.yaml /etc/gravity/config.yaml
EXPOSE 8080
CMD ["gravity-server"]
