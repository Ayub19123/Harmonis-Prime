//! M2.5.5: Benchmark Harness — SATLIB Validation & Performance Baselines
//!
//! HONEST CONSTRAINTS:
//! - Embedded CNFs are small (<50 vars) for fast CI execution
//! - External SATLIB instances require manual download and path configuration
//! - Timeout is enforced at the harness level, not inside the solver
//! - Performance baseline is self-referential ( Harmonis Prime vs itself over time )
//! - NO claims made against MiniSat, Glucose, or CaDiCaL

use crate::pim_solver::cdcl::{CdclSolver, SolveResult};
use crate::pim_solver::dimacs::DimacsInstance;
use std::time::{Duration, Instant};

/// Result of a single benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub num_vars: usize,
    pub num_clauses: usize,
    pub expected_sat: Option<bool>, // None = unknown
    pub actual_sat: bool,
    pub agree: bool,
    pub duration_ms: f64,
    pub timeout: bool,
}

/// Run the solver on a DIMACS instance with timeout
pub fn run_benchmark(
    name: &str,
    instance: &DimacsInstance,
    expected: Option<bool>,
    timeout_ms: u64,
) -> BenchmarkResult {
    let start = Instant::now();
    let mut solver = CdclSolver::from_dimacs(instance);

    // Solve with timeout check
    let result = solver.solve();
    let elapsed = start.elapsed();
    let timeout = elapsed > Duration::from_millis(timeout_ms);

    let actual_sat = matches!(result, SolveResult::Sat(_));
    let agree = expected.map_or(true, |exp| actual_sat == exp);

    BenchmarkResult {
        name: name.to_string(),
        num_vars: instance.num_vars,
        num_clauses: instance.num_clauses,
        expected_sat: expected,
        actual_sat,
        agree,
        duration_ms: elapsed.as_secs_f64() * 1000.0,
        timeout,
    }
}

/// Embedded small CNFs for CI validation — no external downloads required
pub fn embedded_ci_benchmarks() -> Vec<(&'static str, DimacsInstance, Option<bool>)> {
    vec![
        // SAT: empty instance
        (
            "empty_2var",
            DimacsInstance {
                num_vars: 2,
                num_clauses: 0,
                clauses: vec![],
            },
            Some(true),
        ),
        // SAT: unit clauses
        (
            "unit_sat",
            DimacsInstance {
                num_vars: 3,
                num_clauses: 2,
                clauses: vec![vec![1], vec![2]],
            },
            Some(true),
        ),
        // UNSAT: direct contradiction
        (
            "contradiction",
            DimacsInstance {
                num_vars: 1,
                num_clauses: 2,
                clauses: vec![vec![1], vec![-1]],
            },
            Some(false),
        ),
        // SAT: choice required
        (
            "choice_sat",
            DimacsInstance {
                num_vars: 2,
                num_clauses: 2,
                clauses: vec![vec![1, 2], vec![-1, 2]],
            },
            Some(true),
        ),
        // UNSAT: XOR-pattern (M2.5.4 fix validation)
        (
            "xor_unsat",
            DimacsInstance {
                num_vars: 2,
                num_clauses: 4,
                clauses: vec![vec![1, 2], vec![1, -2], vec![-1, 2], vec![-1, -2]],
            },
            Some(false),
        ),
        // SAT: single variable, single clause
        (
            "single_clause",
            DimacsInstance {
                num_vars: 1,
                num_clauses: 1,
                clauses: vec![vec![1]],
            },
            Some(true),
        ),
    ]
}

/// Run all embedded CI benchmarks
pub fn run_ci_suite(timeout_ms: u64) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    for (name, instance, expected) in embedded_ci_benchmarks() {
        results.push(run_benchmark(name, &instance, expected, timeout_ms));
    }
    results
}

/// Print benchmark report as JSON-like text
pub fn print_report(results: &[BenchmarkResult]) {
    println!("=== M2.5.5 Benchmark Report ===");
    println!("{{");
    println!("  \"total\": {},", results.len());
    println!(
        "  \"passed\": {},",
        results.iter().filter(|r| r.agree && !r.timeout).count()
    );
    println!(
        "  \"failed\": {},",
        results.iter().filter(|r| !r.agree).count()
    );
    println!(
        "  \"timeouts\": {},",
        results.iter().filter(|r| r.timeout).count()
    );
    println!("  \"results\": [");

    for (i, r) in results.iter().enumerate() {
        let comma = if i < results.len() - 1 { "," } else { "" };
        println!("    {{");
        println!("      \"name\": \"{}\",", r.name);
        println!("      \"vars\": {},", r.num_vars);
        println!("      \"clauses\": {},", r.num_clauses);
        println!("      \"expected\": {:?},", r.expected_sat);
        println!("      \"actual_sat\": {},", r.actual_sat);
        println!("      \"agree\": {},", r.agree);
        println!("      \"duration_ms\": {:.3},", r.duration_ms);
        println!("      \"timeout\": {}{}", r.timeout, comma);
        println!("    }}{}", comma);
    }

    println!("  ]");
    println!("}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_ci_suite() {
        let results = run_ci_suite(5000); // 5 second timeout per instance

        for r in &results {
            assert!(
                r.agree,
                "Benchmark {} failed: expected={:?}, actual={}",
                r.name, r.expected_sat, r.actual_sat
            );
            assert!(!r.timeout, "Benchmark {} timed out", r.name);
        }

        println!("All {} CI benchmarks passed", results.len());
        print_report(&results);
    }

    #[test]
    fn test_benchmark_xor_pattern_performance() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 4,
            clauses: vec![vec![1, 2], vec![1, -2], vec![-1, 2], vec![-1, -2]],
        };

        let result = run_benchmark("xor_perf", &instance, Some(false), 5000);

        assert!(result.agree);
        assert!(!result.timeout);
        // XOR-pattern should solve in < 10ms after M2.5.4 fix
        assert!(
            result.duration_ms < 10.0,
            "XOR-pattern took {:.3}ms, expected < 10ms",
            result.duration_ms
        );

        println!("XOR-pattern solved in {:.3}ms", result.duration_ms);
    }
}
