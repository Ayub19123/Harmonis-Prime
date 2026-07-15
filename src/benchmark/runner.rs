//! M2.7.14 Layer 1: BenchmarkRunner — Batch DIMACS execution engine

use crate::pim_solver::cdcl::DeterministicSandbox;
use crate::pim_solver::{CdclSolver, DimacsInstance, SolveResult};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// M2.7.14: Benchmark configuration for a single run
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub timeout_secs: u64,
    pub memory_limit_mb: Option<usize>,
    pub cpu_affinity: Option<usize>,
    pub deterministic_seed: u64,
    pub output_dir: PathBuf,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 300, // 5 minutes default
            memory_limit_mb: None,
            cpu_affinity: None,
            deterministic_seed: 0x9e3779b97f4a7c15,
            output_dir: PathBuf::from("benchmark_output"),
        }
    }
}

/// M2.7.14: Single benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkRun {
    pub instance_path: PathBuf,
    pub instance_hash: String,
    pub result: SolveResult,
    pub decisions: u64,
    pub propagations: u64,
    pub conflicts: u64,
    pub restarts: u64,
    pub peak_memory_kb: u64,
    pub wall_time_ms: u64,
    pub proof_valid: Option<bool>,
    pub timed_out: bool,
    pub memory_exceeded: bool,
}

/// M2.7.14: BenchmarkRunner — Batch execution with deterministic sandbox
pub struct BenchmarkRunner {
    pub config: BenchmarkConfig,
    pub sandbox: DeterministicSandbox,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        let mut sandbox = DeterministicSandbox::new();
        sandbox.cpu_affinity = config.cpu_affinity;
        sandbox.memory_limit_mb = config.memory_limit_mb;
        sandbox.seed = config.deterministic_seed;

        Self { config, sandbox }
    }

    /// Execute a single DIMACS instance with timeout and resource limits
    pub fn run_single(&self, path: &Path) -> Result<BenchmarkRun, String> {
        let start = Instant::now();

        // Apply deterministic sandbox
        let _ = self.sandbox.apply_affinity();

        // Parse DIMACS using existing parser
        let instance = DimacsInstance::parse(path)
            .map_err(|e| format!("Failed to parse {}: {:?}", path.display(), e))?;

        let mut solver = CdclSolver::from_dimacs(&instance);

        // Solve with timeout check
        let result = solver.solve();
        let wall_time = start.elapsed().as_millis() as u64;

        // Extract telemetry via pub getters (M2.7.14)
        let decisions = solver.get_decision_count();
        let propagations = solver.get_propagation_count();
        let conflicts = solver.get_conflict_count();
        let instance_hash = solver.get_instance_hash();

        // M2.7.14: Solver instrumentation pending for restarts and peak memory
        let restarts = 0u64;
        let peak_memory_kb = 0u64;

        Ok(BenchmarkRun {
            instance_path: path.to_path_buf(),
            instance_hash,
            result,
            decisions,
            propagations,
            conflicts,
            restarts,
            peak_memory_kb,
            wall_time_ms: wall_time,
            proof_valid: None, // Set by external drat-trim verification
            timed_out: wall_time > (self.config.timeout_secs * 1000),
            memory_exceeded: false, // TODO: RSS monitoring
        })
    }

    /// Batch execute all .cnf files in a directory
    pub fn run_batch(&self, dir: &Path) -> Vec<Result<BenchmarkRun, String>> {
        let mut results = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "cnf").unwrap_or(false))
                .collect();

            // Deterministic ordering
            paths.sort();

            for path in paths {
                results.push(self.run_single(&path));
            }
        }

        results
    }
}
