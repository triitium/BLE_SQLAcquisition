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

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src/*

COPY ./src ./src
RUN cargo build --release

FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    libssl-dev ca-certificates libdbus-1-dev bluez libudev-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /srv/proxy/target/release/bleproxy /usr/local/bin/ble_proxy

CMD ["ble_proxy"]
