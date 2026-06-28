//! SET-5.5: Quantum collapse with Ramanujan mock theta weights
//! Uses mock theta probabilities for heuristic decision making

use crate::quantum::approximation::{Amplitude, CollapseResult, QuantumState};
use crate::ramanujan::mock_theta::mock_theta_weight;

/// Collapse a quantum state using Ramanujan mock theta weights
pub fn collapse_with_ramanujan(state: &mut QuantumState, seed: u64, q: f64) -> CollapseResult {
    // Override amplitudes with mock theta weights
    let total = state.amplitudes.len();
    for (i, amp) in state.amplitudes.iter_mut().enumerate() {
        let weight = mock_theta_weight(i, total, q);
        let norm = weight.sqrt();
        *amp = Amplitude::new(norm, 0.0);
    }
    // Normalize to satisfy Born rule
    let norm = state
        .amplitudes
        .iter()
        .map(|a| a.probability())
        .sum::<f64>()
        .sqrt();
    if norm > 0.0 {
        for amp in &mut state.amplitudes {
            amp.real /= norm;
            amp.imag /= norm;
        }
    }
    state.collapse(seed)
}
