use crate::cre::causal_graph::{NodeId, Weight};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub node_id: NodeId,
    pub observed_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalProbability {
    pub parents: Vec<NodeId>,
    pub probability: Weight,
}

pub struct BayesianNetwork {
    pub conditional_probabilities: HashMap<NodeId, Vec<ConditionalProbability>>,
    pub priors: HashMap<NodeId, Weight>,
    pub evidence: Vec<Evidence>,
}

impl BayesianNetwork {
    pub fn new() -> Self {
        Self {
            conditional_probabilities: HashMap::new(),
            priors: HashMap::new(),
            evidence: Vec::new(),
        }
    }

    pub fn set_prior(&mut self, node: NodeId, probability: Weight) {
        self.priors.insert(node, probability.clamp(0.0, 1.0));
    }

    pub fn add_conditional(&mut self, node: NodeId, parents: Vec<NodeId>, probability: Weight) {
        self.conditional_probabilities
            .entry(node)
            .or_default()
            .push(ConditionalProbability {
                parents,
                probability: probability.clamp(0.0, 1.0),
            });
    }

    pub fn observe(&mut self, evidence: Evidence) {
        self.evidence.push(evidence);
    }

    pub fn query_probability(&self, target: &NodeId) -> Weight {
        *self.priors.get(target).unwrap_or(&0.5)
    }

    pub fn clear_evidence(&mut self) {
        self.evidence.clear();
    }

    pub fn node_count(&self) -> usize {
        self.priors.len()
    }
}
