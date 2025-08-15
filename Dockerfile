# Use the official Rust image as base
FROM rust:1.75 as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Copy static files and templates
COPY static ./static
COPY templates ./templates

# Build the application in release mode
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/web-server-report .

# Copy static files and templates
COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates

# Expose the port (Railway will set the PORT environment variable)
EXPOSE $PORT

# Run the application
CMD ["./web-server-report"]
