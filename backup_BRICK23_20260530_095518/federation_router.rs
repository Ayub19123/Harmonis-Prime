use std::collections::HashMap;

pub struct FederationRouter {
    local_shard_id: String,
    registry: HashMap<String, ShardRoute>,
}

#[derive(Clone)]
struct ShardRoute {
    endpoint: String,
    is_local: bool,
}

impl FederationRouter {
    pub fn new(node_id: u64) -> Self {
        let local_shard_id = format!("node-{}", node_id);
        FederationRouter {
            local_shard_id,
            registry: HashMap::new(),
        }
    }

    pub fn register(&mut self, shard_id: String, endpoint: String, is_local: bool) {
        let route = ShardRoute { endpoint, is_local };
        self.registry.insert(shard_id, route);
    }

    pub fn resolve(&self, shard_id: &str, local_node_id: u64) -> (String, bool) {
        if let Some(route) = self.registry.get(shard_id) {
            return (route.endpoint.clone(), route.is_local);
        }
        let endpoint = format!("http://127.0.0.1:{}", 8000 + local_node_id);
        (endpoint, false)
    }
}
