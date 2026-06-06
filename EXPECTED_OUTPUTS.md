# Expected Outputs - Harmonis Prime Reproducible Benchmark
## Version: v7.1.0-BRICK51.3-ABSOLUTE-ZERO
## Commit: b00b398

### Build Verification
`cargo check --bin benchmark``nExpected: 0 errors, 0 warnings

### Graph Benchmark
`cargo run --release --bin benchmark -- 10000 0x51C3_2026_0613``nExpected: metrics.json with 10,000 entries

### Consensus Simulation
`cargo run --release --bin benchmark_consensus -- 10000 0x51C3_2026_0613``nExpected: metrics_consensus.json with 10,000 entries

### Test Suite
`cargo test``nExpected: All tests pass

### Security Audit
`cargo audit``nExpected: 0 vulnerabilities

### What These Prove
- Telemetry infrastructure works
- Deterministic, reproducible results
- Honest workload labels

### What These Do NOT Prove
- Real multi-node cluster performance
- Energy efficiency (not measured)
- World-record performance (no baseline)

SOVEREIGN PRINCIPLE: Claims = Artifacts. Nothing more. Nothing less.
