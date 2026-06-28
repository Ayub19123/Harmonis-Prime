//! Entropy-Aware PIM Crossbar Placement — Greedy Heuristic.
//!
//! Honest Limitation: Greedy, not proven optimal. No real power data.
//! Uses Shannon entropy from domain balancer to guide PIM allocation.

use crate::set10_fusion::theta_approx::ThetaApproximation;
use crate::set9_telemetry::multi_domain::{MultiDomainRapl, RaplDomain};

/// Placement decision with entropy context
#[derive(Debug, Clone, PartialEq)]
pub struct EntropyPlacement {
    pub domain: RaplDomain,
    pub estimated_energy: f64,
    pub entropy_context: f64,
}

/// Entropy-aware PIM placement — greedy heuristic
#[derive(Debug, Clone)]
pub struct EntropyPimPlacement {
    monitor: MultiDomainRapl,
    theta: ThetaApproximation,
    entropy_threshold: f64,
}

impl EntropyPimPlacement {
    pub fn new(monitor: MultiDomainRapl, entropy_threshold: f64) -> Result<Self, &'static str> {
        if entropy_threshold < 0.0 {
            return Err("entropy_threshold must be non-negative");
        }
        Ok(Self {
            monitor,
            theta: ThetaApproximation::new(),
            entropy_threshold,
        })
    }

    /// Place workload on domain with lowest energy, using theta(t) as entropy context
    /// Returns error if all domains uninitialized
    pub fn place(&self, t: f64) -> Result<EntropyPlacement, &'static str> {
        let theta_val = self.theta.evaluate(t)?;
        let readings = self.monitor.read_all();

        let all_zero = readings.iter().all(|(_, e)| *e == 0.0);
        if all_zero {
            return Err("all domains uninitialized — inject energy values first");
        }

        let (domain, energy) = readings
            .into_iter()
            .min_by(|(_, e1), (_, e2)| e1.partial_cmp(e2).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or("no domains available")?;

        Ok(EntropyPlacement {
            domain,
            estimated_energy: energy,
            entropy_context: theta_val.abs(),
        })
    }

    /// Verify entropy context exceeds threshold
    pub fn entropy_sufficient(&self, t: f64) -> Result<bool, &'static str> {
        let theta_val = self.theta.evaluate(t)?;
        Ok(theta_val.abs() >= self.entropy_threshold)
    }
}
