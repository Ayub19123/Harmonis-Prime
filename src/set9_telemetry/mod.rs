//! SET-9: Multi-Domain Telemetry & Energy Balancing
//!
//! Honest Limitations:
//! - All RAPL reads are simulated — no physical hardware access
//! - Per-domain thermal models are 1D lumped RC — no FEM, no 2D diffusion
//! - JLO correlation uses f64 arithmetic — no hardware ground truth
//! - Domain balancer is greedy heuristic — not proven optimal
//!
//! The precision is eternal.

pub mod domain_balancer;
pub mod jlo_correlation;
pub mod multi_domain;
pub mod thermal_rc;

#[cfg(test)]
mod tests;

pub use domain_balancer::{DomainBalancer, WorkloadPlacement};
pub use jlo_correlation::{DomainPair, JloCorrelation};
pub use multi_domain::{MultiDomainRapl, RaplDomain};
pub use thermal_rc::{DomainThermalModel, ThermalParams};
