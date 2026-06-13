//! SET-5.2: RAPL Hardware-in-the-Loop Energy Bindings
//! Invariant: Hardware-measured joules correlate with software-estimated JLO
//! Platform: Linux (intel-rapl sysfs interface). Windows uses software fallback.

use std::time::Instant;
use crate::energy::monitor::{EnergyMonitor, EnergySample, JloReport};

/// RAPL domain identifiers for Intel processors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RaplDomain {
    Package,      // Entire CPU package
    Core,         // CPU cores only
    Uncore,       // GPU / uncore logic
    Dram,         // Memory controller
    Psu,          // Platform / PSU level
}

impl RaplDomain {
    pub fn sysfs_path(&self) -> &'static str {
        match self {
            RaplDomain::Package => "intel-rapl:0",
            RaplDomain::Core => "intel-rapl:0:0",
            RaplDomain::Uncore => "intel-rapl:0:1",
            RaplDomain::Dram => "intel-rapl:0:2",
            RaplDomain::Psu => "intel-rapl:0:3",
        }
    }
}

/// Raw RAPL sensor reading
#[derive(Debug, Clone)]
pub struct RaplReading {
    pub domain: RaplDomain,
    pub energy_uj: u64,          // Microjoules since boot
    pub timestamp: Instant,
    pub max_energy_range_uj: u64, // Wraparound threshold
}

/// Hardware-in-the-loop RAPL monitor
#[derive(Debug)]
pub struct RaplHardwareMonitor {
    domain: RaplDomain,
    baseline: Option<RaplReading>,
    samples: Vec<EnergySample>,
    start: Instant,
    read_errors: u64,
}

impl RaplHardwareMonitor {
    pub fn new(domain: RaplDomain) -> Self {
        Self {
            domain,
            baseline: None,
            samples: Vec::new(),
            start: Instant::now(),
            read_errors: 0,
        }
    }

    /// Read RAPL energy counter from sysfs (Linux only)
    #[cfg(target_os = "linux")]
    fn read_raw(&self) -> Option<RaplReading> {
        let base_path = format!("/sys/class/powercap/{}/", self.domain.sysfs_path());
        
        let energy_uj = std::fs::read_to_string(format!("{}energy_uj", base_path))
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()?;
        
        let max_range = std::fs::read_to_string(format!("{}max_energy_range_uj", base_path))
            .ok()?
            .trim()
            .parse::<u64>()
            .unwrap_or(u64::MAX);

        Some(RaplReading {
            domain: self.domain,
            energy_uj,
            timestamp: Instant::now(),
            max_energy_range_uj: max_range,
        })
    }

    #[cfg(not(target_os = "linux"))]
    fn read_raw(&self) -> Option<RaplReading> {
        None // Hardware telemetry unavailable on non-Linux platforms
    }

    /// Detect wraparound (counter overflow)
    fn detect_wraparound(&self, current: &RaplReading, previous: &RaplReading) -> bool {
        current.energy_uj < previous.energy_uj
    }

    pub fn is_available(&self) -> bool {
        self.read_raw().is_some()
    }

    pub fn error_rate(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.read_errors as f64 / self.samples.len() as f64
        }
    }
}

impl EnergyMonitor for RaplHardwareMonitor {
    fn sample(&mut self, label: &str) -> EnergySample {
        match self.read_raw() {
            Some(reading) => {
                let joules = if let Some(ref baseline) = self.baseline {
                    if self.detect_wraparound(&reading, baseline) {
                        // Handle wraparound: add max range to delta
                        let wrapped_delta = reading.energy_uj + 
                            (reading.max_energy_range_uj - baseline.energy_uj);
                        wrapped_delta as f64 / 1_000_000.0
                    } else {
                        (reading.energy_uj - baseline.energy_uj) as f64 / 1_000_000.0
                    }
                } else {
                    self.baseline = Some(reading.clone());
                    0.0 // First reading establishes baseline
                };

                let sample = EnergySample {
                    operation_label: label.to_string(),
                    joules,
                    duration: self.start.elapsed(),
                    timestamp: Instant::now(),
                    core_id: 0,
                };

                self.samples.push(sample.clone());
                sample
            }
            None => {
                self.read_errors += 1;
                EnergySample {
                    operation_label: format!("{}_error", label),
                    joules: 0.0,
                    duration: self.start.elapsed(),
                    timestamp: Instant::now(),
                    core_id: 0,
                }
            }
        }
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
        self.baseline = None;
        self.start = Instant::now();
        self.read_errors = 0;
    }
}

/// Correlation analysis between software and hardware JLO measurements
#[derive(Debug, Clone)]
pub struct JloCorrelation {
    pub software_jlo: f64,
    pub hardware_jlo: f64,
    pub correlation_coefficient: f64,
    pub error_bound: f64,
    pub samples_count: usize,
}

impl JloCorrelation {
    /// Pearson correlation coefficient between two measurement streams
    pub fn compute(software: &[f64], hardware: &[f64]) -> Result<Self, &'static str> {
        if software.len() != hardware.len() || software.len() < 2 {
            return Err("Insufficient samples for correlation");
        }

        let n = software.len() as f64;
        let sum_s: f64 = software.iter().sum();
        let sum_h: f64 = hardware.iter().sum();
        let mean_s = sum_s / n;
        let mean_h = sum_h / n;

        let covariance: f64 = software.iter().zip(hardware.iter())
            .map(|(s, h)| (s - mean_s) * (h - mean_h))
            .sum();

        let var_s: f64 = software.iter().map(|s| (s - mean_s).powi(2)).sum();
        let var_h: f64 = hardware.iter().map(|h| (h - mean_h).powi(2)).sum();

        if var_s == 0.0 || var_h == 0.0 {
            return Err("Zero variance in measurements");
        }

        let correlation = covariance / (var_s.sqrt() * var_h.sqrt());
        let error = (mean_s - mean_h).abs() / mean_h;

        Ok(Self {
            software_jlo: mean_s,
            hardware_jlo: mean_h,
            correlation_coefficient: correlation,
            error_bound: error,
            samples_count: software.len(),
        })
    }

    /// Verify invariant: |r| ≥ 0.95 and error ≤ 20%
    pub fn verify_invariant(&self) -> bool {
        self.correlation_coefficient.abs() >= 0.95 && self.error_bound <= 0.20
    }
}