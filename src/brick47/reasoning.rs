//! BRICK-47 Pillar 5: Multi-Layer Reasoning Engine
//! Cross-layer correlation across infra, traces, logs, decisions, resources
//! Benchmark: >= 80% cross-layer correlation accuracy

use crate::brick47::types::{MetricSample, SystemLayer};
use std::collections::HashMap;

/// CorrelationResult: Cross-layer anomaly correlation
#[derive(Clone, Debug)]
pub struct CorrelationResult {
    pub anomaly_id: String,
    pub correlated_layers: Vec<(SystemLayer, f64)>,
    pub composite_score: f64,
    pub explanation: String,
}

/// MultiLayerReasoningEngine: Cross-layer intelligence synthesis
pub struct MultiLayerReasoningEngine {
    layer_metrics: HashMap<SystemLayer, Vec<MetricSample>>,
    correlation_threshold: f64,
    correlation_count: u64,
    accurate_correlations: u64,
}

impl MultiLayerReasoningEngine {
    pub fn new(threshold: f64) -> Self {
        Self {
            layer_metrics: HashMap::new(),
            correlation_threshold: threshold.clamp(0.0, 1.0),
            correlation_count: 0,
            accurate_correlations: 0,
        }
    }

    pub fn ingest_metric(&mut self, sample: MetricSample) {
        let layer = sample.layer.clone();
        self.layer_metrics
            .entry(layer.clone())
            .or_insert_with(Vec::new)
            .push(sample);

        if let Some(samples) = self.layer_metrics.get_mut(&layer) {
            if samples.len() > 1000 {
                samples.remove(0);
            }
        }
    }

    pub fn correlate(
        &mut self,
        anomaly_layer: &SystemLayer,
        anomaly_metric: &str,
        window_secs: f64,
    ) -> CorrelationResult {
        self.correlation_count += 1;

        let now = std::time::Instant::now();
        let mut correlations = Vec::new();

        for (layer, samples) in &self.layer_metrics {
            if layer == anomaly_layer {
                continue;
            }

            let window_samples: Vec<_> = samples
                .iter()
                .filter(|s| now.duration_since(s.timestamp).as_secs_f64() <= window_secs)
                .collect();

            if window_samples.len() >= 2 {
                let values: Vec<f64> = window_samples.iter().map(|s| s.value).collect();
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance =
                    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                let std_dev = variance.sqrt();

                let latest = window_samples.last().unwrap().value;
                let z_score = if std_dev > 0.0 {
                    (latest - mean).abs() / std_dev
                } else {
                    0.0
                };

                if z_score > 2.0 {
                    let confidence = (z_score / 5.0).min(1.0);
                    correlations.push((layer.clone(), confidence));
                }
            }
        }

        correlations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let composite = if correlations.is_empty() {
            0.0
        } else {
            let sum: f64 = correlations.iter().map(|(_, c)| c).sum();
            (sum / correlations.len() as f64).min(1.0)
        };

        let explanation = if correlations.len() >= 2 {
            format!(
                "Cross-layer anomaly: {} correlated with {} layers (confidence={:.3})",
                anomaly_metric,
                correlations.len(),
                composite
            )
        } else {
            format!(
                "Isolated anomaly: {} limited cross-layer correlation",
                anomaly_metric
            )
        };

        if composite >= self.correlation_threshold {
            self.accurate_correlations += 1;
        }

        CorrelationResult {
            anomaly_id: format!("corr_{}_{}", anomaly_layer.as_str(), anomaly_metric),
            correlated_layers: correlations,
            composite_score: composite,
            explanation,
        }
    }

    pub fn accuracy(&self) -> f64 {
        if self.correlation_count == 0 {
            return 0.0;
        }
        self.accurate_correlations as f64 / self.correlation_count as f64
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.correlation_count, self.accurate_correlations)
    }
}
