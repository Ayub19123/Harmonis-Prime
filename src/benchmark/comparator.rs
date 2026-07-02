
//! M2.7.14 Layer 2: BaselineComparator — Tag-to-tag performance comparison

use std::collections::HashMap;

/// M2.7.14: Par-2 (Penalized Average Runtime) score
/// timeout_penalty is typically 2x timeout (e.g., 5000s for 2500s timeout)
pub fn par2_score(runtimes_ms: &[u64], timeout_ms: u64) -> f64 {
    let mut total = 0.0;
    for &rt in runtimes_ms {
        if rt >= timeout_ms {
            total += 2.0 * (timeout_ms as f64 / 1000.0); // Penalty in seconds
        } else {
            total += rt as f64 / 1000.0; // Actual runtime in seconds
        }
    }
    total / runtimes_ms.len() as f64
}

/// M2.7.14: Performance delta between two benchmark runs
#[derive(Debug, Clone)]
pub struct PerformanceDelta {
    pub instance: String,
    pub baseline_time_ms: u64,
    pub current_time_ms: u64,
    pub delta_pct: f64, // Positive = slower, Negative = faster
}

/// M2.7.14: BaselineComparator — Cross-reference current vs historical
pub struct BaselineComparator {
    pub epsilon_pct: f64, // e.g., 5.0 = 5% tolerance
}

impl BaselineComparator {
    pub fn new(epsilon_pct: f64) -> Self {
        Self { epsilon_pct }
    }

    /// Compare current runtimes against baseline
    pub fn compare(
        &self,
        baseline: &HashMap<String, u64>,
        current: &HashMap<String, u64>,
    ) -> Vec<PerformanceDelta> {
        let mut deltas = Vec::new();
        
        for (instance, &current_time) in current {
            if let Some(&baseline_time) = baseline.get(instance) {
                let delta_pct = if baseline_time == 0 {
                    0.0
                } else {
                    ((current_time as f64 - baseline_time as f64) / baseline_time as f64) * 100.0
                };
                
                deltas.push(PerformanceDelta {
                    instance: instance.clone(),
                    baseline_time_ms: baseline_time,
                    current_time_ms: current_time,
                    delta_pct,
                });
            }
        }
        
        deltas
    }

    /// Flag regressions exceeding epsilon threshold
    pub fn flag_regressions(&self, deltas: &[PerformanceDelta]) -> Vec<PerformanceDelta> {
        deltas.iter()
            .filter(|d| d.delta_pct > self.epsilon_pct)
            .cloned()
            .collect()
    }
}
