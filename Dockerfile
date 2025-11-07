# Railway-optimized Dockerfile - Single stage build
FROM rust:1.83-bookworm

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN cargo build --release

# Copy binary to working directory and make executable  
RUN cp ./target/release/web-server-report ./web-server-report && \
    chmod +x ./web-server-report

# Setup runtime user
RUN useradd -ms /bin/bash appuser && \
    chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8000

# Environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    HOST="0.0.0.0" \
    PORT="8000"

# Start the application
CMD ["./web-server-report"]
