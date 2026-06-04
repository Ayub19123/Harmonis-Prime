use crate::cre::causal_graph::CausalGraph;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct CausalContribution {
    pub cause_id: String,
    pub cause_type: String,
    pub probability: f64,
    pub path: Vec<String>,
    pub intervention_hint: String,
}

#[derive(Debug, Clone)]
pub struct RootCauseReport {
    pub symptom: String,
    pub timestamp: u64,
    pub ranked_causes: Vec<CausalContribution>,
    pub confidence: f64,
}

pub struct RootCauseAnalyzer;

impl RootCauseAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, graph: &CausalGraph, symptom: &str, depth: usize) -> RootCauseReport {
        let mut ranked_causes = Vec::new();
        for (id, event) in &graph.nodes {
            if self.has_path(graph, id, symptom, depth) {
                ranked_causes.push(CausalContribution {
                    cause_id: id.to_string(),
                    cause_type: format!("{:?}", event.event_type),
                    probability: 0.75,
                    path: vec![id.to_string(), symptom.to_string()],
                    intervention_hint: format!("Inspect node {} and upstream ancestors", id),
                });
            }
        }
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        RootCauseReport {
            symptom: symptom.to_string(),
            timestamp,
            ranked_causes,
            confidence: 0.92,
        }
    }

    fn has_path(&self, graph: &CausalGraph, from: &str, to: &str, max_depth: usize) -> bool {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back((from.to_string(), 0usize));
        visited.insert(from.to_string());
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            for child in graph.get_children(&current) {
                if child.target == to {
                    return true;
                }
                if !visited.contains(&child.target) {
                    visited.insert(child.target.clone());
                    queue.push_back((child.target.clone(), depth + 1));
                }
            }
        }
        false
    }

    pub fn is_causal(&self, graph: &CausalGraph, cause: &str, effect: &str) -> bool {
        self.has_path(graph, cause, effect, 10)
    }
}
