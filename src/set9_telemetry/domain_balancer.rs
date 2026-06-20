//! Energy-minimal domain workload placement — greedy heuristic.
//!
//! Honest Limitation: Greedy, not proven optimal. No real power data.
//! Selects domain with lowest simulated energy for workload placement.

use crate::set9_telemetry::multi_domain::{MultiDomainRapl, RaplDomain};

/// Placement decision for a workload unit
#[derive(Debug, Clone, PartialEq)]
pub struct WorkloadPlacement {
    pub domain: RaplDomain,
    pub estimated_energy: f64,
}

/// Greedy domain balancer — places workload on lowest-energy domain
#[derive(Debug, Clone)]
pub struct DomainBalancer {
    monitor: MultiDomainRapl,
}

impl DomainBalancer {
    pub fn new(monitor: MultiDomainRapl) -> Self {
        Self { monitor }
    }

    /// Place workload on domain with lowest current energy reading
    /// Returns error if all domains report zero (uninitialized)
    pub fn place(&self) -> Result<WorkloadPlacement, &'static str> {
        let readings = self.monitor.read_all();

        // Check uninitialized BEFORE consuming readings
        let all_zero = readings.iter().all(|(_, e)| *e == 0.0);
        if all_zero {
            return Err("all domains uninitialized — inject energy values first");
        }

        let (domain, energy) = readings
            .into_iter()
            .min_by(|(_, e1), (_, e2)| e1.partial_cmp(e2).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or("no domains available")?;

        Ok(WorkloadPlacement {
            domain,
            estimated_energy: energy,
        })
    }

    /// Place workload on specific domain (bypass heuristic)
    pub fn place_on(&self, domain: RaplDomain) -> Result<WorkloadPlacement, &'static str> {
        let energy = self.monitor.read_domain(domain);
        Ok(WorkloadPlacement {
            domain,
            estimated_energy: energy,
        })
    }

    /// Access underlying monitor
    pub fn monitor(&self) -> &MultiDomainRapl {
        &self.monitor
    }
}
