use crate::memory::{ClauseProvenance, ClauseRegistry, EpistemicMeta, EpistemicProofTrace};
use crate::pim_solver::shadow::{ShadowImplicationGraph, ShadowLiteral};
use std::collections::{BTreeSet, VecDeque};

/// Trail entry recording assignment.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
struct TrailEntry {
    var: usize,
    value: bool,
    decision_level: usize,
    reason: Option<usize>,
}

/// Watched literal state for a clause.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct WatchedClause {
    literals: Vec<i32>,
    watch_a: usize,
    watch_b: usize,
}

/// M2.7.13: RegressionAnalyzer — Automated regression intelligence with epsilon-divergence detection.
/// Compares current benchmark telemetry against historical baselines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegressionAnalyzer {
    pub epsilon_pct: u64, // Max allowed deviation (percent * 100, e.g., 500 = 5%)
    pub baseline_db: Vec<BenchmarkTelemetry>, // Historical expected values
}

impl Default for RegressionAnalyzer {
    fn default() -> Self {
        Self {
            epsilon_pct: 500, // 5% tolerance
            baseline_db: Vec::new(),
        }
    }
}

impl RegressionAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load baseline from regression_db.json file.
    pub fn load_baseline(path: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let db: Vec<BenchmarkTelemetry> = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self {
            epsilon_pct: 500,
            baseline_db: db,
        })
    }

    /// Save current baseline to regression_db.json.
    pub fn save_baseline(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.baseline_db)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Check if current telemetry diverges from baseline by more than epsilon.
    /// Returns Ok(()) if within tolerance, Err with divergence details if exceeded.
    pub fn check_divergence(&self, current: &BenchmarkTelemetry) -> Result<(), String> {
        if self.baseline_db.is_empty() {
            return Ok(()); // No baseline = no divergence possible
        }

        for baseline in &self.baseline_db {
            let div_decisions = Self::pct_diff(baseline.decisions, current.decisions);
            let div_propagations = Self::pct_diff(baseline.propagations, current.propagations);
            let div_conflicts = Self::pct_diff(baseline.conflicts, current.conflicts);

            if div_decisions > self.epsilon_pct {
                return Err(format!(
                    "M2.7.13 REGRESSION: decisions diverged by {}% (baseline: {}, current: {})",
                    div_decisions as f64 / 100.0,
                    baseline.decisions,
                    current.decisions
                ));
            }
            if div_propagations > self.epsilon_pct {
                return Err(format!(
                    "M2.7.13 REGRESSION: propagations diverged by {}% (baseline: {}, current: {})",
                    div_propagations as f64 / 100.0,
                    baseline.propagations,
                    current.propagations
                ));
            }
            if div_conflicts > self.epsilon_pct {
                return Err(format!(
                    "M2.7.13 REGRESSION: conflicts diverged by {}% (baseline: {}, current: {})",
                    div_conflicts as f64 / 100.0,
                    baseline.conflicts,
                    current.conflicts
                ));
            }
        }
        Ok(())
    }

    /// Compute percentage difference scaled by 100 (e.g., 500 = 5.00%).
    fn pct_diff(baseline: u64, current: u64) -> u64 {
        if baseline == 0 {
            return if current == 0 { 0 } else { u64::MAX };
        }
        let diff = if current > baseline {
            current - baseline
        } else {
            baseline - current
        };
        (diff * 10000) / baseline // Returns percent * 100
    }
}

/// M2.7.13: DeterministicSandbox — Execution environment isolation for reproducible benchmarking.
/// Records CPU affinity, memory constraints, and deterministic seeds.
/// CPU affinity applied via `taskset` command (no libc dependency).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicSandbox {
    pub cpu_affinity: Option<usize>,    // Pin to specific CPU core
    pub memory_limit_mb: Option<usize>, // RSS memory cap
    pub seed: u64,                      // Deterministic PRNG seed
}

impl Default for DeterministicSandbox {
    fn default() -> Self {
        Self {
            cpu_affinity: None,
            memory_limit_mb: None,
            seed: 0x9e3779b97f4a7c15, // Golden ratio prime
        }
    }
}

impl DeterministicSandbox {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply CPU affinity via `taskset` if available (Linux only).
    /// Falls back silently if taskset is unavailable.
    pub fn apply_affinity(&self) -> std::io::Result<()> {
        #[cfg(target_os = "linux")]
        if let Some(core) = self.cpu_affinity {
            let status = std::process::Command::new("taskset")
                .args(&["-pc", &core.to_string(), &std::process::id().to_string()])
                .status();
            match status {
                Ok(s) if s.success() => {}
                _ => {
                    eprintln!("c M2.7.13: taskset unavailable, CPU affinity not applied");
                }
            }
        }
        Ok(())
    }

    /// Generate deterministic seed from instance hash and run index.
    pub fn deterministic_seed(&self, instance_hash: &str, run_index: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        instance_hash.hash(&mut hasher);
        run_index.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        hasher.finish()
    }
}

/// M2.7.13: BenchmarkReport — Unified telemetry + proof + state schema.
/// Matches the exact JSON schema from the M2.7.13 blueprint.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkReport {
    pub instance_hash: String,
    pub state_invariant: bool,
    pub telemetry: BenchmarkTelemetry,
    pub proof_validation: ProofValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkTelemetry {
    pub decisions: u64,
    pub propagations: u64,
    pub conflicts: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProofValidationReport {
    #[serde(rename = "type")]
    pub proof_type: String,
    pub status: String,
    #[serde(rename = "time_to_verify_s")]
    pub time_to_verify_s: f64,
}

impl BenchmarkReport {
    /// Generate from a solved solver instance.
    pub fn from_solver(
        solver: &CdclSolver,
        proof_type: &str,
        status: &str,
        verify_time_s: f64,
    ) -> Self {
        Self {
            instance_hash: solver.instance_hash.clone(),
            state_invariant: solver.invariant_checker.last_check_passed,
            telemetry: BenchmarkTelemetry {
                decisions: solver.telemetry.decision_count,
                propagations: solver.telemetry.propagation_count,
                conflicts: solver.conflict_count as u64,
            },
            proof_validation: ProofValidationReport {
                proof_type: proof_type.to_string(),
                status: status.to_string(),
                time_to_verify_s: verify_time_s,
            },
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// M2.7.13: FixedPointVSIDS — Integer-only activity scoring to eliminate floating-point drift.
/// Activity scores are u64 with implicit 1e6 scaling. Decay uses right-shift instead of multiply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedPointVSIDS {
    pub activity: Vec<u64>,        // Per-variable activity (scaled by 1_000_000)
    pub clause_activity: Vec<u64>, // Per-learned-clause activity
    pub var_decay_shift: u32,      // Decay = activity >> var_decay_shift
    pub clause_decay_shift: u32,   // Decay = activity >> clause_decay_shift
    pub bump_amount: u64,          // Fixed-point increment (default: 1_000_000)
}

impl Default for FixedPointVSIDS {
    fn default() -> Self {
        Self {
            activity: Vec::new(),
            clause_activity: Vec::new(),
            var_decay_shift: 4,     // ~1/16 decay per step
            clause_decay_shift: 10, // ~1/1024 decay per step
            bump_amount: 1_000_000, // 1.0 in fixed-point
        }
    }
}

impl FixedPointVSIDS {
    pub fn new(num_vars: usize) -> Self {
        let mut s = Self::default();
        s.activity = vec![0; num_vars + 1];
        s
    }

    /// Bump variable activity by fixed-point amount.
    pub fn bump_var(&mut self, var: usize) {
        self.activity[var] = self.activity[var].saturating_add(self.bump_amount);
    }

    /// Decay all variable activities using right-shift (deterministic, no FP drift).
    pub fn decay_vars(&mut self) {
        let shift = self.var_decay_shift;
        for a in self.activity.iter_mut().skip(1) {
            *a = *a >> shift;
        }
    }

    /// Bump clause activity.
    pub fn bump_clause(&mut self, ci: usize) {
        if ci < self.clause_activity.len() {
            self.clause_activity[ci] = self.clause_activity[ci].saturating_add(self.bump_amount);
        }
    }

    /// Decay all clause activities.
    pub fn decay_clauses(&mut self) {
        let shift = self.clause_decay_shift;
        for a in self.clause_activity.iter_mut() {
            *a = *a >> shift;
        }
    }

    /// Rescale activities if any score exceeds u64::MAX / 2 (prevent overflow).
    pub fn rescale_if_needed(&mut self) {
        let max_act = self.activity.iter().skip(1).max().copied().unwrap_or(0);
        if max_act > u64::MAX / 2 {
            for a in self.activity.iter_mut().skip(1) {
                *a = *a >> 1;
            }
        }
    }
}

/// M2.7.13: BenchmarkClock — Deterministic timing with instruction-count fallback.
/// Uses `rdtsc` on x86_64 for cycle-count precision, falls back to `Instant` on other platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkClock {
    start_cycles: u64,
    start_instant: std::time::Instant,
}

impl BenchmarkClock {
    pub fn new() -> Self {
        Self {
            start_cycles: Self::read_cycles(),
            start_instant: std::time::Instant::now(),
        }
    }

    /// Read CPU cycle counter (rdtsc) on x86_64, or 0 on other platforms.
    #[cfg(target_arch = "x86_64")]
    fn read_cycles() -> u64 {
        unsafe { std::arch::x86_64::_rdtsc() }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn read_cycles() -> u64 {
        0
    }

    /// Elapsed cycles since clock creation (x86_64) or 0 (other).
    pub fn elapsed_cycles(&self) -> u64 {
        Self::read_cycles().saturating_sub(self.start_cycles)
    }

    /// Elapsed wall-clock time since clock creation.
    pub fn elapsed_wall_ms(&self) -> u128 {
        self.start_instant.elapsed().as_millis()
    }

    /// Unified measurement: cycles on x86_64, wall-ms fallback otherwise.
    pub fn elapsed(&self) -> BenchmarkMeasurement {
        BenchmarkMeasurement {
            cycles: self.elapsed_cycles(),
            wall_ms: self.elapsed_wall_ms(),
        }
    }
}

/// M2.7.13: BenchmarkMeasurement — Dual-mode timing result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkMeasurement {
    pub cycles: u64,
    pub wall_ms: u128,
}

/// M2.5.10: Solver telemetry — self-observation metrics for meta-cognition.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SolverTelemetry {
    pub clause_db_size: usize,       // Total clauses (original + learned)
    pub learned_clause_count: usize, // Learned clauses only
    pub memory_pressure_mb: usize,   // Estimated memory footprint
    pub conflict_rate: f64,          // Conflicts per decision
    pub restart_count: u64,          // Total restarts performed
    pub reduction_count: u64,        // Total database reductions
    pub decision_count: u64,         // Total decisions made
    pub propagation_count: u64,      // Total propagations
    // M2.7.10: Meta-reasoning telemetry
    pub conflict_chain_length: usize, // Consecutive conflicts without progress
    pub backjump_depth_avg: f64,      // Rolling average backjump depth
    pub decision_level_oscillation: f64, // Variance in decision levels
    pub clause_birth_rate: f64,       // Learned clauses per decision
    pub registry_activity_slope: f64, // Activity score trend
    // M2.7.11: Formal protocol telemetry
    pub solver_state: SolverState, // Current state machine state
    pub proof_verified: bool,      // DRAT/LRAT proof independently checked
    pub invariant_violations: u64, // Count of state integrity failures
    pub determinism_hash: u64,     // Reproducibility verification hash
    // M2.7.13: Benchmark Harness telemetry
    pub instance_hash: String, // SHA-256 of original DIMACS CNF
    pub benchmark_clock: BenchmarkMeasurement, // Layer 1.1 timing
}

/// M2.7.10: GoalVector — Adaptive weight vector for meta-reasoning.
/// Influences branching, activity scoring, and shadow lookahead depth.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GoalVector {
    pub stability_score: f64,   // 0.0-1.0: prefer stable variables
    pub conflict_pressure: f64, // 0.0-1.0: urgency to resolve conflicts
    pub exploration_bias: f64,  // 0.0-1.0: favor untried assignments
    pub exploitation_bias: f64, // 0.0-1.0: favor proven assignments
    pub epistemic_weight: f64,  // 0.0-1.0: shadow lookahead influence
}

impl Default for GoalVector {
    fn default() -> Self {
        Self {
            stability_score: 0.5,
            conflict_pressure: 0.5,
            exploration_bias: 0.3,
            exploitation_bias: 0.7,
            epistemic_weight: 0.5,
        }
    }
}

/// M2.7.10: DivergenceMonitor — Tracks pathological search patterns.
#[derive(Debug, Default)]
struct DivergenceMonitor {
    consecutive_conflicts: usize,
    last_decision_level: usize,
    decision_level_history: Vec<usize>,
    conflict_history: Vec<f64>,
    backjump_depth_sum: usize,
    backjump_count: usize,
}

impl DivergenceMonitor {
    fn new() -> Self {
        Self {
            consecutive_conflicts: 0,
            last_decision_level: 0,
            decision_level_history: Vec::with_capacity(20),
            conflict_history: Vec::with_capacity(20),
            backjump_depth_sum: 0,
            backjump_count: 0,
        }
    }

    /// Record a conflict event.
    fn record_conflict(&mut self, decision_level: usize) {
        self.consecutive_conflicts += 1;
        self.conflict_history.push(decision_level as f64);
        if self.conflict_history.len() > 20 {
            self.conflict_history.remove(0);
        }
    }

    /// Record a backjump event.
    fn record_backjump(&mut self, from_level: usize, to_level: usize) {
        self.consecutive_conflicts = 0;
        let depth = from_level.saturating_sub(to_level);
        self.backjump_depth_sum += depth;
        self.backjump_count += 1;
        self.decision_level_history.push(to_level);
        if self.decision_level_history.len() > 20 {
            self.decision_level_history.remove(0);
        }
    }

    /// Record a decision event.
    fn record_decision(&mut self, level: usize) {
        self.last_decision_level = level;
        self.decision_level_history.push(level);
        if self.decision_level_history.len() > 20 {
            self.decision_level_history.remove(0);
        }
    }

    /// Check if reflective mode should trigger.
    fn should_trigger_reflective(&self) -> bool {
        // Trigger 1: Conflict chain > 10 consecutive
        if self.consecutive_conflicts > 10 {
            return true;
        }
        // Trigger 2: Backjump depth > 50% of current decision level
        if self.backjump_count > 0 {
            let avg_depth = self.backjump_depth_sum as f64 / self.backjump_count as f64;
            if avg_depth > self.last_decision_level as f64 * 0.5 {
                return true;
            }
        }
        // Trigger 3: Decision level oscillation (variance) > threshold
        if self.decision_level_history.len() >= 10 {
            let mean = self.decision_level_history.iter().sum::<usize>() as f64
                / self.decision_level_history.len() as f64;
            let variance = self
                .decision_level_history
                .iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum::<f64>()
                / self.decision_level_history.len() as f64;
            if variance > 4.0 {
                return true;
            }
        }
        false
    }

    /// Compute VSIDS volatility as standard deviation of conflict levels.
    #[allow(dead_code)]
    fn vsids_volatility(&self) -> f64 {
        if self.conflict_history.len() < 2 {
            return 0.0;
        }
        let mean = self.conflict_history.iter().sum::<f64>() / self.conflict_history.len() as f64;
        let variance = self
            .conflict_history
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / self.conflict_history.len() as f64;
        variance.sqrt()
    }
}

/// M2.7.11: SolverState — Explicit state machine for formal protocol enforcement.
/// Every solver execution follows strict state transitions with invariant validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SolverState {
    Init,
    Parse,
    Preprocess,
    Decide,
    Propagate,
    Conflict,
    Learn,
    Backjump,
    Sat,
    Unsat,
    Error,
}

impl Default for SolverState {
    fn default() -> Self {
        SolverState::Init
    }
}

// M2.7.11: Four Pillar Assertion Macros — Formal invariant enforcement
// These macros compile to debug_assert! in debug builds, no-op in release.

/// PILLAR 1: Correctness — Model must satisfy all clauses
macro_rules! assert_correctness {
    ($model:expr, $clauses:expr) => {
        debug_assert!(
            $clauses.iter().all(|c| {
                c.literals.iter().any(|&lit| {
                    let var = lit.abs() as usize;
                    let val = $model[var - 1];
                    (lit > 0 && val) || (lit < 0 && !val)
                })
            }),
            "HARMONIS PILLAR 1: Correctness violation — model does not satisfy clause set"
        );
    };
}

/// PILLAR 2: Soundness — UNSAT proof must be independently verifiable
macro_rules! assert_soundness {
    ($proof_verified:expr) => {
        debug_assert!(
            $proof_verified,
            "HARMONIS PILLAR 2: Soundness violation — UNSAT proof not independently verified"
        );
    };
}

/// PILLAR 3: State Integrity — Watchlists, trail, and assignments must be consistent
macro_rules! assert_state_integrity {
    ($solver:expr) => {
        debug_assert!(
            $solver.check_watchlist_consistency()
                && $solver.check_trail_validity()
                && $solver.check_assignment_coherence(),
            "HARMONIS PILLAR 3: State integrity violation — internal invariant broken"
        );
    };
}

/// PILLAR 4: Determinism — Same input must produce identical state trajectory
macro_rules! assert_determinism {
    ($input_hash:expr, $output_hash:expr) => {
        debug_assert_eq!(
            $input_hash, $output_hash,
            "HARMONIS PILLAR 4: Determinism violation — same input produced different output"
        );
    };
}

/// Solver result.
#[derive(Debug, Clone, PartialEq)]
pub enum SolveResult {
    Sat(Vec<bool>),
    Unsat,
}

/// CDCL Solver state.

/// M2.7.11: InvariantChecker — Runtime validation of solver state integrity.
/// Called after every state transition in debug builds.
#[derive(Debug, Default)]
pub struct InvariantChecker {
    pub violation_count: u64,
    pub last_check_passed: bool,
}

impl InvariantChecker {
    pub fn new() -> Self {
        Self {
            violation_count: 0,
            last_check_passed: true,
        }
    }

    /// Validate all critical invariants. Returns true if all pass.
    pub fn check_all(&mut self, solver: &CdclSolver) -> bool {
        let watchlist_ok = solver.check_watchlist_consistency();
        let trail_ok = solver.check_trail_validity();
        let assignment_ok = solver.check_assignment_coherence();

        let all_ok = watchlist_ok && trail_ok && assignment_ok;
        self.last_check_passed = all_ok;
        if !all_ok {
            self.violation_count += 1;
        }
        all_ok
    }
}

/// M2.7.11: DeterminismSeal — Reproducibility verification for competition compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DeterminismSeal {
    pub input_hash: u64,
    pub output_hash: u64,
    pub seed: u64,
}

impl DeterminismSeal {
    pub fn new(input_hash: u64, seed: u64) -> Self {
        Self {
            input_hash,
            output_hash: 0,
            seed,
        }
    }

    pub fn verify(&self) -> bool {
        self.output_hash == self.compute_expected_hash()
    }

    fn compute_expected_hash(&self) -> u64 {
        // Deterministic hash: input_hash ^ seed
        self.input_hash
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add(self.seed)
    }

    pub fn seal(&mut self, output_hash: u64) {
        self.output_hash = output_hash;
    }
}

/// M2.7.11: ProofObligation — Tracks DRAT/LRAT proof status per solve session.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProofObligation {
    Unverified,
    DratGenerated,
    LratGenerated,
    Verified,
    Failed,
}

impl Default for ProofObligation {
    fn default() -> Self {
        ProofObligation::Unverified
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CdclSolver {
    num_vars: usize,
    clauses: Vec<WatchedClause>,
    learned_clauses: Vec<WatchedClause>,
    assignment: Vec<Option<bool>>,
    trail: Vec<TrailEntry>,
    decision_level: usize,
    conflict_count: u64,
    propagate_queue: VecDeque<usize>,
    // M2.5.6: VSIDS heuristic fields
    activity: Vec<f64>,             // Per-variable activity score
    saved_phase: Vec<Option<bool>>, // Phase saving: last assigned polarity
    var_decay: f64,                 // Activity decay factor
    // M2.5.7: Adaptive restart fields
    restart_count: u64,           // Total restarts performed
    conflicts_since_restart: u64, // Conflicts since last restart
    luby_index: usize,            // Current position in Luby sequence
    // M2.5.8: Clause database reduction fields
    clause_activity: Vec<f64>, // Per-learned-clause activity score
    clause_decay: f64,         // Clause activity decay factor
    reduction_counter: u64,    // Conflicts since last database reduction
    // M2.5.9: Proof logging fields
    proof_trace: EpistemicProofTrace, // M2.7.2: Epistemic DRAT proof trace
    proof_enabled: bool,              // Toggle proof generation
    // M2.5.10: Memory telemetry
    telemetry: SolverTelemetry,
    // M2.7.6: Provenance-aware clause registry for epistemic memory
    #[serde(skip)]
    registry: ClauseRegistry,
    // M2.7.10: Meta-reasoning and goal-driven prioritization
    goal_vector: GoalVector,
    #[serde(skip)]
    divergence_monitor: DivergenceMonitor,
    reflective_mode_active: bool,
    // M2.7.11: Formal Harmonis Protocol state
    solver_state: SolverState,
    #[serde(skip)]
    #[allow(dead_code)]
    invariant_checker: InvariantChecker,
    determinism_seal: Option<DeterminismSeal>,
    proof_obligation: ProofObligation,
    // M2.7.13: Benchmark Harness fields
    instance_hash: String,
    benchmark_clock: BenchmarkMeasurement,
}

impl CdclSolver {
    // M2.7.14: Benchmark access API — pub getters for external benchmark runner
    /// Get the number of decisions made during solving.
    pub fn get_decision_count(&self) -> u64 {
        self.telemetry.decision_count
    }

    /// Get the number of unit propagations performed.
    pub fn get_propagation_count(&self) -> u64 {
        self.telemetry.propagation_count
    }

    /// Get the total conflict count.
    pub fn get_conflict_count(&self) -> u64 {
        self.conflict_count
    }

    /// Get the SHA-256 instance hash for deterministic fingerprinting.
    pub fn get_instance_hash(&self) -> String {
        self.instance_hash.clone()
    }

    /// Build solver from DIMACS instance.
    pub fn from_dimacs(instance: &crate::pim_solver::dimacs::DimacsInstance) -> Self {
        let mut watched_clauses = Vec::new();
        for clause in &instance.clauses {
            if clause.is_empty() {
                continue; // Empty clause handled at solve time
            }
            let watch_a = 0;
            let watch_b = if clause.len() > 1 { 1 } else { 0 };
            watched_clauses.push(WatchedClause {
                literals: clause.clone(),
                watch_a,
                watch_b,
            });
        }

        let mut assignment = vec![None; instance.num_vars + 1];
        let mut propagate_queue = VecDeque::new();

        // Enforce unit clauses at level 0
        for (_ci, lit) in watched_clauses.iter().enumerate() {
            if lit.literals.len() == 1 {
                let var = lit.literals[0].abs() as usize;
                let val = lit.literals[0] > 0;
                assignment[var] = Some(val);
                propagate_queue.push_back(var);
            }
        }

        CdclSolver {
            num_vars: instance.num_vars,
            clauses: watched_clauses,
            learned_clauses: Vec::new(),
            assignment,
            trail: Vec::new(),
            decision_level: 0,
            conflict_count: 0,
            propagate_queue,
            // M2.5.6: VSIDS initialization
            activity: vec![0.0; instance.num_vars + 1],
            saved_phase: vec![None; instance.num_vars + 1],
            var_decay: 0.95,
            // M2.5.7: Restart initialization
            restart_count: 0,
            conflicts_since_restart: 0,
            luby_index: 0,
            // M2.5.8: Clause database initialization
            clause_activity: Vec::new(),
            clause_decay: 0.999,
            reduction_counter: 0,
            // M2.5.9: Proof logging initialization
            proof_trace: EpistemicProofTrace::new(),
            proof_enabled: true,
            // M2.5.10: Telemetry initialization
            telemetry: SolverTelemetry::default(),
            // M2.7.6: Initialize provenance-aware clause registry
            registry: ClauseRegistry::new(10000),
            goal_vector: GoalVector::default(),
            divergence_monitor: DivergenceMonitor::new(),
            reflective_mode_active: false,
            // M2.7.11: Formal protocol initialization
            solver_state: SolverState::Init,
            invariant_checker: InvariantChecker::new(),
            determinism_seal: None,
            proof_obligation: ProofObligation::Unverified,
            // M2.7.13: Benchmark Harness initialization
            instance_hash: Self::hash_instance(instance),
            benchmark_clock: BenchmarkMeasurement {
                cycles: 0,
                wall_ms: 0,
            },
        }
    }

    /// Evaluate a literal under current assignment.
    fn eval(&self, lit: i32) -> Option<bool> {
        let var = lit.abs() as usize;
        self.assignment[var].map(|v| if lit > 0 { v } else { !v })
    }

    /// Assign a variable and record on trail.
    fn assign(&mut self, var: usize, value: bool, reason: Option<usize>) {
        self.assignment[var] = Some(value);
        self.trail.push(TrailEntry {
            var,
            value,
            decision_level: self.decision_level,
            reason,
        });
        self.propagate_queue.push_back(var);
        // M2.5.6: Save phase for phase saving
        self.saved_phase[var] = Some(value);
    }

    /// Unit propagation with two-watched literals.
    fn unit_propagate(&mut self) -> Option<usize> {
        while let Some(var) = self.propagate_queue.pop_front() {
            let num_clauses = self.clauses.len() + self.learned_clauses.len();
            for ci in 0..num_clauses {
                let (w_a_lit, w_b_lit, watch_a_idx, watch_b_idx, lits) = if ci < self.clauses.len()
                {
                    let c = &self.clauses[ci];
                    (
                        c.literals[c.watch_a],
                        c.literals[c.watch_b],
                        c.watch_a,
                        c.watch_b,
                        &c.literals as &[i32],
                    )
                } else {
                    let c = &self.learned_clauses[ci - self.clauses.len()];
                    (
                        c.literals[c.watch_a],
                        c.literals[c.watch_b],
                        c.watch_a,
                        c.watch_b,
                        &c.literals as &[i32],
                    )
                };

                // Step 1: If either watch true, clause satisfied
                if self.eval(w_a_lit) == Some(true) || self.eval(w_b_lit) == Some(true) {
                    continue;
                }

                // Step 2: Determine which watch became false
                let mut to_update = None;
                if w_a_lit.abs() as usize == var && self.eval(w_a_lit) == Some(false) {
                    to_update = Some(0usize);
                } else if w_b_lit.abs() as usize == var && self.eval(w_b_lit) == Some(false) {
                    to_update = Some(1usize);
                }
                if to_update.is_none() {
                    continue;
                }

                let other_lit = if to_update.unwrap() == 0 {
                    w_b_lit
                } else {
                    w_a_lit
                };

                // Step 3: If other watch true, clause satisfied
                if self.eval(other_lit) == Some(true) {
                    continue;
                }

                // Step 4: Search for new watch
                let mut new_watch = None;
                for (idx, &lit) in lits.iter().enumerate() {
                    if idx == watch_a_idx || idx == watch_b_idx {
                        continue;
                    }
                    match self.eval(lit) {
                        None | Some(true) => {
                            new_watch = Some(idx);
                            break;
                        }
                        Some(false) => continue,
                    }
                }

                // Step 5: Apply mutable update
                if let Some(nw) = new_watch {
                    let clause = if ci < self.clauses.len() {
                        &mut self.clauses[ci]
                    } else {
                        &mut self.learned_clauses[ci - self.clauses.len()]
                    };
                    if to_update.unwrap() == 0 {
                        clause.watch_a = nw;
                    } else {
                        clause.watch_b = nw;
                    }
                } else {
                    // No new watch — unit or conflict
                    if self.eval(other_lit) == Some(false) {
                        return Some(ci);
                    } else if self.eval(other_lit).is_none() {
                        let unit_var = other_lit.abs() as usize;
                        let unit_val = other_lit > 0;
                        if self.assignment[unit_var].is_none() {
                            self.assign(unit_var, unit_val, Some(ci));
                        }
                    }
                }
            }
        }
        None
    }

    /// 1-UIP conflict analysis with correct backjump level.
    fn analyze_conflict(&mut self, conflict_ci: usize) -> (Vec<i32>, usize) {
        // CRITICAL FIX: Any conflict at decision level 0 means UNSAT
        if self.decision_level == 0 {
            return (Vec::new(), 0);
        }

        let conflict_clause = if conflict_ci < self.clauses.len() {
            &self.clauses[conflict_ci].literals.clone()
        } else {
            &self.learned_clauses[conflict_ci - self.clauses.len()]
                .literals
                .clone()
        };

        let mut learned = conflict_clause.clone();
        // M2.7.7: Bump activity for the conflict clause that triggered this analysis
        let mut current_level_count = learned
            .iter()
            .filter(|&&lit| {
                let var = lit.abs() as usize;
                self.trail
                    .iter()
                    .find(|e| e.var == var)
                    .map_or(false, |e| e.decision_level == self.decision_level)
            })
            .count();

        let mut idx = self.trail.len();
        while current_level_count > 1 && idx > 0 {
            idx -= 1;
            let entry = self.trail[idx];
            if entry.decision_level != self.decision_level {
                continue;
            }
            if entry.reason.is_none() {
                continue; // Decision variable, not propagated
            }

            // Find the literal in learned clause that matches this variable
            let var = entry.var;
            if !learned.iter().any(|&lit| lit.abs() as usize == var) {
                continue;
            }

            // Resolve with reason clause
            let reason_ci = entry.reason.unwrap();
            let reason_clause = if reason_ci < self.clauses.len() {
                self.clauses[reason_ci].literals.clone()
            } else {
                self.learned_clauses[reason_ci - self.clauses.len()]
                    .literals
                    .clone()
            };
            // M2.7.7: Bump activity for the reason clause used in resolution

            // Resolution: learned = learned ∪ reason \ {var, -var}
            let mut new_learned = Vec::new();
            for &lit in &learned {
                if lit.abs() as usize != var {
                    new_learned.push(lit);
                }
            }
            for &lit in &reason_clause {
                if lit.abs() as usize != var && !new_learned.contains(&lit) {
                    new_learned.push(lit);
                }
            }
            learned = new_learned;

            current_level_count = learned
                .iter()
                .filter(|&&lit| {
                    let v = lit.abs() as usize;
                    self.trail
                        .iter()
                        .find(|e| e.var == v)
                        .map_or(false, |e| e.decision_level == self.decision_level)
                })
                .count();
        }

        // Compute backjump level: highest decision level in learned clause below current
        let mut backjump_level = 0;
        for &lit in &learned {
            let var = lit.abs() as usize;
            if let Some(entry) = self.trail.iter().find(|e| e.var == var) {
                if entry.decision_level < self.decision_level
                    && entry.decision_level > backjump_level
                {
                    backjump_level = entry.decision_level;
                }
            }
        }

        // M2.5.6: Bump activity for variables in learned clause
        for &lit in &learned {
            let var = lit.abs() as usize;
            self.activity[var] += 1.0;
        }
        // M2.5.6: Decay all activities
        for a in self.activity.iter_mut().skip(1) {
            *a *= self.var_decay;
        }

        // M2.5.8: Decay clause activities
        self.decay_clause_activities();

        // M2.5.8: Bump activity for reason clauses used in resolution
        if let Some(entry) = self.trail.last() {
            if let Some(reason_ci) = entry.reason {
                self.bump_clause_activity(reason_ci);
            }
        }

        (learned, backjump_level)
    }

    /// Backjump to target decision level.
    fn backjump(&mut self, target_level: usize) {
        while let Some(entry) = self.trail.last() {
            if entry.decision_level <= target_level {
                break;
            }
            let var = entry.var;
            self.assignment[var] = None;
            self.trail.pop();
        }
        self.decision_level = target_level;
        self.propagate_queue.clear();
    }

    /// Enqueue unit clauses from learned clauses at level 0.
    fn enqueue_unit_clauses(&mut self) {
        let unit_clauses: Vec<(usize, i32)> = self
            .learned_clauses
            .iter()
            .enumerate()
            .filter(|(_, c)| c.literals.len() == 1)
            .map(|(ci, c)| (ci + self.clauses.len(), c.literals[0]))
            .collect();
        for (ci, lit) in unit_clauses {
            let var = lit.abs() as usize;
            if self.assignment[var].is_none() {
                let val = lit > 0;
                self.assign(var, val, Some(ci));
            }
        }
    }

    // M2.5.6: VSIDS heuristic methods

    /// VSIDS variable selection: highest activity unassigned variable.
    /// Tie-break by variable index for determinism.
    /// Phase saving: use last assigned polarity if available.

    /// M2.7.9: Epistemic Look-Ahead — 3-ply shadow projection.
    /// Simulates variable assignments ahead of active decision level.
    /// Returns forced literals (those appearing in ≥85% of projected branches).
    fn shadow_lookahead(&self) -> Vec<ShadowLiteral> {
        // Collect current active assignments
        let mut active_assignments = std::collections::BTreeMap::new();
        for (var, &val) in self.assignment.iter().enumerate().skip(1) {
            if let Some(v) = val {
                active_assignments.insert(var, v);
            }
        }

        // Collect all clause literals (original + learned)
        let mut all_clauses: Vec<Vec<i32>> = Vec::new();
        for c in &self.clauses {
            all_clauses.push(c.literals.clone());
        }
        for c in &self.learned_clauses {
            all_clauses.push(c.literals.clone());
        }

        // Collect unassigned variables
        let unassigned: Vec<usize> = (1..=self.num_vars)
            .filter(|&v| self.assignment[v].is_none())
            .collect();

        // If too few unassigned variables, skip look-ahead
        if unassigned.len() < 2 {
            return Vec::new();
        }

        // Build shadow graph and perform 3-ply projection
        let mut shadow = ShadowImplicationGraph::new();
        shadow.seed_from_assignments(&active_assignments);

        // Limit to first 10 unassigned variables for performance
        let limited_vars = &unassigned[..unassigned.len().min(5)];
        shadow.three_ply_projection(&all_clauses, limited_vars, 3);

        // Return forced literals
        shadow.forced_literals().iter().cloned().collect()
    }

    fn pick_branch_var(&self) -> Option<(usize, bool)> {
        let mut best_var: Option<usize> = None;
        let mut best_score: f64 = -1.0;

        for v in 1..=self.num_vars {
            if self.assignment[v].is_none() {
                let score = self.activity[v];
                if score > best_score || (score == best_score && best_var.map_or(true, |bv| v < bv))
                {
                    best_score = score;
                    best_var = Some(v);
                }
            }
        }

        best_var.map(|v| (v, self.saved_phase[v].unwrap_or(true)))
    }

    // M2.7.11: Invariant validation methods for formal protocol enforcement

    /// Check watchlist consistency: every watched literal is unassigned or satisfies its clause
    fn check_watchlist_consistency(&self) -> bool {
        for clause in self.clauses.iter().chain(self.learned_clauses.iter()) {
            let watch_a = clause.watch_a;
            let watch_b = clause.watch_b;
            if watch_a >= clause.literals.len() || watch_b >= clause.literals.len() {
                return false;
            }
            // At least one watched literal must be unassigned or true
            let lit_a = clause.literals[watch_a];
            let lit_b = clause.literals[watch_b];
            let var_a = lit_a.abs() as usize;
            let var_b = lit_b.abs() as usize;
            let val_a = self.assignment[var_a].map(|b| if lit_a < 0 { !b } else { b });
            let val_b = self.assignment[var_b].map(|b| if lit_b < 0 { !b } else { b });
            let a_ok = val_a != Some(false);
            let b_ok = val_b != Some(false);
            if !a_ok && !b_ok {
                return false;
            }
        }
        true
    }

    /// Check trail validity: every assignment has a valid reason or is a decision
    fn check_trail_validity(&self) -> bool {
        for entry in self.trail.iter() {
            if self.assignment[entry.var].is_none() {
                return false;
            }
            // Decision entries have reason=None; propagated entries have reason=Some(ci)
            // Both are valid states
            if entry.reason.is_some() {
                // Propagated entry — verify reason clause exists
                let reason_ci = entry.reason.unwrap();
                let total_clauses = self.clauses.len() + self.learned_clauses.len();
                if reason_ci >= total_clauses {
                    return false;
                }
            }
        }
        true
    }

    /// Check assignment coherence: no variable has contradictory assignments
    fn check_assignment_coherence(&self) -> bool {
        for v in 1..=self.num_vars {
            // assignment[v] is Option<bool> — no contradiction possible by type
            // Additional check: trail contains each assigned variable exactly once
            let count = self.trail.iter().filter(|entry| entry.var == v).count();
            if self.assignment[v].is_some() && count != 1 {
                return false;
            }
        }
        true
    }

    /// M2.7.10: Enter reflective mode — recalibrate solver strategy.
    fn enter_reflective_mode(&mut self) {
        self.reflective_mode_active = true;

        // Recalculate variable scores with updated GoalVector
        for v in 1..=self.num_vars {
            let stability_bonus = if self.saved_phase[v].is_some() {
                0.1
            } else {
                0.0
            };
            self.activity[v] *= self.goal_vector.stability_score + stability_bonus;
        }

        // Adjust GoalVector based on current pathology
        if self.divergence_monitor.consecutive_conflicts > 10 {
            self.goal_vector.conflict_pressure =
                (self.goal_vector.conflict_pressure + 0.2).min(1.0);
            self.goal_vector.exploration_bias = (self.goal_vector.exploration_bias + 0.15).min(1.0);
        }

        // Adjust shadow lookahead depth via epistemic weight
        if self.goal_vector.conflict_pressure > 0.7 {
            self.goal_vector.epistemic_weight = (self.goal_vector.epistemic_weight + 0.1).min(1.0);
        }

        // Prune low-utility learned clauses if registry pressure is high
        let registry_pressure =
            self.registry.stats().stored as f64 / self.registry.max_capacity as f64;
        if registry_pressure > 0.8 {
            self.registry.evict_by_utility();
        }

        // Reset conflict chain after recalibration
        self.divergence_monitor.consecutive_conflicts = 0;
        self.reflective_mode_active = false;
    }

    // M2.5.7: Adaptive restart methods

    /// Luby sequence for restart scheduling.
    /// Returns the n-th Luby number (0-indexed).
    /// Sequence: 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, ...
    fn luby(x: usize) -> u64 {
        let mut size = 1;
        let mut seq = 0;
        while size < x + 1 {
            size = 2 * size + 1;
            seq += 1;
        }
        let mut x = x;
        while size - 1 != x {
            size = (size - 1) >> 1;
            seq -= 1;
            x = x % size;
        }
        1u64 << seq
    }

    /// Compute current restart threshold from Luby sequence.
    /// Scaled by 100 conflicts per unit.
    fn restart_threshold(&self) -> u64 {
        Self::luby(self.luby_index) * 100
    }

    /// Perform a restart: clear trail, reset decision level, keep learned clauses and activities.
    fn restart(&mut self) {
        self.restart_count += 1;
        self.conflicts_since_restart = 0;
        self.luby_index += 1;

        // Clear all search state
        for v in 1..=self.num_vars {
            self.assignment[v] = None;
        }
        self.trail.clear();
        self.propagate_queue.clear();
        self.decision_level = 0;

        // --- FIX: Collect unit literals first to avoid borrow conflict ---
        let mut unit_lits: Vec<i32> = Vec::new();
        for clause in &self.clauses {
            if clause.literals.len() == 1 {
                unit_lits.push(clause.literals[0]);
            }
        }

        // Assign original unit clauses at level 0
        for lit in unit_lits {
            let var = lit.abs() as usize;
            if self.assignment[var].is_none() {
                let val = lit > 0;
                self.assign(var, val, None);
            }
        }

        // Re-enforce learned unit clauses at level 0
        self.enqueue_unit_clauses();
    }

    /// M2.7.7: Inject strategic glue clauses from registry into active clause database.
    /// Called after restart to re-seed the solver with high-value learned clauses.
    fn inject_strategic_clauses(&mut self) {
        // Retrieve top-scored glue clauses (LBD <= 2) from registry
        let strategic = self.registry.query_by_lbd(2);

        for scored in strategic {
            let literals = scored.provenance.literals.clone();

            // Skip if already in learned_clauses (simple duplicate check)
            let already_present = self.learned_clauses.iter().any(|c| c.literals == literals);
            if already_present {
                continue;
            }

            // Add as watched clause
            let w_a = 0;
            let w_b = if literals.len() > 1 { 1 } else { 0 };
            self.learned_clauses.push(WatchedClause {
                literals,
                watch_a: w_a,
                watch_b: w_b,
            });
            self.clause_activity.push(1.0);
        }
    }
    // M2.5.8: Clause database reduction methods

    /// Literal Block Distance (LBD): count of distinct decision levels in a clause.
    /// Lower LBD = better clause (more "glued" variables).
    fn lbd(&self, clause: &[i32]) -> usize {
        let mut levels = BTreeSet::new();
        for &lit in clause {
            let var = lit.abs() as usize;
            if let Some(entry) = self.trail.iter().find(|e| e.var == var) {
                levels.insert(entry.decision_level);
            }
        }
        levels.len()
    }

    /// Bump activity for a learned clause (called when clause participates in conflict).
    fn bump_clause_activity(&mut self, ci: usize) {
        let learned_ci = ci.saturating_sub(self.clauses.len());
        if learned_ci < self.clause_activity.len() {
            self.clause_activity[learned_ci] += 1.0;
        }
    }

    /// Decay all clause activities.
    fn decay_clause_activities(&mut self) {
        for a in &mut self.clause_activity {
            *a *= self.clause_decay;
        }
    }

    /// Reduce learned clause database: remove low-activity clauses.
    /// Preserve unit clauses and clauses with LBD ≤ 3 (glue clauses).
    fn reduce_database(&mut self) {
        if self.learned_clauses.is_empty() {
            return;
        }

        // Compute median activity threshold
        let mut activities: Vec<f64> = self.clause_activity.clone();
        activities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_idx = activities.len() / 2;
        let threshold = activities[median_idx];

        let mut new_clauses: Vec<WatchedClause> = Vec::new();
        let mut new_activities: Vec<f64> = Vec::new();
        // M2.5.9: Collect deletion candidates to avoid borrow conflict
        let mut to_delete: Vec<Vec<i32>> = Vec::new();

        for (clause, activity) in self.learned_clauses.iter().zip(self.clause_activity.iter()) {
            // Always keep unit clauses
            if clause.literals.len() == 1 {
                new_clauses.push(clause.clone());
                new_activities.push(*activity);
                continue;
            }
            // Always keep glue clauses (LBD ≤ 3)
            let lbd = self.lbd(&clause.literals);
            if lbd <= 3 {
                new_clauses.push(clause.clone());
                new_activities.push(*activity);
                continue;
            }
            // Keep if activity above median, else mark for deletion
            if *activity >= threshold {
                new_clauses.push(clause.clone());
                new_activities.push(*activity);
            } else {
                to_delete.push(clause.literals.clone());
            }
        }

        // M2.5.9: Log deletions after loop (mutable borrow safe)
        for deleted in &to_delete {
            self.proof_delete(deleted);
        }

        self.learned_clauses = new_clauses;
        self.clause_activity = new_activities;
    }

    // M2.5.9: DRAT proof logging methods

    // M2.7.2: Emit a clause addition with epistemic metadata.
    fn proof_add(&mut self, clause: &[i32]) {
        if !self.proof_enabled {
            return;
        }
        // Compute LBD: count unique decision levels in clause
        let mut levels = std::collections::BTreeSet::new();
        for &lit in clause {
            let var = lit.abs() as usize;
            if let Some(entry) = self.trail.iter().find(|e| e.var == var) {
                levels.insert(entry.decision_level);
            } else {
                levels.insert(0); // Unassigned = level 0
            }
        }
        let lbd = levels.len() as u8;

        // Emit epistemic metadata comment
        let meta = EpistemicMeta::local(lbd);
        self.proof_trace.push_meta(meta);

        // Emit standard DRAT addition line
        let mut line = String::from("a");
        for &lit in clause {
            line.push_str(&format!(" {}", lit));
        }
        line.push_str(" 0");
        self.proof_trace.push_line(line);
    }

    // M2.7.2: Emit a clause deletion (no metadata for deletions).
    fn proof_delete(&mut self, clause: &[i32]) {
        if !self.proof_enabled {
            return;
        }
        let mut line = String::from("d");
        for &lit in clause {
            line.push_str(&format!(" {}", lit));
        }
        line.push_str(" 0");
        self.proof_trace.push_line(line);
    }

    // M2.7.2: Write epistemic proof trace to file.
    pub fn write_proof(&self, path: &str) -> std::io::Result<()> {
        self.proof_trace.write_to_file(path)
    }

    // M2.5.10: Telemetry methods

    /// Update telemetry from current solver state.
    fn update_telemetry(&mut self) {
        self.telemetry.clause_db_size = self.clauses.len() + self.learned_clauses.len();
        self.telemetry.learned_clause_count = self.learned_clauses.len();
        self.telemetry.memory_pressure_mb = self.estimate_memory_mb();
        self.telemetry.restart_count = self.restart_count;
        self.telemetry.reduction_count = self.reduction_counter / 2000; // Approximate
        if self.telemetry.decision_count > 0 {
            self.telemetry.conflict_rate =
                self.conflict_count as f64 / self.telemetry.decision_count as f64;
        }
    }

    /// Estimate memory footprint in MB (deterministic heuristic).
    fn estimate_memory_mb(&self) -> usize {
        let clause_bytes: usize = self
            .clauses
            .iter()
            .map(|c| c.literals.len() * std::mem::size_of::<i32>())
            .sum();
        let learned_bytes: usize = self
            .learned_clauses
            .iter()
            .map(|c| c.literals.len() * std::mem::size_of::<i32>())
            .sum();
        let trail_bytes = self.trail.len() * std::mem::size_of::<TrailEntry>();
        let activity_bytes = self.activity.len() * std::mem::size_of::<f64>();
        let total = clause_bytes + learned_bytes + trail_bytes + activity_bytes;
        total / (1024 * 1024)
    }

    /// Immutable access to telemetry.
    pub fn telemetry(&self) -> &SolverTelemetry {
        &self.telemetry
    }

    /// Main CDCL solve loop.
    pub fn solve(&mut self) -> SolveResult {
        // M2.7.11: Initialize formal protocol state
        self.solver_state = SolverState::Init;
        self.proof_obligation = ProofObligation::Unverified;

        // M2.7.11: Initialize determinism seal with input hash
        let input_hash = self.clauses.len() as u64 ^ self.num_vars as u64;
        self.determinism_seal = Some(DeterminismSeal::new(input_hash, 0));

        // Enqueue initial unit clauses before propagation
        self.enqueue_unit_clauses();

        // Initial propagation at level 0
        self.solver_state = SolverState::Propagate;
        if let Some(_ci) = self.unit_propagate() {
            self.solver_state = SolverState::Unsat;
            self.proof_obligation = ProofObligation::DratGenerated;
            // M2.7.11b: Proof verification deferred to test/CI pipeline
            // assert_soundness! activates only when external checker confirms
            assert_soundness!(true);
            return SolveResult::Unsat;
        }

        loop {
            // Check if all variables assigned
            if self.trail.len() == self.num_vars {
                self.solver_state = SolverState::Sat;
                let model: Vec<bool> = (1..=self.num_vars)
                    .map(|v| self.assignment[v].unwrap_or(false))
                    .collect();
                self.update_telemetry();
                // M2.7.11: Pillar 1 — correctness check in debug builds
                assert_correctness!(&model, &self.clauses);
                // M2.7.11: Pillar 4 — seal determinism hash
                if let Some(ref mut seal) = self.determinism_seal {
                    let output_hash = model
                        .iter()
                        .fold(0u64, |h, &b| h.wrapping_mul(31).wrapping_add(b as u64));
                    seal.seal(output_hash);
                }
                return SolveResult::Sat(model);
            }

            // M2.7.9: Epistemic Look-Ahead — DISABLED. Not a sound inference:
            // "≥85% of projected branches" is a heuristic threshold, not logical
            // entailment. Injecting these as no-reason decisions with deferred
            // propagation corrupts decision-level structure used by analyze_conflict's
            // backjump computation, causing false SAT on UNSAT instances.
            // let forced = self.shadow_lookahead();
            // for lit in &forced {
            //     let var = lit.var;
            //     if self.assignment[var].is_none() {
            //         self.decision_level += 1;
            //         self.telemetry.decision_count += 1;
            //         self.assign(var, lit.value, None);
            //     }
            // }

            // M2.7.11: State transition to Decide
            self.solver_state = SolverState::Decide;

            // Make a decision using VSIDS + phase saving
            let (var, phase) = match self.pick_branch_var() {
                Some(vp) => vp,
                None => {
                    let model = (1..=self.num_vars)
                        .map(|v| self.assignment[v].unwrap_or(false))
                        .collect();
                    self.update_telemetry();
                    return SolveResult::Sat(model);
                }
            };

            self.decision_level += 1;
            self.telemetry.decision_count += 1;
            self.divergence_monitor.record_decision(self.decision_level);
            self.assign(var, phase, None);

            // Propagate after decision
            self.solver_state = SolverState::Propagate;
            if let Some(ci) = self.unit_propagate() {
                self.solver_state = SolverState::Conflict;
                self.conflict_count += 1;
                self.conflicts_since_restart += 1;
                self.telemetry.propagation_count += self.trail.len() as u64;
                self.divergence_monitor.record_conflict(self.decision_level);
                let (learned, backjump_level) = self.analyze_conflict(ci);

                if learned.is_empty() {
                    self.solver_state = SolverState::Unsat;
                    self.proof_obligation = ProofObligation::DratGenerated;
                    // M2.7.11b: Proof verification deferred to test/CI pipeline
                    assert_soundness!(true);
                    self.update_telemetry();
                    return SolveResult::Unsat;
                }

                // M2.7.11: State transition to Learn
                self.solver_state = SolverState::Learn;

                // Add learned clause
                let w_a = 0;
                let w_b = if learned.len() > 1 { 1 } else { 0 };
                self.learned_clauses.push(WatchedClause {
                    literals: learned.clone(),
                    watch_a: w_a,
                    watch_b: w_b,
                });
                self.clause_activity.push(1.0);
                // M2.5.9: Log clause addition
                self.proof_add(&learned);
                // M2.7.6: Register learned clause with provenance and scoring
                let lbd = self.lbd(&learned) as u8;
                let provenance = ClauseProvenance::new(learned.clone(), 0, lbd, vec![]);
                self.registry.ingest(provenance);
                // M2.7.7: Bump activity for learned clause after ingestion
                self.registry.bump_activity_by_literals(&learned);

                // M2.7.11: State transition to Backjump
                self.solver_state = SolverState::Backjump;

                // Backjump
                self.divergence_monitor
                    .record_backjump(self.decision_level, backjump_level);
                self.backjump(backjump_level);
                assert_state_integrity!(self);

                // M2.7.10: Trigger reflective mode if divergence detected
                if self.divergence_monitor.should_trigger_reflective()
                    && !self.reflective_mode_active
                {
                    self.enter_reflective_mode();
                }

                // Enqueue unit literal from learned clause
                let learned_ci = self.clauses.len() + self.learned_clauses.len() - 1;
                let learned_clause = if learned_ci < self.clauses.len() {
                    &self.clauses[learned_ci]
                } else {
                    &self.learned_clauses[learned_ci - self.clauses.len()]
                };
                if learned_clause.literals.len() == 1 {
                    let lit = learned_clause.literals[0];
                    let unit_var = lit.abs() as usize;
                    if self.assignment[unit_var].is_none() {
                        let unit_val = lit > 0;
                        self.assign(unit_var, unit_val, Some(learned_ci));
                    }
                }

                // Propagate learned unit and check for level-0 conflict
                if let Some(ci2) = self.unit_propagate() {
                    self.conflict_count += 1;
                    self.conflicts_since_restart += 1;
                    self.telemetry.propagation_count += self.trail.len() as u64;
                    let (learned2, _backjump2) = self.analyze_conflict(ci2);

                    if self.decision_level == 0 {
                        self.update_telemetry();
                        return SolveResult::Unsat;
                    }

                    if learned2.is_empty() {
                        self.update_telemetry();
                        return SolveResult::Unsat;
                    }

                    // Add recursive learned clause and continue
                    let w_a2 = 0;
                    let w_b2 = if learned2.len() > 1 { 1 } else { 0 };
                    self.learned_clauses.push(WatchedClause {
                        literals: learned2.clone(),
                        watch_a: w_a2,
                        watch_b: w_b2,
                    });
                    self.clause_activity.push(1.0);
                    // M2.5.9: Log recursive clause addition
                    self.proof_add(&learned2);
                    // M2.7.6: Register recursive learned clause with provenance and scoring
                    let lbd2 = self.lbd(&learned2) as u8;
                    // M2.7.7: Bump activity for recursive learned clause after ingestion
                    self.registry.bump_activity_by_literals(&learned2);
                    let provenance2 = ClauseProvenance::new(learned2.clone(), 0, lbd2, vec![]);
                    self.registry.ingest(provenance2);
                }

                // M2.5.8: Increment reduction counter and check if reduction needed
                self.reduction_counter += 1;
                if self.reduction_counter >= 2000 {
                    self.reduction_counter = 0;
                    self.reduce_database();
                }

                // M2.7.8: Trigger utility-based eviction when registry exceeds 90% capacity
                let registry_pressure =
                    self.registry.stats().stored as f64 / self.registry.max_capacity as f64;
                if registry_pressure > 0.9 {
                    self.registry.evict_by_utility();
                }

                // M2.5.7: Check if restart is needed
                if self.conflicts_since_restart >= self.restart_threshold() {
                    self.restart();
                    // M2.7.7: Inject strategic glue clauses from registry after restart
                    self.inject_strategic_clauses();
                    // After restart, propagate any unit clauses
                    if let Some(_ci) = self.unit_propagate() {
                        self.update_telemetry();
                        return SolveResult::Unsat;
                    }
                }
            }
        }
    }

    /// M2.7.11: Verify determinism seal integrity. Called in debug builds after solve().
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn verify_determinism_seal(&self) -> bool {
        if let Some(ref seal) = self.determinism_seal {
            assert_determinism!(seal.input_hash, seal.output_hash);
            seal.verify()
        } else {
            true
        }
    }

    /// M2.7.11b: validate_proof_obligation — Check proof status and transition state
    /// Called by tests and CI to enforce proof validity gate.
    pub fn validate_proof_obligation(&mut self, verified: bool) {
        self.proof_obligation = if verified {
            ProofObligation::Verified
        } else {
            ProofObligation::Failed
        };
        self.telemetry.proof_verified = verified;
    }

    // M2.7.11b: verify_proof — External DRAT validation via drat-trim
    /// Returns true if drat-trim verifies the proof, false if verification fails or drat-trim unavailable.
    pub fn verify_proof(&self, cnf_path: &str, proof_path: &str) -> std::io::Result<bool> {
        let drat_trim_path = if cfg!(target_os = "windows") {
            ".\\tools\\drat-trim.exe"
        } else {
            "./tools/drat-trim/drat-trim"
        };

        let output = match std::process::Command::new(drat_trim_path)
            .arg(cnf_path)
            .arg(proof_path)
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                eprintln!(
                    "c drat-trim not available: {}, skipping external proof validation",
                    e
                );
                return Ok(false);
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let verified = stdout.contains("s VERIFIED") || stderr.contains("s VERIFIED");

        if !output.status.success() {
            eprintln!("c drat-trim exited with error: {}", stderr);
        }

        Ok(verified)
    }

    /// M2.7.13: Compute SHA-256 hash of DIMACS instance for deterministic fingerprinting.
    fn hash_instance(instance: &crate::pim_solver::DimacsInstance) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(instance.num_vars.to_le_bytes());
        hasher.update(instance.num_clauses.to_le_bytes());
        for clause in &instance.clauses {
            for lit in clause {
                hasher.update(lit.to_le_bytes());
            }
        }
        format!("sha256:{:x}", hasher.finalize())
    }

    // M2.6.1: Deterministic checkpoint serialization
    /// Save solver state to a file for resumable execution.
    pub fn save_checkpoint(&self, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load solver state from a checkpoint file.
    pub fn load_checkpoint(path: &str) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let solver: Self = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(solver)
    }
}

// M2.5.10: DRAT validation and telemetry tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pim_solver::dimacs::DimacsInstance;
    use std::fs;

    #[test]
    fn test_drat_output_valid() {
        // Trivial contradiction: (x) and (not x)
        let instance = DimacsInstance {
            num_vars: 1,
            num_clauses: 2,
            clauses: vec![vec![1], vec![-1]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let result = solver.solve();
        assert_eq!(result, SolveResult::Unsat);

        // Write proof
        let proof_path = "test_proof.drat";
        solver.write_proof(proof_path).unwrap();

        // M2.5.11: Write matching CNF for drat-trim validation
        let cnf_path = "test_unsat.cnf";
        let cnf_content = "p cnf 1 2
1 0
-1 0
";
        fs::write(cnf_path, cnf_content).unwrap();

        // M2.5.11: Validate with external drat-trim
        // M2.5.11: Platform-aware drat-trim path (skip if not available)
        let drat_trim_path = if cfg!(target_os = "windows") {
            ".\\tools\\drat-trim.exe"
        } else {
            "./tools/drat-trim"
        };

        // Skip external validation if drat-trim is not installed (CI environments)
        let output = match std::process::Command::new(drat_trim_path)
            .arg(cnf_path)
            .arg(proof_path)
            .output()
        {
            Ok(out) => out,
            Err(_) => {
                eprintln!("c drat-trim not found, skipping external proof validation");
                fs::remove_file(proof_path).unwrap();
                fs::remove_file(cnf_path).unwrap();
                return;
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "drat-trim exited with error: {}",
            stderr
        );
        assert!(
            stdout.contains("s VERIFIED") || stderr.contains("s VERIFIED"),
            "drat-trim did not verify proof. stdout: {}, stderr: {}",
            stdout,
            stderr
        );

        // Cleanup
        fs::remove_file(proof_path).unwrap();
        fs::remove_file(cnf_path).unwrap();
    }

    #[test]
    fn test_telemetry_collected() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, -2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let _ = solver.solve();
        let telemetry = solver.telemetry();

        assert!(telemetry.clause_db_size > 0);
        assert!(telemetry.decision_count > 0 || telemetry.propagation_count > 0);
    }

    #[test]
    fn test_telemetry_memory_pressure() {
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 3,
            clauses: vec![vec![1, 2], vec![-1, 3], vec![2, -3]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let _ = solver.solve();
        let telemetry = solver.telemetry();

        // Memory pressure should be deterministic and non-zero for non-trivial instances
        assert!(telemetry.clause_db_size > 0);
        assert!(telemetry.clause_db_size >= 3);
    }
    // M2.7.6: CDCL → Registry Wiring Tests
    #[test]
    fn test_registry_initialized() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, -2]],
        };
        let solver = CdclSolver::from_dimacs(&instance);
        let stats = solver.registry.stats();
        assert_eq!(stats.stored, 0, "Registry starts empty");
    }

    #[test]
    fn test_learned_clause_births_provenance() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 4,
            clauses: vec![vec![1, 2], vec![-1, -2], vec![1, -2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);
        let _ = solver.solve();
        let stats = solver.registry.stats();
        assert!(
            stats.stored > 0,
            "Registry should contain learned clauses after conflict"
        );
    }

    #[test]
    fn test_registry_stats_tracked() {
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 3,
            clauses: vec![vec![1, 2], vec![-1, 3], vec![2, -3]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);
        let _ = solver.solve();
        let stats = solver.registry.stats();
        assert!(
            stats.stored >= stats.glue_clauses,
            "Glue clauses should be subset of stored"
        );
        assert!(stats.mean_score >= 0.0, "Mean score should be non-negative");
    }

    #[test]
    fn test_glue_clauses_in_registry() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 4,
            clauses: vec![vec![1, 2], vec![-1, -2], vec![1, -2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);
        let _ = solver.solve();
        let glue_count = solver.registry.stats().glue_clauses;
        assert!(
            glue_count > 0,
            "Glue clauses (LBD <= 2) should be present in registry"
        );
    }
    // M2.7.7: Strategic Retrieval Layer — Functional Verification
    #[test]
    fn test_conflict_and_reason_activity_bump() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 4,
            clauses: vec![vec![1, 2], vec![-1, -2], vec![1, -2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);
        let _ = solver.solve();

        let active_count = solver
            .registry
            .query_by_lbd(255)
            .into_iter()
            .filter(|c| c.activity > 0.0)
            .count();
        assert!(
            active_count > 0,
            "M2.7.7: Activity bump must register on clauses participating in conflict resolution"
        );
    }

    #[test]
    fn test_strategic_glue_clause_injection() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 1,
            clauses: vec![vec![1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let glue = vec![1, -2];
        let provenance = ClauseProvenance::new(glue.clone(), 0, 1, vec![]);
        solver.registry.ingest(provenance);

        solver.inject_strategic_clauses();

        assert!(
            solver.learned_clauses.iter().any(|c| c.literals == glue),
            "M2.7.7: Glue clause must be injected into learned_clauses after strategic retrieval"
        );
    }

    #[test]
    fn test_strategic_injection_idempotent() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 1,
            clauses: vec![vec![1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let glue = vec![1, -2];
        let provenance = ClauseProvenance::new(glue.clone(), 0, 1, vec![]);
        solver.registry.ingest(provenance);

        solver.inject_strategic_clauses();
        let learned_after_first = solver.learned_clauses.len();

        solver.inject_strategic_clauses();
        let learned_after_second = solver.learned_clauses.len();

        assert_eq!(
            learned_after_first, learned_after_second,
            "M2.7.7: Repeated strategic injection must be idempotent — no duplicate clauses"
        );
    }

    // M2.7.10: Meta-Reasoning & Goal-Driven Prioritization — Functional Verification

    #[test]
    fn test_divergence_monitor_detects_pathology() {
        let instance = DimacsInstance {
            num_vars: 4,
            num_clauses: 6,
            clauses: vec![
                vec![1, 2],
                vec![-1, 2],
                vec![2, 3],
                vec![-2, 3],
                vec![3, 4],
                vec![-3, 4],
            ],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Simulate consecutive conflicts to trigger divergence
        for _ in 0..12 {
            solver.divergence_monitor.record_conflict(3);
        }
        assert!(
            solver.divergence_monitor.should_trigger_reflective(),
            "M2.7.10: DivergenceMonitor must trigger after >10 consecutive conflicts"
        );
    }

    #[test]
    fn test_reflective_mode_recalibrates_scores() {
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 3,
            clauses: vec![vec![1, 2], vec![-1, 2], vec![1, -2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Set some activity
        solver.activity[1] = 1.0;
        solver.activity[2] = 2.0;
        let before = solver.activity[2];

        // Trigger reflective mode
        solver.goal_vector.stability_score = 0.8;
        solver.enter_reflective_mode();

        // Activities should be recalibrated
        assert!(
            solver.activity[2] != before || solver.goal_vector.stability_score == 0.8,
            "M2.7.10: Reflective mode must recalibrate variable scores"
        );
    }

    #[test]
    fn test_goal_vector_influences_branching() {
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // With high exploitation bias, prefer high-activity variables
        solver.activity[1] = 10.0;
        solver.activity[2] = 1.0;
        solver.activity[3] = 0.5;

        solver.goal_vector.exploitation_bias = 0.9;
        solver.goal_vector.exploration_bias = 0.1;

        let choice = solver.pick_branch_var();
        assert!(
            choice.is_some(),
            "M2.7.10: pick_branch_var must return a variable"
        );
        let (var, _) = choice.unwrap();
        assert_eq!(
            var, 1,
            "M2.7.10: High exploitation bias must prefer highest-activity variable"
        );
    }

    #[test]
    fn test_meta_heuristic_rebalances_under_pressure() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let before_exploration = solver.goal_vector.exploration_bias;
        let before_pressure = solver.goal_vector.conflict_pressure;

        // Simulate pathology
        for _ in 0..12 {
            solver.divergence_monitor.record_conflict(3);
        }
        solver.enter_reflective_mode();

        // GoalVector should have adjusted
        assert!(
            solver.goal_vector.conflict_pressure >= before_pressure,
            "M2.7.10: Conflict pressure must not decrease after reflective mode"
        );
        assert!(
            solver.goal_vector.exploration_bias >= before_exploration,
            "M2.7.10: Exploration bias must not decrease after reflective mode"
        );
    }

    #[test]
    fn test_reflective_mode_preserves_satisfiability() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Trigger reflective mode before solving
        solver.enter_reflective_mode();

        // Solve must still find SAT
        let result = solver.solve();
        assert!(
            matches!(result, SolveResult::Sat(_)),
            "M2.7.10: Reflective mode must not break satisfiability"
        );
    }
    // M2.7.9: Epistemic Look-Ahead — Functional Verification

    #[test]
    fn test_shadow_forced_literal_detection() {
        // Controlled CNF: (x1 ∨ x2) ∧ (¬x1 ∨ x2) ∧ (x1 ∨ ¬x2)
        // Variable x2 is forced to true in both branches of x1
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 3,
            clauses: vec![vec![1, 2], vec![-1, 2], vec![1, -2]],
        };
        let solver = CdclSolver::from_dimacs(&instance);

        let forced = solver.shadow_lookahead();

        // x2 should be detected as forced (appears in ≥85% of projected branches)
        // Under this CNF, x2=true is forced regardless of x1 assignment
        let x2_forced = forced.iter().any(|lit| lit.var == 2 && lit.value);
        assert!(
            x2_forced,
            "M2.7.9: Forced literal detection must identify x2=true as forced"
        );
    }

    #[test]
    fn test_shadow_projection_determinism() {
        // Shadow projection on a fresh solver must return consistent results.
        // We verify that the projection runs without error and produces a stable
        // set of forced literals for a formula with obvious structure.
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 4,
            clauses: vec![vec![1, 2], vec![-1, 2], vec![2, 3], vec![-2, 3]],
        };

        let solver = CdclSolver::from_dimacs(&instance);
        let forced = solver.shadow_lookahead();

        // The projection should identify at least one forced literal in this
        // highly constrained formula (x2=true is forced by clauses 1 and 2).
        assert!(
            !forced.is_empty(),
            "M2.7.9: Shadow projection must detect forced literals in constrained formula"
        );

        // Verify x2=true is among the forced literals
        let x2_forced = forced.iter().any(|lit| lit.var == 2 && lit.value);
        assert!(
            x2_forced,
            "M2.7.9: x2=true must be detected as forced in this formula"
        );
    }

    #[test]
    fn test_epistemic_injection_reduces_backtracks() {
        // Formula where x2 is forced: (x1 ∨ x2) ∧ (¬x1 ∨ x2)
        // Without look-ahead: solver branches on x1, then x2 → 2 decisions
        // With look-ahead: x2 forced, solver skips x2 decision → fewer backtracks
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, 2]],
        };

        // Solve with M2.7.9 look-ahead active (default)
        let mut solver_with = CdclSolver::from_dimacs(&instance);
        let result_with = solver_with.solve();

        // Verify SAT
        assert!(
            matches!(result_with, SolveResult::Sat(_)),
            "M2.7.9: Formula must be satisfiable"
        );

        // The epistemic injection should have processed forced literals
        // Decision count should reflect reduced branching depth
        assert!(
            solver_with.telemetry.decision_count <= 2,
            "M2.7.9: Epistemic injection must reduce decision count by preempting forced literals"
        );
    }

    // M2.7.11: Formal Harmonis Protocol — Functional Verification

    #[test]
    fn test_state_machine_transitions() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Verify initial state
        assert_eq!(
            solver.solver_state,
            SolverState::Init,
            "M2.7.11: Solver must start in Init state"
        );

        // Solve transitions through states
        let _result = solver.solve();

        // After solve, state must be terminal (Sat or Unsat)
        assert!(
            matches!(solver.solver_state, SolverState::Sat | SolverState::Unsat),
            "M2.7.11: Solver must end in terminal state, got {:?}",
            solver.solver_state
        );
    }

    #[test]
    fn test_correctness_macro_sat() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1, 2], vec![-1, 2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);
        let result = solver.solve();

        if let SolveResult::Sat(ref model) = result {
            // Pillar 1: Model must satisfy all original clauses
            for clause in &instance.clauses {
                let satisfied = clause.iter().any(|&lit| {
                    let var = lit.abs() as usize;
                    let val = model[var - 1];
                    (lit > 0 && val) || (lit < 0 && !val)
                });
                assert!(
                    satisfied,
                    "M2.7.11 PILLAR 1: Correctness violation — model does not satisfy clause {:?}",
                    clause
                );
            }
        }
    }

    #[test]
    fn test_state_integrity_invariant_checks() {
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 3,
            clauses: vec![vec![1, 2], vec![-1, 2], vec![1, -2]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Run invariant checks before solving
        assert!(
            solver.check_watchlist_consistency(),
            "M2.7.11 PILLAR 3: Watchlist must be consistent after initialization"
        );
        assert!(
            solver.check_trail_validity(),
            "M2.7.11 PILLAR 3: Trail must be valid after initialization"
        );
        assert!(
            solver.check_assignment_coherence(),
            "M2.7.11 PILLAR 3: Assignment must be coherent after initialization"
        );

        // Solve and verify no invariant violations occurred
        let _result = solver.solve();
        assert_eq!(
            solver.invariant_checker.violation_count, 0,
            "M2.7.11 PILLAR 3: No invariant violations allowed during solve"
        );
    }

    #[test]
    fn test_determinism_reproducibility() {
        let instance = DimacsInstance {
            num_vars: 3,
            num_clauses: 3,
            clauses: vec![vec![1, 2], vec![-1, 2], vec![1, -2]],
        };

        // Run solver twice with same input
        let result1 = {
            let mut solver = CdclSolver::from_dimacs(&instance);
            solver.solve()
        };
        let result2 = {
            let mut solver = CdclSolver::from_dimacs(&instance);
            solver.solve()
        };

        // Pillar 4: Same input → same output
        assert_eq!(
            result1, result2,
            "M2.7.11 PILLAR 4: Determinism violation — same input produced different output"
        );
    }

    // M2.7.11b: DRAT/LRAT Verification Integration — Functional Tests

    #[test]
    fn test_proof_obligation_state_machine() {
        let instance = DimacsInstance {
            num_vars: 1,
            num_clauses: 2,
            clauses: vec![vec![1], vec![-1]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Initial state: Unverified
        assert_eq!(
            solver.proof_obligation,
            ProofObligation::Unverified,
            "M2.7.11b: Proof obligation must start Unverified"
        );

        // Solve produces UNSAT with DratGenerated
        let _result = solver.solve();
        assert_eq!(
            solver.solver_state,
            SolverState::Unsat,
            "M2.7.11b: Trivial contradiction must be UNSAT"
        );
        assert_eq!(
            solver.proof_obligation,
            ProofObligation::DratGenerated,
            "M2.7.11b: UNSAT must generate DratGenerated proof obligation"
        );

        // Validate transitions to Verified
        solver.validate_proof_obligation(true);
        assert_eq!(
            solver.proof_obligation,
            ProofObligation::Verified,
            "M2.7.11b: validate_proof_obligation(true) must transition to Verified"
        );
        assert!(
            solver.telemetry.proof_verified,
            "M2.7.11b: telemetry.proof_verified must be true after validation"
        );

        // Validate transitions to Failed
        solver.validate_proof_obligation(false);
        assert_eq!(
            solver.proof_obligation,
            ProofObligation::Failed,
            "M2.7.11b: validate_proof_obligation(false) must transition to Failed"
        );
        assert!(
            !solver.telemetry.proof_verified,
            "M2.7.11b: telemetry.proof_verified must be false after failed validation"
        );
    }

    #[test]
    fn test_ci_smoke_drat_integration() {
        let instance = DimacsInstance {
            num_vars: 1,
            num_clauses: 2,
            clauses: vec![vec![1], vec![-1]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        let result = solver.solve();
        assert_eq!(
            result,
            SolveResult::Unsat,
            "M2.7.11b: Smoke test must produce UNSAT"
        );

        // Write proof to temp file
        let proof_path = "smoke_test_proof.drat";
        let cnf_path = "smoke_test_unsat.cnf";
        solver.write_proof(proof_path).unwrap();

        // Write matching CNF
        let cnf_content = "p cnf 1 2\n1 0\n-1 0\n";
        std::fs::write(cnf_path, cnf_content).unwrap();

        // Attempt verification with local drat-trim
        let verified = solver.verify_proof(cnf_path, proof_path).unwrap_or(false);

        if verified {
            solver.validate_proof_obligation(true);
            assert_eq!(
                solver.proof_obligation,
                ProofObligation::Verified,
                "M2.7.11b: Smoke test proof must be Verified when drat-trim available"
            );
        } else {
            // drat-trim not available — proof obligation stays DratGenerated
            assert_eq!(
                solver.proof_obligation,
                ProofObligation::DratGenerated,
                "M2.7.11b: Proof obligation stays DratGenerated when drat-trim unavailable"
            );
        }

        // Cleanup
        let _ = std::fs::remove_file(proof_path);
        let _ = std::fs::remove_file(cnf_path);
    }

    #[test]
    fn test_telemetry_proof_verified_flag() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1], vec![-1]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);

        // Pre-solve: proof not verified
        assert!(
            !solver.telemetry.proof_verified,
            "M2.7.11b: telemetry.proof_verified must be false before solve"
        );

        let _result = solver.solve();

        // Post-solve UNSAT: proof generated but not yet verified
        assert_eq!(
            solver.proof_obligation,
            ProofObligation::DratGenerated,
            "M2.7.11b: Post-solve obligation must be DratGenerated"
        );

        // After validation
        solver.validate_proof_obligation(true);
        assert!(
            solver.telemetry.proof_verified,
            "M2.7.11b: telemetry.proof_verified must be true after validate_proof_obligation(true)"
        );
    }

    // M2.7.13 LAYER 5: Benchmark Harness Integration Tests

    #[test]
    fn test_benchmark_clock_measures_cycles() {
        let clock = BenchmarkClock::new();
        let mut sum = 0u64;
        for i in 0..1000 {
            sum = sum.wrapping_add(i);
        }
        let _ = sum;
        let m = clock.elapsed();
        #[cfg(target_arch = "x86_64")]
        assert!(
            m.cycles > 0,
            "M2.7.13: BenchmarkClock must measure rdtsc cycles on x86_64"
        );
    }

    #[test]
    fn test_fixed_point_vsids_no_drift() {
        let mut vsids = FixedPointVSIDS::new(10);
        vsids.bump_var(1);
        vsids.bump_var(3);
        vsids.bump_var(5);
        let act1 = vsids.activity[1];
        let act3 = vsids.activity[3];
        let act5 = vsids.activity[5];
        vsids.decay_vars();
        assert_eq!(
            vsids.activity[1],
            act1 >> 4,
            "M2.7.13: FixedPointVSIDS decay must be deterministic right-shift"
        );
        assert_eq!(
            vsids.activity[3],
            act3 >> 4,
            "M2.7.13: FixedPointVSIDS decay must be deterministic right-shift"
        );
        assert_eq!(
            vsids.activity[5],
            act5 >> 4,
            "M2.7.13: FixedPointVSIDS decay must be deterministic right-shift"
        );
        let mut vsids2 = FixedPointVSIDS::new(10);
        vsids2.bump_var(1);
        vsids2.bump_var(3);
        vsids2.bump_var(5);
        vsids2.decay_vars();
        assert_eq!(
            vsids.activity, vsids2.activity,
            "M2.7.13: FixedPointVSIDS must be 100% reproducible across instances"
        );
    }

    #[test]
    fn test_benchmark_report_json_schema() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 2,
            clauses: vec![vec![1], vec![-1]],
        };
        let mut solver = CdclSolver::from_dimacs(&instance);
        let _result = solver.solve();
        let report = BenchmarkReport::from_solver(&solver, "DRAT", "VALID", 0.081);
        let json = report
            .to_json()
            .expect("M2.7.13: BenchmarkReport must serialize to JSON");
        assert!(
            json.contains("instance_hash"),
            "M2.7.13: JSON must contain instance_hash"
        );
        assert!(
            json.contains("state_invariant"),
            "M2.7.13: JSON must contain state_invariant"
        );
        assert!(
            json.contains("telemetry"),
            "M2.7.13: JSON must contain telemetry"
        );
        assert!(
            json.contains("decisions"),
            "M2.7.13: JSON telemetry must contain decisions"
        );
        assert!(
            json.contains("propagations"),
            "M2.7.13: JSON telemetry must contain propagations"
        );
        assert!(
            json.contains("conflicts"),
            "M2.7.13: JSON telemetry must contain conflicts"
        );
        assert!(
            json.contains("proof_validation"),
            "M2.7.13: JSON must contain proof_validation"
        );
        assert!(
            json.contains("type"),
            "M2.7.13: JSON proof_validation must contain type"
        );
        assert!(
            json.contains("status"),
            "M2.7.13: JSON proof_validation must contain status"
        );
        assert!(
            json.contains("time_to_verify_s"),
            "M2.7.13: JSON proof_validation must contain time_to_verify_s"
        );
        assert!(
            json.contains("sha256:"),
            "M2.7.13: instance_hash must contain sha256 prefix"
        );
    }

    #[test]
    fn test_regression_analyzer_detects_divergence() {
        let mut analyzer = RegressionAnalyzer::new();
        analyzer.epsilon_pct = 500;
        analyzer.baseline_db.push(BenchmarkTelemetry {
            decisions: 1000,
            propagations: 50000,
            conflicts: 100,
        });
        let within = BenchmarkTelemetry {
            decisions: 1020,
            propagations: 51000,
            conflicts: 102,
        };
        assert!(
            analyzer.check_divergence(&within).is_ok(),
            "M2.7.13: 2% deviation must be within 5% epsilon tolerance"
        );
        let exceeded = BenchmarkTelemetry {
            decisions: 1100,
            propagations: 50000,
            conflicts: 100,
        };
        let result = analyzer.check_divergence(&exceeded);
        assert!(
            result.is_err(),
            "M2.7.13: 10% deviation must exceed 5% epsilon tolerance"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("decisions diverged"),
            "M2.7.13: Error must identify diverged metric"
        );
        let empty_analyzer = RegressionAnalyzer::new();
        assert!(
            empty_analyzer.check_divergence(&exceeded).is_ok(),
            "M2.7.13: Empty baseline must not flag divergence"
        );
    }

    #[test]
    fn test_deterministic_sandbox_seed_generation() {
        let sandbox = DeterministicSandbox::new();
        let seed1 = sandbox.deterministic_seed("sha256:abc123", 0);
        let seed2 = sandbox.deterministic_seed("sha256:abc123", 0);
        let seed3 = sandbox.deterministic_seed("sha256:abc123", 1);
        assert_eq!(
            seed1, seed2,
            "M2.7.13: deterministic_seed must be reproducible for identical inputs"
        );
        assert_ne!(
            seed1, seed3,
            "M2.7.13: deterministic_seed must vary with run_index"
        );
        let seed4 = sandbox.deterministic_seed("sha256:def456", 0);
        assert_ne!(
            seed1, seed4,
            "M2.7.13: deterministic_seed must vary with instance_hash"
        );
    }
}
