# Use official Rust image with specific version that works
FROM rust:1.70-slim as builder

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies - this layer will be cached
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY static ./static
COPY templates ./templates

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary and assets
COPY --from=builder /app/target/release/web-server-report .
COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates

# Create non-root user
RUN useradd -r -u 1001 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8000

CMD ["./web-server-report"]
