//! SET-3: Energy Monitor — Joules-Per-Logical-Operation (JLO)
//! Cross-platform energy telemetry with hardware-in-the-loop support

use std::time::{Duration, Instant};

/// Energy measurement for a single logical operation
#[derive(Debug, Clone)]
pub struct EnergySample {
    pub operation_label: String,
    pub joules: f64,
    pub duration: Duration,
    pub timestamp: Instant,
    pub core_id: usize,
}

/// JLO report for a batch of operations
#[derive(Debug, Clone)]
pub struct JloReport {
    pub total_joules: f64,
    pub total_operations: u64,
    pub joules_per_op: f64,
    pub avg_latency_micros: f64,
    pub thermal_efficiency: f64, // ops/joule
}

/// Cross-platform energy monitor trait
pub trait EnergyMonitor {
    fn sample(&mut self, label: &str) -> EnergySample;
    fn report(&self) -> JloReport;
    fn reset(&mut self);
}

/// Linux RAPL (Running Average Power Limit) monitor
/// Reads from /sys/class/powercap/intel-rapl/
#[derive(Debug)]
pub struct RaplMonitor {
    domain: String,
    baseline_joules: f64,
    samples: Vec<EnergySample>,
    start: Instant,
}

impl RaplMonitor {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            baseline_joules: 0.0,
            samples: Vec::new(),
            start: Instant::now(),
        }
    }

    /// Return the RAPL domain identifier
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Read RAPL energy counter (Linux-specific, stub for other platforms)
    fn read_rapl_counter(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string(format!(
                "/sys/class/powercap/intel-rapl/{}/energy_uj",
                self.domain
            ))
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|uj| uj / 1_000_000.0) // microjoules → joules
            .unwrap_or(0.0)
        }
        #[cfg(not(target_os = "linux"))]
        {
            0.0 // Stub: hardware telemetry unavailable
        }
    }
}

impl EnergyMonitor for RaplMonitor {
    fn sample(&mut self, label: &str) -> EnergySample {
        let current = self.read_rapl_counter();
        let delta = current - self.baseline_joules;
        self.baseline_joules = current;

        let sample = EnergySample {
            operation_label: label.to_string(),
            joules: delta,
            duration: self.start.elapsed(),
            timestamp: Instant::now(),
            core_id: 0,
        };

        self.samples.push(sample.clone());
        sample
    }

    fn report(&self) -> JloReport {
        let total_joules: f64 = self.samples.iter().map(|s| s.joules).sum();
        let total_ops = self.samples.len() as u64;
        let total_duration_micros: f64 = self.samples
            .iter()
            .map(|s| s.duration.as_micros() as f64)
            .sum();

        JloReport {
            total_joules,
            total_operations: total_ops,
            joules_per_op: if total_ops > 0 {
                total_joules / total_ops as f64
            } else {
                0.0
            },
            avg_latency_micros: if total_ops > 0 {
                total_duration_micros / total_ops as f64
            } else {
                0.0
            },
            thermal_efficiency: if total_joules > 0.0 {
                total_ops as f64 / total_joules
            } else {
                0.0
            },
        }
    }

    fn reset(&mut self) {
        self.samples.clear();
        self.baseline_joules = self.read_rapl_counter();
        self.start = Instant::now();
    }
}

/// Software-based energy estimator (fallback when RAPL unavailable)
#[derive(Debug)]
pub struct SoftwareEnergyMonitor {
    samples: Vec<EnergySample>,
    start: Instant,
    estimated_joules_per_op: f64,
}

impl SoftwareEnergyMonitor {
    pub fn new(estimated_joules_per_op: f64) -> Self {
        Self {
            samples: Vec::new(),
            start: Instant::now(),
            estimated_joules_per_op,
        }
    }
}

impl EnergyMonitor for SoftwareEnergyMonitor {
    fn sample(&mut self, label: &str) -> EnergySample {
        let sample = EnergySample {
            operation_label: label.to_string(),
            joules: self.estimated_joules_per_op,
            duration: self.start.elapsed(),
            timestamp: Instant::now(),
            core_id: 0,
        };
        self.samples.push(sample.clone());
        sample
    }

    fn report(&self) -> JloReport {
        let total_joules: f64 = self.samples.iter().map(|s| s.joules).sum();
        let total_ops = self.samples.len() as u64;

        JloReport {
            total_joules,
            total_operations: total_ops,
            joules_per_op: self.estimated_joules_per_op,
            avg_latency_micros: 0.0,
            thermal_efficiency: if total_joules > 0.0 {
                total_ops as f64 / total_joules
            } else {
                0.0
            },
        }
    }

    fn reset(&mut self) {
        self.samples.clear();
        self.start = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rapl_monitor_domain_active() {
        let monitor = RaplMonitor::new("intel-rapl:0");
        assert_eq!(monitor.domain(), "intel-rapl:0");
    }
}