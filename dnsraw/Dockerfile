# Multi-stage build for DNS server
# Stage 1: Build the application
# Using nightly because project uses edition2024
FROM rustlang/rust:nightly-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests first for better layer caching
# This allows Docker to cache dependencies if only source code changes
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies first (optimization)
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release && \
    rm -rf src target/release/deps/dnsraw*

# Copy actual source code
COPY src ./src

# Build the actual application (tests not needed for production)
RUN cargo build --release

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies including curl for healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
# Using a specific UID/GID for consistency
RUN groupadd -g 1000 dnsuser && \
    useradd -r -u 1000 -g dnsuser -m -s /sbin/nologin dnsuser

WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/dnsraw /app/dnsraw

# Copy default blocklist if it exists
COPY dnsblock.txt /app/dnsblock.txt

# Create directory for data persistence
RUN mkdir -p /app/data && \
    chown -R dnsuser:dnsuser /app

# Switch to non-root user
# Note: Port 53 requires privileged access, handle via docker-compose
USER dnsuser

# Expose DNS port (UDP) and API port (TCP)
EXPOSE 53/udp
EXPOSE 8080/tcp

# Health check for the API endpoint
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the DNS server
CMD ["/app/dnsraw"]
