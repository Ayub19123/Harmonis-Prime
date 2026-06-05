use crate::pom::operational_memory::{compute_hash, OpPayload, OperationalEntry};
use crate::types::ClusterId;

pub struct GovernanceChain {
    sequence: u64,
    last_hash: [u8; 32],
}

impl GovernanceChain {
    pub fn new(genesis_hash: [u8; 32]) -> Self {
        Self {
            sequence: 0,
            last_hash: genesis_hash,
        }
    }

    pub fn record_vote(
        &mut self,
        proposal_id: String,
        voter: ClusterId,
        decision: bool,
        weight: u64,
        epoch_id: u64,
        term: u64,
    ) -> OperationalEntry {
        self.sequence += 1;

        let entry = OperationalEntry {
            sequence: self.sequence,
            epoch_id,
            term,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            payload: OpPayload::GovernanceVote {
                proposal_id,
                voter,
                decision,
                weight,
            },
            parent_hash: self.last_hash,
            raft_index: 0,
            entry_hash: [0u8; 32],
        };

        self.last_hash = compute_hash(&entry);
        entry
    }

    pub fn record_policy(
        &mut self,
        policy_id: String,
        hash: [u8; 32],
        enactment_epoch: u64,
        term: u64,
    ) -> OperationalEntry {
        self.sequence += 1;

        let entry = OperationalEntry {
            sequence: self.sequence,
            epoch_id: enactment_epoch,
            term,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            payload: OpPayload::PolicyEnacted {
                policy_id,
                hash,
                enactment_epoch,
            },
            parent_hash: self.last_hash,
            raft_index: 0,
            entry_hash: [0u8; 32],
        };

        self.last_hash = compute_hash(&entry);
        entry
    }

    pub fn head_hash(&self) -> [u8; 32] {
        self.last_hash
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}
