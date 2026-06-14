//! SET-5.5: Ramanujan Mathematical Invariant Tests
//! Invariant: Mock theta convergence, HCN correctness, partition advantage

#[cfg(test)]
mod tests {
    use crate::ramanujan::mock_theta::{mock_theta_f, mock_theta_weight};
    use crate::ramanujan::hcn::highly_composite_numbers;

    #[test]
    fn test_mock_theta_convergence() {
        let q = 0.5;
        let result_10 = mock_theta_f(q, 10);
        let result_20 = mock_theta_f(q, 20);
        // Convergence check: 10 vs 20 terms should be within 1%
        let diff = (result_20 - result_10).abs();
        assert!(diff < 0.01, "Mock theta failed convergence: diff={}", diff);
        assert!(result_10 > 0.0, "Mock theta must be positive");
    }

    #[test]
    fn test_mock_theta_weight_distribution() {
        let total = 100;
        let q = 0.7;
        let mut weights = Vec::new();
        for i in 0..total {
            weights.push(mock_theta_weight(i, total, q));
        }
        // Weights should decrease monotonically
        for i in 1..weights.len() {
            assert!(weights[i] <= weights[i-1], "Weight must decrease monotonically");
        }
        // All weights positive
        assert!(weights.iter().all(|&w| w > 0.0), "All weights must be positive");
    }

    #[test]
    fn test_highly_composite_numbers() {
        let hcn = highly_composite_numbers(10);
        // Known first 10 HCN: 1, 2, 4, 6, 12, 24, 36, 48, 60, 120
        let expected = vec![1, 2, 4, 6, 12, 24, 36, 48, 60, 120];
        assert_eq!(hcn, expected, "HCN sequence must match Ramanujan canon");
    }

    #[test]
    fn test_hcn_divisor_advantage() {
        let hcn = highly_composite_numbers(5);
        // Each HCN must have strictly more divisors than previous
        let mut prev_divisors = 0;
        for &num in &hcn {
            let div_count = divisor_count(num);
            assert!(div_count > prev_divisors, "HCN {} must have > {} divisors", num, prev_divisors);
            prev_divisors = div_count;
        }
    }

    fn divisor_count(n: u64) -> u64 {
        let mut count = 0;
        let limit = (n as f64).sqrt() as u64;
        for i in 1..=limit {
            if n % i == 0 {
                count += 1;
                if i != n / i { count += 1; }
            }
        }
        count
    }
}
