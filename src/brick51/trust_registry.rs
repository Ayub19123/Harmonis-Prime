//! BRICK-51 Layer 1: Trust Registry
//! Verifiable node identities and reputation scores
//! CMF-524: Decentralised resource sharing with verifiable trust

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct NodeIdentity {
    pub node_id: String,
    pub public_key_hash: String,
    pub reputation: f64,
    pub trust_score: f64,
    pub compute_offered: u64,
    pub compute_consumed: u64,
}

pub struct TrustRegistry {
    nodes: HashMap<String, NodeIdentity>,
    transactions: u64,
    verified_barters: u64,
}

impl TrustRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            transactions: 0,
            verified_barters: 0,
        }
    }

    pub fn register(&mut self, id: &str, key_hash: &str) {
        let node = NodeIdentity {
            node_id: id.to_string(),
            public_key_hash: key_hash.to_string(),
            reputation: 1.0,
            trust_score: 1.0,
            compute_offered: 0,
            compute_consumed: 0,
        };
        self.nodes.insert(id.to_string(), node);
    }

    pub fn barter_compute(&mut self, from: &str, to: &str, cycles: u64) -> bool {
        self.transactions += 1;
        let from_node = self.nodes.get(from).cloned();
        let to_node = self.nodes.get(to).cloned();

        match (from_node, to_node) {
            (Some(mut f), Some(mut t)) => {
                if f.compute_offered >= cycles && f.trust_score > 0.5 {
                    f.compute_offered -= cycles;
                    t.compute_consumed += cycles;
                    self.verified_barters += 1;
                    self.nodes.insert(from.to_string(), f);
                    self.nodes.insert(to.to_string(), t);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn offer_compute(&mut self, node_id: &str, cycles: u64) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.compute_offered += cycles;
        }
    }

    pub fn trust_rate(&self) -> f64 {
        if self.transactions == 0 {
            return 1.0;
        }
        self.verified_barters as f64 / self.transactions as f64
    }

    pub fn stats(&self) -> (u64, u64, f64) {
        (self.transactions, self.verified_barters, self.trust_rate())
    }
}
