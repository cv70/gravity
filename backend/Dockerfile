# Build stage
FROM rust:1.75-slim as builder
WORKDIR /app

# Copy manifests first for dependency caching
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create a dummy main.rs for dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY backend/src ./src

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim
WORKDIR /app

# Install CA certificates for HTTPS
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/backend ./backend

# Expose port
EXPOSE 8080

# Run the binary directly
CMD ["./backend"]