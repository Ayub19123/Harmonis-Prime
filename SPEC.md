# Harmonis Prime Specification (HBS-1.1)

## Verified Architecture (BRICK-42 through BRICK-50)

### Quantum-Classical Simulation Layer
- **Location:** `src/brick42/quantum/`
- **Components:**
  - `qpu_engine.rs` — `QuantumState { amplitudes: Vec<Complex64> }` with coherence budget tracking
  - `annealing.rs` — `QuantumAnnealingSolver` with confidence scoring
  - `qkd.rs` — `FinancialQKDNetwork` with `QuantumAuditTrail`
  - `quantum_synapse.rs` — `compute_amplitudes()` with normalization, `estimate_confidence()`
- **Backends:** `Simulated` (functional), `DWave`, `IBMQiskit`, `EdgeQPU` (stubs)

### Formal Verification Engine
- **Location:** `src/fv/`
- **Components:**
  - `invariant_checker.rs` — `InvariantChecker` with `RaftConsensusSafety`, `CausalConsistency`, `GovernanceEnforcement`
  - `model_checker.rs` — `check_bounded(spec, depth)` with state exploration
  - `proof_generator.rs` — `generate_invariant_proof()` with inductive step certificates
  - `tla_spec.rs` — TLA+-style initial predicates (`ALL n IN Nodes : node_n_log = empty`)

### Policy-Driven Autonomy
- **Location:** `src/autonomy/`
- **Components:**
  - `policy_runtime.rs` — `PolicyPredicate` enum with 8 base types + `CompositeAnd/Or`
  - Runtime evaluation: `State → Bool` with recursive composite logic

### Causal State Transitions
- **Location:** `src/brick49/`, `src/state_machine.rs`
- **Components:**
  - `causality_engine.rs` — `transition(new_state_id) → CausalityProof`
  - `state_machine.rs` — `apply(command) → String`
  - `replay/ledger.rs` — monotonic sequence invariants

### Additional Verified Layers
- **BRICK-45:** Chaos injection (`chaos_engine.rs`, `chaos_runner.rs`)
- **BRICK-46:** Cognitive engine (`cognitive.rs`, `quantum_synapse.rs`)
- **BRICK-47:** Decision loop (`decision_loop.rs`, `counterfactual.rs`)
- **BRICK-50:** Quantum-classical coupling (`quantum_classical_coupling.rs`)

## Honest Limitations

| Claim in Blueprint | Code Reality | Honest Framing |
|-------------------|-------------|----------------|
| "Quantum substrate" | Simulated backend only | Simulated quantum backend with amplitude vectors |
| "Density matrix evolution" | `compute_amplitudes()` uses pseudo-random, not von Neumann equation | Amplitude estimation with normalization; full Hamiltonian evolution is research |
| "Real-time flawless execution" | Wall-clock timing with ±30% variance | Honest wall-clock measurement on consumer hardware |
| "Mesh DAG topology" | `SharedMemoryGraph` is undirected HashMap | Graph abstraction exists; DAG enforcement is spec-only |
| "Physical QPU integration" | `DWave`, `IBMQiskit`, `EdgeQPU` are enum variants | Backend stubs defined; only `Simulated` is functional |

## Research Roadmap

| Phase | Target | Current Gap | Estimated Timeline |
|-------|--------|-------------|-------------------|
| 1 | Wire quantum amplitudes to graph mutations | `SharedMemoryGraph` is classical HashMap | Month 1 |
| 2 | Implement full density matrix evolution | `compute_amplitudes()` uses PRNG, not `[H, ρ]` | Month 2–3 |
| 3 | Physical QPU backend integration | Only `Simulated` backend functional | Month 3–6 |
| 4 | Mesh DAG topology enforcement | No cycle detection in graph layer | Month 1–2 |

## Sovereign Principle

**Claims = Code. Every assertion in this document references a specific file and function.**

If a claim cannot be traced to `src/`, it is marked as research or limitation.

## Tag Reference

| Tag | Commit | Purpose |
|-----|--------|---------|
| `v7.1.0-BRICK51.3-HW-AUDIT` | `7f6ad71` | Latest verified baseline |
| `v7.1.0-BRICK51.3-TIMING-HONESTY` | `6905b5f` | Honest timing documentation |
| `v7.1.0-BRICK51.3-FINAL-CORRECT` | `93b8993` | Corrected 10K benchmarks |

---

*This specification is a living document. It is honest about what exists, what does not, and what comes next.*
