//! Cross-domain JLO (Joules-per-Logical-Operation) correlation.
//!
//! Honest Limitation: f64 arithmetic, no hardware ground truth.
//! Pearson correlation coefficient — not causal inference.

use crate::set9_telemetry::multi_domain::RaplDomain;

/// Identifies a pair of domains for correlation analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomainPair {
    pub a: RaplDomain,
    pub b: RaplDomain,
}

impl DomainPair {
    pub fn new(a: RaplDomain, b: RaplDomain) -> Result<Self, &'static str> {
        if std::mem::discriminant(&a) == std::mem::discriminant(&b) {
            return Err("domain pair must contain distinct domains");
        }
        Ok(Self { a, b })
    }
}

/// JLO correlation between two domains' energy consumption
#[derive(Debug, Clone, PartialEq)]
pub struct JloCorrelation {
    pub pair: DomainPair,
    pub pearson_r: f64,
    pub samples: usize,
}

impl JloCorrelation {
    /// Compute Pearson correlation coefficient between two measurement streams
    pub fn compute(pair: DomainPair, a: &[f64], b: &[f64]) -> Result<Self, &'static str> {
        if a.len() != b.len() || a.len() < 2 {
            return Err("streams must have equal length >= 2");
        }

        let n = a.len() as f64;
        let mean_a: f64 = a.iter().sum::<f64>() / n;
        let mean_b: f64 = b.iter().sum::<f64>() / n;

        let covariance: f64 = a.iter().zip(b.iter())
            .map(|(x, y)| (x - mean_a) * (y - mean_b))
            .sum();

        let var_a: f64 = a.iter().map(|x| (x - mean_a).powi(2)).sum();
        let var_b: f64 = b.iter().map(|y| (y - mean_b).powi(2)).sum();

        if var_a == 0.0 || var_b == 0.0 {
            return Err("zero variance in one or both streams");
        }

        let r = covariance / (var_a.sqrt() * var_b.sqrt());

        // Clamp to [-1, 1] to handle floating-point drift
        let r_clamped = r.clamp(-1.0, 1.0);

        Ok(Self {
            pair,
            pearson_r: r_clamped,
            samples: a.len(),
        })
    }

    /// Verify invariant: |r| >= 0.95 indicates strong correlation
    pub fn is_strong(&self) -> bool {
        self.pearson_r.abs() >= 0.95
    }

    /// Verify invariant: opposite sign energy is physically suspicious
    pub fn is_positive(&self) -> bool {
        self.pearson_r >= 0.0
    }
}