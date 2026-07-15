# Harmonis Prime — Formal Methods & Correctness (M2.7.14)

## 1. Correctness Invariants

### CDCL Core Invariants

| Invariant | Mathematical Statement | Test Coverage |
|---|---|---|
| Unit Propagation Soundness | If a clause becomes unit, the implied literal must be assigned consistently | test_unit_propagation |
| Conflict Detection | If all literals in a clause are falsified, a conflict must be recorded | test_conflict_detection |
| 1-UIP Clause Learning | The learned clause must be asserting | test_uip_clause_learning |
| Trail Consistency | The assignment trail must never contain contradictory literals | test_trail_consistency |
| Decision Level Monotonicity | Decision levels must increase monotonically during search | test_decision_levels |

### Determinism Invariants

| Invariant | Enforcement | Verification |
|---|---|---|
| Fixed Seed | DeterministicSandbox.seed = 0x9e3779b97f4a7c15 | test_deterministic_execution |
| CPU Affinity | Optional core pinning via sandbox.apply_affinity() | test_cpu_affinity |
| Deterministic Ordering | BTreeSet/sort() for instance selection | test_batch_ordering |

## 2. Proof Generation & Verification

Pipeline: CdclSolver.solve() -> proof.drat -> drat-trim -> VERIFIED/REJECTED

| Criterion | Requirement | Implementation |
|---|---|---|
| Clause Addition | Every added clause must be a logical consequence | 1-UIP learning |
| Clause Deletion | Deletion must preserve satisfiability | Standard DRAT format |
| Termination | Proof must end with empty clause for UNSAT | solve() post-condition |

## 3. Traceability Matrix

| Code Element | Mathematical Concept | Test | Location |
|---|---|---|---|
| CdclSolver::solve() | CDCL algorithm (Een, Sorensson 2003) | 215 tests | src/pim_solver/cdcl.rs |
| BenchmarkRunner::run_single() | Deterministic execution environment | test_benchmark_runner | src/benchmark/runner.rs |
| BaselineComparator::compare() | PAR-2 scoring (SAT Competition standard) | test_par2_score | src/benchmark/comparator.rs |
| VersionHistory::record_run() | Persistent audit logging | test_version_history | src/benchmark/history.rs |

## 4. Formal Verification Gaps (Honest)

| Property | Status | Plan |
|---|---|---|
| Complete CDCL correctness proof | Not formalized | Academic collaboration |
| Memory safety | Verified | Rust borrow checker |
| Thread safety | Verified | Rust Send/Sync |
| Determinism across architectures | x86_64 only | ARM64 validation queued |
| Proof checker correctness | Trusted base (drat-trim) | Coq/Isabelle queued |

## 5. References

- Een, N., & Sorensson, N. (2003). An Extensible SAT-solver. SAT 2003.
- Heule, M. J. H., et al. (2013). Trimming while checking clausal proofs. FMCAD 2013.
- SAT Competition 2027 Rules. https://satcompetition.org/
