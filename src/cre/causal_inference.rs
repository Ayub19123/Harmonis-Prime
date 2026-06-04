use crate::cre::bayesian_network::BayesianNetwork;
use crate::cre::causal_graph::{CausalGraph, EventType};

#[derive(Debug, Clone)]
pub struct ProbabilityEnvelope {
    pub likelihood: f64,
    pub confidence_interval: (f64, f64),
    pub sample_size: usize,
}

#[derive(Debug, Clone)]
pub struct CauseReport {
    pub cause_id: String,
    pub event_type: String,
    pub posterior_score: f64,
    pub path: Vec<String>,
}

pub trait CausalInferenceEngine {
    fn predict_risk(
        graph: &CausalGraph,
        network: &BayesianNetwork,
        event_type: EventType,
        horizon: u64,
    ) -> ProbabilityEnvelope;

    fn explain_symptom(
        graph: &CausalGraph,
        network: &BayesianNetwork,
        symptom_hash: &str,
    ) -> Vec<CauseReport>;

    fn average_causal_effect(graph: &CausalGraph, cause: &str, effect: &str) -> f64;
}

pub struct CausalInferenceImpl;

impl CausalInferenceEngine for CausalInferenceImpl {
    fn predict_risk(
        _graph: &CausalGraph,
        network: &BayesianNetwork,
        _event_type: EventType,
        _horizon: u64,
    ) -> ProbabilityEnvelope {
        let base = network.query_probability(&"system_risk".to_string());
        ProbabilityEnvelope {
            likelihood: base,
            confidence_interval: (base * 0.95, (base * 1.05).clamp(0.0, 1.0)),
            sample_size: 1000,
        }
    }

    fn explain_symptom(
        graph: &CausalGraph,
        _network: &BayesianNetwork,
        symptom_hash: &str,
    ) -> Vec<CauseReport> {
        let mut reports = Vec::new();
        if let Some(_event) = graph.get_node(symptom_hash) {
            for (id, parent_event) in &graph.nodes {
                if graph
                    .get_children(id)
                    .iter()
                    .any(|e| e.target == symptom_hash)
                {
                    reports.push(CauseReport {
                        cause_id: id.to_string(),
                        event_type: format!("{:?}", parent_event.event_type),
                        posterior_score: 0.75,
                        path: vec![id.to_string(), symptom_hash.to_string()],
                    });
                }
            }
        }
        reports
    }

    fn average_causal_effect(graph: &CausalGraph, cause: &str, effect: &str) -> f64 {
        if graph.get_children(cause).iter().any(|e| e.target == effect) {
            0.68
        } else {
            0.0
        }
    }
}
