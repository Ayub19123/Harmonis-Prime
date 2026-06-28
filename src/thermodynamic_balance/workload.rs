use crate::thermodynamic_balance::entropy::EntropyEngine;

/// Workload drift detector using KL divergence.
///
/// Compares expected workload distribution against observed.
/// Flags drift when D_KL(observed || expected) > threshold.
#[derive(Debug, Clone)]
pub struct WorkloadDriftDetector {
    entropy: EntropyEngine,
    threshold: f64,
}

impl WorkloadDriftDetector {
    pub fn new(threshold: f64) -> Result<Self, &'static str> {
        if threshold < 0.0 {
            return Err("threshold must be non-negative");
        }
        Ok(Self {
            entropy: EntropyEngine::new(),
            threshold,
        })
    }

    /// Returns true if drift detected (KL > threshold).
    pub fn detect_drift(&self, expected: &[f64], observed: &[f64]) -> Result<bool, &'static str> {
        let kl = self.entropy.kl_divergence(observed, expected)?;
        Ok(kl > self.threshold)
    }

    /// Returns raw KL divergence magnitude.
    pub fn drift_magnitude(&self, expected: &[f64], observed: &[f64]) -> Result<f64, &'static str> {
        self.entropy.kl_divergence(observed, expected)
    }
}
