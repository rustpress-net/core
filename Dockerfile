# Build stage
# Using rust:latest for edition2024 support required by dependencies
FROM rust:slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY plugins ./plugins

# Set RUSTFLAGS to allow warnings for MVP release
ENV RUSTFLAGS="-Aunused -Amismatched_lifetime_syntaxes -Adependency_on_unit_never_type_fallback -Aunused_comparisons"

# Build all release binaries
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies including curl for healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy all binaries from builder
COPY --from=builder /app/target/release/rustpress /usr/local/bin/rustpress
COPY --from=builder /app/target/release/rustpress-cli /usr/local/bin/rustpress-cli
COPY --from=builder /app/target/release/rustpress-migrate /usr/local/bin/rustpress-migrate

# Copy default configuration and assets
COPY config ./config
COPY themes ./themes
COPY public ./public
COPY migrations ./migrations

# Create necessary directories
RUN mkdir -p /app/uploads /app/logs

# Set environment variables
ENV HOST=0.0.0.0
ENV PORT=8080
ENV RUST_LOG=info

# Expose the port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the server
CMD ["rustpress"]
