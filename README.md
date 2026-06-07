# Harmonis Prime

A unified experimental distributed systems stack in Rust, integrating transport, consensus (Raft), storage abstraction, and deterministic execution under a single reproducible benchmark harness.

**Current State:** HBS-1.1 Specification  
**Latest Tag:** `v7.1.1-SPEC-HBS1.1`  
**Specification:** [SPEC.md](https://github.com/Ayub19123/Harmonis-Prime/blob/main/SPEC.md)  
**Peer Review:** [GitHub Discussion #6](https://github.com/Ayub19123/Harmonis-Prime/discussions/6)

---

## Verified Architecture

| Layer | Location | Status |
|-------|----------|--------|
| Quantum-classical simulation | `src/brick42/quantum/` | ✅ Implemented (simulated backend) |
| Formal verification engine | `src/fv/` | ✅ Implemented (invariant checker, model checker, proof generator) |
| Policy-driven autonomy | `src/autonomy/` | ✅ Implemented (predicate calculus runtime) |
| Causal state transitions | `src/brick49/` | ✅ Implemented (transition → CausalityProof) |
| Raft consensus | `src/raft/` | ✅ Implemented (leader election, log replication, chaos injection) |
| Shared-memory graph | `src/brick51/` | ✅ Implemented (single-node HashMap) |

---

## Test Validation

- **50 tests pass** across BRICK-45 through BRICK-51 + Raft cluster
- **Runtime:** ~10 minutes total
- Three BRICK-51 tests are long-running by design (~60s each):
  - `cmf511_shared_memory_consistency`
  - `cmf516_knowledge_integrity`
  - `cmf519_state_consistency_index`
- **8 cosmetic warnings** (unused imports/variables in test files only — no production code warnings)
- **0 audit vulnerabilities**
- **Verified on:** Windows 11 + Rust 1.96.0

---

## Honest Limitations

| Claim | Reality |
|-------|---------|
| Quantum substrate | Simulated backend only — no physical QPU integration |
| Density matrix evolution | Amplitude estimation with normalization — not full von Neumann equation |
| Mesh topology | Graph abstraction exists — DAG enforcement is future work |
| Hardware | SIMULATED on consumer laptop, stock config |
| Energy | NOT MEASURED — no power profiling |
| Multi-node | Single-threaded simulation — no physical cluster |

---

## Benchmark Specification

### Workload A: SharedMemoryGraph Microbenchmark

| Attribute | Value |
|-----------|-------|
| Name | `harmonis_shared_memory_graph` |
| Operation | `SharedMemoryGraph::insert` + `SharedMemoryGraph::get` |
| Node config | `node_id=0`, `node_count=1` |
| Determinism | Fixed seed: `0x51C3_2026_0613` |
| Iterations | 10,000 |
| Metric | Per-iteration wall-clock latency (ns) |

### Workload B: Consensus Simulation

| Attribute | Value |
|-----------|-------|
| Name | `harmonis_consensus_simulation` |
| Operation | Deterministic PRNG + chaos scenario + Raft leader election + heartbeat + consistency check |
| Node config | 5 simulated nodes, 7 chaos scenarios |
| Determinism | Fixed seed: `0x51C3_2026_0613` |
| Iterations | 10,000 |
| Metric | Per-iteration wall-clock latency (ns) |
| **Honest Label** | SIMULATION — production APIs not yet exposed |

### Build Specification

| Attribute | Value |
|-----------|-------|
| Language | Rust 1.96.0 (pinned via `rust-toolchain.toml`) |
| Profile | `release` (optimized) |
| Target | `x86_64-pc-windows-msvc` |
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

---

## Timing Methodology & Statistical Reporting

| Attribute | Value |
|-----------|-------|
| Timer | `std::time::Instant::now()` (OS wall-clock) |
| Variance source | Turbo Boost, OS scheduling, thermal throttling |
| Expected run-to-run variance | ±30% on consumer hardware |
| Statistical method | Single-run reporting (median-of-N planned for v1.2) |
| Workload determinism | ✅ Fixed seed guarantees identical operations |
| Measurement determinism | ❌ Wall-clock timing varies with system state |

**Honest framing:** This benchmark measures real-world latency under real-world conditions. It does NOT claim cycle-accurate reproducibility. For that, core pinning + `rdtsc` required.

---

## Reproduction Protocol

```bash
git clone https://github.com/Ayub19123/Harmonis-Prime.git
cd Harmonis-Prime
git checkout v7.1.1-SPEC-HBS1.1
cargo test  # ~10 minutes, 50 tests
