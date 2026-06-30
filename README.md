.# Harmonis Prime

## Current Status
- **199 tests passing** | 0 failed | 8 ignored (Odlyzko cache)
- **Latest sealed brick:** M2.7.10 — Meta-Reasoning & Goal-Driven Prioritization
- **Commit:** `696d86a`
- **Target:** SAT Competition 2027 (March 2027 submission window)


[![Tests](https://img.shields.io/badge/tests-170%2F170%20passing-brightgreen)](https://github.com/Ayub19123/Harmonis-Prime/actions)
[![CI](https://img.shields.io/badge/CI-Ubuntu%20%7C%20Windows%20%7C%20macOS-blue)](https://github.com/Ayub19123/Harmonis-Prime/actions)
[![DRAT](https://img.shields.io/badge/DRAT-verified%20s%20VERIFIED-success)](https://github.com/Ayub19123/Harmonis-Prime)
[![Rust](https://img.shields.io/badge/rust-1.96%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
[![DOI](https://img.shields.io/badge/DOI-10.5281%2Fzenodo.15542022-blue)](https://doi.org/10.5281/zenodo.15542022)
[![Whitepaper](https://img.shields.io/badge/whitepaper-v1.2-lightgrey)](https://figshare.com/articles/software/Harmonis_Prime_v6_2_0_SET_8_GM/26103446)
Overview
Harmonis Prime is a SAT Competition 2027 candidate solver built on four inviolable pillars:
Table
Pillar	Principle	Enforcement
Correctness	Every algorithmic path is verified by deterministic tests	cargo test --lib — 170 tests, zero tolerance
Auditability	Every proof carries epistemic provenance	DRAT + c epistemic origin_id lbd timestamp metadata
Reproducibility	Identical inputs produce identical outputs across platforms	CI matrix: Ubuntu / Windows / macOS
Competitiveness	Performance gains are measured, never assumed	Benchmark baselines per brick, no regression without evidence
This is a research-grade solver, not a production distributed system.
Design Objective
Modern SAT solvers typically separate core search, proof generation, and distributed clause sharing into independent subsystems. Each has its own drift. Each has its own validation culture.
We asked: Can one test harness govern all three — and make every learned clause auditable?
System Status
Phase 1 — Correctness Foundation ✅ SEALED
Table
Brick	Description	Status	Commit
M2.5	DIMACS parser + CDCL core	✅ Sealed	—
M2.5.1	Telemetry + DRAT proof generation	✅ Sealed	—
M2.5.2	Unit propagation (watched literals)	✅ Sealed	—
M2.5.3	Conflict analysis (1-UIP)	✅ Sealed	—
M2.5.4	Decision heuristics (VSIDS)	✅ Sealed	—
Phase 2 — Validation & Performance ✅ SEALED
Table
Brick	Description	Status	Commit
M2.5.5	CI benchmarks (GitHub Actions matrix)	✅ Sealed	—
M2.5.6	Memory telemetry + activity decay	✅ Sealed	—
M2.5.7	RAPL energy telemetry	✅ Sealed	—
M2.5.8	Deterministic replay	✅ Sealed	—
M2.5.9	Clause database hardening	✅ Sealed	—
M2.5.10	Epistemic DRAT logging foundation	✅ Sealed	—
M2.5.11	drat-trim integration (Windows + Linux)	✅ Sealed	s VERIFIED
Phase 3 — Interface & Packaging ✅ SEALED
Table
Brick	Description	Status	Evidence
M2.6	CLI binary (SAT/UNSAT exit codes 10/20)	✅ Sealed	Cross-platform
M2.6.1	Checkpointing (procedural memory)	✅ Sealed	save/load state
M2.6.2	Linux cross-compilation	✅ Sealed	ELF 64-bit native
M2.6.3	Linux DRAT validation	✅ Sealed	CI green, s VERIFIED
M2.6.5	Docker + AWS packaging	✅ Sealed	harmonis-prime:sat-2027
Phase 4 — Distributed Memory 🔥 ACTIVE
Table
Brick	Description	Status	Commit
M2.7.1	Local Clause Registry (Lit-Pack protocol)	✅ SEALED	fda2fb0
M2.7.2	Epistemic DRAT Logging (LBD provenance)	✅ SEALED	1371904
M2.7.3	DHT Bootstrap (libp2p mesh)	🔲 Ready	Pending
M2.7.4	ClauseRegistry ↔ CDCL integration	🔲 Ready	Pending
M2.7.5	Conflict-aware routing	🔲 Future	Post-competition
Phase 5 — Hardware Layers 📋 FUTURE (Post-Competition)
Table
Brick	Description	Timeline
SET-6C	PUF Identity	Summit
SET-6D	HBM Telemetry	Summit
SET-6E	CoreSight Tracing	Summit
M2.8	Neural Heuristic Engine	Summit
M2.10	GPU Offloading	Summit
Core Verification Command
bash
# Full regression suite (170 tests)
cargo test --lib -- --nocapture

# Expected output:
# test result: ok. 170 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out
SAT Solver Binary
bash
# Build competition binary (no default features — SAT Competition path)
cargo build --release --bin sat_solver --no-default-features

# SAT instance
./target/release/sat_solver test_sat.cnf
# → s SATISFIABLE, v 1 -2 0, exit code 10

# UNSAT instance with DRAT proof
./target/release/sat_solver test_unsat.cnf --proof proof.drat
# → s UNSATISFIABLE, exit code 20

# Validate proof
drat-trim test_unsat.cnf proof.drat
# → s VERIFIED
Docker Container (SAT Competition Ready)
bash
# Build multi-stage image
docker build -t harmonis-prime:sat-2027 .

# Run containerized solver
docker run --rm -v $(pwd)/instance.cnf:/solver/instance.cnf   harmonis-prime:sat-2027 instance.cnf
System Architecture
Mermaid
Code
Preview
DIMACS Parser
CDCL Core Engine
Unit Propagation
Conflict Analysis / 1-UIP
VSIDS Decision Heuristics
Clause Learning
Epistemic DRAT Logger
DRAT Proof Filec epistemic origin_id lbdtimestamp
Local Clause Registry
Lit-Pack Binary Protocol
Epistemic FilteringUtility = α·LBD + β·Size +γ·Activity
DHT Meshlibp2p Kademlia — M2.7.3
CheckpointingM2.6.1 Procedural Memory
TelemetryRAPL / Memory / Conflicts
CI Matrix
Ubuntu + Windows +macOS
Architecture → Code Mapping
Table
Layer	Source Directory	Key Modules	Brick
SAT Core	src/pim_solver/	cdcl.rs — CDCL engine, conflict analysis	M2.5–M2.5.4
Proof Logging	src/memory/	proof.rs — Epistemic DRAT trace	M2.7.2
Clause Registry	src/memory/	registry.rs, packet.rs — Lit-Pack protocol	M2.7.1
Distributed Memory	src/memory/	dht.rs — DHT placeholder (M2.7.3)	M2.7.3
Telemetry	src/pim_solver/	telemetry.rs — SolverTelemetry	M2.5.6
Checkpointing	src/pim_solver/	checkpoint.rs — State save/load	M2.6.1
CLI Binary	src/bin/	sat_solver.rs — DIMACS + proof output	M2.6
CI/CD	.github/workflows/	rust.yml — Matrix build + DRAT validation	M2.6.3
Docker	Dockerfile	Multi-stage: rust builder + debian runtime	M2.6.5
Epistemic DRAT Proof Format
Standard DRAT is extended with c epistemic comment lines that carry clause provenance:
plain
c epistemic origin_id=0 lbd=3 timestamp=1751234567
a 1 -2 3 0
c epistemic origin_id=0 lbd=2 timestamp=1751234568
a -4 5 0
Table
Field	Type	Meaning
origin_id	u8	0 = local solver; 1–255 = distributed peer
lbd	u8	Literal Block Distance (unique decision levels in clause)
timestamp	u32	Birth epoch — seconds since UNIX epoch
Compatibility: drat-trim ignores all c lines. Standard DRAT checkers work unchanged.
Measurement Definitions
Table
Term	What We Mean	What We Measure	Test Enforcing
Correctness	Solver returns correct SAT/UNSAT	DRAT proof validates via drat-trim	test_drat_output_valid
Determinism	Identical CNF → identical result	Bit-exact reproduction	test_deterministic_replay
Auditability	Every learned clause is traceable	Epistemic metadata in DRAT	test_local_meta_serialization
Performance	No regression without evidence	Benchmark baselines per brick	CI matrix timing
Reproducibility	Clean clone → full build + test	cargo test --lib passes	GitHub Actions
What the System Provides Today
1. CDCL SAT Solver Core
Watched literal unit propagation
1-UIP conflict analysis with clause learning
VSIDS decision heuristics with activity decay
Clause database reduction (LRU + activity)
Restart policy (Luby + glucose-style)
2. Epistemic DRAT Proof Logging
Every learned clause carries origin_id, lbd, timestamp
Dynamic LBD computation from trail decision levels
serde serialization for checkpoint compatibility
Backward compatible with drat-trim and all standard checkers
3. Local Clause Registry (M2.7.1)
Lit-Pack binary protocol: Header(32b) + Metadata(32b) + Payload(i32[])
Epistemic filtering: Utility(C) = α·LBD + β·Size + γ·Activity
Content-hash deduplication
LBD-threshold query for strategic clause retrieval
4. Cross-Platform Build
Native Windows (MSVC), Linux (GCC), macOS (Clang)
Docker container for SAT Competition submission
CI matrix: ubuntu-latest, windows-latest, macos-latest
5. Deterministic Checkpointing (M2.6.1)
Full solver state save/load via serde
Procedural memory for long-running instances
Cross-platform state portability
Competition Compliance
Table
Requirement	Status	Evidence
DIMACS CNF input	✅	src/bin/sat_solver.rs
SAT exit code 10	✅	std::process::exit(10)
UNSAT exit code 20	✅	std::process::exit(20)
DRAT proof output	✅	--proof <file.drat> flag
Proof verifiable by drat-trim	✅	s VERIFIED in CI
Docker container	✅	harmonis-prime:sat-2027
Single static binary	✅	--no-default-features build
Reproducibility Protocol
Minimal Run
bash
git clone https://github.com/Ayub19123/Harmonis-Prime.git
cd Harmonis-Prime
cargo test --lib -- --nocapture
With DRAT Validation
bash
# Linux only (drat-trim compiled in CI)
cargo test --lib test_drat_output_valid -- --nocapture
Build Competition Binary
bash
cargo build --release --bin sat_solver --no-default-features
Docker Build
bash
docker build -t harmonis-prime:sat-2027 .
Scientific Positioning
Explicit Non-Claims
This project does NOT claim:
Formal proof of SAT solver completeness (standard CDCL properties apply)
Production distributed system deployment (M2.7.3+ is research)
Hardware acceleration (FPGA/SIMD — future work)
"First-of-its-kind" status (we stand on MiniSAT, CaDiCaL, Maple)
Aerospace-grade certification (no DO-178C/ECSS)
Validation Methodology
Deterministic unit testing (170 tests)
DRAT proof validation via drat-trim (external checker)
CI matrix reproducibility (3 platforms)
Benchmark baselines per brick (no regression without evidence)
Epistemic metadata audit trail (every clause traceable)
Roadmap
Phase 1–3: SAT Competition 2027 Foundation ✅ COMPLETE
All bricks sealed. Solver builds, tests, proves, and packages across all platforms.
Phase 4: Distributed Memory 🔥 ACTIVE
Table
Feature	Status	Brick
Local Clause Registry	✅ Sealed	M2.7.1
Epistemic DRAT Logging	✅ Sealed	M2.7.2
DHT Bootstrap (libp2p)	🔲 Ready	M2.7.3
Registry ↔ CDCL Integration	🔲 Ready	M2.7.4
Conflict-Aware Routing	🔲 Future	M2.7.5
Phase 5: Summit Vision 📋 FUTURE (Post-March 2027)
Table
Feature	Status	Brick
PUF Identity	Not started	SET-6C
HBM Telemetry	Not started	SET-6D
CoreSight Tracing	Not started	SET-6E
Neural Heuristic Engine	Not started	M2.8
GPU Offloading	Not started	M2.10
Academic Resources
Table
Resource	Link	DOI
Whitepaper v1.2 (Zenodo)	zenodo.org/record/15542022	10.5281/zenodo.15542022
Whitepaper v1.2 (Figshare)	figshare.com/articles/26103446	—
GitHub Repository	github.com/Ayub19123/Harmonis-Prime	—
GitHub Actions CI	github.com/Ayub19123/Harmonis-Prime/actions	—
SAT Competition 2027	satcompetition.github.io/2027	—
Engagement Prompts
We invite architecture feedback and contributions:
Open question: How should distributed clause exchange interact with CDCL restart policies?
Help wanted: libp2p Kademlia integration in Rust — peer discovery optimization
Architecture feedback: What heuristic should SET-8 (Neural Engine) learn first — branching or phase?
Notes on Design Philosophy
The system is structured around:
Test-first development — no brick without tests
Explicit failure modeling — every error path is handled
Deterministic execution guarantees — bitwise reproducible
Separation of simulation vs. physical hardware — PIM is software-only
All claims must be verifiable through cargo test, not external assertion
The Four Pillars are inviolable. Every brick is laid to aboloute mathematical industrial standard. No drift. No gaps. All corners covered.
Harmonis Prime rises in silence. Resilient. With zero fear, zero emotion, seeing around corners. Peace with precision.
