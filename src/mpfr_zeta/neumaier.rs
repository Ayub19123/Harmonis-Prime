//! SET-12: Kahan-Neumaier Compensated Summation
//! LIMITATION: f64 only. MPFR path uses rug::Float native precision.
//! LIMITATION: Single-threaded. SIMD batching in M2.3.

/// Neumaier's compensated summation algorithm.
/// More robust than Kahan when terms have alternating signs or
/// wildly differing magnitudes.
///
/// Reference: Neumaier, A. "Rundungsfehleranalyse einiger Verfahren
/// zur Summation endlicher Summen." ZAMM, 1974.
pub fn neumaier_sum_f64(terms: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0; // compensation for lost low-order bits
    for &x in terms {
        let t = sum + x;
        if sum.abs() >= x.abs() {
            c += (sum - t) + x;
        } else {
            c += (x - t) + sum;
        }
        sum = t;
    }
    sum + c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neumaier_empty() {
        let result = neumaier_sum_f64(&[]);
        assert_eq!(result, 0.0, "Empty sum must be 0");
    }

    #[test]
    fn test_neumaier_determinism() {
        let terms = vec![0.1; 10];
        let a = neumaier_sum_f64(&terms);
        let b = neumaier_sum_f64(&terms);
        assert_eq!(a, b, "Neumaier sum must be deterministic");
    }

    #[test]
    fn test_neumaier_exact_recovery() {
        // Classic catastrophic cancellation test:
        // 1 + 1e100 + 1 - 1e100 = 2
        // Naive f64 summation gives 0.0 (both 1.0s are absorbed).
        // Neumaier recovers the exact result 2.0.
        let terms = vec![1.0, 1e100, 1.0, -1e100];
        let result = neumaier_sum_f64(&terms);
        assert_eq!(
            result, 2.0,
            "Neumaier must recover exact result 2.0, got {}",
            result
        );
    }

    #[test]
    fn test_neumaier_precision_invariant() {
        // 10_000 × 0.0001 = 1.0 exactly.
        // Naive summation drifts; Neumaier stays within f64 epsilon.
        let terms = vec![0.0001f64; 10_000];
        let result = neumaier_sum_f64(&terms);
        let epsilon = 1e-12;
        assert!(
            (result - 1.0).abs() < epsilon,
            "Neumaier precision: |{} - 1.0| = {} > {}",
            result,
            (result - 1.0).abs(),
            epsilon
        );
    }
}
