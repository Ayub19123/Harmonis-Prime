//! SET-5.3: Long-Run Sovereign Organism Invariant Tests (CI‑friendly)

use sovereign_core::endurance::harness::{EnduranceHarness, EnduranceConfig};
use sovereign_core::endurance::checkpoint::CheckpointEngine;

#[test]
fn heap_growth_bounded_invariant() {
    // 0.0003 hours ≈ 1.08 seconds → enough for at least one checkpoint
    let config = EnduranceConfig {
        duration_hours: 0.0003,
        checkpoint_interval_secs: 1,
        max_heap_growth_percent_per_hour: 1000.0,
        max_entropy_variance: 1e-3,
        operations_per_checkpoint: 1000,
    };
    let mut harness = EnduranceHarness::new(config);
    let report = harness.run_simulated();
    assert!(report.heap_growth_rate_per_hour < 10000.0);
    assert!(report.invariant_passed);
}

#[test]
fn entropy_drift_zero_invariant() {
    let config = EnduranceConfig {
        duration_hours: 0.0003,
        checkpoint_interval_secs: 1,
        max_heap_growth_percent_per_hour: 1000.0,
        max_entropy_variance: 1e-3,
        operations_per_checkpoint: 500,
    };
    let mut harness = EnduranceHarness::new(config);
    let report = harness.run_simulated();
    assert!(report.entropy_variance <= 1e-3);
}

#[test]
fn determinism_hash_stable_invariant() {
    let mut engine = CheckpointEngine::new();
    let cp1 = engine.seal(0.5, 1000, 1_000_000);
    let cp2 = engine.seal(0.5, 1000, 1_000_000);
    assert_eq!(cp1.determinism_hash, cp2.determinism_hash);
}

#[test]
fn liveness_sustained_invariant() {
    let config = EnduranceConfig {
        duration_hours: 0.0003,
        checkpoint_interval_secs: 1,
        max_heap_growth_percent_per_hour: 1000.0,
        max_entropy_variance: 1e-3,
        operations_per_checkpoint: 100,
    };
    let mut harness = EnduranceHarness::new(config);
    let report = harness.run_simulated();
    assert!(report.total_operations > 0);
    assert!(report.success_rate >= 0.99);
    assert!(report.checkpoints >= 1);  // At least one checkpoint
}
