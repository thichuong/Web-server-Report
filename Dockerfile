# Build stage - use Alpine for smaller download
FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig

WORKDIR /app

# Copy all files
COPY . .

# Build for musl (static linking - no runtime deps needed)
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release

# Runtime stage - minimal distroless image
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

# Copy the binary and assets
COPY --from=builder /app/target/release/web-server-report .
COPY --from=builder /app/static ./static
COPY --from=builder /app/templates ./templates

EXPOSE 8000

CMD ["./web-server-report"]
