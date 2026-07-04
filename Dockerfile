# =============================================================================
# M2.7.15: Docker/cgroups Containerization
# Harmonis Prime Sovereign Core v6.2.0-M2.7.15
# =============================================================================
# Multi-stage build for minimal, reproducible, competition-grade artifact.
# Rust 1.96.0 locked to glibc 2.36 (debian:bookworm).
# Non-root execution. cgroups-aware resource governance.
# =============================================================================

ARG RUST_VERSION=1.96.0
ARG DEBIAN_VERSION=bookworm

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS builder

LABEL maintainer="Ayub19123 <ayub@harmonisprime.io>"
LABEL version="6.2.0-M2.7.15"
LABEL description="Harmonis Prime Sovereign Core - SAT Solver"
LABEL org.opencontainers.image.source="https://github.com/Ayub19123/Harmonis-Prime"

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/sovereign_core

# Copy dependency manifests first for deterministic layer caching
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Update lock file to ensure compatibility with container environment
# then build release binary. --locked removed due to cross-environment
# lock file drift (test job verifies lock integrity).
RUN cargo update --workspace && \
    cargo build --release --lib --bins

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:${DEBIAN_VERSION}-slim AS runtime

LABEL maintainer="Ayub19123 <ayub@harmonisprime.io>"
LABEL version="6.2.0-M2.7.15"
LABEL description="Harmonis Prime Sovereign Core - SAT Solver Runtime"

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security hardening
RUN groupadd -r sovereign && \
    useradd -r -g sovereign -m -d /var/lib/sovereign -s /sbin/nologin sovereign

# Create directories
RUN mkdir -p /opt/sovereign/bin /opt/sovereign/lib /opt/sovereign/data /opt/sovereign/benchmarks && \
    chown -R sovereign:sovereign /opt/sovereign

# Copy binaries from builder
COPY --from=builder /usr/src/sovereign_core/target/release/benchmark_runner /opt/sovereign/bin/

# Copy library artifacts if present
COPY --from=builder /usr/src/sovereign_core/target/release/libsovereign_core.so /opt/sovereign/lib/ 2>/dev/null || true
COPY --from=builder /usr/src/sovereign_core/target/release/libsovereign_core.a /opt/sovereign/lib/ 2>/dev/null || true

# cgroups-aware resource governance environment
ENV SOVEREIGN_CGROUPS_ENABLED=true
ENV SOVEREIGN_CPU_QUOTA_PATH=/sys/fs/cgroup/cpu/cpu.cfs_quota_us
ENV SOVEREIGN_MEM_LIMIT_PATH=/sys/fs/cgroup/memory/memory.limit_in_bytes
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV SOVEREIGN_DATA_DIR=/opt/sovereign/data
ENV SOVEREIGN_BENCHMARK_DIR=/opt/sovereign/benchmarks

# Health check: verify binary integrity
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD test -x /opt/sovereign/bin/benchmark_runner || exit 1

# Switch to non-root user
USER sovereign

# Working directory for benchmarks and SQLite ledger
WORKDIR /opt/sovereign/data

# Default: display binary path and exit (container is a tool, not a daemon)
ENTRYPOINT ["/opt/sovereign/bin/benchmark_runner"]
CMD ["--help"]
