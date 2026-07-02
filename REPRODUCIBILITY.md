# Harmonis Prime — Reproducibility Guide (M2.7.14)

## 1. Environment Constraints

| Requirement | Version | Verification |
|-------------|---------|------------|
| Rust | 1.96.0 | `rustc --version` |
| Cargo | 1.96.0 | `cargo --version` |
| OS | Linux x86_64 | `uname -a` |
| Git | >= 2.30 | `git --version` |
| drat-trim | Latest (compiled from source) | `tools/drat-trim` |

## 2. Build Instructions

    # Clone repository
    git clone https://github.com/Ayub19123/Harmonis-Prime.git
    cd Harmonis-Prime

    # Verify tag
    git checkout v6.2.0-M2.7.14

    # Build release profile
    cargo build --release

    # Verify zero warnings
    cargo check --lib --bins 2>&1 | grep -E "^error|^warning" | wc -l
    # Expected: 0

## 3. Test Reproduction

    # Full test suite
    cargo test --lib

    # Expected output:
    # test result: ok. 215 passed; 0 failed; 8 ignored; 0 measured; 7 filtered out

## 4. Benchmark Reproduction

    # Create benchmark directory
    mkdir -p benchmarks/cnf

    # Add DIMACS .cnf files (see BENCHMARKS.md for datasets)

    # Run benchmark with JSON export
    cargo run --bin benchmark_runner -- --input-dir ./benchmarks/cnf --format json --output-dir ./results

    # Verify output
    ls -la results/benchmark_*.json

## 5. Proof Verification Reproduction

    # Generate UNSAT proof
    cargo run --bin sat_solver -- --input test_unsat.cnf --proof proof.drat

    # Verify with drat-trim
    ./tools/drat-trim test_unsat.cnf proof.drat
    # Expected: s VERIFIED

## 6. Version History Reproduction

    # Initialize SQLite ledger
    cargo run --bin benchmark_runner -- --input-dir ./benchmarks/cnf --db-path ./history.db --git-tag v6.2.0-M2.7.14

    # Query previous runs
    sqlite3 history.db "SELECT git_tag, COUNT(*) FROM benchmark_runs GROUP BY git_tag;"

## 7. Artifact Evaluation Checklist

| Check | Command | Expected Result |
|-------|---------|---------------|
| Build clean | `cargo build --release` | Success, 0 warnings |
| Tests pass | `cargo test --lib` | 215 passed, 0 failed |
| Binary compiles | `cargo check --bins` | 0 errors, 0 warnings |
| Benchmark runs | `cargo run --bin benchmark_runner -- --help` | Help text displayed |
| Proof validates | `./tools/drat-trim test_unsat.cnf proof.drat` | `s VERIFIED` |
| Version history | `sqlite3 history.db ".schema"` | Tables exist |

## 8. Containerization (Queued)

Docker/Singularity containers for bit-exact replication:
- Status: Planned for M2.7.15
- Will include: pinned Rust toolchain, pre-compiled drat-trim, benchmark datasets

## 9. Hardware Dependence Disclosure

| Factor | Impact | Mitigation |
|--------|--------|------------|
| CPU clock speed | Runtime variance | Wall-clock timeout normalization |
| Thermal throttling | Sustained performance | Documented ambient temperature |
| Memory bandwidth | Large instance solving | Documented RAM configuration |
| Cache hierarchy | Propagation speed | Fixed CPU affinity (`benchmark_runner --cpu-affinity`) |

## 10. Threats to Validity

- **Benchmark selection bias**: Current datasets are limited to custom/test instances
- **Hardware dependence**: Results from 8-core x86_64 may not generalize
- **Compiler version**: Rust 1.96.0 optimizations may differ from future versions
- **OS effects**: Linux scheduler noise vs. Windows
- **Measurement precision**: `std::time::Instant` wall-clock granularity
- **Cache/thermal effects**: Not instrumented in M2.7.14
