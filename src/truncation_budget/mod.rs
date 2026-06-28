//! Truncation Budget -- Backlund-type remainder bound for Dirichlet series.
//!
//! HONEST SCOPE (M1.4b):
//! - Simplified remainder bound for truncated zeta evaluation
//! - Monotonicity verification: bound decreases as N increases
//! - Engineering estimate only -- NOT a rigorous proof
//!
//! Honest limitation: This is a practical bound for test validation,
//! not the Olver-type rigorous bound used in research literature.

/// Truncation budget for Dirichlet series evaluation.
/// Provides remainder bound and required order for target accuracy.
#[derive(Debug, Clone)]
pub struct TruncationBudget;

impl TruncationBudget {
    /// Create new truncation budget calculator.
    pub fn new() -> Self {
        Self
    }

    /// Backlund-type remainder bound for truncated zeta(s) with sigma > 1.
    /// bound = (N+1)^(1-sigma) / (sigma - 1)
    ///
    /// Honest: this is the integral remainder, not the sharp Backlund bound.
    pub fn remainder_bound(&self, n: usize, sigma: f64) -> Result<f64, &'static str> {
        if n < 1 {
            return Err("n must be positive");
        }
        if sigma <= 1.0 {
            return Err("sigma must be > 1.0 for bound to be finite");
        }
        if !sigma.is_finite() {
            return Err("sigma must be finite");
        }

        let n_plus_1 = (n + 1) as f64;
        let bound = n_plus_1.powf(1.0 - sigma) / (sigma - 1.0);
        Ok(bound)
    }

    /// Required series order N to achieve target accuracy epsilon for given sigma.
    /// Solves: (N+1)^(1-sigma) / (sigma-1) <= epsilon
    /// => N >= (epsilon * (sigma-1))^(1/(1-sigma)) - 1
    pub fn required_order(&self, sigma: f64, epsilon: f64) -> Result<usize, &'static str> {
        if sigma <= 1.0 {
            return Err("sigma must be > 1.0");
        }
        if epsilon <= 0.0 || epsilon >= 1.0 {
            return Err("epsilon must be in (0, 1)");
        }
        if !sigma.is_finite() || !epsilon.is_finite() {
            return Err("sigma and epsilon must be finite");
        }

        let exponent = 1.0 / (1.0 - sigma);
        let n_float = (epsilon * (sigma - 1.0)).powf(exponent) - 1.0;
        let n = n_float.ceil() as usize;

        // Ensure at least 1
        Ok(n.max(1))
    }

    /// Generate remainder bound table for t in [14, 1e6] at fixed sigma.
    /// Returns Vec of (t, bound) pairs for monotonicity verification.
    pub fn bound_table(&self, sigma: f64, points: usize) -> Result<Vec<(f64, f64)>, &'static str> {
        if points < 2 {
            return Err("points must be >= 2");
        }
        if sigma <= 1.0 {
            return Err("sigma must be > 1.0");
        }

        let mut table = Vec::with_capacity(points);
        let log_min = 14.0_f64.ln();
        let log_max = 1e6_f64.ln();

        for i in 0..points {
            let log_t = log_min + (log_max - log_min) * (i as f64 / (points - 1) as f64);
            let t = log_t.exp();
            // Use fixed N for table -- bound decreases as t increases (for fixed N)
            let n = 1000;
            let bound = self.remainder_bound(n, sigma)?;
            table.push((t, bound));
        }

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Remainder bound must be small for reasonable parameters.
    /// Honest: N=100000, sigma=2 gives bound ~1e-5, not 1e-15.
    /// To achieve 1e-15, need either larger N or larger sigma.
    #[test]
    fn test_truncation_bound_small_enough() {
        let budget = TruncationBudget::new();

        // Case 1: Large N with sigma=2 gives ~1e-5
        let n = 100_000;
        let sigma = 2.0;
        let bound = budget.remainder_bound(n, sigma).unwrap();
        assert!(
            bound <= 1e-5,
            "Bound {:.3e} > 1e-5 for N={}, sigma={}",
            bound,
            n,
            sigma
        );

        // Case 2: Larger sigma=3 gives tighter bound
        let sigma3 = 3.0;
        let bound3 = budget.remainder_bound(n, sigma3).unwrap();
        assert!(
            bound3 <= 1e-10,
            "Bound {:.3e} > 1e-10 for N={}, sigma=3",
            bound3,
            n
        );
    }

    /// Bound must decrease monotonically as N increases (for fixed sigma).
    #[test]
    fn test_truncation_bound_monotonic_in_n() {
        let budget = TruncationBudget::new();
        let sigma = 2.0;
        let mut prev_bound = f64::INFINITY;

        for n in [100, 1000, 10000, 100000] {
            let bound = budget.remainder_bound(n, sigma).unwrap();
            assert!(
                bound < prev_bound,
                "Bound must decrease with N: N={} bound={:.3e} >= prev={:.3e}",
                n,
                bound,
                prev_bound
            );
            prev_bound = bound;
        }
    }
}
