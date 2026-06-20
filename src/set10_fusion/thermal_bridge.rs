//! PIM + Thermodynamic Bridge — Heuristic coupling.
//!
//! Honest Limitation: Heuristic only. No physical hardware ground truth.
//! Temperature from RC model feeds into PIM energy estimate.

use crate::set9_telemetry::thermal_rc::{DomainThermalModel, ThermalParams};

/// Bridge between thermal model and PIM energy estimation
#[derive(Debug, Clone)]
pub struct PimThermalBridge {
    thermal: DomainThermalModel,
    base_energy_per_op: f64,
    thermal_coefficient: f64,
}

impl PimThermalBridge {
    pub fn new(
        thermal_params: ThermalParams,
        base_energy_per_op: f64,
        thermal_coefficient: f64,
    ) -> Result<Self, &'static str> {
        if base_energy_per_op <= 0.0 {
            return Err("base_energy_per_op must be positive");
        }
        if thermal_coefficient < 0.0 {
            return Err("thermal_coefficient must be non-negative");
        }
        Ok(Self {
            thermal: DomainThermalModel::new(thermal_params),
            base_energy_per_op,
            thermal_coefficient,
        })
    }

    /// Advance thermal state and return energy estimate for next operation
    /// Energy = base + thermal_coefficient * (current_temp - ambient)
    pub fn step_and_estimate(&mut self, power: f64, dt: f64) -> Result<f64, &'static str> {
        self.thermal.step(power, dt)?;
        let temp_rise = self.thermal.temperature() - self.thermal.ambient();
        let energy = self.base_energy_per_op + self.thermal_coefficient * temp_rise;
        Ok(energy)
    }

    /// Current temperature from thermal model
    pub fn temperature(&self) -> f64 {
        self.thermal.temperature()
    }

    /// Ambient temperature
    pub fn ambient(&self) -> f64 {
        self.thermal.ambient()
    }
}
