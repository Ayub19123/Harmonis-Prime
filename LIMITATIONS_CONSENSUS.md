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
> injection and Raft consensus produce measurable, reproducible latency results."

## What This Does NOT Prove
> "Zero-latency guarantees. Picosecond precision. Air-gapped security.
> World-record performance. These require controlled testbeds."

## Sovereign Principle
**Claims = Measurements. Nothing more. Nothing less.**

