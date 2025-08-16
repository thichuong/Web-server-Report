# Use Ubuntu for better compatibility
FROM ubuntu:22.04 as builder

# Avoid prompts from apt
ENV DEBIAN_FRONTEND=noninteractive

# Install Rust and build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main for dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && rm -rf src target/release/deps/web_server_report*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary and required assets
COPY --from=builder /app/target/release/web-server-report ./web-server-report

# Copy the new dashboard structure
COPY --from=builder /app/dashboards ./dashboards

# Copy shared components and assets
COPY --from=builder /app/shared_components ./shared_components
COPY --from=builder /app/shared_assets ./shared_assets

# Set proper permissions
RUN chmod +x ./web-server-report

# Create non-root user
RUN useradd -r -u 1001 -m appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8000

# Health check for WebSocket and HTTP endpoints
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Environment variables documentation
ENV DATABASE_URL="" \
    TAAPI_SECRET="" \
    REDIS_URL="redis://localhost:6379" \
    HOST="0.0.0.0" \
    PORT="8000"

CMD ["./web-server-report"]
