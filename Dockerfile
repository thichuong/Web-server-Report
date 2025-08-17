# Railway-optimized Dockerfile với SSL/TLS fixes
FROM rust:1.83-slim-bookworm as builder

# Install dependencies cần thiết cho SSL/TLS
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Update CA certificates
RUN update-ca-certificates

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main for dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src target/release/deps/web_server_report*

# Copy source code
COPY . .

# Build the application với optimizations cho production
ENV RUSTFLAGS="-C target-cpu=x86-64-v3"
RUN cargo build --release

# Runtime stage với Google Distroless (minimal & secure)
FROM gcr.io/distroless/cc-debian12:latest

# Install minimal runtime dependencies với SSL support cho distroless
# Distroless đã có CA certificates và SSL libraries built-in

WORKDIR /app

# Copy the binary từ builder stage với executable permissions
COPY --from=builder --chmod=755 /app/target/release/web-server-report ./web-server-report

# Copy dashboard structure và assets
COPY --from=builder /app/dashboards ./dashboards
COPY --from=builder /app/shared_components ./shared_components
COPY --from=builder /app/shared_assets ./shared_assets

# Distroless images don't support creating users, already runs as non-root
# USER nonroot (built-in non-root user)

# Expose port 8000
EXPOSE 8000

# Health check endpoint - simplified for distroless (no curl available)
# Railway will handle external health checks
# HEALTHCHECK not available in distroless, Railway monitors via HTTP

# Environment variables với defaults
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    DATABASE_URL="" \
    TAAPI_SECRET="" \
    REDIS_URL="redis://localhost:6379" \
    HOST="0.0.0.0"

# Start command
CMD ["./web-server-report"]
