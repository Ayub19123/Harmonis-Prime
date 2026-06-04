use std::collections::HashMap;

pub struct GlobalLogReplicator {
    pending: HashMap<String, u64>, // tx_id -> proposed_term
    committed: Vec<String>,
}

impl GlobalLogReplicator {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            committed: Vec::new(),
        }
    }
    
    pub fn propose(&mut self, tx_id: String, term: u64) {
        self.pending.insert(tx_id, term);
    }
    
    pub fn commit(&mut self, tx_id: &str) {
        if self.pending.remove(tx_id).is_some() {
            self.committed.push(tx_id.to_string());
        }
    }
    
    pub fn is_pending(&self, tx_id: &str) -> bool {
        self.pending.contains_key(tx_id)
    }
    
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}