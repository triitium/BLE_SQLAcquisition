# Stage 1: Build Rust project
FROM rust:1.90.0-slim-bullseye AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    libdbus-1-dev \
    libglib2.0-dev \
    libudev-dev \
    bluez \
    dbus \
    dbus-user-session \
    bluetooth \
    libbluetooth-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /srv/proxy

# Copy Cargo manifests first (cache dependencies)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src/*

# Copy full source and build
COPY ./src ./src
RUN cargo build --release

# Stage 2: Runtime image (Ubuntu 24.04)
FROM ubuntu:24.04

# Install runtime dependencies for BLE and Postgres
RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates libdbus-1-dev bluez libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /usr/src/app/target/release/bleproxy /usr/local/bin/ble_proxy

# Entrypoint
CMD ["ble_proxy"]
