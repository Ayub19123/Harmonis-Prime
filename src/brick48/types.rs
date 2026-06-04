//! BRICK-48: Predictive Sovereignty â€” Shared Types
//! Deterministic, bounded, zero-drift type system for predictive autonomy

use std::time::Instant;

/// ProvenanceEntry: Immutable cryptographic lineage record
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProvenanceEntry {
    pub action_id: String,
    pub brick46_dna_hash: String,
    pub merkle_root: String,
    pub timestamp: Instant,
    pub zk_proof: Vec<u8>,
}

impl ProvenanceEntry {
    pub fn new(action_id: &str, dna_hash: &str, merkle_root: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            brick46_dna_hash: dna_hash.to_string(),
            merkle_root: merkle_root.to_string(),
            timestamp: Instant::now(),
            zk_proof: Vec::new(),
        }
    }

    pub fn with_zk_proof(mut self, proof: Vec<u8>) -> Self {
        self.zk_proof = proof;
        self
    }
}

/// PredictiveEvent: Future-state projection with confidence
#[derive(Clone, Debug)]
pub struct PredictiveEvent {
    pub event_id: String,
    pub predicted_time: Instant,
    pub confidence: f64,
    pub severity: f64,
    pub layer: String,
    pub preemptive_action_id: Option<String>,
}

impl PredictiveEvent {
    pub fn new(id: &str, confidence: f64, severity: f64, layer: &str) -> Self {
        Self {
            event_id: id.to_string(),
            predicted_time: Instant::now(),
            confidence: confidence.clamp(0.0, 1.0),
            severity: severity.clamp(0.0, 1.0),
            layer: layer.to_string(),
            preemptive_action_id: None,
        }
    }

    pub fn with_preemptive_action(mut self, action_id: &str) -> Self {
        self.preemptive_action_id = Some(action_id.to_string());
        self
    }
}

/// HorizonForecast: Multi-horizon demand projection
#[derive(Clone, Debug)]
pub struct HorizonForecast {
    pub forecast_id: String,
    pub horizon_seconds: u64,
    pub predicted_load: f64,
    pub predicted_memory: f64,
    pub predicted_network: f64,
    pub confidence: f64,
}

impl HorizonForecast {
    pub fn new(
        id: &str,
        horizon: u64,
        load: f64,
        memory: f64,
        network: f64,
        confidence: f64,
    ) -> Self {
        Self {
            forecast_id: id.to_string(),
            horizon_seconds: horizon,
            predicted_load: load.max(0.0),
            predicted_memory: memory.max(0.0),
            predicted_network: network.max(0.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// PreemptiveRemediation: Action taken before problem materializes
#[derive(Clone, Debug)]
pub struct PreemptiveRemediation {
    pub action_id: String,
    pub target_event_id: String,
    pub action_type: String,
    pub resource_delta: f64,
    pub execution_time_ms: f64,
    pub resolved_before_materialization: bool,
}

impl PreemptiveRemediation {
    pub fn new(action_id: &str, target: &str, action_type: &str, delta: f64) -> Self {
        Self {
            action_id: action_id.to_string(),
            target_event_id: target.to_string(),
            action_type: action_type.to_string(),
            resource_delta: delta,
            execution_time_ms: 0.0,
            resolved_before_materialization: false,
        }
    }

    pub fn mark_resolved(mut self, exec_ms: f64) -> Self {
        self.execution_time_ms = exec_ms;
        self.resolved_before_materialization = true;
        self
    }
}

/// ThermodynamicProfile: Energy/heat metrics for UTE certification
#[derive(Clone, Debug)]
pub struct ThermodynamicProfile {
    pub baseline_heat_joules: f64,
    pub forecast_heat_joules: f64,
    pub theoretical_minimum_joules: f64,
    pub efficiency_ratio: f64,
}

impl ThermodynamicProfile {
    pub fn new(baseline: f64, forecast: f64, theoretical: f64) -> Self {
        let ratio = if forecast > 0.0 {
            theoretical / forecast
        } else {
            0.0
        };
        Self {
            baseline_heat_joules: baseline,
            forecast_heat_joules: forecast,
            theoretical_minimum_joules: theoretical,
            efficiency_ratio: ratio.clamp(0.0, 1.0),
        }
    }
}

/// AutonomyMetrics: TSA certification tracking
#[derive(Clone, Debug)]
pub struct AutonomyMetrics {
    pub start_time: Instant,
    pub human_interventions: u64,
    pub decisions_autonomous: u64,
    pub health_score: f64,
}

impl AutonomyMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            human_interventions: 0,
            decisions_autonomous: 0,
            health_score: 1.0,
        }
    }
}

/// AdversarialScenario: CSR certification chaos injection
#[derive(Clone, Debug)]
pub enum AdversarialScenario {
    GeopoliticalShift {
        region: String,
        impact_score: f64,
    },
    InfrastructureCollapse {
        node_count: u64,
        cascade_risk: f64,
    },
    CosmicDataSpike {
        magnitude: f64,
        duration_ms: u64,
    },
    ByzantineSurge {
        agent_count: u64,
        deception_level: f64,
    },
    QuantumDecoherence {
        qubit_loss_rate: f64,
    },
}

impl AdversarialScenario {
    pub fn noise_level(&self) -> f64 {
        match self {
            AdversarialScenario::GeopoliticalShift { impact_score, .. } => *impact_score,
            AdversarialScenario::InfrastructureCollapse { cascade_risk, .. } => *cascade_risk,
            AdversarialScenario::CosmicDataSpike { magnitude, .. } => *magnitude,
            AdversarialScenario::ByzantineSurge {
                deception_level, ..
            } => *deception_level,
            AdversarialScenario::QuantumDecoherence {
                qubit_loss_rate, ..
            } => *qubit_loss_rate,
        }
    }
}
