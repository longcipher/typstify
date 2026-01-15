# Stage 1: Build typstify binary
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --package typstify

# Stage 2: Runtime environment
FROM gcr.io/distroless/cc-debian12

# Install required runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy built binary from builder stage
COPY --from=builder /usr/src/app/target/release/typstify /usr/local/bin/typstify

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set working directory (GitHub Action mounts repo here)
WORKDIR /github/workspace

# Use entrypoint script for flexible working directory support
ENTRYPOINT ["/entrypoint.sh"]

# Default command (can be overridden)
CMD ["build"]
