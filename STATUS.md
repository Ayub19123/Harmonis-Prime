# Harmonis Prime — SovereignCore Brick Status

| Brick | Status | Commit | Tests | Description |
|-------|--------|--------|-------|-------------|
| M2.5 | ✅ SEALED | 80681f2 | 164/164 | DIMACS CNF adapter + CDCL engine |
| M2.5.1 | ✅ SEALED | 80681f2 | 164/164 | Minimal CDCL with telemetry & DRAT |
| M2.5.5 | ✅ SEALED | 80681f2 | 6/6 benchmarks | CI benchmark suite (XOR, contradiction, unit SAT) |
| **M2.6** | **✅ SEALED** | **32178f** | **164/164 + CLI validated** | **SAT Competition 2027 CLI binary** |
| M2.6.1 | 🔲 READY | — | — | Checkpoint serialization (procedural memory) |
| M2.7 | 🔲 READY | — | — | Distributed clause registry / DHT mesh |

## M2.6 CLI Interface
\\\ash
cargo run --bin sat_solver <input.cnf> [--proof <file.drat>] [--mem-profile] [--clause-db-stats]
\\\
- Exit 10 = SATISFIABLE
- Exit 20 = UNSATISFIABLE
- Exit 1 = Error
- stdout: DIMACS results | stderr: telemetry & proof logs

## Latest Validation
- Full regression: 164 passed, 0 failed, 8 ignored
- SAT sweep: exit 10 ✅
- UNSAT sweep: exit 20 ✅
- DRAT proof artifact: generated ✅
- Telemetry flags: active ✅
