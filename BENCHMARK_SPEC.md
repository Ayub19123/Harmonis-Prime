# Harmonis Prime Benchmark Specification v1.0
## Immutable Reference: v7.1.0-BRICK51.3-FINAL-BENCH (commit 481da5f)

### Classification
- Type: Custom distributed systems simulation benchmark
- Category: Chaos-injected consensus latency measurement
- Standard Alignment: MLPerf Tiny (inspired, not certified)
- Status: Pre-submission — awaiting independent reproduction

### Workload A: SharedMemoryGraph Microbenchmark
| Attribute | Value |
|-----------|-------|
| Name | harmonis_shared_memory_graph |
| Operation | SharedMemoryGraph::insert + SharedMemoryGraph::get |
| Node config | node_id=0, node_count=1 |
| Determinism | Fixed seed: 0x51C3_2026_0613 |
| Iterations | 10,000 |
| Metric | Per-iteration wall-clock latency (ns) |

### Workload B: Consensus Simulation
| Attribute | Value |
|-----------|-------|
| Name | harmonis_consensus_simulation |
| Operation | Deterministic PRNG + chaos scenario + Raft leader election + heartbeat + consistency check |
| Node config | 5 simulated nodes, 7 chaos scenarios |
| Determinism | Fixed seed: 0x51C3_2026_0613 |
| Iterations | 10,000 |
| Metric | Per-iteration wall-clock latency (ns) |
| **HONEST LABEL** | SIMULATION — production APIs not yet exposed |

### Build Specification
| Attribute | Value |
|-----------|-------|
| Language | Rust 1.96.0 (pinned via rust-toolchain.toml) |
| Profile | release (optimized) |
| Target | x86_64-pc-windows-msvc |
| Dependencies | Zero external benchmark dependencies |

### Hardware Context
| Attribute | Value |
|-----------|-------|
| Machine | Consumer laptop, stock configuration |
| CPU | 11th Gen Intel i7-1165G7 |
| RAM | 16GB DDR4 |
| OS | Windows 11 |
| Core pinning | NOT IMPLEMENTED |
| Turbo locking | NOT IMPLEMENTED |
| Energy measurement | NOT AVAILABLE |

### Honest Limitations
1. Not MLPerf-certified: Custom benchmark awaiting standard mapping
2. Not formally verified: No Lean 4 proofs
3. Not multi-node: Single-threaded simulation only
4. Not energy-profiled: No RAPL or power meter
5. Not baseline-compared: No reference system yet

### What This Proves
> On standard consumer hardware, deterministic chaos-injected consensus simulation produces measurable, reproducible latency results.

### What This Does NOT Prove
> Production-grade consensus performance. Zero-latency guarantees. Energy efficiency. World-record performance. Formal correctness.

### Reproduction Protocol
`powershell
git clone https://github.com/Ayub19123/Harmonis-Prime.git
cd Harmonis-Prime
git checkout v7.1.0-BRICK51.3-FINAL-BENCH
./run_benchmark.ps1
