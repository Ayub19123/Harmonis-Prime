# Harmonis Prime

[![Tests](https://img.shields.io/badge/tests-149%2F149%20passing-brightgreen)](https://github.com/Ayub19123/Harmonis-Prime)
[![Zero Drift](https://img.shields.io/badge/drift-0%25-blue)](https://github.com/Ayub19123/Harmonis-Prime)
[![Rust](https://img.shields.io/badge/rust-1.96%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![DOI](https://img.shields.io/badge/DOI-10.5281%2Fzenodo.15542022-blue)](https://doi.org/10.5281/zenodo.15542022)
[![Whitepaper](https://img.shields.io/badge/whitepaper-v1.1-lightgrey)](https://figshare.com/articles/software/Harmonis_Prime_v6_2_0_SET_8_GM/26103446)

> **Harmonis Prime** is a Rust-based experimental research system for validating deterministic distributed system behavior, numerical correctness via high-precision reference oracles, energy-aware computation modeling, and identity verification primitives under reproducible test governance.

---

## Overview

Harmonis Prime is a modular verification framework that unifies:

- Correctness validation
- Numerical computation
- Security enforcement simulation
- Energy telemetry modeling
- System simulation

into a single test-governed architecture.

**This is a research framework, not a production distributed system.**

---

## Design Objective

Modern distributed systems typically separate correctness validation, numerical computation, security enforcement, and energy telemetry into independent tools. Each has its own drift. Each has its own test culture.

We asked: **Can one test harness govern all four?**

---

## System Status

| Subsystem | Description | Status |
|-----------|-------------|--------|
| Deterministic test harness | Full regression suite | ✅ Stable |
| MPFR Zeta oracle (`mpfr_zeta`) | High-precision reference evaluation (400-bit) | ✅ Stable |
| Energy telemetry model | PMU + simulated DVFS + EMA filtering | ✅ Stable |
| Identity subsystem | PUF simulation + NIST-style validation | ✅ Stable |
| Kernel enforcement layer | eBPF / seccomp simulation | ✅ Stable |
| Network calculus engine | Delay bound modeling (min-plus algebra) | ✅ Stable |
| Thermodynamic model | Entropy / KL divergence / stability checks | ✅ Stable |
| PIM solver prototype | Clause evaluation simulation | ⚠️ Experimental |
| Hardware integration | RAPL / NUMA placeholders | ⚠️ Partial |

---

## Core Verification Command

```bash
cargo test --lib -- --nocapture
Expected output:
plain
running 149 tests
test result: ok. 149 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
finished in 0.54s
System Architecture
Mermaid
Code
Preview
HAL: Hardware AbstractionLayer
Kernel Enforcement LayereBPF / seccomp / netfilter
Identity LayerPUF + NIST validation
Network Calculus EngineLatency / bounds / min-plus
Energy Telemetry LayerPMU / DVFS / EMA
Mathematical Oracle LayerMPFR Zeta / truncation /theta
PIM Solver LayerClause evaluation simulation
Test Governance LayerDeterministic validationharness
Each layer is validated independently and integrated through deterministic test execution.
Architecture → Code Mapping
Table
Layer	Source Directory	Key Modules
Hardware Abstraction	src/hal/	atomic_boot.rs — fingerprint, compliance
Kernel Enforcement	src/kernel_enforcement/	ebpf.rs, seccomp.rs, netfilter.rs
Identity	src/identity/	puf.rs, auth.rs, nist.rs
Network Calculus	src/network_calculus/	curves.rs — min-plus, delay bounds
Energy Telemetry	src/energy_telemetry/	telemetry.rs — PMU, EMA, thermal
Thermodynamic Balance	src/thermodynamic_balance/	entropy.rs — Shannon, KL divergence
Mathematical Oracle	src/mpfr_zeta/	oracle.rs, neumaier.rs, truncation.rs
PIM Solver	src/pim_solver/	solver.rs — crossbar simulation
Integration	src/set10_fusion/	theta_approx.rs — thermal bridge
Measurement Definitions
Table
Term	What We Mean	What We Measure	Test Enforcing	Excluded
Determinism	Identical inputs produce identical outputs	Bit-exact reproduction across runs	test_zeta_determinism	Hardware clock jitter, OS scheduling
Stability	Consistent output given identical input	Test execution time variance	test_thermodynamic_determinism	Non-deterministic RNG (seeded ChaCha20)
Energy bound	Simulated power within thermal limits	EMA-filtered PMU counter drift	test_energy_telemetry_drift_within_1_percent	Actual hardware RAPL (Linux only)
Identity assurance	Cryptographic binding to hardware	PUF response stability, NIST monobit	test_puf_deterministic_per_node	Physical tampering, side-channels
Numerical precision	MPFR vs f64 agreement within bounds	Relative error ≤ 1e-12	test_neumaier_precision_invariant	Extreme t-values requiring >10⁶ terms
What the System Provides Today
1. Deterministic Execution
All core modules are tested under repeatable conditions. No stochastic behavior is introduced in production paths.
2. High-Precision Numerical Oracle
MPFR-based ζ(½ + it) evaluation using 400-bit arithmetic
Kahan–Neumaier compensated summation (exact recovery: 2.0 from catastrophic cancellation)
Backlund-style truncation bounds with monotonicity verification
Deterministic fallback to IEEE f64 when rug is unavailable
Property: Given identical inputs and build environment, outputs are bitwise reproducible.
3. Energy-Aware Modeling
Simulated DVFS scaling
EMA-based smoothing
PMU-style counters (platform dependent)
4. Identity Verification Layer
PUF-based node identity simulation
NIST SP 800-22 style randomness validation
Replay protection via monotonic nonce logic
5. Kernel Enforcement Model
Simulated syscall filtering
Network-level drop policies
Isolation-oriented execution model
6. Network Calculus Layer
Min-plus algebra implementation
Delay and backlog bounds
Leaky bucket / token bucket models
7. Thermodynamic Consistency Layer
Entropy constraints
KL divergence bounds
Energy stability invariants
Experimental / Research Components
Table
Component	Status	Note
AVX-512 SIMD kernel	Research	std::simd experimental
NUMA optimization	Research	Single-node only
RAPL integration	Partial	Linux-dependent
FPGA acceleration	Not implemented	Future work
MPI distributed execution	Not implemented	Future work
Odlyzko dataset validation	Pending	Manual setup
Reproducibility Protocol
Minimal Run
bash
git clone https://github.com/Ayub19123/Harmonis-Prime.git
cd Harmonis-Prime
cargo test --lib -- --nocapture
With MPFR Support
bash
cargo test --lib --features mpfr mpfr_zeta -- --nocapture
Lint Verification
bash
cargo clippy -- -D warnings
Note: 114 pre-existing clippy warnings in legacy modules (brick41-51, POM, autonomy) are being addressed in a dedicated maintenance session. SET-12 itself is clean.
Reviewer Verification Checklist
Table
Criterion	Verification	Test
Determinism	Identical runs produce identical outputs	test_zeta_determinism
Numerical correctness	MPFR aligns with f64 within bounds	test_neumaier_precision_invariant
Isolation	No external network dependency	test_zero_external_api_calls
Energy model	EMA convergence stable	test_ema_filter_convergence
Reproducibility	Fresh clone → full suite passes	cargo test --lib -- --nocapture
Scientific Positioning
Explicit Non-Claims
This project does NOT claim:
Formal proof of distributed system correctness
Production deployment readiness
Cryptographic or security certification
Hardware acceleration (FPGA/SIMD full deployment)
Completeness of distributed system modeling
"First-of-its-kind" status (no literature survey conducted)
"Aerospace-grade" certification (no DO-178C/ECSS standard referenced)
Validation Methodology
System correctness is verified using:
Deterministic unit testing
Invariant-based validation
Reproducibility across clean builds
Numerical comparison (MPFR vs f64 fallback)
Energy simulation consistency checks
No subsystem is considered valid without test coverage.
Research Framing
Hypothesis
Deterministic invariant-based validation can provide consistent correctness guarantees across heterogeneous system layers.
Evaluation Strategy
Invariant-based testing
Deterministic replay validation
Cross-module consistency checks
Numerical stability benchmarking
Simulated energy coherence analysis
Roadmap
Phase 2 (Current)
Table
Feature	Status
Odlyzko dataset integration	In progress
SIMD acceleration (AVX-512)	Planned
NUMA optimization	Planned
RAPL integration	Planned
Phase 3 (Planned)
Table
Feature	Status
FPGA acceleration	Not started
Distributed cluster execution	Not started
Hardware co-design	Not started
Phase 4 (Future)
Table
Feature	Status
Formal verification integration	Optional
Certification-grade evaluation	Optional
Academic Resources
Table
Resource	Link	DOI
Whitepaper v1.1 (Zenodo)	zenodo.org/record/15542022	10.5281/zenodo.15542022
Whitepaper v1.1 (Figshare)	figshare.com/articles/26103446	—
GitHub Repository	github.com/Ayub19123/Harmonis-Prime	—
GitHub Discussions	github.com/Ayub19123/Harmonis-Prime/discussions	—
Engagement Prompts
We invite architecture feedback and contributions:
Open question: Where would you deploy a zero-drift system?
Help wanted: MPFR Z(t) on Windows — rug compatibility
Architecture feedback: What domain should SET-13 integrate?
Notes on Design Philosophy
The system is structured around:
Test-first development
Explicit failure modeling
Deterministic execution guarantees
Separation of simulation vs. physical hardware assumptions
All claims must be verifiable through cargo test, not external assertion.
