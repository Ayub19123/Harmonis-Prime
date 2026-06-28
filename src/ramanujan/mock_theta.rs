//! SET-5.5: Ramanujan mock theta functions for heuristic parsing
//! Approximations of mock theta functions f(q) = Σ q^{n(n+1)} / (q;q)_n

/// Compute Ramanujan mock theta f(q) with given q (0<q<1)
/// Approximation using first N terms
pub fn mock_theta_f(q: f64, terms: usize) -> f64 {
    let mut sum = 0.0;
    let mut q_pochhammer = 1.0;
    for n in 0..terms {
        // q^{n(n+1)}
        let exponent = n * (n + 1);
        let q_term = q.powi(exponent as i32);
        sum += q_term / q_pochhammer;
        // update (q;q)_n = ∏_{k=1}^{n} (1 - q^k)
        q_pochhammer *= 1.0 - q.powi((n + 1) as i32);
    }
    sum
}

/// Use mock theta as a probability weight for quantum state selection
pub fn mock_theta_weight(index: usize, total: usize, q: f64) -> f64 {
    let t = index as f64 / total as f64;
    let weight = mock_theta_f(q, 10) * (1.0 - t).powi(2);
    weight.max(1e-6)
}
