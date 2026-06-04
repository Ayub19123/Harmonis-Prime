use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeConfig {
    pub node_id: u64,
    pub http_port: u16,
    pub raft_port: u16,
    pub federation_port: u16,
    pub data_dir: String,
    pub region: String,
    pub jurisdiction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: u64,
    pub host: String,
    pub http_port: u16,
    pub raft_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftConfig {
    pub election_timeout_min_ms: u64,
    pub election_timeout_max_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_entries_per_append: usize,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            election_timeout_min_ms: 300,
            election_timeout_max_ms: 600,
            heartbeat_interval_ms: 80,
            max_entries_per_append: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: u64,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
    pub conflict_index: u64,
    pub conflict_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteRequest {
    pub term: u64,
    pub candidate_id: u64,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub index: u64,
    pub term: u64,
    pub command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Delete { key: String },
    Noop,
}


// =============================================================================

// =============================================================================

// =============================================================================
// BRICK-23: CROSS-CLUSTER CONSENSUS TYPES
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClusterId(pub String);

impl ClusterId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalOperation {
    pub op: String,
    pub key: String,
    pub value: Option<String>,
    pub target_cluster: ClusterId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FederationPayload {
    JoinRequest {
        cluster_nodes: Vec<String>,
        shard_range: (String, String),
    },
    Heartbeat {
        leader_id: u64,
        commit_index: u64,
        timestamp: u64,
    },
    GlobalProposal {
        transaction_id: String,
        operations: Vec<GlobalOperation>,
    },
    GlobalCommitAck {
        transaction_id: String,
        cluster_commit_index: u64,
    },
    ShardMigration {
        shard_id: String,
        from_cluster: ClusterId,
        to_cluster: ClusterId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationMessage {
    pub from_cluster: ClusterId,
    pub to_cluster: ClusterId,
    pub term: u64,
    pub payload: FederationPayload,
}