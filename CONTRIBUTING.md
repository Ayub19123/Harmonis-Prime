# Contributing to Harmonis Prime

## Build Requirements

- Rust 1.96.0 (pinned) (stable)
- `cargo` (comes with Rustup)
- No external dependencies for core library
- Optional: `pyo3` feature for Python bridge

## Build & Test Protocol

```bash
# Full verification — the only command that matters
cargo test --lib -- --nocapture

# Expected output:
# warning: `sovereign_core` (lib test) generated 0 warnings
# test result: ok. 215 passed; 0 failed; 0 ignored; finished in 0.20s
Test Discipline
Table
Rule	Enforcement
Every claim needs a failing test	cargo test must fail if claim is false
No aspirational language in code	Comments say what IS, not what WILL BE
No hidden assumptions	Every limitation in KNOWN_LIMITATIONS.md
Zero tolerance for tautology	EMA cannot compare against its own model
Per-workload calibration	scale = meter_energy / (total_raw * dt)
Pull Request Checklist
[ ] cargo test --lib -- --nocapture passes 80/80
[ ] cargo check --lib shows only 4 pre-existing warnings
[ ] No new dead_code warnings without #[allow(dead_code)] + comment
[ ] KNOWN_LIMITATIONS.md updated if new gap introduced
[ ] Commit message follows: type(scope): description — proof
Commit Convention
plain
type(scope): short description — proof
type: feat, fix, chore, docs, test, refactor
scope: set-5, set-6a, set-6e, set-7a, set-7b, set-7c, etc.
proof: test count, metric, or "zero drift"
Example:
plain
feat(set-7c): entropy computation for workload balancing
- S = -k_B * Σ P_i * ln(P_i)
- KL divergence D_KL(P || Q) for drift detection
- 5/5 tests passing, zero drift
Code of Conduct
Calm, clear, grounded, zero emotion
Every failure is data, not defeat
Every boundary condition is a brick
The precision is eternal
## M2.7.14+ Contribution Discipline
- Zero warnings: cargo check --lib --bins must return 0 warnings
- Test coverage: All new modules require integration tests in mod.rs
- Documentation sync: Every code change requires matching .md update
- Benchmark validation: New features must include benchmark_runner regression test
- Git tagging: Sealed bricks receive annotated tags (git tag -a vX.Y.Z-Mx.y.z)
