//! BRICK-51 Layer 1: Awareness Monitor
//! Collects telemetry for Awareness Index (â‰¥95%)
//! CMF-519: Each node reports 20 metrics; â‰¥95% correctly known by all within 100ms

use std::collections::HashMap;

pub struct AwarenessMonitor {
    metrics: HashMap<String, f64>,
    coverage_count: u64,
    total_checks: u64,
    node_count: usize,
}

impl AwarenessMonitor {
    pub fn new(node_count: usize) -> Self {
        Self {
            metrics: HashMap::new(),
            coverage_count: 0,
            total_checks: 0,
            node_count,
        }
    }

    pub fn report_metric(&mut self, node_id: &str, metric_id: &str, value: f64) {
        let key = format!("{}:{}", node_id, metric_id);
        self.metrics.insert(key, value);
    }

    pub fn check_coverage(&mut self) -> f64 {
        let expected = self.node_count * 20; // 20 metrics per node
        let actual = self.metrics.len();
        self.total_checks += 1;
        if actual >= expected {
            self.coverage_count += 1;
        }
        actual as f64 / expected.max(1) as f64
    }

    pub fn awareness_index(&self) -> f64 {
        if self.total_checks == 0 {
            return 1.0;
        }
        self.coverage_count as f64 / self.total_checks as f64
    }

    pub fn stats(&self) -> (usize, u64, f64) {
        (
            self.metrics.len(),
            self.total_checks,
            self.awareness_index(),
        )
    }
}
