//! BRICK-46 Phase 1: Homeostasis Loop
//! Autonomic self-regulation — continuous SLO scoring with bounded history

use crate::brick46::types::{HealthSnapshot, SloMetric};
use std::time::Instant;

/// Configuration for homeostatic regulation
#[derive(Clone, Debug)]
pub struct HomeostasisConfig {
    pub slo_targets: Vec<SloMetric>,
    pub max_history: usize,
    pub anomaly_threshold: f64,
    pub energy_budget_max: f64,
}

impl HomeostasisConfig {
    pub fn production_default() -> Self {
        Self {
            slo_targets: vec![
                SloMetric::new("availability", 0.9999, 0.4),
                SloMetric::new("latency_p99", 100.0, 0.3),
                SloMetric::new("error_rate", 0.001, 0.2),
                SloMetric::new("throughput", 10000.0, 0.1),
            ],
            max_history: 10000,
            anomaly_threshold: 0.85,
            energy_budget_max: 1.0,
        }
    }
}

/// HomeostasisLoop: The autonomic nervous system of Harmonis Prime
#[derive(Debug)]
pub struct HomeostasisLoop {
    config: HomeostasisConfig,
    history: Vec<HealthSnapshot>,
    observation_count: u64,
}

impl HomeostasisLoop {
    pub fn new(config: HomeostasisConfig) -> Self {
        Self {
            config,
            history: Vec::with_capacity(1000),
            observation_count: 0,
        }
    }

    pub fn record_observation(
        &mut self,
        error_rate: f64,
        latency_p99_ms: f64,
        energy_budget_used: f64,
        throughput: f64,
    ) -> HealthSnapshot {
        let slo_score =
            self.compute_slo_score(error_rate, latency_p99_ms, energy_budget_used, throughput);
        let mut violations = Vec::new();
        if error_rate > 0.01 {
            violations.push(format!("error_rate_exceeded: {:.5}", error_rate));
        }
        if latency_p99_ms > 1000.0 {
            violations.push(format!("latency_p99_exceeded: {:.2}ms", latency_p99_ms));
        }
        if energy_budget_used > self.config.energy_budget_max {
            violations.push(format!("energy_budget_exceeded: {:.2}", energy_budget_used));
        }
        let snapshot = HealthSnapshot {
            timestamp: Instant::now(),
            slo_score,
            error_rate,
            latency_p99_ms,
            energy_budget_used,
            violations,
        };
        self.history.push(snapshot.clone());
        self.observation_count += 1;
        if self.history.len() > self.config.max_history {
            let overflow = self.history.len() - self.config.max_history;
            self.history.drain(0..overflow);
        }
        snapshot
    }

    fn compute_slo_score(
        &self,
        error_rate: f64,
        latency_p99_ms: f64,
        energy_budget_used: f64,
        throughput: f64,
    ) -> f64 {
        let mut score = 1.0;
        if error_rate > 0.0 {
            score -= (error_rate * 20.0).min(0.5);
        }
        if latency_p99_ms > 0.0 {
            score -= (latency_p99_ms / 2000.0).min(0.3);
        }
        if energy_budget_used > self.config.energy_budget_max {
            score -= ((energy_budget_used - self.config.energy_budget_max) * 0.5).min(0.2);
        }
        if throughput > 5000.0 {
            score += 0.05;
        }
        if throughput > 10000.0 {
            score += 0.05;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn latest(&self) -> Option<&HealthSnapshot> {
        self.history.last()
    }

    pub fn history(&self) -> &[HealthSnapshot] {
        &self.history
    }

    pub fn rolling_average(&self, n: usize) -> Option<f64> {
        let take = n.min(self.history.len());
        if take == 0 {
            return None;
        }
        let sum: f64 = self
            .history
            .iter()
            .rev()
            .take(take)
            .map(|h| h.slo_score)
            .sum();
        Some(sum / take as f64)
    }

    pub fn trend(&self, window: usize) -> &'static str {
        if self.history.len() < window * 2 {
            return "insufficient_data";
        }
        let recent: f64 = self
            .history
            .iter()
            .rev()
            .take(window)
            .map(|h| h.slo_score)
            .sum();
        let previous: f64 = self
            .history
            .iter()
            .rev()
            .skip(window)
            .take(window)
            .map(|h| h.slo_score)
            .sum();
        let recent_avg = recent / window as f64;
        let prev_avg = previous / window as f64;
        if recent_avg > prev_avg + 0.05 {
            "improving"
        } else if recent_avg < prev_avg - 0.05 {
            "degrading"
        } else {
            "stable"
        }
    }

    pub fn observation_count(&self) -> u64 {
        self.observation_count
    }
}
