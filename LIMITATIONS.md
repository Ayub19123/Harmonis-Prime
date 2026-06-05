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

> "On standard consumer hardware with stock configuration, deterministic chaos
> injection and Raft consensus produce measurable, reproducible latency results.
> The seed-locked execution guarantees identical workload paths; the wall-clock
> variance reflects real-world scheduler and thermal behavior."

## What This Does NOT Prove

> "Zero-latency guarantees. Picosecond precision. Air-gapped security.
> World-record performance. These require controlled testbeds with documented
> hardware configuration, core pinning, turbo locking, and background quiescence."

## Path to Controlled Testbed

| Requirement | Acquisition Target | Estimated Timeline |
|---|---|---|
| Core pinning | Desktop motherboard with UEFI control | Month 2-3 |
| Turbo locking | BIOS/UEFI disable + static voltage | Month 2-3 |
| Energy measurement | Intel RAPL (Linux) or Power Gadget (Windows) | Month 2-3 |
| Background quiescence | Dedicated benchmark partition, minimal OS | Month 3-4 |
| Air-gapped execution | Physical isolation, no network stack | Month 6+ |

## Sovereign Principle

**Claims = Measurements. Nothing more. Nothing less.**

