# Build stage
FROM rust:1.75 as builder

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /usr/src/app/target/release/web-server-report .

# Copy static files and templates
COPY static ./static
COPY templates ./templates

# Create a non-root user
RUN useradd -r -u 1001 appuser && chown -R appuser:appuser /app
USER appuser

# Expose the port
EXPOSE 8000

# Run the binary
CMD ["./web-server-report"]
