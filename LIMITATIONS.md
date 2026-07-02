# Honest Limitations — Harmonis Prime Telemetry

## Measurement Boundaries
| Limitation | Status | Impact |
|---|---|---|
| Core pinning | NOT IMPLEMENTED | Thread migration ±5-15% latency variance |
| Turbo/boost locking | NOT IMPLEMENTED | Frequency scaling affects iteration timing |
| Swap isolation | NOT IMPLEMENTED | Pagefile activity may introduce disk I/O jitter |
| Background quiescence | NOT IMPLEMENTED | Services consume CPU cycles unpredictably |
| Energy measurement | NOT AVAILABLE | No RAPL access, no Intel Power Gadget |
| GPU isolation | NOT IMPLEMENTED | iGPU shares memory bandwidth with CPU |

## What This Proves
> "On standard consumer hardware with stock configuration, deterministic
> benchmark execution produces measurable, reproducible SAT results."

## What This Does NOT Prove
> "Zero-latency guarantees. Picosecond precision. Air-gapped security.
> World-record performance. These require controlled testbeds."

## Sovereign Principle
**Claims = Measurements. Nothing more. Nothing less.**


## Timing Methodology & Variance (Added 2026-06-07)

**Measurement:** std::time::Instant::now() — OS-scheduled wall-clock time
**Hardware:** Consumer laptop, stock configuration (no core pinning, no turbo lock)
**Expected variance:** ±30% between runs due to:
- CPU frequency scaling (Turbo Boost)
- OS scheduler jitter
- Thermal throttling
- Memory allocation variance

**This is NOT a bug.** This is honest measurement of real-world conditions.
For <5% variance, core pinning and turbo locking are required.

**Reproducibility guarantee:** The workload path (seed → operations) IS deterministic.
The latency measurement IS NOT deterministic on uncontrolled hardware.
﻿# Known Limitations – Harmonis Prime (as of 2026-06-17)

## Integration Tests (System‑Level Validation)
- **Claim**: "Validates 100% of UNSAT proofs via drat-trim"
- **Current evidence**: Unit tests only (71/71 passing)
- **Missing**: Cross‑platform validation (ARM64, Windows primary), Docker containerization, real-time RSS monitoring
- **Resolution**: Phase 2 – build 	ests/integration/ harness with simulated network layer
- **Impact**: The claim is theoretically sound but not empirically verified at system level

## Performance Benchmarks
- **Current**: 0.27s execution observed
- **Missing**: criterion.rs benchmarks with statistical confidence
- **Resolution**: Phase 2 – add benchmarks for latency, throughput, memory under load

## Fuzzing
- **Current**: None
- **Missing**: cargo fuzz targets for PUF extraction, clause evaluation, network calculus curves
- **Resolution**: Phase 2 – security hardening

## Real Hardware Validation
- **Current**: Software simulation only
- **Missing**: ARM CoreSight PMU, physical power meter, FPGA PIM fabric
- **Resolution**: Phase 2 – ARM integration, MRAM/FeRAM prototyping

## Honest Discipline
Every claim in the repository is either:
- ✅ **Sealed** – backed by executable tests
- ⏳ **Documented** – labelled as pending

No claim is hidden. No failure is ignored.

## M2.7.14 Limitations
- Docker/OCI containerization: queued for M2.7.15
- cgroups resource throttling: queued for M2.7.15
- SAT Competition benchmark mirror routing: queued
- CI/CD auto-trigger on git push: queued for M2.7.16
- Cactus plot generation: queued
- Grafana visualization: queued for M2.7.17
- OpenTuner / Bayesian heuristic optimization: queued
- Python JSON Logger integration: queued
- Granular bottleneck isolation (problem-class regression): queued
