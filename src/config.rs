use crate::types::{NodeConfig, PeerInfo, RaftConfig};

pub struct SovereignConfig {
    pub node: NodeConfig,
    pub peers: Vec<PeerInfo>,
    pub raft: RaftConfig,
}

impl SovereignConfig {
    pub fn default_for_node(node_id: u64) -> Self {
        Self {
            node: NodeConfig {
                node_id,
                http_port: (8000 + node_id) as u16,
                raft_port: (9000 + node_id) as u16,
                federation_port: (8080 + node_id) as u16,
                data_dir: format!("./data{}", node_id),
                region: "Sovereign_Alpha_1".to_string(),
                jurisdiction: "Global_Root".to_string(),
            },
            peers: vec![
                PeerInfo {
                    id: 1,
                    host: "127.0.0.1".to_string(),
                    http_port: 8001,
                    raft_port: 9001,
                },
                PeerInfo {
                    id: 2,
                    host: "127.0.0.1".to_string(),
                    http_port: 8002,
                    raft_port: 9002,
                },
                PeerInfo {
                    id: 3,
                    host: "127.0.0.1".to_string(),
                    http_port: 8003,
                    raft_port: 9003,
                },
            ],
            raft: RaftConfig::default(),
        }
    }

    pub fn get_peer_endpoints(&self) -> Vec<String> {
        self.peers
            .iter()
            .filter(|p| p.id != self.node.node_id)
            .map(|p| format!("http://{}:{}", p.host, p.raft_port))
            .collect()
    }
}
