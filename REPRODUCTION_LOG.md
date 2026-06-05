# REPRODUCTION_LOG.md — Harmonis Prime Sovereign Baseline

## Canonical Commit
| Field | Value |
|-------|-------|
| Repository | https://github.com/Ayub19123/Harmonis-Prime |
| Branch | main |
| Commit | 'dd6d76e42422cb7a5afbadd7fde7d2814aca7d3c' |
| Tag | v7.1.0-BRICK51.3-SOVEREIGN |
| Date | 2026-06-05 |

## Test Results — VERIFIED
| Suite | Tests | Passed | Failed | Duration |
|-------|-------|--------|--------|----------|
| BRICK-50 | 6 | 6 | 0 | 0.01s |
| BRICK-51 | 13 | 13 | 0 | 1275.04s |
| Raft Cluster | 4 | 4 | 0 | 0.00s |
| Doc-tests | 0 | 0 | 0 | 0.00s |
| **TOTAL** | **23** | **23** | **0** | **~1275s** |

## Audit Results — VERIFIED
| Command | Result |
|---------|--------|
| `cargo audit` | 0 vulnerabilities (41 crates scanned) |

## Execution Environment (Actual Machine)
| Field | Value | How Verified |
|-------|-------|--------------|
| CPU | [FILL from `wmic cpu get name`] | Command output |
| Memory | [FILL from `wmic computersystem get totalphysicalmemory`] | Command output |
| OS | Windows 11 Pro [FILL from `winver`] | `winver` |
| Rust Version | [FILL from `rustc --version`] | Command output |
| Cargo Version | [FILL from `cargo --version`] | Command output |
| Hardware Tag | SIMULATED | Honest default |

## Honest Limitations
- Standard laptop configuration, not core-pinned
- Turbo/boost enabled, not locked
- Swap/pagefile active
- Background services running (browser, Discord, etc.)
- Network connected during execution
- Not air-gapped, not clean-room

## What This Proves
> "On this specific machine, with standard consumer configuration, the deterministic test suite produces consistent results. Seed-locked chaos injection and hardened Raft consensus verified."

## What This Does NOT Prove
> "Zero-latency guarantees. Picosecond precision. Air-gapped security. World-record performance. These require controlled testbeds documented separately."

## Independent Reproduction Witness
| Field | Value |
|-------|-------|
| Reproducer | [To be filled by external engineer] |
| Machine | [OS, CPU, RAM] |
| Rust Version | [rustc --version] |
| Result | [Pass/Fail with notes] |
| Date | [Reproduction date] |

**Status:** ✅ Baseline verified locally. Awaiting independent reproduction.
