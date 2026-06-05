//! BRICK-46 Phase 4: Cognitive Node
//! Cerebral cortex — anomaly prediction, multi-variable synthesis

use crate::brick46::types::{CognitiveSignal, HealthSnapshot};

#[derive(Clone, Debug)]
pub enum CognitiveModel {
    RuleBased,
    Statistical,
    QuantumHybrid,
}

#[derive(Clone, Debug)]
pub struct CognitiveConfig {
    pub model: CognitiveModel,
    pub anomaly_threshold: f64,
    pub correlation_window: usize,
    pub max_signals: usize,
}

impl CognitiveConfig {
    pub fn production_default() -> Self {
        Self {
            model: CognitiveModel::QuantumHybrid,
            anomaly_threshold: 0.75,
            correlation_window: 100,
            max_signals: 1000,
        }
    }
}

pub struct CognitiveNode {
    config: CognitiveConfig,
    signal_history: Vec<CognitiveSignal>,
    inference_count: u64,
}

impl CognitiveNode {
    pub fn new(config: CognitiveConfig) -> Self {
        Self {
            config,
            signal_history: Vec::with_capacity(1000),
            inference_count: 0,
        }
    }

    pub fn analyze(
        &mut self,
        health: &HealthSnapshot,
        incoming: &[CognitiveSignal],
    ) -> Option<CognitiveSignal> {
        let bounded_incoming = if incoming.len() > self.config.max_signals {
            &incoming[..self.config.max_signals]
        } else {
            incoming
        };
        let base_anomaly = 1.0 - health.slo_score;
        let signal_contribution: f64 = bounded_incoming
            .iter()
            .map(|s| s.anomaly_score)
            .sum::<f64>()
            * 0.05;
        let mut anomaly_score = (base_anomaly + signal_contribution).clamp(0.0, 1.0);
        match self.config.model {
            CognitiveModel::RuleBased => {}
            CognitiveModel::Statistical => {
                let variance = self.compute_variance();
                anomaly_score += variance * 0.1;
            }
            CognitiveModel::QuantumHybrid => {
                anomaly_score = (anomaly_score * 1.2).min(1.0);
            }
        }
        for signal in bounded_incoming {
            self.signal_history.push(signal.clone());
        }
        if self.signal_history.len() > self.config.correlation_window {
            let overflow = self.signal_history.len() - self.config.correlation_window;
            self.signal_history.drain(0..overflow);
        }
        self.inference_count += 1;
        if anomaly_score < self.config.anomaly_threshold {
            return None;
        }
        let action = self.synthesize_action(health, anomaly_score, bounded_incoming);
        Some(CognitiveSignal {
            correlation_id: format!("cognitive::synthesis::{}", self.inference_count),
            summary: format!("Anomaly detected: slo_score={:.3}, error_rate={:.5}, latency_p99={:.2}ms, anomaly={:.3}",
                health.slo_score, health.error_rate, health.latency_p99_ms, anomaly_score),
            anomaly_score,
            recommended_action: Some(action),
            timestamp: std::time::Instant::now(),
        })
    }

    fn synthesize_action(
        &self,
        health: &HealthSnapshot,
        anomaly_score: f64,
        signals: &[CognitiveSignal],
    ) -> String {
        if health.error_rate > 0.05 {
            return "emergency_circuit_break".to_string();
        }
        if health.latency_p99_ms > 5000.0 {
            return "scale_compute_pool".to_string();
        }
        if anomaly_score > 0.95 {
            return "full_failover".to_string();
        }
        if !signals.is_empty() && signals.iter().any(|s| s.anomaly_score > 0.9) {
            return "increase_redundancy".to_string();
        }
        if health.energy_budget_used > 0.9 {
            return "throttle_non_critical".to_string();
        }
        "investigate_anomaly".to_string()
    }

    fn compute_variance(&self) -> f64 {
        if self.signal_history.len() < 2 {
            return 0.0;
        }
        let scores: Vec<f64> = self
            .signal_history
            .iter()
            .map(|s| s.anomaly_score)
            .collect();
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64;
        variance.sqrt()
    }

    pub fn stats(&self) -> u64 {
        self.inference_count
    }
}
