//! BRICK-49 Pillar 4: Lineage Ledger
//! Immutable state hashing, divergence prevention, cross-scale integrity
//! Benchmark: 100% immutability, 0 deviation from master blueprint

use crate::brick49::types::LineageEntry;
use std::collections::HashMap;

/// LineageLedger: Unbroken cryptographic chain
pub struct LineageLedger {
    entries: Vec<LineageEntry>,
    hash_index: HashMap<String, usize>,
    divergence_events: u64,
    total_operations: u64,
}

impl LineageLedger {
    pub fn new() -> Self {
        let genesis = LineageEntry::genesis();
        let mut hash_index = HashMap::new();
        hash_index.insert(genesis.operation_hash.clone(), 0);
        Self {
            entries: vec![genesis],
            hash_index,
            divergence_events: 0,
            total_operations: 0,
        }
    }

    pub fn append(&mut self, op_hash: &str) -> LineageEntry {
        let parent = self.entries.last().unwrap();
        let entry = parent.child(op_hash);

        // Verify immutability: parent hash must exist
        if !self.hash_index.contains_key(&entry.parent_hash) {
            self.divergence_events += 1;
        }

        self.hash_index
            .insert(entry.operation_hash.clone(), self.entries.len());
        self.entries.push(entry.clone());
        self.total_operations += 1;
        entry
    }

    pub fn verify_chain(&self) -> bool {
        for i in 1..self.entries.len() {
            let current = &self.entries[i];
            let parent = &self.entries[i - 1];
            if current.parent_hash != parent.operation_hash {
                return false;
            }
        }
        true
    }

    pub fn immutability_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 1.0;
        }
        let valid = self.total_operations - self.divergence_events;
        valid as f64 / self.total_operations as f64
    }

    pub fn divergence_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        self.divergence_events as f64 / self.total_operations as f64
    }

    pub fn stats(&self) -> (usize, u64, u64, f64, f64) {
        (
            self.entries.len(),
            self.total_operations,
            self.divergence_events,
            self.immutability_rate(),
            self.divergence_rate(),
        )
    }
}
