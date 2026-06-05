use crate::cluster_identity::ClusterIdentity;
use crate::types::ClusterId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The atomic unit of operational memory.
/// Every entry is causally linked to its predecessor via parent_hash.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationalEntry {
    pub sequence: u64,         // Global monotonic sequence
    pub epoch_id: u64,         // Epoch boundary marker
    pub term: u64,             // Raft global term at time of entry
    pub timestamp: u64,        // UNIX epoch nanoseconds
    pub payload: OpPayload,    // What happened
    pub parent_hash: [u8; 32], // SHA-256 of serialized previous entry
    pub raft_index: u64,       // Cross-link to BRICK-18 log position
    pub entry_hash: [u8; 32],  // SHA-256 of this entry's content
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OpPayload {
    EpochSealed {
        term: u64,
        leader_id: u64,
        next_epoch: u64,
    },
    ClusterJoined {
        cluster_id: ClusterId,
        identity: ClusterIdentity,
    },
    ClusterLeft {
        cluster_id: ClusterId,
        reason: String,
        timestamp: u64,
    },
    ShardMigrated {
        shard_id: String,
        from_cluster: ClusterId,
        to_cluster: ClusterId,
        tx_id: String,
    },
    GovernanceVote {
        proposal_id: String,
        voter: ClusterId,
        decision: bool,
        weight: u64,
    },
    PolicyEnacted {
        policy_id: String,
        hash: [u8; 32],
        enactment_epoch: u64,
    },
    TopologyDelta {
        added: Vec<ClusterId>,
        removed: Vec<ClusterId>,
        epoch: u64,
    },
}

/// A verified checkpoint of operational state at a given epoch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub epoch_id: u64,
    pub sequence: u64, // Last included sequence number
    pub timestamp: u64,
    pub topology_checksum: [u8; 32], // Merkle root of GlobalRegistry state
    pub governance_checksum: [u8; 32], // Merkle root of GovernanceChain state
    pub data: Vec<u8>,               // Serialized snapshot payload
    pub signature: Option<Vec<u8>>,  // PQC-signed by cluster leader (BRICK-29)
}

/// Compute SHA-256 hash of serializable data.
pub fn compute_hash<T: Serialize>(data: &T) -> [u8; 32] {
    let json = serde_json::to_string(data).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// Verify causal chain integrity between two entries.
pub fn verify_causal_link(parent: &OperationalEntry, child: &OperationalEntry) -> bool {
    let expected_parent_hash = compute_hash(parent);
    child.parent_hash == expected_parent_hash
}
