//! SET-6E: Software PMU Simulation, EMA Filter, DVFS Model
//! 
//! Phase 1 (x86_64): Deterministic timers, synthetic workload profiles
//! Phase 2 (ARM): Real CoreSight PMU integration


/// DVFS profile: voltage and frequency operating point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DvfsProfile {
    pub voltage_v: f64,      // Volts
    pub frequency_hz: f64,   // Hz
    pub capacitance_f: f64,  // Farads (simplified)
}

impl DvfsProfile {
    pub fn new(voltage_v: f64, frequency_hz: f64, capacitance_f: f64) -> Result<Self, &'static str> {
        if voltage_v <= 0.0 || frequency_hz <= 0.0 || capacitance_f <= 0.0 {
            return Err("DVFS parameters must be positive");
        }
        if !voltage_v.is_finite() || !frequency_hz.is_finite() || !capacitance_f.is_finite() {
            return Err("DVFS parameters must be finite");
        }
        Ok(Self { voltage_v, frequency_hz, capacitance_f })
    }
}

/// Compute dynamic power: P_dyn = C * V^2 * f
pub fn compute_dynamic_power(profile: &DvfsProfile) -> f64 {
    profile.capacitance_f * profile.voltage_v.powi(2) * profile.frequency_hz
}

/// Exponential Moving Average filter for drift calibration
/// E_t = alpha * E_measured + (1-alpha) * E_model
#[derive(Debug, Clone)]
pub struct EmaFilter {
    pub alpha: f64,           // smoothing factor (0.0 - 1.0)
    pub ema_value: f64,       // current filtered value
}

impl EmaFilter {
    pub fn new(alpha: f64, initial_value: f64) -> Result<Self, &'static str> {
        if alpha < 0.0 || alpha > 1.0 {
            return Err("Alpha must be in [0.0, 1.0]");
        }
        if !alpha.is_finite() || !initial_value.is_finite() {
            return Err("Parameters must be finite");
        }
        Ok(Self { alpha, ema_value: initial_value })
    }

    /// Apply EMA filter to new measurement
    pub fn update(&mut self, measured: f64, model: f64) -> f64 {
        // E_t = alpha * measured + (1-alpha) * model
        self.ema_value = self.alpha * measured + (1.0 - self.alpha) * model;
        self.ema_value
    }

    /// Get current drift relative to model
    pub fn drift(&self, model: f64) -> f64 {
        if model == 0.0 {
            0.0
        } else {
            (self.ema_value - model).abs() / model
        }
    }
}

/// Simulated PMU counter (64-bit, with overflow handling)
#[derive(Debug, Clone)]
pub struct PmuSimulator {
    pub counter: u64,
    pub overflow_count: u64,
    pub last_read: u64,
}

impl PmuSimulator {
    pub fn new() -> Self {
        Self {
            counter: 0,
            overflow_count: 0,
            last_read: 0,
        }
    }

    /// Simulate counter increment (deterministic for testing)
    pub fn increment(&mut self, delta: u64) {
        let (new_counter, overflowed) = self.counter.overflowing_add(delta);
        if overflowed {
            self.overflow_count += 1;
        }
        self.counter = new_counter;
    }

    /// Read counter with overflow tracking
    pub fn read(&mut self) -> u64 {
        let delta = self.counter.wrapping_sub(self.last_read);
        self.last_read = self.counter;
        delta
    }

    /// Simulate 64-bit wraparound for testing
    pub fn set_near_overflow(&mut self) {
        self.counter = u64::MAX - 100;
        self.last_read = 0;
    }
}

/// Telemetry drift measurement
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryDrift {
    pub measured: f64,
    pub model: f64,
    pub drift_ratio: f64,
    pub within_tolerance: bool,
}

/// Energy telemetry session
#[derive(Debug, Clone)]
pub struct EnergyTelemetry {
    pub profile: DvfsProfile,
    pub ema: EmaFilter,
    pub pmu: PmuSimulator,
    pub power_model: f64,  // theoretical baseline
}

impl EnergyTelemetry {
    pub fn new(profile: DvfsProfile, alpha: f64, initial_power: f64) -> Result<Self, &'static str> {
        let ema = EmaFilter::new(alpha, initial_power)?;
        let pmu = PmuSimulator::new();
        let power_model = compute_dynamic_power(&profile);
        
        Ok(Self {
            profile,
            ema,
            pmu,
            power_model,
        })
    }

    /// Sample power and update EMA filter
    pub fn sample(&mut self, measured_power: f64) -> TelemetryDrift {
        let _filtered = self.ema.update(measured_power, self.power_model);
        let drift_ratio = self.ema.drift(self.power_model);
        
        TelemetryDrift {
            measured: measured_power,
            model: self.power_model,
            drift_ratio,
            within_tolerance: drift_ratio <= 0.01, // <= 1%
        }
    }

    /// Simulate DVFS frequency change
    pub fn change_frequency(&mut self, new_frequency_hz: f64) -> Result<(), &'static str> {
        if new_frequency_hz <= 0.0 || !new_frequency_hz.is_finite() {
            return Err("Frequency must be positive and finite");
        }
        self.profile.frequency_hz = new_frequency_hz;
        self.power_model = compute_dynamic_power(&self.profile);
        Ok(())
    }
}

/// Apply EMA filter (standalone function)
pub fn apply_ema_filter(alpha: f64, measured: f64, model: f64, _previous_ema: f64) -> f64 {
    alpha * measured + (1.0 - alpha) * model
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_power_basic() {
        let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
        let power = compute_dynamic_power(&profile);
        // P = 1e-9 * 1^2 * 1e9 = 1.0 W
        assert!((power - 1.0).abs() < 1e-6, "Expected ~1.0 W, got {}", power);
    }

    #[test]
    fn test_ema_filter_update() {
        let mut ema = EmaFilter::new(0.5, 1.0).unwrap();
        let result = ema.update(1.2, 1.0);
        // E = 0.5 * 1.2 + 0.5 * 1.0 = 1.1
        assert!((result - 1.1).abs() < 1e-9, "Expected 1.1, got {}", result);
    }

    #[test]
    fn test_pmu_counter_overflow() {
        let mut pmu = PmuSimulator::new();
        pmu.set_near_overflow();
        pmu.increment(200); // wraps around
        assert_eq!(pmu.overflow_count, 1);
        let delta = pmu.read();
        assert_eq!(delta, 99);
    }
}


