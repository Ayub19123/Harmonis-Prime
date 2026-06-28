//! SET-5.5: Ramanujan quantum utility invariant tests
//! INVARIANT: collapse_with_ramanujan biases toward lower indices statistically
//! Mock theta weights decrease with index → lower indices have higher probability

use proptest::prelude::*;
use sovereign_core::quantum::approximation::QuantumStateBuilder;
use sovereign_core::quantum::ramanujan_collapse::collapse_with_ramanujan;

proptest! {
    /// INVARIANT: Over many trials, Ramanujan collapse selects lower indices
    /// more frequently than uniform random (statistical bias, not determinism).
    #[test]
    fn ramanujan_statistical_bias(
        size in 4usize..8,
        trials in 100usize..200,
        seed_base in 0u64..10000,
    ) {
        // Build uniform-amplitude state (all equal before Ramanujan weighting)
        let mut state = QuantumStateBuilder::new();
        for _ in 0..size {
            state = state.add_basis_state(1.0 / (size as f64).sqrt(), 0.0);
        }
        let state = state.build("test").unwrap();

        // Count selections across many trials
        let mut counts = vec![0usize; size];
        for t in 0..trials {
            let mut trial_state = state.clone();
            let result = collapse_with_ramanujan(&mut trial_state, seed_base + t as u64, 0.5);
            counts[result.selected_index] += 1;
        }

        // Index 0 should be selected most frequently (highest mock theta weight)
        // max_count computed via sorted
        let idx0_count = counts[0];

        // Statistical invariant: index 0 is in the top 2 most-selected indices
        // (allows for probabilistic variance while proving bias)
        let mut sorted: Vec<_> = counts.iter().enumerate().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1)); // descending by count

        prop_assert!(
            sorted[0].0 == 0 || sorted[1].0 == 0,
            "Index 0 (highest weight) not in top 2 selections. Counts: {:?}",
            counts
        );

        // Additional: index 0 count should exceed uniform expectation (trials/size)
        let uniform_expectation = trials / size;
        prop_assert!(
            idx0_count >= uniform_expectation,
            "Index 0 count {} < uniform expectation {}. Counts: {:?}",
            idx0_count, uniform_expectation, counts
        );
    }
}
