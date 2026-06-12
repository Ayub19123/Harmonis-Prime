//! SET-3: Quantum state approximation invariant tests
//! Born rule + decoherence + normalization

use proptest::prelude::*;
use sovereign_core::quantum::approximation::{QuantumState, QuantumStateBuilder, Amplitude};

proptest! {
    /// INVARIANT: Born rule — Σ|α_i|² = 1
    #[test]
    fn born_rule_normalization(
        a in -1.0_f64..1.0,
        b in -1.0_f64..1.0,
    ) {
        let norm = (a * a + b * b).sqrt();
        if norm < 1e-9 {
            return Ok(());
        }
        
        let state = QuantumStateBuilder::new()
            .add_basis_state(a / norm, 0.0)
            .add_basis_state(b / norm, 0.0)
            .build("test")
            .unwrap();
        
        let total_prob: f64 = state.amplitudes.iter().map(|amp| amp.probability()).sum();
        prop_assert!((total_prob - 1.0).abs() < 1e-9,
            "Born rule violated: Σ|α_i|² = {}", total_prob);
    }

    /// INVARIANT: Decoherence produces valid classical outcomes
    #[test]
    fn decoherence_validity(
        seed in 0u64..u64::MAX,
    ) {
        let state = QuantumStateBuilder::new()
            .add_basis_state(1.0 / 2.0_f64.sqrt(), 0.0)
            .add_basis_state(1.0 / 2.0_f64.sqrt(), 0.0)
            .build("superposition")
            .unwrap();
        
        let result = state.collapse(seed);
        
        // Selected index must be valid
        prop_assert!(result.selected_index < state.amplitudes.len(),
            "Invalid collapse index: {}", result.selected_index);
        
        // Probability must match |α|²
        let expected_prob = state.amplitudes[result.selected_index].probability();
        prop_assert!((result.probability - expected_prob).abs() < 1e-9,
            "Probability mismatch: {} vs {}", result.probability, expected_prob);
    }

    /// INVARIANT: Quantum morphing preserves normalization
    #[test]
    fn morphing_preserves_normalization(
        t in 0.0_f64..1.0,
    ) {
        let state_a = QuantumStateBuilder::new()
            .add_basis_state(1.0, 0.0)
            .add_basis_state(0.0, 0.0)
            .build("basis_0")
            .unwrap();
        
        let state_b = QuantumStateBuilder::new()
            .add_basis_state(0.0, 0.0)
            .add_basis_state(1.0, 0.0)
            .build("basis_1")
            .unwrap();
        
        let morphed = state_a.interpolate(&state_b, t);
        let total_prob: f64 = morphed.amplitudes.iter().map(|a| a.probability()).sum();
        
        prop_assert!((total_prob - 1.0).abs() < 1e-9,
            "Morphing broke normalization: Σ|α_i|² = {}", total_prob);
    }
}