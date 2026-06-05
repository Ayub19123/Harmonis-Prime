//! BRICK-47 Pillar 6: Drift Detection System
//! Detect logic/metric drift; trigger recalibration
//! Benchmark: Metric drift >= 90%, policy drift >= 85%

use crate::brick47::types::MetricSample;
use std::collections::VecDeque;
use std::time::Instant;

/// DriftType: Classification of detected drift
#[derive(Clone, Debug, PartialEq)]
pub enum DriftType {
    MetricShift,
    PolicyDegradation,
    ModelDecay,
    DistributionDrift,
}

/// DriftEvent: Recorded drift occurrence
#[derive(Clone, Debug)]
pub struct DriftEvent {
    pub drift_type: DriftType,
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub deviation: f64,
    pub confidence: f64,
    pub timestamp: Instant,
}

/// DriftDetectionSystem: Self-monitoring for metric and policy drift
pub struct DriftDetectionSystem {
    baselines: VecDeque<(String, f64, Instant)>,
    metric_history: VecDeque<MetricSample>,
    max_history: usize,
    metric_drift_count: u64,
    policy_drift_count: u64,
    metric_detected: u64,
    policy_detected: u64,
    metric_threshold: f64,
    policy_threshold: f64,
}

impl DriftDetectionSystem {
    pub fn new(max_history: usize) -> Self {
        Self {
            baselines: VecDeque::with_capacity(max_history),
            metric_history: VecDeque::with_capacity(max_history),
            max_history,
            metric_drift_count: 0,
            policy_drift_count: 0,
            metric_detected: 0,
            policy_detected: 0,
            metric_threshold: 0.15, // 15% deviation triggers metric drift
            policy_threshold: 0.20, // 20% deviation triggers policy drift
        }
    }

    pub fn set_thresholds(&mut self, metric: f64, policy: f64) {
        self.metric_threshold = metric.clamp(0.01, 0.5);
        self.policy_threshold = policy.clamp(0.01, 0.5);
    }

    /// Establish baseline for a metric
    pub fn establish_baseline(&mut self, name: &str, value: f64) {
        self.baselines
            .push_back((name.to_string(), value, Instant::now()));
        if self.baselines.len() > self.max_history {
            self.baselines.pop_front();
        }
    }

    /// Record metric sample and check for drift
    pub fn observe(&mut self, sample: MetricSample) -> Option<DriftEvent> {
        self.metric_history.push_back(sample.clone());
        if self.metric_history.len() > self.max_history {
            self.metric_history.pop_front();
        }

        // Find baseline
        let baseline = self.baselines.iter().find(|(n, _, _)| n == &sample.name);
        if baseline.is_none() {
            return None;
        }

        let (_, base_val, _) = baseline.unwrap();
        let deviation = (sample.value - base_val).abs() / base_val.abs().max(1e-10);

        let (threshold, drift_type) = if sample.name.starts_with("policy_") {
            (self.policy_threshold, DriftType::PolicyDegradation)
        } else {
            (self.metric_threshold, DriftType::MetricShift)
        };

        if deviation > threshold {
            let confidence = (deviation / (threshold * 2.0)).min(1.0);

            if drift_type == DriftType::MetricShift {
                self.metric_drift_count += 1;
                self.metric_detected += 1;
            } else {
                self.policy_drift_count += 1;
                self.policy_detected += 1;
            }

            Some(DriftEvent {
                drift_type,
                metric_name: sample.name.clone(),
                baseline_value: *base_val,
                current_value: sample.value,
                deviation,
                confidence,
                timestamp: Instant::now(),
            })
        } else {
            None
        }
    }

    /// Periodic recalibration: update baselines from recent stable window
    pub fn recalibrate(&mut self, metric_name: &str, window_samples: usize) {
        let samples: Vec<_> = self
            .metric_history
            .iter()
            .filter(|s| s.name == metric_name)
            .rev()
            .take(window_samples)
            .collect();

        if samples.len() >= 3 {
            let mean = samples.iter().map(|s| s.value).sum::<f64>() / samples.len() as f64;
            self.establish_baseline(metric_name, mean);
        }
    }

    pub fn metric_drift_accuracy(&self) -> f64 {
        if self.metric_drift_count == 0 {
            return 1.0;
        }
        self.metric_detected as f64 / self.metric_drift_count as f64
    }

    pub fn policy_drift_accuracy(&self) -> f64 {
        if self.policy_drift_count == 0 {
            return 1.0;
        }
        self.policy_detected as f64 / self.policy_drift_count as f64
    }

    pub fn stats(&self) -> (u64, u64, u64, u64) {
        (
            self.metric_drift_count,
            self.policy_drift_count,
            self.metric_detected,
            self.policy_detected,
        )
    }
}
