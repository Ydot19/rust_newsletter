# Remove the cross-compilation setup
FROM docker.io/library/rust:1.83 as builder

WORKDIR /app
COPY . .

RUN cargo build --release  # Build for native architecture

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Update the copy path
COPY --from=builder /app/target/release/newsletter_service .

EXPOSE 8081
CMD ["./newsletter_service"]
