//! Per-domain 1D lumped RC thermal model.
//!
//! Honest Limitation: 1D lumped RC only. No FEM, no 2D diffusion,
//! no anisotropic conductivity, no boundary effects.

/// Thermal parameters for a single domain
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalParams {
    pub ambient_temp: f64,      // K
    pub thermal_resistance: f64, // K/W
    pub thermal_capacitance: f64, // J/K
}

impl ThermalParams {
    pub fn new(ambient: f64, resistance: f64, capacitance: f64) -> Result<Self, &'static str> {
        if ambient <= 0.0 {
            return Err("ambient temperature must be positive");
        }
        if resistance <= 0.0 {
            return Err("thermal resistance must be positive");
        }
        if capacitance <= 0.0 {
            return Err("thermal capacitance must be positive");
        }
        Ok(Self {
            ambient_temp: ambient,
            thermal_resistance: resistance,
            thermal_capacitance: capacitance,
        })
    }
}

/// Per-domain thermal model with independent RC parameters
#[derive(Debug, Clone)]
pub struct DomainThermalModel {
    params: ThermalParams,
    current_temp: f64,
}

impl DomainThermalModel {
    pub fn new(params: ThermalParams) -> Self {
        let t_amb = params.ambient_temp;
        Self {
            params,
            current_temp: t_amb,
        }
    }

    /// Advance thermal state by dt seconds with constant power
    /// T_new = T_amb + P·R + (T_old - T_amb - P·R)·exp(-dt/(R·C))
    pub fn step(&mut self, power: f64, dt: f64) -> Result<(), &'static str> {
        if dt < 0.0 {
            return Err("dt must be non-negative");
        }
        if !power.is_finite() || !dt.is_finite() {
            return Err("power and dt must be finite");
        }

        let t_amb = self.params.ambient_temp;
        let r = self.params.thermal_resistance;
        let c = self.params.thermal_capacitance;
        let steady_rise = power * r;
        let tau = r * c;
        let exp_factor = (-dt / tau).exp();

        self.current_temp = t_amb + steady_rise
            + (self.current_temp - t_amb - steady_rise) * exp_factor;

        Ok(())
    }

    pub fn temperature(&self) -> f64 {
        self.current_temp
    }

    pub fn ambient(&self) -> f64 {
        self.params.ambient_temp
    }

    /// Steady-state temperature for given power: T_ambient + P·R
    pub fn steady_state(&self, power: f64) -> f64 {
        self.params.ambient_temp + power * self.params.thermal_resistance
    }

    /// Thermal parameters (immutable)
    pub fn params(&self) -> ThermalParams {
        self.params
    }
}