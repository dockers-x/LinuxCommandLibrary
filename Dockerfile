# Multi-stage build for optimized Docker image
FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Download database file with retry logic
RUN for i in 1 2 3 4 5; do \
    echo "Attempt $i to download database..." && \
    curl -L --retry 5 --retry-delay 3 --retry-max-time 300 --max-time 300 \
      --connect-timeout 30 -o /tmp/database.db \
      "https://raw.githubusercontent.com/SimonSchubert/LinuxCommandLibrary/master/assets/database.db" \
      && break || sleep 10; \
    done && \
    test -f /tmp/database.db && \
    ls -lh /tmp/database.db

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user and data directory
RUN useradd -r -s /bin/false -m -d /app appuser && \
    mkdir -p /app/data && \
    chown -R appuser:appuser /app

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/LinuxCommandLibrary ./LinuxCommandLibrary

# Copy database file to runtime environment
COPY --from=builder /tmp/database.db ./database.db

# Make binary executable
RUN chmod +x ./LinuxCommandLibrary

# Set environment variables with defaults
ENV RUST_LOG=info
ENV DATABASE_PATH=/app/database.db
ENV SERVER_ADDR=0.0.0.0:8080
ENV ENABLE_CORS=true

# Change ownership to app user
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./LinuxCommandLibrary"]