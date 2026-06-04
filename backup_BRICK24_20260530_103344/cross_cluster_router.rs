use crate::types::ClusterId;
use crate::global_registry::GlobalRegistry;

pub struct CrossClusterRouter {
    registry: GlobalRegistry,
    local_cluster: ClusterId,
}

impl CrossClusterRouter {
    pub fn new(local_cluster: ClusterId) -> Self {
        Self {
            registry: GlobalRegistry::new(),
            local_cluster,
        }
    }
    
    pub fn register_cluster(&mut self, identity: crate::cluster_identity::ClusterIdentity) {
        self.registry.register(identity);
    }
    
    pub fn route_key(&self, key: &str) -> ClusterId {
        let clusters: Vec<_> = self.registry.all_clusters();
        if clusters.is_empty() {
            return self.local_cluster.clone();
        }
        let hash = key.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let idx = (hash as usize) % clusters.len();
        ClusterId(clusters[idx].cluster_id.clone())
    }
    
    pub fn is_local(&self, cluster_id: &ClusterId) -> bool {
        cluster_id == &self.local_cluster
    }
    
    pub fn local_cluster(&self) -> &ClusterId {
        &self.local_cluster
    }
}