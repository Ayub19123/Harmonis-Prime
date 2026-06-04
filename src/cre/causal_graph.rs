use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type NodeId = String;
pub type Weight = f64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    ClusterJoined,
    ShardMigrated,
    PolicyEnacted,
    EpochSealed,
    GovernanceVote,
    TopologyDelta,
    LeaderFailover,
    QuorumRebalanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvent {
    pub event_type: EventType,
    pub timestamp: u128,
    pub hash: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectedEdge {
    pub target: NodeId,
    pub weight: Weight,
}

pub struct CausalGraph {
    pub nodes: HashMap<NodeId, CausalEvent>,
    pub edges: HashMap<NodeId, Vec<DirectedEdge>>,
}

impl CausalGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, id: NodeId, event: CausalEvent) {
        self.nodes.insert(id, event);
    }

    pub fn add_causal_link(&mut self, source: NodeId, target: NodeId, weight: Weight) {
        let clamped = weight.clamp(0.0, 1.0);
        self.edges.entry(source).or_default().push(DirectedEdge {
            target,
            weight: clamped,
        });
    }

    pub fn get_node(&self, id: &str) -> Option<&CausalEvent> {
        self.nodes.get(id)
    }

    pub fn get_children(&self, source: &str) -> Vec<&DirectedEdge> {
        self.edges
            .get(source)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }
}
