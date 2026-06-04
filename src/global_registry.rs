use crate::cluster_identity::ClusterIdentity;
use crate::types::ClusterId;
use std::collections::HashMap;

pub struct GlobalRegistry {
    clusters: HashMap<ClusterId, ClusterIdentity>,
}

impl GlobalRegistry {
    pub fn new() -> Self {
        Self {
            clusters: HashMap::new(),
        }
    }

    pub fn register(&mut self, identity: ClusterIdentity) {
        self.clusters
            .insert(ClusterId(identity.cluster_id.clone()), identity);
    }

    pub fn get(&self, id: &ClusterId) -> Option<&ClusterIdentity> {
        self.clusters.get(id)
    }

    pub fn all_clusters(&self) -> Vec<&ClusterIdentity> {
        self.clusters.values().collect()
    }

    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }
}
