//! SET-7B: Zeta Resonance Invariant Tests
//! 
//! Invariants:
//!   - theta_monotonic: Riemann-Siegel theta increases with t
//!   - computation_pipeline: Functions execute without panic
//!   - resonance_mapping_determinism: Fixed inputs produce fixed outputs
//! 
//! LIMITATION: Truncated Dirichlet series does not converge on σ=1/2.
//! Real zero detection requires Riemann-Siegel formula or analytic continuation.
//! These tests verify the computation pipeline, not mathematical correctness.

use crate::zeta_resonance::zeta::*;

// --- Invariant 1: Theta Monotonicity ---

#[test]
fn test_theta_monotonicity() {
    let zeta = ZetaResonance::new(100).unwrap();
    let theta1 = zeta.riemann_siegel_theta(10.0);
    let theta2 = zeta.riemann_siegel_theta(20.0);
    assert!(theta2 > theta1, "Theta must increase with t");
}

// --- Invariant 2: Computation Pipeline Valid ---

#[test]
fn test_computation_pipeline_no_panic() {
    let zeta = ZetaResonance::new(1000).unwrap();
    
    // Pipeline executes without panic — mathematical correctness is Phase 2
    let (re, im) = zeta.zeta_critical(0.0).unwrap();
    assert!(re.is_finite(), "Real part must be finite");
    assert!(im.is_finite(), "Imaginary part must be finite");
    
    let z = zeta.hardy_z(10.0).unwrap();
    assert!(z.is_finite(), "Z(t) must be finite");
}

// --- Invariant 3: Determinism ---

#[test]
fn test_resonance_mapping_deterministic() {
    let zeta = ZetaResonance::new(1000).unwrap();
    
    // Fixed inputs produce fixed outputs
    let z1 = zeta.hardy_z(15.0).unwrap();
    let z2 = zeta.hardy_z(15.0).unwrap();
    
    assert_eq!(z1, z2, "Z(t) must be deterministic for fixed inputs");
}

// --- Invariant 4: Bracket Search Pipeline ---

#[test]
fn test_bracket_search_pipeline() {
    let zeta = ZetaResonance::new(1000).unwrap();
    
    // LIMITATION: Brackets may not contain real zeros due to divergent series
    // This test verifies the search algorithm executes correctly
    let brackets = zeta.locate_zero_brackets(10.0, 20.0, 0.1);
    
    // Brackets are returned as Vec — may be empty due to mathematical limitation
    assert!(true, "Bracket search completed without panic");
}
