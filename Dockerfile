# Use a more lightweight Rust image
FROM rust:1.75-slim-bookworm as builder

# Install minimal build dependencies in a single layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy source file to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies first (this layer will be cached)
RUN cargo build --release && rm -rf src target/release/deps/web_server_report*

# Copy the actual source code
COPY src ./src

# Copy static files and templates (needed for build verification)
COPY static ./static
COPY templates ./templates

# Build the application in release mode with optimizations
RUN CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build --release

# Use distroless for minimal runtime footprint
FROM gcr.io/distroless/cc-debian12

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/web-server-report /app/

# Copy static files and templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/templates /app/templates

# Set working directory
WORKDIR /app

# Expose the port
EXPOSE 8000

# Run the application
CMD ["./web-server-report"]
