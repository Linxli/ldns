# Multi-stage build for DNS TUI
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

# Copy manifests and source code
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Create minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
# Using a specific UID/GID for consistency
RUN groupadd -g 1000 tuiuser && \
    useradd -r -u 1000 -g tuiuser -m -s /sbin/nologin tuiuser

WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/ldnsTUI /app/ldnsTUI

# Change ownership to non-root user
RUN chown -R tuiuser:tuiuser /app

# Switch to non-root user
USER tuiuser

# Set environment variable for DNS API URL (can be overridden in docker-compose)
ENV DNS_API_URL=http://dns-server:8080

# Run the TUI application
# Note: TUI applications require a TTY, so run with docker compose run -it or docker run -it
CMD ["/app/ldnsTUI"]
