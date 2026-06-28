//! SET-6E: Software PMU Simulation, EMA Filter, DVFS Model
//!
//! Phase 1 (x86_64): Deterministic timers, synthetic workload profiles
//! Phase 2 (ARM): Real CoreSight PMU integration

/// DVFS profile: voltage and frequency operating point
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DvfsProfile {
    pub voltage_v: f64,     // Volts
    pub frequency_hz: f64,  // Hz
    pub capacitance_f: f64, // Farads (simplified)
}

impl DvfsProfile {
    pub fn new(
        voltage_v: f64,
        frequency_hz: f64,
        capacitance_f: f64,
    ) -> Result<Self, &'static str> {
        if voltage_v <= 0.0 || frequency_hz <= 0.0 || capacitance_f <= 0.0 {
            return Err("DVFS parameters must be positive");
        }
        if !voltage_v.is_finite() || !frequency_hz.is_finite() || !capacitance_f.is_finite() {
            return Err("DVFS parameters must be finite");
        }
        Ok(Self {
            voltage_v,
            frequency_hz,
            capacitance_f,
        })
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
    pub alpha: f64,     // smoothing factor (0.0 - 1.0)
    pub ema_value: f64, // current filtered value
}

impl EmaFilter {
    pub fn new(alpha: f64, initial_value: f64) -> Result<Self, &'static str> {
        if alpha < 0.0 || alpha > 1.0 {
            return Err("Alpha must be in [0.0, 1.0]");
        }
        if !alpha.is_finite() || !initial_value.is_finite() {
            return Err("Parameters must be finite");
        }
        Ok(Self {
            alpha,
            ema_value: initial_value,
        })
    }

    /// Apply EMA filter to new measurement
    pub fn update(&mut self, measured: f64, model: f64) -> f64 {
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
    pub power_model: f64, // theoretical baseline
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

// --- Independent Estimators for Ground-Truth Validation ---

/// Workload type for generating independent signals across two paths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Workload {
    Idle,
    Bursty,
    SustainedHigh,
    Ramping,
}

impl Workload {
    /// Generate time-domain voltage/current signal (physical meter path).
    /// Returns Vec<(time_s, voltage_v, current_a)>.
    pub fn power_signal(&self, num_samples: usize, dt: f64) -> Vec<(f64, f64, f64)> {
        let mut samples = Vec::with_capacity(num_samples);
        let (base_v, base_i) = match self {
            Workload::Idle => (3.3, 0.5),
            Workload::Bursty => (3.3, 2.0),
            Workload::SustainedHigh => (3.3, 4.0),
            Workload::Ramping => (3.3, 0.5),
        };

        for i in 0..num_samples {
            let t = i as f64 * dt;
            let (v, i_a) = match self {
                Workload::Idle => (base_v, base_i + 0.02 * (t * 0.1).sin()),
                Workload::Bursty => {
                    let burst = if (i % 100) < 10 { 1.0 } else { 0.0 };
                    (base_v, base_i + burst * 3.0)
                }
                Workload::SustainedHigh => (base_v, base_i + 0.5 * (t * 0.05).sin()),
                Workload::Ramping => {
                    let ramp = (t / 10.0).min(3.0);
                    (base_v, base_i + ramp)
                }
            };
            samples.push((t, v, i_a));
        }
        samples
    }

    /// Generate performance counters (PMU path).
    /// Returns Vec<(cpu_cycles, cache_misses, memory_accesses)>.
    pub fn pmu_counters(&self, num_samples: usize) -> Vec<(u64, u64, u64)> {
        let mut counters = Vec::with_capacity(num_samples);
        let (base_cycles, base_misses, base_mem) = match self {
            Workload::Idle => (100_000, 10_000, 50_000),
            Workload::Bursty => (1_000_000, 200_000, 500_000),
            Workload::SustainedHigh => (5_000_000, 500_000, 2_000_000),
            Workload::Ramping => (100_000, 10_000, 50_000),
        };

        for i in 0..num_samples {
            let (cycles, misses, mem) = match self {
                Workload::Idle => (base_cycles, base_misses, base_mem),
                Workload::Bursty => {
                    let burst = if (i % 100) < 10 { 3 } else { 1 };
                    (base_cycles * burst, base_misses * burst, base_mem * burst)
                }
                Workload::SustainedHigh => (base_cycles, base_misses, base_mem),
                Workload::Ramping => {
                    let ramp = (i as f64 / num_samples as f64).min(1.0);
                    let scale = 1.0 + ramp * 4.0;
                    (
                        (base_cycles as f64 * scale) as u64,
                        (base_misses as f64 * scale) as u64,
                        (base_mem as f64 * scale) as u64,
                    )
                }
            };
            counters.push((cycles, misses, mem));
        }
        counters
    }
}

/// Power model for PMU-based energy estimation.
#[derive(Debug, Clone)]
pub struct PowerModel {
    pub cycle_energy_j: f64,      // Joules per cycle
    pub cache_miss_energy_j: f64, // Joules per cache miss
    pub memory_energy_j: f64,     // Joules per memory access
}

impl PowerModel {
    pub fn new(cycle_energy_j: f64, cache_miss_energy_j: f64, memory_energy_j: f64) -> Self {
        Self {
            cycle_energy_j,
            cache_miss_energy_j,
            memory_energy_j,
        }
    }

    /// Estimate power from a single counter sample.
    pub fn estimate_power(&self, cycles: u64, misses: u64, mem: u64) -> f64 {
        self.cycle_energy_j * cycles as f64
            + self.cache_miss_energy_j * misses as f64
            + self.memory_energy_j * mem as f64
    }
}

/// PMU Estimator — indirect, model-based energy calculation.
pub struct PmuEstimator {
    pub model: PowerModel,
    pub alpha: f64,
    pub ema_power: f64,
}

impl PmuEstimator {
    pub fn new(model: PowerModel, alpha: f64) -> Result<Self, &'static str> {
        if alpha < 0.0 || alpha > 1.0 {
            return Err("Alpha must be in [0.0, 1.0]");
        }
        Ok(Self {
            model,
            alpha,
            ema_power: 0.0,
        })
    }

    /// Estimate total energy from PMU counter stream.
    pub fn estimate_energy(&mut self, counters: &[(u64, u64, u64)], dt: f64) -> f64 {
        let mut total_energy = 0.0;
        for (cycles, misses, mem) in counters {
            let instant_power = self.model.estimate_power(*cycles, *misses, *mem);
            self.ema_power = self.alpha * instant_power + (1.0 - self.alpha) * self.ema_power;
            total_energy += self.ema_power * dt;
        }
        total_energy
    }
}

/// Physical Meter — direct V·I integration with ADC quantization.
pub struct PhysicalMeter {
    pub adc_bits: u32,
    pub sampling_jitter: f64,
    pub seed: u64,
}

impl PhysicalMeter {
    pub fn new(adc_bits: u32, sampling_jitter: f64, seed: u64) -> Result<Self, &'static str> {
        if adc_bits == 0 || adc_bits > 32 {
            return Err("ADC bits must be in [1, 32]");
        }
        Ok(Self {
            adc_bits,
            sampling_jitter,
            seed,
        })
    }

    /// Measure energy by integrating V·I over time with ADC effects.
    pub fn measure_energy(&self, signal: &[(f64, f64, f64)]) -> f64 {
        let mut total_energy = 0.0;
        let quantization_step_v = 5.0 / (2_u64.pow(self.adc_bits) as f64);
        let quantization_step_i = 10.0 / (2_u64.pow(self.adc_bits) as f64);

        let mut rng_state = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);

        for window in signal.windows(2) {
            let (t1, v1, i1) = window[0];
            let (t2, _v2, _i2) = window[1];
            let dt = t2 - t1;

            let v_quant = (v1 / quantization_step_v).round() * quantization_step_v;
            let i_quant = (i1 / quantization_step_i).round() * quantization_step_i;

            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let jitter = ((rng_state as f64 / u64::MAX as f64) - 0.5) * 2.0 * self.sampling_jitter;
            let dt_eff = (dt + jitter).max(0.0);

            let power = v_quant * i_quant;
            total_energy += power * dt_eff;
        }
        total_energy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_power_basic() {
        let profile = DvfsProfile::new(1.0, 1_000_000_000.0, 1e-9).unwrap();
        let power = compute_dynamic_power(&profile);
        assert!((power - 1.0).abs() < 1e-6, "Expected ~1.0 W, got {}", power);
    }

    #[test]
    fn test_ema_filter_update() {
        let mut ema = EmaFilter::new(0.5, 1.0).unwrap();
        let result = ema.update(1.2, 1.0);
        assert!((result - 1.1).abs() < 1e-9, "Expected 1.1, got {}", result);
    }

    #[test]
    fn test_pmu_counter_overflow() {
        let mut pmu = PmuSimulator::new();
        pmu.set_near_overflow();
        pmu.increment(200);
        assert_eq!(pmu.overflow_count, 1);
        let delta = pmu.read();
        assert_eq!(delta, 99);
    }
}
