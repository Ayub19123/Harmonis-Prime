//! SET-3: Quantum State Approximation
//! Non-binary probabilistic state distributions for cognitive decision-making
//! |ψ⟩ = α|0⟩ + β|1⟩ where |α|² + |β|² = 1

use std::f64::consts::PI;

/// Quantum amplitude: complex probability amplitude
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Amplitude {
    pub real: f64,
    pub imag: f64,
}

impl Amplitude {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    /// |α|² = real² + imag²
    pub fn probability(&self) -> f64 {
        self.real.powi(2) + self.imag.powi(2)
    }

    /// Normalize to unit probability
    pub fn normalized(&self) -> Self {
        let norm = self.probability().sqrt();
        if norm > 0.0 {
            Self {
                real: self.real / norm,
                imag: self.imag / norm,
            }
        } else {
            *self
        }
    }
}

/// Quantum state |ψ⟩ = Σ α_i |i⟩
#[derive(Debug, Clone)]
pub struct QuantumState {
    pub amplitudes: Vec<Amplitude>,
    pub label: String,
}

/// Result of quantum measurement (state collapse)
#[derive(Debug, Clone)]
pub struct CollapseResult {
    pub selected_index: usize,
    pub probability: f64,
    pub classical_value: f64,
    pub decoherence_time_nanos: u64,
}

impl QuantumState {
    /// Verify Born rule: Σ |α_i|² = 1
    pub fn verify_normalization(&self) -> Result<(), &'static str> {
        let total_prob: f64 = self.amplitudes.iter().map(|a| a.probability()).sum();
        if (total_prob - 1.0).abs() > 1e-9 {
            return Err("Quantum state not normalized: Σ|α_i|² ≠ 1");
        }
        Ok(())
    }

    /// Measure (collapse) the quantum state into a classical outcome
    /// Uses weighted random selection based on |α_i|²
    pub fn collapse(&self, seed: u64) -> CollapseResult {
        let start = std::time::Instant::now();
        
        let probs: Vec<f64> = self.amplitudes.iter().map(|a| a.probability()).collect();
        let total: f64 = probs.iter().sum();
        
        // Deterministic pseudo-random for reproducibility
        let mut rng = seed;
        let roll = loop {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let candidate = (rng as f64 / u64::MAX as f64) * total;
            if candidate.is_finite() {
                break candidate;
            }
        };

        let mut cumulative = 0.0;
        let mut selected = 0;
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if roll <= cumulative {
                selected = i;
                break;
            }
        }

        let decoherence = start.elapsed().as_nanos() as u64;

        CollapseResult {
            selected_index: selected,
            probability: probs[selected],
            classical_value: selected as f64,
            decoherence_time_nanos: decoherence,
        }
    }

    /// Compute von Neumann entropy S = -Tr(ρ log ρ)
    /// For pure states: S = 0
    /// For mixed states: S > 0
    pub fn von_neumann_entropy(&self) -> f64 {
        self.amplitudes
            .iter()
            .map(|a| {
                let p = a.probability();
                if p > 0.0 { -p * p.ln() } else { 0.0 }
            })
            .sum()
    }

    /// Interpolate between two quantum states (quantum morphing)
    pub fn interpolate(&self, other: &Self, t: f64) -> Self {
        let n = self.amplitudes.len().max(other.amplitudes.len());
        let mut new_amps = Vec::with_capacity(n);
        
        for i in 0..n {
            let a = self.amplitudes.get(i).copied().unwrap_or(Amplitude::new(0.0, 0.0));
            let b = other.amplitudes.get(i).copied().unwrap_or(Amplitude::new(0.0, 0.0));
            new_amps.push(Amplitude::new(
                a.real * (1.0 - t) + b.real * t,
                a.imag * (1.0 - t) + b.imag * t,
            ));
        }

        // Re-normalize
        let total_prob: f64 = new_amps.iter().map(|a| a.probability()).sum();
        let norm = total_prob.sqrt();
        if norm > 0.0 {
            for a in &mut new_amps {
                a.real /= norm;
                a.imag /= norm;
            }
        }

        Self {
            amplitudes: new_amps,
            label: format!("{}↔{}", self.label, other.label),
        }
    }
}

/// Builder for creating valid quantum states
pub struct QuantumStateBuilder {
    amplitudes: Vec<Amplitude>,
}

impl QuantumStateBuilder {
    pub fn new() -> Self {
        Self { amplitudes: Vec::new() }
    }

    pub fn add_basis_state(mut self, real: f64, imag: f64) -> Self {
        self.amplitudes.push(Amplitude::new(real, imag));
        self
    }

    pub fn build(self, label: &str) -> Result<QuantumState, &'static str> {
        let state = QuantumState {
            amplitudes: self.amplitudes,
            label: label.to_string(),
        };
        state.verify_normalization()?;
        Ok(state)
    }
}