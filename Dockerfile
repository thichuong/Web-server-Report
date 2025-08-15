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

# Copy the binary and assets with proper names
COPY --from=builder /app/target/release/web-server-report ./web-server-report
# Copy the entire reorganized `crypto_dashboard` tree into the runtime image.
# Using a single path keeps the layout consistent with the repo move and
# simplifies runtime file resolution (server expects `crypto_dashboard/...`).
# If you need legacy `/static` or `/templates` paths inside the image for
# backward compatibility, we can add copies or symlinks here — currently the
# server serves `crypto_dashboard/assets` and loads templates from
# `crypto_dashboard/templates`.
COPY --from=builder /app/crypto_dashboard ./crypto_dashboard

# Set proper permissions
RUN chmod +x ./web-server-report

# Create non-root user
RUN useradd -r -u 1001 -m appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8000

CMD ["./web-server-report"]
