# Multi-stage Dockerfile for building a release Rust binary and packaging static assets
# Build stage
FROM rust:1.72 as builder
WORKDIR /usr/src/app

# Copy Cargo manifests and fetch dependencies to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() { println!(\"compile-stub\"); }" > src/main.rs
RUN cargo fetch

# Copy the rest of the source and assets
COPY . .

# Allow toggling sqlx offline mode at build time (Railway can pass a build-arg)
ARG SQLX_OFFLINE=1
ENV SQLX_OFFLINE=${SQLX_OFFLINE}

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
# Install ca-certificates and OpenSSL runtime (native-tls/sqlx need libssl)
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy the compiled binary and static assets
COPY --from=builder /usr/src/app/target/release/web-server-report /app/web-server-report
COPY --from=builder /usr/src/app/static /app/static
COPY --from=builder /usr/src/app/templates /app/templates

EXPOSE 8000

# Prefer Railway-provided PORT and bind to 0.0.0.0 by default via HOST env
ENV RUST_LOG=info
ENV HOST=0.0.0.0

CMD ["/app/web-server-report"]
