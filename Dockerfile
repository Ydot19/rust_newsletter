FROM docker.io/library/rust:1.81 as builder

# Install cross-compilation tools
RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Add ARM64 target
RUN rustup target add aarch64-unknown-linux-gnu

# Set the working directory in the container
WORKDIR /app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release --target aarch64-unknown-linux-gnu

# use the latest bookworm to align with the version of rusn
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/aarch64-unknown-linux-gnu/release/newsletter_service /usr/local/bin/newsletter_service

# Expose port 8081
EXPOSE 8081

# Run the binary
CMD ["newsletter_service"]
