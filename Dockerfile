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

CMD ["./web-server-report"]
