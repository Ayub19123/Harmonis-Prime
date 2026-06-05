//! BRICK-47 Pillar 1: Causal Event Graph (CEG)
//! Node-edge causal map with confidence scoring and root cause reconstruction
//! Benchmark: >= 85% correct root cause under chaos

use crate::brick47::types::{CausalEdge, CausalNode};
use std::collections::{HashMap, VecDeque};

/// CausalEventGraph: Directed graph of system events with weighted edges
pub struct CausalEventGraph {
    nodes: HashMap<String, CausalNode>,
    edges: Vec<CausalEdge>,
    adjacency: HashMap<String, Vec<(String, f64, f64)>>,
    max_nodes: usize,
}

impl CausalEventGraph {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(max_nodes),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            max_nodes,
        }
    }

    pub fn add_event(&mut self, node: CausalNode) {
        if self.nodes.len() >= self.max_nodes {
            if let Some(first_key) = self.nodes.keys().next().cloned() {
                self.nodes.remove(&first_key);
                self.adjacency.remove(&first_key);
            }
        }
        let id = node.id.clone();
        self.nodes.insert(id.clone(), node);
        self.adjacency.entry(id).or_insert_with(Vec::new);
    }

    pub fn add_causal_link(&mut self, edge: CausalEdge) {
        self.edges.push(edge.clone());
        self.adjacency
            .entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push((edge.to.clone(), edge.weight, edge.confidence));
    }

    pub fn root_cause(&self, event_id: &str) -> Option<(String, f64, Vec<String>)> {
        if !self.nodes.contains_key(event_id) {
            return None;
        }

        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back((event_id.to_string(), 1.0, vec![event_id.to_string()]));

        let mut best_path: Option<(String, f64, Vec<String>)> = None;

        while let Some((current, confidence, path)) = queue.pop_front() {
            if let Some(neighbors) = self.adjacency.get(&current) {
                for (to, weight, edge_conf) in neighbors {
                    let new_conf = confidence * weight * edge_conf;
                    let mut new_path = path.clone();
                    new_path.push(to.clone());

                    if let Some(existing) = visited.get(to) {
                        if new_conf <= *existing {
                            continue;
                        }
                    }
                    visited.insert(to.clone(), new_conf);
                    queue.push_back((to.clone(), new_conf, new_path.clone()));

                    if !self.adjacency.contains_key(to)
                        || self.adjacency.get(to).unwrap().is_empty()
                    {
                        if best_path.is_none() || new_conf > best_path.as_ref().unwrap().1 {
                            best_path = Some((to.clone(), new_conf, new_path));
                        }
                    }
                }
            }
        }

        best_path
    }

    pub fn causal_chain(&self, root_id: &str, symptom_id: &str) -> Option<Vec<String>> {
        let mut path = Vec::new();
        let mut current = root_id.to_string();
        path.push(current.clone());

        let max_hops = 20;
        for _ in 0..max_hops {
            if current == symptom_id {
                return Some(path);
            }
            if let Some(neighbors) = self.adjacency.get(&current) {
                if let Some((next, _, _)) = neighbors
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                {
                    current = next.clone();
                    path.push(current.clone());
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if current == symptom_id {
            Some(path)
        } else {
            None
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.adjacency.clear();
    }
}
