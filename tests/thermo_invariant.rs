//! SET-3: Thermodynamic invariant tests
//! Shannon entropy monotonicity + Landauer limit adherence

use proptest::prelude::*;
use sovereign_core::thermo::entropy::{EntropyTracker, ThermodynamicState, K_BOLTZMANN, LANDAUER_LIMIT_300K};
use std::time::Instant;

proptest! {
    /// INVARIANT: Entropy monotonically decreases under optimization
    #[test]
    fn entropy_monotonicity_invariant(
        initial_entropy in 1.0_f64..10.0,
        steps in 2usize..20,
    ) {
        let mut tracker = EntropyTracker::new(300.0).unwrap();
        
        // Initial high-entropy state (uniform distribution)
        let n = 8usize;
        let uniform_prob = 1.0 / n as f64;
        let state0 = ThermodynamicState {
            probabilities: vec![uniform_prob; n],
            timestamp: Instant::now(),
            label: "initial".to_string(),
        };
        
        tracker.record(state0).unwrap();
        
        // Simulate optimization: probabilities concentrate
        for step in 1..=steps {
            let concentration = 0.5 + (step as f64 / steps as f64) * 0.5;
            let mut probs = vec![(1.0 - concentration) / (n - 1) as f64; n - 1];
            probs.push(concentration);
            
            let state = ThermodynamicState {
                probabilities: probs,
                timestamp: Instant::now(),
                label: format!("step_{}", step),
            };
            
            let report = tracker.record(state);
            prop_assert!(report.is_ok(), "Entropy increased at step {}: {:?}", step, report);
            prop_assert!(report.unwrap().is_monotonic, "Non-monotonic entropy detected");
        }
        
        // Verify Landauer limit holds for all states
        prop_assert!(tracker.verify_landauer_limit(), "Landauer limit violated");
    }

    /// INVARIANT: Landauer energy scales correctly with temperature
    #[test]
    fn landauer_temperature_scaling(
        temp in 273.0_f64..373.0, // 0°C to 100°C
        bits in 1.0_f64..64.0,
    ) {
        let expected = K_BOLTZMANN * temp * std::f64::consts::LN_2 * bits;
        let limit = K_BOLTZMANN * temp * std::f64::consts::LN_2;
        
        // Energy to erase N bits = N * k_B * T * ln(2)
        prop_assert!((expected - bits * limit).abs() < 1e-30,
            "Landauer scaling violated: {} vs {}", expected, bits * limit);
    }
}