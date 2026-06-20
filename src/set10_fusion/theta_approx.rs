//! Riemann-Siegel theta(t) approximation via Stirling's formula.
//!
//! Honest Limitation: f64 precision only. No Odlyzko dataset comparison.
//! Formula: θ(t) ≈ (t/2)·ln(t/2π) − t/2 − π/8 + 1/(48t) + ...

/// Stirling's approximation for Riemann-Siegel theta function
#[derive(Debug, Clone)]
pub struct ThetaApproximation;

impl ThetaApproximation {
    pub fn new() -> Self {
        Self
    }

    /// θ(t) ≈ (t/2)·ln(t/2π) − t/2 − π/8 + 1/(48t)
    /// Valid for t > 1.0
    pub fn evaluate(&self, t: f64) -> Result<f64, &'static str> {
        if t <= 0.0 {
            return Err("t must be positive");
        }
        if !t.is_finite() {
            return Err("t must be finite");
        }

        let t_over_2 = t / 2.0;
        let ln_term = (t / (2.0 * std::f64::consts::PI)).ln();
        let main = t_over_2 * ln_term - t_over_2;
        let correction = -std::f64::consts::PI / 8.0 + 1.0 / (48.0 * t);

        Ok(main + correction)
    }

    /// First derivative: dθ/dt ≈ (1/2)·ln(t/2π)
    /// Must be positive and monotonically increasing for t > 1/(2π)
    pub fn derivative(&self, t: f64) -> Result<f64, &'static str> {
        if t <= 0.0 {
            return Err("t must be positive");
        }
        let result = 0.5 * (t / (2.0 * std::f64::consts::PI)).ln();
        Ok(result)
    }

    /// Verify monotonicity: θ(t₂) > θ(t₁) for t₂ > t₁ > 1/(2π)
    pub fn is_monotonic(&self, t1: f64, t2: f64) -> Result<bool, &'static str> {
        if t1 >= t2 {
            return Err("t2 must be greater than t1");
        }
        let theta1 = self.evaluate(t1)?;
        let theta2 = self.evaluate(t2)?;
        Ok(theta2 > theta1)
    }
}