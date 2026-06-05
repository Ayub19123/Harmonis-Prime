use crate::brick41::foundation::trust_layer::{AuditEntry, TrustLayer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Ledger: Byzantine fault-tolerant consensus for multi-agent coordination
/// BRICK-41 Phase 1: Foundation — Distributed State Integrity
#[derive(Debug, Clone)]
pub struct Ledger {
    pub node_id: String,
    pub trust: TrustLayer,
    pub state: Arc<Mutex<HashMap<String, LedgerValue>>>,
    pub quorum_size: usize,
    pub replicas: Vec<String>,
    pub committed_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct LedgerValue {
    pub key: String,
    pub value: String,
    pub version: u64,
    pub timestamp_ns: u128,
    pub proposer: String,
    pub signatures: Vec<String>,
    pub status: CommitStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommitStatus {
    Proposed,
    PrePrepared,
    Prepared,
    Committed,
}

#[derive(Debug, Clone)]
pub struct ConsensusProposal {
    pub sequence: u64,
    pub digest: String,
    pub view: u64,
    pub value: LedgerValue,
}

impl Ledger {
    pub fn new(node_id: &str, quorum_size: usize) -> Self {
        Self {
            node_id: node_id.to_string(),
            trust: TrustLayer::new(),
            state: Arc::new(Mutex::new(HashMap::new())),
            quorum_size,
            replicas: Vec::new(),
            committed_sequence: 0,
        }
    }

    pub fn add_replica(&mut self, replica_id: &str) {
        self.replicas.push(replica_id.to_string());
    }

    pub fn propose(&mut self, key: &str, value: &str) -> ConsensusProposal {
        let seq = self.committed_sequence + 1;
        let timestamp = Self::now_ns();
        let digest = Self::digest(key, value, seq, timestamp);

        let ledger_value = LedgerValue {
            key: key.to_string(),
            value: value.to_string(),
            version: seq,
            timestamp_ns: timestamp,
            proposer: self.node_id.clone(),
            signatures: vec![self.node_id.clone()],
            status: CommitStatus::Proposed,
        };

        self.trust
            .append(&self.node_id, "propose", &format!("{}={}", key, digest));

        ConsensusProposal {
            sequence: seq,
            digest,
            view: 1,
            value: ledger_value,
        }
    }

    pub fn pre_prepare(&mut self, proposal: &ConsensusProposal) -> bool {
        proposal.value.signatures.len() >= 1
    }

    pub fn prepare(&mut self, proposal: &mut ConsensusProposal, replica_sigs: Vec<String>) -> bool {
        proposal.value.signatures.extend(replica_sigs);
        let sig_count = proposal.value.signatures.len();
        self.trust.append(
            &self.node_id,
            "prepare",
            &format!("seq={} sigs={}", proposal.sequence, sig_count),
        );
        sig_count >= self.quorum_size
    }

    pub fn commit(&mut self, proposal: ConsensusProposal) -> bool {
        let mut state = self.state.lock().unwrap();
        let key = proposal.value.key.clone();

        state.insert(
            key.clone(),
            LedgerValue {
                key: key.clone(),
                value: proposal.value.value.clone(),
                version: proposal.sequence,
                timestamp_ns: proposal.value.timestamp_ns,
                proposer: proposal.value.proposer,
                signatures: proposal.value.signatures,
                status: CommitStatus::Committed,
            },
        );

        self.committed_sequence = proposal.sequence;
        self.trust.append(
            &self.node_id,
            "commit",
            &format!("{}@{}", key, proposal.sequence),
        );

        true
    }

    pub fn get(&self, key: &str) -> Option<LedgerValue> {
        let state = self.state.lock().unwrap();
        state.get(key).cloned()
    }

    pub fn verify_consensus(&self, proposal: &ConsensusProposal) -> bool {
        let unique_sigs: std::collections::HashSet<_> = proposal.value.signatures.iter().collect();
        unique_sigs.len() >= self.quorum_size
    }

    pub fn audit_trail(&self) -> Vec<AuditEntry> {
        self.trust.replay(0)
    }

    fn digest(key: &str, value: &str, seq: u64, timestamp: u128) -> String {
        use sha2::{Digest, Sha256};
        let data = format!("{}:{}:{}:{}", key, value, seq, timestamp);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn now_ns() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }
}
