use std::collections::HashMap;

pub struct CausalMemory {
    pub edge_weights: HashMap<(String, String), f64>,
    pub prior_distributions: HashMap<String, f64>,
    pub model_version: String,
    pub state_hash: String,
}

impl CausalMemory {
    pub fn new(version: String, hash: String) -> Self {
        Self {
            edge_weights: HashMap::new(),
            prior_distributions: HashMap::new(),
            model_version: version,
            state_hash: hash,
        }
    }

    pub fn seal_epoch(&self) -> String {
        self.state_hash.clone()
    }

    pub fn update_edge_weight(&mut self, source: String, target: String, weight: f64) {
        self.edge_weights
            .insert((source, target), weight.clamp(0.0, 1.0));
    }

    pub fn get_edge_weight(&self, source: &str, target: &str) -> f64 {
        *self
            .edge_weights
            .get(&(source.to_string(), target.to_string()))
            .unwrap_or(&0.5)
    }

    pub fn update_prior(&mut self, node: String, prior: f64) {
        self.prior_distributions.insert(node, prior.clamp(0.0, 1.0));
    }

    pub fn get_prior(&self, node: &str) -> f64 {
        *self.prior_distributions.get(node).unwrap_or(&0.5)
    }
}
