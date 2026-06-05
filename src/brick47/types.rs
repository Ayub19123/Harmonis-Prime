//! BRICK-47: Self-Healing Observability â€” Shared Types
//! Deterministic, bounded, zero-drift type system for all observability layers

use std::time::Instant;

/// CausalNode: Event node in the Causal Event Graph
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CausalNode {
    pub id: String,
    pub layer: String,
    pub timestamp: Instant,
    pub event_type: String,
}

impl CausalNode {
    pub fn new(id: &str, layer: &str, event_type: &str) -> Self {
        Self {
            id: id.to_string(),
            layer: layer.to_string(),
            timestamp: Instant::now(),
            event_type: event_type.to_string(),
        }
    }
}

/// CausalEdge: Directed edge with confidence weight
#[derive(Clone, Debug)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    pub weight: f64,
    pub confidence: f64,
}

impl CausalEdge {
    pub fn new(from: &str, to: &str, weight: f64, confidence: f64) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            weight: weight.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// IncidentRecord: Semantic incident for Temporal Memory Engine
#[derive(Clone, Debug)]
pub struct IncidentRecord {
    pub id: String,
    pub pattern_signature: String,
    pub context: String,
    pub remediation: String,
    pub outcome: String,
    pub confidence: f64,
    pub timestamp: Instant,
    pub access_count: u64,
}

impl IncidentRecord {
    pub fn new(id: &str, pattern: &str, context: &str, remediation: &str, outcome: &str) -> Self {
        Self {
            id: id.to_string(),
            pattern_signature: pattern.to_string(),
            context: context.to_string(),
            remediation: remediation.to_string(),
            outcome: outcome.to_string(),
            confidence: 1.0,
            timestamp: Instant::now(),
            access_count: 0,
        }
    }

    /// Logarithmic decay: older incidents lose relevance
    pub fn decay_score(&self, now: Instant) -> f64 {
        let age_secs = now.duration_since(self.timestamp).as_secs_f64().max(1.0);
        let decay = self.confidence / (1.0 + age_secs.ln());
        decay * (1.0 + (self.access_count as f64).ln().max(0.0))
    }
}

/// RemediationAction: Proposed fix with metadata
#[derive(Clone, Debug)]
pub struct RemediationAction {
    pub action_id: String,
    pub description: String,
    pub target_layer: String,
    pub estimated_impact_ms: f64,
    pub rollback_steps: Vec<String>,
}

impl RemediationAction {
    pub fn new(id: &str, description: &str, target_layer: &str, impact_ms: f64) -> Self {
        Self {
            action_id: id.to_string(),
            description: description.to_string(),
            target_layer: target_layer.to_string(),
            estimated_impact_ms: impact_ms,
            rollback_steps: Vec::new(),
        }
    }

    pub fn with_rollback(mut self, steps: Vec<&str>) -> Self {
        self.rollback_steps = steps.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// AuditEntry: Immutable trace for full audit coverage
#[derive(Clone, Debug)]
pub struct AuditEntry {
    pub stage: String,
    pub timestamp: Instant,
    pub details: String,
    pub success: bool,
}

impl AuditEntry {
    pub fn new(stage: &str, details: &str, success: bool) -> Self {
        Self {
            stage: stage.to_string(),
            timestamp: Instant::now(),
            details: details.to_string(),
            success,
        }
    }
}

/// SystemLayer: Enumeration of observable layers
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SystemLayer {
    Infrastructure,
    Application,
    Network,
    Decision,
    Resource,
    Quantum,
}

impl SystemLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemLayer::Infrastructure => "infra",
            SystemLayer::Application => "app",
            SystemLayer::Network => "network",
            SystemLayer::Decision => "decision",
            SystemLayer::Resource => "resource",
            SystemLayer::Quantum => "quantum",
        }
    }
}

/// MetricSample: Point-in-time metric for drift detection
#[derive(Clone, Debug)]
pub struct MetricSample {
    pub name: String,
    pub value: f64,
    pub timestamp: Instant,
    pub layer: SystemLayer,
}

impl MetricSample {
    pub fn new(name: &str, value: f64, layer: SystemLayer) -> Self {
        Self {
            name: name.to_string(),
            value,
            timestamp: Instant::now(),
            layer,
        }
    }
}

/// SimulationResult: Counterfactual simulation output
#[derive(Clone, Debug)]
pub struct SimulationResult {
    pub action_id: String,
    pub stability_delta: f64,
    pub performance_delta: f64,
    pub cascade_risk: f64,
    pub approved: bool,
    pub reason: String,
}

impl SimulationResult {
    pub fn approved(action_id: &str, stability: f64, perf: f64, risk: f64) -> Self {
        Self {
            action_id: action_id.to_string(),
            stability_delta: stability,
            performance_delta: perf,
            cascade_risk: risk,
            approved: true,
            reason: "Simulation passed all invariants".to_string(),
        }
    }

    pub fn rejected(action_id: &str, reason: &str) -> Self {
        Self {
            action_id: action_id.to_string(),
            stability_delta: 0.0,
            performance_delta: 0.0,
            cascade_risk: 1.0,
            approved: false,
            reason: reason.to_string(),
        }
    }
}
