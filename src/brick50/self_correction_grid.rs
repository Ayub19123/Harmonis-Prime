//! BRICK-50 Pillar 4: Real-Time Self-Correction Grid
//! Non-linear feedback: lim(t→∞) Δx(t) = 0

#[derive(Clone)]
pub struct CorrectionState {
    pub delta_x: f64,
    pub convergence_rate: f64,
    pub iterations: u64,
}

pub struct SelfCorrectionGrid {
    states: Vec<CorrectionState>,
    target_convergence: f64,
    total_corrections: u64,
}

impl SelfCorrectionGrid {
    pub fn new(target_convergence: f64) -> Self {
        Self {
            states: Vec::new(),
            target_convergence: target_convergence.max(1e-15),
            total_corrections: 0,
        }
    }

    pub fn correct(&mut self, current_delta: f64) -> CorrectionState {
        self.total_corrections += 1;
        let k = 0.1;
        let new_delta = current_delta * (-k as f64).exp();
        let convergence =
            (self.target_convergence - new_delta.abs()).max(0.0) / self.target_convergence;
        let state = CorrectionState {
            delta_x: new_delta,
            convergence_rate: convergence,
            iterations: self.total_corrections,
        };
        self.states.push(state.clone());
        state
    }

    pub fn converged(&self) -> bool {
        if let Some(last) = self.states.last() {
            last.delta_x.abs() < self.target_convergence
        } else {
            false
        }
    }

    pub fn stats(&self) -> (u64, f64, bool) {
        let converged = self.converged();
        let last_delta = self.states.last().map(|s| s.delta_x.abs()).unwrap_or(1.0);
        (self.total_corrections, last_delta, converged)
    }
}
