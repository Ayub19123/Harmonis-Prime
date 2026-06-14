# Known Limitations – Harmonis Prime (SET-5.6)

## Honest Disclosure

The following claims are **NOT** validated in this release:

### Quantum advantage
- Ramanujan‑driven collapse only tested up to 20 dimensions.
- No physical QPU integration; simulated amplitude estimation only.

### Fluid intelligence generalisation
- Euler thermodynamic loops validated on simulated workloads only.
- Not tested on real‑time market data, sensor fusion, or adversarial network traffic.

### Energy measurement
- RAPL JLO correlation is software‑estimated on Windows.
- True hardware‑in‑the‑loop reduction requires Linux with RAPL‑capable silicon.

### Multi‑node / sovereignty
- No air‑gapped physical cluster deployed.
- Cross‑cluster federation is an architectural target, not implemented.

### Performance benchmarks
- Wall‑clock timing on consumer laptop (i7‑1165G7) – variance ±30%.
- No core pinning, turbo locking, or real‑time guarantees.

## What is validated
- 106/106 tests pass with zero warnings in sealed modules.
- Sub‑microsecond Py03 round‑trip (0.30 µs).
- Laminar flow (Reynolds <2300) and monotonic entropy in Euler loops.
- Statistical bias >0.5 in Ramanujan HCN heuristic.

For full details, see [WHITEPAPER_HBS2_0.md](./WHITEPAPER_HBS2_0.md).
