use crate::cre::causal_graph::CausalGraph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub target_node: String,
    pub forced_value: f64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct OutcomeDelta {
    pub expected_state_hash: String,
    pub risk_delta: f64,
    pub confidence: f64,
    pub explanation: String,
}

pub struct CounterfactualEngine;

impl CounterfactualEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        _world: &CausalGraph,
        intervention: &Intervention,
        _target: &str,
    ) -> OutcomeDelta {
        let risk_delta = -intervention.forced_value.clamp(0.0, 1.0) * 0.15;
        OutcomeDelta {
            expected_state_hash: format!(
                "sim_{:016x}",
                intervention
                    .target_node
                    .as_bytes()
                    .iter()
                    .fold(0u64, |a, b| a.wrapping_add(*b as u64))
            ),
            risk_delta,
            confidence: 0.82,
            explanation: format!(
                "If {} is forced to {:.2}, risk delta = {:.3}",
                intervention.target_node, intervention.forced_value, risk_delta
            ),
        }
    }

    pub fn find_minimal_intervention(
        &self,
        _world: &CausalGraph,
        _target: &str,
        _desired_outcome: f64,
    ) -> Vec<Intervention> {
        vec![]
    }

    pub fn analyze_scenarios(
        &self,
        world: &CausalGraph,
        interventions: &[Intervention],
        target: &str,
    ) -> Vec<OutcomeDelta> {
        interventions
            .iter()
            .map(|i| self.evaluate(world, i, target))
            .collect()
    }
}
