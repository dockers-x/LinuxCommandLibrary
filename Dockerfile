# Use official fixed Rust version image as build environment
FROM rust:bookworm AS builder

# Set working directory
WORKDIR /app

# Install necessary build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Download database file
RUN wget https://github.com/SimonSchubert/LinuxCommandLibrary/raw/master/assets/database.db -O /tmp/database.db

# Copy Cargo.toml and Cargo.lock to working directory
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies (create dummy main.rs to cache dependencies)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build release version
RUN cargo build --release

# Create minimal runtime image
FROM debian:bookworm-slim

# Set necessary environment variables
ENV LOG_LEVEL=info
ENV PORT=8080
ENV DATABASE_PATH=/app/database.db
ENV SERVER_ADDR=0.0.0.0:8080
ENV ENABLE_CORS=true
ENV RUST_LOG=info

# Set working directory
WORKDIR /app

# Install necessary runtime libraries
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl3 \
    libsqlite3-0 \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Copy executable from build stage to this image
COPY --from=builder /app/target/release/LinuxCommandLibrary /app/LinuxCommandLibrary

# Copy database file to runtime environment
COPY --from=builder /tmp/database.db /app/database.db

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Run executable
CMD ["./LinuxCommandLibrary"]