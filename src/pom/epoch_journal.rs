use crate::pom::operational_memory::{OpPayload, OperationalEntry};
use std::sync::{Arc, Mutex};

pub struct EpochJournal {
    current_epoch: u64,
    current_term: u64,
    sequence: Arc<Mutex<u64>>,
}

impl EpochJournal {
    pub fn new() -> Self {
        Self {
            current_epoch: 1,
            current_term: 1,
            sequence: Arc::new(Mutex::new(0)),
        }
    }

    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    pub fn current_term(&self) -> u64 {
        self.current_term
    }

    pub fn advance_epoch(&mut self, leader_id: u64) -> OperationalEntry {
        let mut seq = self.sequence.lock().unwrap();
        *seq += 1;

        let entry = OperationalEntry {
            sequence: *seq,
            epoch_id: self.current_epoch,
            term: self.current_term,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            payload: OpPayload::EpochSealed {
                term: self.current_term,
                leader_id,
                next_epoch: self.current_epoch + 1,
            },
            parent_hash: [0u8; 32], // Will be set by caller with actual chain
            raft_index: 0,
            entry_hash: [0u8; 32], // Will be computed by caller
        };

        self.current_epoch += 1;
        entry
    }

    pub fn seal_epoch(&mut self, term: u64, leader_id: u64) -> OperationalEntry {
        self.current_term = term;
        self.advance_epoch(leader_id)
    }
}
