FROM docker.io/rust:1.81 AS builder

COPY . /app

WORKDIR /app

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends --assume-yes \
    libprotobuf-dev \
    build-essential \
    clang-tools-16 \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    && apt-get clean

RUN cargo build --release

FROM docker.io/debian:bookworm-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates curl build-essential

WORKDIR /app

# copy the runtime files
COPY --from=builder /app/target/release/namada-monitoring /app/monitoring

ENTRYPOINT ["/app/monitoring"]