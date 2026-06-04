use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterIdentity {
    pub cluster_id: String,
    pub region: String,
    pub jurisdiction: String,
    pub public_key: String,
    pub raft_endpoint: String,
    pub http_endpoint: String,
    pub federation_endpoint: String,
    pub created_at: u64,
    pub version: String,
}

impl ClusterIdentity {
    pub fn generate(
        region: String,
        jurisdiction: String,
        public_key: String,
        host: String,
        raft_port: u16,
        http_port: u16,
        federation_port: u16,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            cluster_id: format!("cluster-{}-{}", region, created_at),
            region,
            jurisdiction,
            public_key,
            raft_endpoint: format!("http://{}:{}", host, raft_port),
            http_endpoint: format!("http://{}:{}", host, http_port),
            federation_endpoint: format!("http://{}:{}", host, federation_port),
            created_at,
            version: "v6.3.0-BRICK23".to_string(),
        }
    }
}