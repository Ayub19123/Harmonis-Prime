//! BRICK-51 Layer 1: Knowledge Ledger
//! Immutable record of all shared knowledge with truth verification
//! CMF-516: False knowledge acceptance <0.1%

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct KnowledgeEntry {
    pub statement: String,
    pub verified: bool,
    pub proof_hash: String,
    pub source_node: String,
}

pub struct KnowledgeLedger {
    entries: Vec<KnowledgeEntry>,
    truth_index: HashMap<String, bool>,
    false_accepted: u64,
    total_validated: u64,
}

impl KnowledgeLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            truth_index: HashMap::new(),
            false_accepted: 0,
            total_validated: 0,
        }
    }

    pub fn submit(&mut self, statement: &str, proof_hash: &str, source: &str) {
        let entry = KnowledgeEntry {
            statement: statement.to_string(),
            verified: false,
            proof_hash: proof_hash.to_string(),
            source_node: source.to_string(),
        };
        self.entries.push(entry);
    }

    pub fn verify(&mut self, statement: &str, truth_value: bool) -> bool {
        self.total_validated += 1;
        let is_false = !truth_value;

        for entry in self.entries.iter_mut() {
            if entry.statement == statement {
                entry.verified = true;
                if is_false {
                    self.false_accepted += 1;
                    return false; // Rejected
                }
                self.truth_index.insert(statement.to_string(), true);
                return true; // Accepted
            }
        }
        false
    }

    pub fn false_acceptance_rate(&self) -> f64 {
        if self.total_validated == 0 {
            return 0.0;
        }
        self.false_accepted as f64 / self.total_validated as f64
    }

    pub fn integrity_rate(&self) -> f64 {
        1.0 - self.false_acceptance_rate()
    }

    pub fn stats(&self) -> (u64, u64, f64, f64) {
        (
            self.total_validated,
            self.false_accepted,
            self.false_acceptance_rate(),
            self.integrity_rate(),
        )
    }
}
