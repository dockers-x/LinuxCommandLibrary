# Multi-stage build for Linux Command Library

# Stage 1: Download database
FROM alpine:latest AS database-downloader
RUN apk add --no-cache wget
WORKDIR /data
RUN wget https://github.com/SimonSchubert/LinuxCommandLibrary/raw/master/assets/database.db -O database.db

# Stage 2: Build the Rust application
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual application
RUN cargo build --release

# Stage 3: Runtime image
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache sqlite-libs libgcc

WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/LinuxCommandLibrary /app/LinuxCommandLibrary

# Copy the database from database-downloader stage
COPY --from=database-downloader /data/database.db /app/database.db

# Environment variables
ENV DATABASE_PATH=/app/database.db
ENV SERVER_ADDR=0.0.0.0:8080
ENV ENABLE_CORS=true
ENV RUST_LOG=info

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Run the application
CMD ["/app/LinuxCommandLibrary"]
