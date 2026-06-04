use crate::hal::fingerprint::{ComplianceResult, GoldenMasterSpec, HalError, HardwareFingerprint};

// Re-export GoldenMasterSpec for CLI access

#[derive(Debug, Clone, PartialEq)]
pub enum BootOutcome {
    Compliant {
        score: f64,
        fingerprint_id: String,
        bindings: HardwareBindings,
    },
    Degraded {
        score: f64,
        violations: Vec<String>,
        fallback_bindings: HardwareBindings,
    },
    CriticalFailure {
        reason: String,
        score: f64,
        fingerprint_id: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareBindings {
    pub cpu_threads: usize,
    pub gpu_devices: Vec<GpuBinding>,
    pub memory_pool_bytes: u64,
    pub telemetry_handle: TelemetryHandle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuBinding {
    pub device_id: String,
    pub memory_bytes: u64,
    pub compute_units: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryHandle {
    pub stream_id: String,
    pub sampling_rate_hz: u64,
}

pub struct AtomicBootSequence {
    golden_master: GoldenMasterSpec,
    boot_log: Vec<String>,
    outcome: Option<BootOutcome>,
}

impl AtomicBootSequence {
    pub fn new(gm: GoldenMasterSpec) -> Self {
        Self {
            golden_master: gm,
            boot_log: Vec::new(),
            outcome: None,
        }
    }

    pub fn execute(&mut self) -> &BootOutcome {
        self.log("HARMONIS PRIME -- ATOMIC BOOT SEQUENCE INITIATED");

        self.log("Step 1: Generating hardware fingerprint...");
        let fingerprint = match HardwareFingerprint::generate() {
            Ok(fp) => {
                self.log(&format!("   Fingerprint generated: {}", fp.fingerprint_id));
                fp
            }
            Err(e) => {
                self.log(&format!("   Fingerprint generation failed: {}", e));
                let outcome = BootOutcome::CriticalFailure {
                    reason: format!("HAL detection failed: {}", e),
                    score: 0.0,
                    fingerprint_id: "unknown".to_string(),
                };
                self.outcome = Some(outcome);
                return self.outcome.as_ref().unwrap();
            }
        };

        self.log("Step 2: Verifying against Golden Master...");
        let compliance = fingerprint.verify_against_gm(&self.golden_master);

        match compliance {
            ComplianceResult::Compliant { score } => {
                self.log(&format!(
                    "   Compliance score: {:.2}% -- BOOT PERMITTED",
                    score * 100.0
                ));
                self.log("   Zero-drift barrier: PASSED");

                self.log("Step 3: Binding hardware resources...");
                let bindings = match self.bind_hardware(&fingerprint) {
                    Ok(b) => {
                        self.log(&format!("   CPU threads: {}", b.cpu_threads));
                        self.log(&format!("   GPU devices: {}", b.gpu_devices.len()));
                        self.log(&format!(
                            "   Memory pool: {} MB",
                            b.memory_pool_bytes / (1024 * 1024)
                        ));
                        b
                    }
                    Err(e) => {
                        self.log(&format!("   Hardware binding failed: {}", e));
                        let outcome = BootOutcome::CriticalFailure {
                            reason: format!("Hardware binding failed: {}", e),
                            score,
                            fingerprint_id: fingerprint.fingerprint_id,
                        };
                        self.outcome = Some(outcome);
                        return self.outcome.as_ref().unwrap();
                    }
                };

                let outcome = BootOutcome::Compliant {
                    score,
                    fingerprint_id: fingerprint.fingerprint_id,
                    bindings,
                };
                self.outcome = Some(outcome);
            }

            ComplianceResult::NonCompliant { score, violations } => {
                self.log(&format!(
                    "   Compliance score: {:.2}% -- DEGRADED MODE",
                    score * 100.0
                ));
                for v in &violations {
                    self.log(&format!("      Violation: {}", v));
                }

                self.log("Step 3: Binding fallback hardware (degraded mode)...");
                let fallback = self.bind_fallback_hardware(&fingerprint);
                self.log(&format!(
                    "   Degraded bindings: CPU threads: {}",
                    fallback.cpu_threads
                ));

                let outcome = BootOutcome::Degraded {
                    score,
                    violations,
                    fallback_bindings: fallback,
                };
                self.outcome = Some(outcome);
            }

            ComplianceResult::CriticalFailure { reason } => {
                self.log(&format!("   CRITICAL FAILURE: {}", reason));
                self.log("   Boot halted. System refuses to run on untrusted silicon.");

                let outcome = BootOutcome::CriticalFailure {
                    reason,
                    score: 0.0,
                    fingerprint_id: fingerprint.fingerprint_id,
                };
                self.outcome = Some(outcome);
            }
        }

        self.outcome.as_ref().unwrap()
    }

    fn bind_hardware(
        &self,
        fingerprint: &HardwareFingerprint,
    ) -> Result<HardwareBindings, HalError> {
        let cpu_threads = fingerprint.compute.cpu_threads;

        let gpu_devices = fingerprint
            .compute
            .gpu_devices
            .iter()
            .map(|gpu| GpuBinding {
                device_id: gpu.device_id.clone(),
                memory_bytes: gpu.vram_bytes,
                compute_units: (gpu.compute_capability.parse().unwrap_or(0) as f64 * 10.0) as u32,
            })
            .collect();

        let memory_pool_bytes = (fingerprint.memory.available_ram_bytes as f64 * 0.85) as u64;

        let telemetry_handle = TelemetryHandle {
            stream_id: format!("telemetry_{}", fingerprint.fingerprint_id),
            sampling_rate_hz: 1000,
        };

        Ok(HardwareBindings {
            cpu_threads,
            gpu_devices,
            memory_pool_bytes,
            telemetry_handle,
        })
    }

    fn bind_fallback_hardware(&self, fingerprint: &HardwareFingerprint) -> HardwareBindings {
        HardwareBindings {
            cpu_threads: fingerprint.compute.cpu_threads.min(2),
            gpu_devices: Vec::new(),
            memory_pool_bytes: (fingerprint.memory.available_ram_bytes as f64 * 0.5) as u64,
            telemetry_handle: TelemetryHandle {
                stream_id: format!("degraded_{}", fingerprint.fingerprint_id),
                sampling_rate_hz: 100,
            },
        }
    }

    fn log(&mut self, message: &str) {
        self.boot_log.push(message.to_string());
        println!("{}", message);
    }

    pub fn boot_log(&self) -> &[String] {
        &self.boot_log
    }

    pub fn is_booted(&self) -> bool {
        matches!(
            &self.outcome,
            Some(BootOutcome::Compliant { .. }) | Some(BootOutcome::Degraded { .. })
        )
    }

    pub fn outcome(&self) -> Option<&BootOutcome> {
        self.outcome.as_ref()
    }
}

impl Default for GoldenMasterSpec {
    fn default() -> Self {
        Self {
            spec_id: "HARMONIS-GM-v1.0".to_string(),
            min_compute_units: 4,
            min_ram_bytes: 8_000_000_000,
            max_thermal_celsius: 85.0,
            requires_secure_boot: false,
            min_driver_version: "535.0".to_string(),
            requires_rdma: false,
        }
    }
}

pub fn boot_harmonis() -> BootOutcome {
    let mut boot = AtomicBootSequence::new(GoldenMasterSpec::default());
    boot.execute().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_sequence_executes() {
        let outcome = boot_harmonis();
        match outcome {
            BootOutcome::Compliant { score, .. } => {
                assert!(score >= 0.95);
                println!("Boot test passed: Compliant ({:.2}%)", score * 100.0);
            }
            BootOutcome::Degraded { score, .. } => {
                println!("Boot test: Degraded ({:.2}%)", score * 100.0);
            }
            BootOutcome::CriticalFailure { reason, .. } => {
                panic!("Boot test failed: {}", reason);
            }
        }
    }
}
