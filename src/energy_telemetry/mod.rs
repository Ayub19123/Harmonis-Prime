//! SET-6E: CoreSight Energy Telemetry - Cycle-Accurate Power Measurement
//! 
//! Holy Grail Principle:
//!   Less data. Less energy. More precision. The unresolved becomes mere task
//!   when decoded piece by piece with zero emotion, absolute calm, and
//!   mathematical resilience.
//! 
//! Mathematical Foundation:
//!   P_dyn = C * V^2 * f
//!   E_t = alpha * E_measured + (1-alpha) * E_model
//! 
//! Operating Principle:
//!   - Every failure is data, not defeat
//!   - Every boundary condition is a brick
//!   - Every unresolved puzzle becomes executable test
//!   - Fearless. Calm. Clear. Resilient. Zero emotion.

pub mod telemetry;

pub use telemetry::{
    PmuSimulator, DvfsProfile, EmaFilter, 
    EnergyTelemetry, TelemetryDrift,
    compute_dynamic_power, apply_ema_filter,
};


#[cfg(test)]
mod tests;


