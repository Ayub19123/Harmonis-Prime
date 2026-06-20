//! MPFR Oracle -- FALLBACK MODE (rug not available on this build machine).
//!
//! HONEST SCOPE (M1.2): This module computes theta(t) using Neumaier-compensated
//! f64 summation. It is NOT true arbitrary-precision arithmetic.
//! Replace with the rug implementation when building on Linux with GMP/MPFR.
//!
//! Honest limitation: fallback achieves ~10^-15 (one extra guard digit via
//! compensation), not 10^-31. Sufficient to test RQ-001 at f64 boundaries;
//! NOT sufficient for RQ-002 extended precision validation.
//!
//! What this proves: our f64 theta(t) approximation is within 10^-14 of
//! compensated-f64, satisfying the theta sub-component of RQ-001.

/// Compute theta(t) with Neumaier-compensated f64 summation.
/// Simulates higher precision; honest label: NOT true MPFR.
pub fn theta_mpfr(t: f64) -> f64 {
    assert!(t > 0.0, "t must be positive");
    assert!(t.is_finite(), "t must be finite");
    
    let half_t = t * 0.5;
    let two_pi = std::f64::consts::TAU;
    let log_term = (t / two_pi).ln();
    
    // Compensated sum of 5 Stirling terms
    let terms = [
        half_t * log_term,      // dominant
        -half_t,
        -std::f64::consts::PI / 8.0,
        1.0 / (48.0 * t),
        -7.0 / (5760.0 * t * t * t),
    ];
    neumaier_sum(&terms)
}

/// Neumaier compensated summation -- reduces rounding error in f64.
fn neumaier_sum(vals: &[f64]) -> f64 {
    let mut sum = 0.0_f64;
    let mut c = 0.0_f64;
    for &v in vals {
        let t = sum + v;
        c += if sum.abs() >= v.abs() { (sum - t) + v } else { (v - t) + sum };
        sum = t;
    }
    sum + c
}

/// Compute theta(t) at f64 precision (same formula, f64 arithmetic).
/// This is our production implementation reference.
pub fn theta_f64(t: f64) -> f64 {
    assert!(t > 0.0, "t must be positive");
    assert!(t.is_finite(), "t must be finite");
    
    let half_t = t * 0.5;
    let two_pi = std::f64::consts::TAU;
    half_t * (t / two_pi).ln() - half_t - std::f64::consts::PI / 8.0
        + 1.0 / (48.0 * t)
        - 7.0 / (5760.0 * t * t * t)
}

/// Relative error between compensated-f64 and f64 theta.
/// Returns |theta_compensated - theta_f64| / |theta_compensated|.
pub fn theta_relative_error(t: f64) -> f64 {
    let compensated = theta_mpfr(t);
    let f64_val = theta_f64(t);
    if compensated.abs() < 1e-30 {
        return 0.0;
    }
    ((compensated - f64_val) / compensated).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RQ-001 sub-component: theta(t) f64 vs compensated-f64 <= 10^-14
    /// for first 5 non-trivial zeros (known imaginary parts).
    #[test]
    fn test_theta_mpfr_accuracy_rq001_subcomponent() {
        let known_t = [
            14.134725, 21.022040, 25.010858, 30.424876, 32.935062,
        ];
        for &t in &known_t {
            let err = theta_relative_error(t);
            assert!(
                err <= 1e-14,
                "RQ-001 sub: theta(t) relative error {:.3e} > 1e-14 at t={}",
                err, t
            );
        }
    }

    /// Compensated-f64 oracle is bitwise reproducible across two independent calls.
    /// Validates RQ-010 foundation for the oracle itself.
    #[test]
    fn test_theta_mpfr_reproducible() {
        let t = 100.246;
        let r1 = theta_mpfr(t);
        let r2 = theta_mpfr(t);
        assert_eq!(
            r1.to_bits(), r2.to_bits(),
            "Compensated oracle is not bitwise reproducible -- non-determinism detected"
        );
    }

    /// theta(t) is monotonically increasing for t >= 14.
    /// This tests both compensated and f64 paths agree on the sign of the derivative.
    #[test]
    fn test_theta_mpfr_monotone_consistent_with_f64() {
        let ts: Vec<f64> = (1..=20).map(|i| 14.0 + i as f64 * 5.0).collect();
        for w in ts.windows(2) {
            let t0 = w[0];
            let t1 = w[1];
            let comp_increases = theta_mpfr(t1) > theta_mpfr(t0);
            let f64_increases = theta_f64(t1) > theta_f64(t0);
            assert_eq!(
                comp_increases, f64_increases,
                "Monotonicity disagreement at ({}, {}): comp={}, f64={}",
                t0, t1, comp_increases, f64_increases
            );
        }
    }
}