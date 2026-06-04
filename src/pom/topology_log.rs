use crate::cluster_identity::ClusterIdentity;
use crate::pom::operational_memory::{OpPayload, OperationalEntry};
use crate::types::ClusterId;
use std::collections::HashSet;

pub struct TopologyLog {
    known_clusters: HashSet<ClusterId>,
    sequence: u64,
}

impl TopologyLog {
    pub fn new() -> Self {
        Self {
            known_clusters: HashSet::new(),
            sequence: 0,
        }
    }

    pub fn record_join(
        &mut self,
        cluster_id: ClusterId,
        identity: ClusterIdentity,
        parent_hash: [u8; 32],
    ) -> OperationalEntry {
        self.sequence += 1;
        self.known_clusters.insert(cluster_id.clone());

        OperationalEntry {
            sequence: self.sequence,
            epoch_id: 1, // Set by caller
            term: 1,     // Set by caller
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            payload: OpPayload::ClusterJoined {
                cluster_id,
                identity,
            },
            parent_hash,
            raft_index: 0,
            entry_hash: [0u8; 32], // Computed by orchestrator
        }
    }

    pub fn record_leave(
        &mut self,
        cluster_id: ClusterId,
        reason: String,
        parent_hash: [u8; 32],
    ) -> OperationalEntry {
        self.sequence += 1;
        self.known_clusters.remove(&cluster_id);

        OperationalEntry {
            sequence: self.sequence,
            epoch_id: 1,
            term: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            payload: OpPayload::ClusterLeft {
                cluster_id,
                reason,
                timestamp: 0,
            },
            parent_hash,
            raft_index: 0,
            entry_hash: [0u8; 32],
        }
    }

    pub fn cluster_count(&self) -> usize {
        self.known_clusters.len()
    }

    pub fn is_known(&self, cluster_id: &ClusterId) -> bool {
        self.known_clusters.contains(cluster_id)
    }
}
