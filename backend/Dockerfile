# Build stage
FROM rust:1.75-bookworm as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y libssl-dev pkg-config

# Copy cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release

# Now copy actual source code
COPY . .

# Touch main.rs to force rebuild
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim

WORKDIR /app

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/code_share_backend .

# Copy migrations
COPY --from=builder /app/migrations ./migrations

EXPOSE 8080

CMD ["./code_share_backend"]
