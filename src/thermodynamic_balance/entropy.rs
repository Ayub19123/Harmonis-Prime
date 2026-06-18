/// Shannon entropy and KL divergence engine.
/// 
/// All computations use f64. No unsafe code. No external dependencies.
#[derive(Debug, Clone)]
pub struct EntropyEngine;

impl EntropyEngine {
    pub fn new() -> Self {
        Self
    }

    /// Shannon entropy: H = −Σ p_i · ln(p_i)
    /// 
    /// Convention: 0 · ln(0) = 0 (limit continuity)
    pub fn shannon_entropy(&self, probabilities: &[f64]) -> f64 {
        probabilities.iter().map(|&p| {
            if p > 0.0 {
                -p * p.ln()
            } else {
                0.0
            }
        }).sum()
    }

    /// Kullback-Leibler divergence: D_KL(P || Q) = Σ p_i · ln(p_i / q_i)
    /// 
    /// Returns error if:
    /// - Lengths mismatch
    /// - Any probability negative
    /// - Q has zero where P is non-zero (mathematically undefined → +∞)
    pub fn kl_divergence(&self, p: &[f64], q: &[f64]) -> Result<f64, &'static str> {
        if p.len() != q.len() {
            return Err("distributions must have identical length");
        }
        if p.is_empty() {
            return Ok(0.0);
        }

        let mut sum = 0.0;
        for (&p_i, &q_i) in p.iter().zip(q.iter()) {
            if p_i < 0.0 || q_i < 0.0 {
                return Err("probabilities must be non-negative");
            }
            if p_i > 0.0 && q_i == 0.0 {
                return Err("KL divergence undefined: Q has zero probability where P is non-zero");
            }
            if p_i > 0.0 && q_i > 0.0 {
                sum += p_i * (p_i / q_i).ln();
            }
        }
        Ok(sum)
    }
}