FROM rust:1.75-slim
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release
EXPOSE 8080
CMD ["cargo", "run"]
