//! BRICK-46: Synthetic Nervous System — Shared Types
//! Deterministic, bounded, zero-drift type system for all layers

use std::time::Instant;

/// SloMetric: Service Level Objective with weighted scoring
#[derive(Clone, Debug, PartialEq)]
pub struct SloMetric {
    pub name: String,
    pub value: f64,
    pub target: f64,
    pub weight: f64,
}

impl SloMetric {
    pub fn new(name: &str, target: f64, weight: f64) -> Self {
        Self {
            name: name.to_string(),
            value: 0.0,
            target,
            weight: weight.clamp(0.0, 1.0),
        }
    }

    /// Gap from target (negative = below target, positive = above target)
    pub fn gap(&self) -> f64 {
        self.value - self.target
    }

    /// Normalized score (0.0 = worst, 1.0 = best)
    pub fn normalized_score(&self) -> f64 {
        if self.target <= 0.0 {
            return 1.0;
        }
        let ratio = self.value / self.target;
        if ratio >= 1.0 {
            1.0
        } else {
            ratio.clamp(0.0, 1.0)
        }
    }
}

/// HealthSnapshot: Immutable point-in-time system health
#[derive(Clone, Debug)]
pub struct HealthSnapshot {
    pub timestamp: Instant,
    pub slo_score: f64,
    pub error_rate: f64,
    pub latency_p99_ms: f64,
    pub energy_budget_used: f64,
    pub violations: Vec<String>,
}

impl HealthSnapshot {
    pub fn is_healthy(&self) -> bool {
        self.slo_score >= 0.9 && self.error_rate < 0.01 && self.violations.is_empty()
    }
}

/// ReflexEvent: Peripheral nervous system signal
#[derive(Clone, Debug)]
pub struct ReflexEvent {
    pub source_id: String,
    pub signal_type: String,
    pub magnitude: f64,
    pub timestamp: Instant,
}

impl ReflexEvent {
    pub fn new(source_id: &str, signal_type: &str, magnitude: f64) -> Self {
        Self {
            source_id: source_id.to_string(),
            signal_type: signal_type.to_string(),
            magnitude,
            timestamp: Instant::now(),
        }
    }
}

/// CognitiveSignal: Elevated signal from reflex to cognitive layer
#[derive(Clone, Debug)]
pub struct CognitiveSignal {
    pub correlation_id: String,
    pub summary: String,
    pub anomaly_score: f64,
    pub recommended_action: Option<String>,
    pub timestamp: Instant,
}

impl CognitiveSignal {
    pub fn new(correlation_id: &str, summary: &str, anomaly_score: f64) -> Self {
        Self {
            correlation_id: correlation_id.to_string(),
            summary: summary.to_string(),
            anomaly_score: anomaly_score.clamp(0.0, 1.0),
            recommended_action: None,
            timestamp: Instant::now(),
        }
    }

    pub fn with_action(mut self, action: &str) -> Self {
        self.recommended_action = Some(action.to_string());
        self
    }
}

/// QStateKey: Quantum superposition context identifier
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct QStateKey {
    pub context: String,
    pub dimension: usize,
}

/// QStateValue: Quantum amplitude vector with confidence
#[derive(Clone, Debug)]
pub struct QStateValue {
    pub amplitudes: Vec<f64>,
    pub confidence: f64,
}

/// ApiFlowRequest: Zero-friction API mesh ingress
#[derive(Clone, Debug)]
pub struct ApiFlowRequest {
    pub tenant_id: String,
    pub operation: String,
    pub payload_bytes: usize,
    pub priority: u8,
}

impl ApiFlowRequest {
    pub fn new(tenant_id: &str, operation: &str, payload_bytes: usize) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            operation: operation.to_string(),
            payload_bytes,
            priority: 128, // Default mid-priority
        }
    }
}

/// ApiFlowResponse: Zero-friction API mesh egress
#[derive(Clone, Debug)]
pub struct ApiFlowResponse {
    pub success: bool,
    pub message: String,
    pub latency_ms: f64,
}

impl ApiFlowResponse {
    pub fn ok(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            latency_ms: 0.0,
        }
    }

    pub fn err(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            latency_ms: 0.0,
        }
    }
}
