# Harmonis Prime — Benchmark Methodology (M2.7.14)

## 1. Benchmark Philosophy

Harmonis Prime follows the SAT Competition artifact evaluation standard:
- **Deterministic execution**: Identical seeds, identical results
- **Reproducible environments**: Documented hardware, OS, compiler
- **Standardized metrics**: PAR-2, solved instances, peak memory, proof validity
- **Baseline comparison**: Against Kissat, CaDiCaL, and historical self-tags

## 2. Execution Environment

| Attribute | Specification |
|-----------|---------------|
| OS | Linux x86_64 (primary), Windows 10/11 (secondary) |
| Rust Version | 1.96.0 (pinned via `rust-toolchain.toml`) |
| Profile | `release` (optimized) for competition submission |
| CPU | 8 physical cores (documented in `PERFORMANCE.md`) |
| Memory | 6.4 GB available (documented baseline) |
| Timeout | 300 seconds wall-clock per instance (default) |

## 3. Benchmark Runner CLI

```bash
# Batch execution with JSON export
cargo run --bin benchmark_runner -- --input-dir ./benchmarks/cnf --format json --output-dir ./results

# With SQLite version history recording
cargo run --bin benchmark_runner -- --input-dir ./benchmarks/cnf --format csv --db-path ./history.db --git-tag v6.2.0-M2.7.14

# Baseline comparison against previous tag
cargo run --bin benchmark_runner -- --input-dir ./benchmarks/cnf --baseline-tag v6.2.0-M2.7.13 --db-path ./history.db

# Type this exactly:
