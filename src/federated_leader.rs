use crate::types::{ClusterId, FederationMessage, FederationPayload};

pub struct FederatedLeader {
    cluster_id: ClusterId,
    global_term: u64,
}

impl FederatedLeader {
    pub fn new(cluster_id: ClusterId) -> Self {
        Self {
            cluster_id,
            global_term: 1,
        }
    }

    pub fn create_heartbeat(&self, leader_id: u64, commit_index: u64) -> FederationMessage {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        FederationMessage {
            from_cluster: self.cluster_id.clone(),
            to_cluster: ClusterId("global".to_string()),
            term: self.global_term,
            payload: FederationPayload::Heartbeat {
                leader_id,
                commit_index,
                timestamp,
            },
        }
    }

    pub fn create_join_request(
        &self,
        nodes: Vec<String>,
        shard_min: String,
        shard_max: String,
    ) -> FederationMessage {
        FederationMessage {
            from_cluster: self.cluster_id.clone(),
            to_cluster: ClusterId("global".to_string()),
            term: self.global_term,
            payload: FederationPayload::JoinRequest {
                cluster_nodes: nodes,
                shard_range: (shard_min, shard_max),
            },
        }
    }

    pub fn increment_global_term(&mut self) {
        self.global_term += 1;
    }
}
