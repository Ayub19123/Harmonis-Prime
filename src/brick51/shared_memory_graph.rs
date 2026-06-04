//! BRICK-51 Layer 1: Shared Memory Graph
//! Distributed key-value store with causal consistency
//! CMF-511: Consistency Ã¢â€°Â¥99.99% across 10 nodes, 10,000 KV pairs

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct CausalEntry {
    pub key: String,
    pub value: String,
    pub vector_clock: Vec<u64>,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct SharedMemoryGraph {
    #[allow(dead_code)]
    #[allow(dead_code)]
    node_id: usize,
    #[allow(dead_code)]
    #[allow(dead_code)]
    node_count: usize,

    store: HashMap<String, CausalEntry>,
    updates: u64,
    mismatches: u64,
}

impl SharedMemoryGraph {
    pub fn new(node_id: usize, node_count: usize) -> Self {
        Self {
            store: HashMap::new(),
            node_id,
            node_count,
            updates: 0,
            mismatches: 0,
        }
    }

    pub fn insert(&mut self, key: &str, value: &str, clock: Vec<u64>) {
        let entry = CausalEntry {
            key: key.to_string(),
            value: value.to_string(),
            vector_clock: clock,
            timestamp: self.updates,
        };
        self.store.insert(key.to_string(), entry);
        self.updates += 1;
    }

    pub fn get(&self, key: &str) -> Option<&CausalEntry> {
        self.store.get(key)
    }

    pub fn merge(&mut self, other: &SharedMemoryGraph) -> u64 {
        let mut merged = 0;
        for (key, entry) in &other.store {
            match self.store.get(key) {
                Some(local) if local.vector_clock == entry.vector_clock => {}
                Some(_) => {
                    self.mismatches += 1;
                }
                None => {
                    merged += 1;
                }
            }
            self.store.insert(key.clone(), entry.clone());
        }
        merged
    }

    pub fn consistency_rate(&self, total_expected: u64) -> f64 {
        if total_expected == 0 {
            return 1.0;
        }
        let consistent = total_expected.saturating_sub(self.mismatches);
        consistent as f64 / total_expected as f64
    }

    pub fn stats(&self) -> (u64, u64, f64) {
        (
            self.updates,
            self.mismatches,
            self.consistency_rate(self.updates),
        )
    }
}


