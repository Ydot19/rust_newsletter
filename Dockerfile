FROM docker.io/library/rust:1.83 as builder

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y \
    gcc-aarch64-linux-gnu \
    libpq-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add aarch64-unknown-linux-gnu

WORKDIR /app
COPY . .

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

RUN cargo build --release --target aarch64-unknown-linux-gnu

# use the latest bookworm to align with the version of rust
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
