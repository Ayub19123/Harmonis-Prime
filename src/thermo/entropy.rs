//! SET-3: Thermodynamic Entropy Tracking
//! Invariant: dH/dt ≤ 0 after t_convergence (monotonic entropy decrease)
//! Landauer Limit: E_min = k_B * T * ln(2) ≈ 2.87e-21 J at 300K

use std::time::Instant;

/// Boltzmann constant (J/K)
pub const K_BOLTZMANN: f64 = 1.380649e-23;

/// Landauer limit at room temperature (300K) in Joules
/// E_min = k_B * T * ln(2)
pub const LANDAUER_LIMIT_300K: f64 = K_BOLTZMANN * 300.0 * std::f64::consts::LN_2;

/// Shannon entropy of a probability distribution
/// H(X) = -Σ p(x) * log₂(p(x))
#[derive(Debug, Clone)]
pub struct ThermodynamicState {
    pub probabilities: Vec<f64>,
    pub timestamp: Instant,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct EntropyReport {
    pub shannon_entropy: f64,
    pub landauer_energy: f64,
    pub bits_erased: f64,
    pub is_monotonic: bool,
    pub below_landauer: bool,
}

impl ThermodynamicState {
    /// Compute Shannon entropy in bits
    /// H = -Σ p_i * log₂(p_i)
    pub fn shannon_entropy(&self) -> f64 {
        self.probabilities
            .iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| -p * p.log2())
            .sum()
    }

    /// Energy required to erase this state at temperature T (Kelvin)
    /// E = k_B * T * ln(2) * H
    pub fn landauer_energy(&self, temperature_k: f64) -> f64 {
        K_BOLTZMANN * temperature_k * std::f64::consts::LN_2 * self.shannon_entropy()
    }

    /// Verify entropy monotonically decreases from previous state
    /// dH/dt ≤ 0
    pub fn verify_monotonic_decrease(&self, previous: &Self) -> Result<(), ThermoError> {
        let current_h = self.shannon_entropy();
        let previous_h = previous.shannon_entropy();
        
        if current_h > previous_h {
            return Err(ThermoError::EntropyIncreased {
                current: current_h,
                previous: previous_h,
                delta: current_h - previous_h,
            });
        }
        Ok(())
    }

    /// Generate full entropy report
    pub fn report(&self, temperature_k: f64, previous: Option<&Self>) -> EntropyReport {
        let h = self.shannon_entropy();
        let landauer = self.landauer_energy(temperature_k);
        let bits = h;
        
        let is_monotonic = if let Some(prev) = previous {
            self.verify_monotonic_decrease(prev).is_ok()
        } else {
            true
        };

        EntropyReport {
            shannon_entropy: h,
            landauer_energy: landauer,
            bits_erased: bits,
            is_monotonic,
            below_landauer: landauer >= LANDAUER_LIMIT_300K * bits,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ThermoError {
    #[error("Entropy increased: {current:.6} > {previous:.6} (delta: +{delta:.6})")]
    EntropyIncreased { current: f64, previous: f64, delta: f64 },
    
    #[error("Invalid probability distribution: sum = {sum}, expected 1.0")]
    InvalidDistribution { sum: f64 },
    
    #[error("Temperature must be positive: {0}")]
    InvalidTemperature(f64),
}

/// Entropy tracker for consensus state transitions
pub struct EntropyTracker {
    states: Vec<ThermodynamicState>,
    temperature_k: f64,
}

impl EntropyTracker {
    pub fn new(temperature_k: f64) -> Result<Self, ThermoError> {
        if temperature_k <= 0.0 {
            return Err(ThermoError::InvalidTemperature(temperature_k));
        }
        Ok(Self {
            states: Vec::new(),
            temperature_k,
        })
    }

    pub fn record(&mut self, state: ThermodynamicState) -> Result<EntropyReport, ThermoError> {
        if let Some(last) = self.states.last() {
            state.verify_monotonic_decrease(last)?;
        }
        
        let report = state.report(self.temperature_k, self.states.last());
        self.states.push(state);
        Ok(report)
    }

    pub fn verify_landauer_limit(&self) -> bool {
        self.states.iter().all(|s| {
            let e = s.landauer_energy(self.temperature_k);
            let h = s.shannon_entropy();
            e >= LANDAUER_LIMIT_300K * h
        })
    }

    pub fn entropy_trajectory(&self) -> Vec<f64> {
        self.states.iter().map(|s| s.shannon_entropy()).collect()
    }
}