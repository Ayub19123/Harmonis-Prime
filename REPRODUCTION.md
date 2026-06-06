Harmonis Prime – External Reproduction Pack (Industrial Grade)



Version: 1.0

Tag: v7.1.0-BRICK51.3-FINAL-BENCH

Commit: 481da5f

Date: 2026-06-06



Objective:

Verify deterministic execution on an external machine.



This is a reproducibility check ONLY:

\- NOT a performance contest

\- NOT a code review

\- NOT a design evaluation



Only verify: "Does your output exactly match the expected output?"



1\. Immutable Reference

Repository: https://github.com/Ayub19123/Harmonis-Prime

Tag: v7.1.0-BRICK51.3-FINAL-BENCH

Commit: 481da5f



Verification commands:

git clone https://github.com/Ayub19123/Harmonis-Prime.git

cd Harmonis-Prime

git checkout v7.1.0-BRICK51.3-FINAL-BENCH

git rev-parse HEAD   (must output 481da5f)



2\. Environment Requirements (clean environment)

OS: Windows 10/11, Ubuntu 20.04+, macOS 11+

Rust: 1.96.0 (from rust-toolchain.toml)

CPU: x86\_64 or ARM64, 2+ cores

RAM: 4+ GB



Pre-run checks:

rustc --version   # must show 1.96.0

cargo --version



3\. Setup Instructions (copy-paste these lines)

git clone https://github.com/Ayub19123/Harmonis-Prime.git

cd Harmonis-Prime

git checkout v7.1.0-BRICK51.3-FINAL-BENCH

git rev-parse HEAD   (verify it's 481da5f)

rustup install 1.96.0

rustup override set 1.96.0

cargo build --release



If any warning appears: STOP. Open an issue.



4\. Expected Output Contract

Run this command:

cargo run --release --bin benchmark\_consensus -- 10000 0x51C3\_2026\_0613



Terminal must contain:

\- Iterations: 10000

\- Seed: 0x51C3\_2026\_0613

\- Mode: SIMULATION

\- Total runtime: any positive number

\- Exit code 0 (no crash)



Generated files (must exist):

\- metrics.json -> exactly 10,000 entries

\- metrics\_consensus.json -> exactly 10,000 entries

\- REPRODUCTION\_LOG.md -> witness entry



Optional SHA256:

sha256sum metrics.json metrics\_consensus.json



5\. Evidence to Return (copy this template and fill)

Environment:

\- OS:

\- CPU:

\- Rustc version:

\- Cargo version:



Terminal output (first 20 lines and last 20 lines):

...



File validation:

\- metrics.json size: bytes (SHA256: )

\- metrics\_consensus.json size: bytes (SHA256: )

\- Entry counts: both = 10000 (YES/NO)



Confirmation:

\- Execution completed successfully? YES/NO

\- Any deviations? (if yes, describe)



6\. Witness Log

Append to REPRODUCTION\_LOG.md this line:

| \[your name] | YYYY-MM-DD | 481da5f | \[OS] | 1.96.0 | PASS |



7\. Scope Limitations

This pack does NOT validate: performance ranking, AI/ML claims, real-world consensus, energy efficiency, formal proofs. Only validates deterministic execution and output structure.



8\. One-line summary

Run the command on tag v7.1.0-BRICK51.3-FINAL-BENCH and confirm 10,000 entries in two JSON files with zero errors.

