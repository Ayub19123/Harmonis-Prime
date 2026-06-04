use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// StateHasher: Cryptographic fingerprinting of engine state
/// H(state) = SHA-256(serialize(state)) — deterministic, collision-resistant
pub struct StateHasher;

/// EngineSnapshot: Serializable representation of engine state for hashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSnapshot {
    pub term: u64,
    pub commit_index: u64,
    pub node_id: u64,
    pub kv_store_hash: String,
    pub substrate_flags: Vec<(String, bool)>,
    pub timestamp_nanos: u64,
}

/// hash_engine_state: H(s) → 256-bit digest
/// Deterministic: same state always produces same hash
pub fn hash_engine_state(snapshot: &EngineSnapshot) -> String {
    let serialized = serde_json::to_string(snapshot).unwrap_or_else(|_| String::new());

    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let result = hasher.finalize();

    format!("{:x}", result)
}

impl StateHasher {
    /// Create new hasher instance
    pub fn new() -> Self {
        Self
    }

    /// Hash a byte slice directly
    pub fn hash_bytes(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Hash two digests together — for chaining
    pub fn hash_concat(&self, a: &str, b: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(a.as_bytes());
        hasher.update(b.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Verify integrity: H(expected) == H(actual)
    pub fn verify(&self, expected: &str, actual: &str) -> bool {
        expected == actual
    }
}

/// Create snapshot from engine state components
pub fn create_snapshot(
    term: u64,
    commit_index: u64,
    node_id: u64,
    kv_store: &std::collections::HashMap<String, String>,
    substrates: &[(String, bool)],
) -> EngineSnapshot {
    // Hash the KV store separately for efficiency
    let kv_json = serde_json::to_string(kv_store).unwrap_or_default();
    let mut kv_hasher = Sha256::new();
    kv_hasher.update(kv_json.as_bytes());
    let kv_hash = format!("{:x}", kv_hasher.finalize());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    EngineSnapshot {
        term,
        commit_index,
        node_id,
        kv_store_hash: kv_hash,
        substrate_flags: substrates.to_vec(),
        timestamp_nanos: now,
    }
}
