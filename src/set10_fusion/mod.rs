//! SET-10: Fusion Layer — Mathematical Bridge to Industrial Standard
//!
//! Honest Limitations:
//! - theta_approx: Stirling's approximation only, f64 precision, no Odlyzko dataset
//! - extended_series: More Dirichlet terms, sigma > 1 only, diverges at sigma = 1/2
//! - thermal_bridge: Heuristic coupling, no physical hardware ground truth
//! - entropy_placement: Greedy heuristic, not proven optimal
//!
//! The precision is eternal.

pub mod entropy_placement;
pub mod extended_series;
pub mod thermal_bridge;
pub mod theta_approx;

#[cfg(test)]
mod tests;

pub use entropy_placement::EntropyPimPlacement;
pub use extended_series::ExtendedDirichletSeries;
pub use thermal_bridge::PimThermalBridge;
pub use theta_approx::ThetaApproximation;
