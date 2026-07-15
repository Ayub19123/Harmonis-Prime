# Harmonis Prime — Artifact Evaluation Checklist (M2.7.14)

## For SAT Competition 2027 Reviewers

### Quick Verification (5 minutes)

| Step | Command | Expected | Check |
|------|---------|----------|-------|
| 1. Clone | `git clone https://github.com/Ayub19123/Harmonis-Prime.git` | Success | [ ] |
| 2. Checkout | `git checkout v6.2.0-M2.7.14` | Detached HEAD | [ ] |
| 3. Build | `cargo build --release` | Success, 0 warnings | [ ] |
| 4. Test | `cargo test --lib` | 215 passed, 0 failed | [ ] |
| 5. Binary | `cargo check --bins` | 0 errors, 0 warnings | [ ] |

### Functional Verification (15 minutes)

| Step | Command | Expected | Check |
|------|---------|----------|-------|
| 6. Solver | `cargo run --bin sat_solver -- --help` | Help text | [ ] |
| 7. Benchmark | `cargo run --bin benchmark_runner -- --help` | Help text | [ ] |
| 8. Proof | `./tools/drat-trim test_unsat.cnf proof.drat` | `s VERIFIED` | [ ] |

### Reproducibility Verification (30 minutes)

| Step | Command | Expected | Check |
|------|---------|----------|-------|
| 9. Benchmark run | `mkdir -p benchmarks/cnf; cp test_unsat.cnf benchmarks/cnf/; cargo run --bin benchmark_runner -- --input-dir benchmarks/cnf --format json` | JSON output | [ ] |
| 10. Version history | `cargo run --bin benchmark_runner -- --input-dir benchmarks/cnf --db-path history.db --git-tag v6.2.0-M2.7.14` | SQLite file created | [ ] |
| 11. Baseline query | `sqlite3 history.db "SELECT * FROM benchmark_runs;"` | Rows returned | [ ] |

### Documentation Verification

| Document | Purpose | Location | Check |
|----------|---------|----------|-------|
| README.md | Quick start | Root | [ ] |
| ARCHITECTURE.md | System design | Root | [ ] |
| BENCHMARKS.md | Methodology | Root | [ ] |
| REPRODUCIBILITY.md | Reproduction steps | Root | [ ] |
| CHANGELOG.md | Version history | Root | [ ] |
| LIMITATIONS.md | Known boundaries | Root | [ ] |
| BRICK_STATUS.md | Roadmap | Root | [ ] |

### Scoring

| Criterion | Weight | Evidence |
|-----------|--------|----------|
| Build success | 20% | `cargo build --release` |
| Tests pass | 20% | `cargo test --lib` |
| Documentation | 20% | 7+ .md files, all current |
| Reproducibility | 20% | `benchmark_runner` execution |
| Proof validation | 20% | `drat-trim` VERIFIED |

**Total Expected: 100%**

## Contact

- Repository: https://github.com/Ayub19123/Harmonis-Prime
- Issues: https://github.com/Ayub19123/Harmonis-Prime/issues
- Whitepaper: See `WHITEPAPER.md` (M2.7.1)
