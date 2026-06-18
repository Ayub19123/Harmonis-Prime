/// Simplified RC thermal model for PIM crossbar energy dissipation.
/// 
/// Math: T_new = T_ambient + P·R + (T_old − T_ambient − P·R)·exp(−dt/(R·C))
#[derive(Debug, Clone)]
pub struct ThermalModel {
    ambient_temp: f64,
    thermal_resistance: f64,
    thermal_capacitance: f64,
    current_temp: f64,
}

impl ThermalModel {
    pub fn new(ambient_temp: f64, thermal_resistance: f64, thermal_capacitance: f64) -> Result<Self, &'static str> {
        if ambient_temp <= 0.0 {
            return Err("ambient temperature must be positive");
        }
        if thermal_resistance <= 0.0 {
            return Err("thermal resistance must be positive");
        }
        if thermal_capacitance <= 0.0 {
            return Err("thermal capacitance must be positive");
        }
        Ok(Self {
            ambient_temp,
            thermal_resistance,
            thermal_capacitance,
            current_temp: ambient_temp,
        })
    }

    /// Advance thermal state by dt seconds with constant power dissipation.
    pub fn step(&mut self, power: f64, dt: f64) -> Result<(), &'static str> {
        if dt < 0.0 {
            return Err("dt must be non-negative");
        }
        if !power.is_finite() || !dt.is_finite() {
            return Err("power and dt must be finite");
        }

        let steady_state_rise = power * self.thermal_resistance;
        let tau = self.thermal_resistance * self.thermal_capacitance;
        let exp_factor = (-dt / tau).exp();

        self.current_temp = self.ambient_temp + steady_state_rise
            + (self.current_temp - self.ambient_temp - steady_state_rise) * exp_factor;

        Ok(())
    }

    pub fn temperature(&self) -> f64 {
        self.current_temp
    }

    pub fn ambient(&self) -> f64 {
        self.ambient_temp
    }

    /// Steady-state temperature for given power: T_ambient + P·R
    pub fn steady_state(&self, power: f64) -> f64 {
        self.ambient_temp + power * self.thermal_resistance
    }
}