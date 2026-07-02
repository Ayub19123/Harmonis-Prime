# Harmonis Prime — Telemetry & Profiling (M2.7.14)

## 1. Telemetry Architecture

| Level | Data | Consumer | Module |
|---|---|---|---|
| Solver Core | decisions, propagations, conflicts, restarts | BenchmarkRunner | src/pim_solver/cdcl.rs |
| Benchmark Layer | wall_time_ms, peak_memory_kb, proof_valid | MetricsExporter | src/benchmark/runner.rs |
| Version Ledger | git_tag, instance_hash, timestamp | VersionHistory | src/benchmark/history.rs |

## 2. Internal Solver Telemetry

| Field | Type | Access |
|---|---|---|
| decision_count | u64 | get_decision_count() |
| propagation_count | u64 | get_propagation_count() |
| conflict_count | u64 | get_conflict_count() |
| instance_hash | String | get_instance_hash() |

## 3. BenchmarkRun Struct

```rust
pub struct BenchmarkRun {
    pub instance_path: PathBuf,
    pub instance_hash: String,
    pub result: SolveResult,
    pub decisions: u64,
    pub propagations: u64,
    pub conflicts: u64,
    pub restarts: u64,
    pub peak_memory_kb: u64,
    pub wall_time_ms: u64,
    pub proof_valid: Option<bool>,
    pub timed_out: bool,
    pub memory_exceeded: bool,
}
```

## 4. SQLite Schema

**benchmark_runs**: id, git_tag, instance_path, instance_hash, result, decisions, propagations, conflicts, restarts, peak_memory_kb, wall_time_ms, proof_valid, timed_out, memory_exceeded, timestamp

**version_metadata**: git_tag, commit_hash, test_count, compiler_warnings, timestamp

## 5. Profiling Capabilities (Queued)

| Capability | Status |
|---|---|
| LBD logging | Queued (M2.7.15) |
| Glue clause tracking | Queued (M2.7.15) |
| Real-time RSS monitoring | Queued (M2.7.15) |
| Conflict graph export | Queued (M2.7.16) |

## 6. Query Examples

```sql
SELECT instance_path, wall_time_ms, result FROM benchmark_runs WHERE git_tag = 'v6.2.0-M2.7.14';
SELECT result, AVG(wall_time_ms) FROM benchmark_runs GROUP BY result;
```
