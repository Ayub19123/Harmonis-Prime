# Known Limitations – Harmonis Prime (as of 2026-06-17)

## Integration Tests (System‑Level Validation)
- **Claim**: "Survives 10/10 chaos scenarios"
- **Current evidence**: Unit tests only (71/71 passing)
- **Missing**: Cross‑module chaos injection (network partition, Byzantine nodes, thermal throttling, OOM)
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
