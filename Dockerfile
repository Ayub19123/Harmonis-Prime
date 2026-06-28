FROM rust:1.96-slim-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --bin sat_solver --no-default-features

FROM debian:bookworm-slim
WORKDIR /solver
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/sat_solver .
ENTRYPOINT ["./sat_solver"]
