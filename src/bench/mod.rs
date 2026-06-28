//! M2.5.5: Benchmark Harness — External Validation & Performance Baselines
//!
//! HONEST CONSTRAINTS:
//! - Only tests small SATLIB instances (<200 vars) in CI
//! - Large instances require manual runs with timeout
//! - Performance claims only valid against baseline, not MiniSat/Glucose

pub mod harness;
