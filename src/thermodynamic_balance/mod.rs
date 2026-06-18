//! SET-7C: Thermodynamic Workload Balancing
//! 
//! Mathematical Foundation:
//!   - Shannon entropy: S = −k_B · Σ P_i · ln(P_i)  [nats]
//!   - KL divergence: D_KL(P || Q) = Σ P_i · ln(P_i / Q_i)
//!   - RC thermal model: dT/dt = (P − (T−T_ambient)/R) / C
//! 
//! CRITICAL LIMITATION — THERMAL MODEL:
//!   This is a lumped 1D RC model. Real PIM crossbars exhibit:
//!   - 2D heat diffusion with cell-to-cell coupling
//!   - Material-dependent anisotropic conductivity
//!   - Boundary effects at array edges
//!   - We capture first-order transient only. Phase 2: FEM thermal simulation.
//! 
//! CRITICAL LIMITATION — ENTROPY:
//!   We use natural log (nats), not bits. Conversion: 1 bit = ln(2) nats.
//!   We do not enforce Σ p_i = 1.0 — caller must normalize.
//! 
//! Operating Principle:
//!   - Every failure is data, not defeat
//!   - Every boundary condition is a brick
//!   - The precision is eternal

pub mod entropy;
pub mod thermal;
pub mod workload;

#[cfg(test)]
mod tests;

pub use entropy::EntropyEngine;
pub use thermal::ThermalModel;
pub use workload::WorkloadDriftDetector;