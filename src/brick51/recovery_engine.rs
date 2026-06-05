//! BRICK-51 Layer 5: Recovery Engine
//! Autonomous mesh reorganisation after failure
//! CMF-514: Recovery <30 seconds
//! CMF-517: Emergent specialisation ≥20% improvement

use std::collections::HashMap;

pub struct DomainExpertise {
    pub domain: String,
    pub performance: f64,
    pub episodes: u64,
}

pub struct RecoveryEngine {
    nodes_alive: usize,
    nodes_total: usize,
    recovery_time_ms: u64,
    specialisations: HashMap<String, DomainExpertise>,
    recovered_count: u64,
}

impl RecoveryEngine {
    pub fn new(total_nodes: usize) -> Self {
        Self {
            nodes_alive: total_nodes,
            nodes_total: total_nodes,
            recovery_time_ms: 0,
            specialisations: HashMap::new(),
            recovered_count: 0,
        }
    }

    pub fn simulate_failure(&mut self, failed_nodes: usize) {
        self.nodes_alive = self.nodes_alive.saturating_sub(failed_nodes);
    }

    pub fn recover(&mut self, time_ms: u64) {
        self.recovery_time_ms = time_ms;
        self.nodes_alive = self.nodes_total;
        self.recovered_count += 1;
    }

    pub fn train_specialisation(&mut self, domain: &str, performance: f64) {
        let entry = self
            .specialisations
            .entry(domain.to_string())
            .or_insert(DomainExpertise {
                domain: domain.to_string(),
                performance: 0.0,
                episodes: 0,
            });
        entry.episodes += 1;
        entry.performance =
            (entry.performance * (entry.episodes - 1) as f64 + performance) / entry.episodes as f64;
    }

    pub fn specialisation_gain(&self, domain: &str, baseline: f64) -> f64 {
        match self.specialisations.get(domain) {
            Some(exp) if exp.episodes >= 10 => (exp.performance - baseline) / baseline,
            _ => 0.0,
        }
    }

    pub fn recovery_time_ms(&self) -> u64 {
        self.recovery_time_ms
    }

    pub fn availability(&self) -> f64 {
        if self.nodes_total == 0 {
            return 1.0;
        }
        self.nodes_alive as f64 / self.nodes_total as f64
    }

    pub fn stats(&self) -> (usize, usize, u64, f64) {
        (
            self.nodes_alive,
            self.nodes_total,
            self.recovery_time_ms,
            self.availability(),
        )
    }
}
