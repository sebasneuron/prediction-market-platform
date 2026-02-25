# -----  Chef Stage -----
FROM rust:1.88-bullseye AS chef

WORKDIR /app
ARG DATABASE_URL

RUN cargo install cargo-chef
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

# ----- Builder Stage ----

FROM rust:1.88-bullseye AS builder

# install necessary tools for lib rdkafka and protobuf compilation
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    zlib1g-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN cargo install cargo-chef

COPY --from=chef /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json
ENV DATABASE_URL=$DATABASE_URL
ENV SQLX_OFFLINE=true

COPY . .

RUN cargo build --release --workspace

# ---- Runtime stage: Final, small image --

FROM debian:bullseye-slim AS runtime

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/grpc-service .
COPY --from=builder /app/target/release/order-service .
COPY --from=builder /app/target/release/service-api .
COPY --from=builder /app/target/release/websocket-service .

CMD ['./grpc-service']