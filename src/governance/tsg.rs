use crate::hal::atomic_boot::HardwareBindings;
use crate::hal::fingerprint::HardwareFingerprint;

#[derive(Debug, Clone, PartialEq)]
pub struct TrustSafetyGuard {
    pub thermal_ceiling_celsius: f64,
    pub power_ceiling_watts: u32,
    pub memory_ceiling_bytes: u64,
    pub cpu_ceiling_percent: f64,
    pub requires_secure_boot: bool,
    pub requires_driver_signature: bool,
    pub anomaly_threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SafetyCheck {
    Pass {
        metric: String,
        value: f64,
        limit: f64,
    },
    Warning {
        metric: String,
        value: f64,
        limit: f64,
        severity: f64,
    },
    Violation {
        metric: String,
        value: f64,
        limit: f64,
        action: SafetyAction,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SafetyAction {
    Throttle,
    Halt,
    Degrade,
    LogOnly,
}

impl TrustSafetyGuard {
    pub fn production() -> Self {
        Self {
            thermal_ceiling_celsius: 85.0,
            power_ceiling_watts: 750,
            memory_ceiling_bytes: 12_000_000_000,
            cpu_ceiling_percent: 95.0,
            requires_secure_boot: false,
            requires_driver_signature: true,
            anomaly_threshold: 0.15,
        }
    }

    pub fn evaluate(
        &self,
        fingerprint: &HardwareFingerprint,
        bindings: &HardwareBindings,
    ) -> Vec<SafetyCheck> {
        let mut checks = Vec::new();

        // Thermal ceiling
        if fingerprint.thermal.max_sustained_load_celsius > self.thermal_ceiling_celsius {
            checks.push(SafetyCheck::Violation {
                metric: "thermal".to_string(),
                value: fingerprint.thermal.max_sustained_load_celsius,
                limit: self.thermal_ceiling_celsius,
                action: SafetyAction::Throttle,
            });
        } else if fingerprint.thermal.max_sustained_load_celsius
            > self.thermal_ceiling_celsius * 0.9
        {
            checks.push(SafetyCheck::Warning {
                metric: "thermal".to_string(),
                value: fingerprint.thermal.max_sustained_load_celsius,
                limit: self.thermal_ceiling_celsius,
                severity: 0.8,
            });
        } else {
            checks.push(SafetyCheck::Pass {
                metric: "thermal".to_string(),
                value: fingerprint.thermal.max_sustained_load_celsius,
                limit: self.thermal_ceiling_celsius,
            });
        }

        // Memory ceiling
        if bindings.memory_pool_bytes > self.memory_ceiling_bytes {
            checks.push(SafetyCheck::Violation {
                metric: "memory".to_string(),
                value: bindings.memory_pool_bytes as f64,
                limit: self.memory_ceiling_bytes as f64,
                action: SafetyAction::Throttle,
            });
        } else {
            checks.push(SafetyCheck::Pass {
                metric: "memory".to_string(),
                value: bindings.memory_pool_bytes as f64,
                limit: self.memory_ceiling_bytes as f64,
            });
        }

        // Secure boot enforcement
        if self.requires_secure_boot && !fingerprint.firmware.secure_boot_enabled {
            checks.push(SafetyCheck::Violation {
                metric: "secure_boot".to_string(),
                value: 0.0,
                limit: 1.0,
                action: SafetyAction::Halt,
            });
        }

        // Driver signature enforcement
        if self.requires_driver_signature && !fingerprint.drivers.signature_valid {
            checks.push(SafetyCheck::Violation {
                metric: "driver_signature".to_string(),
                value: 0.0,
                limit: 1.0,
                action: SafetyAction::Halt,
            });
        }

        checks
    }

    pub fn has_critical_violation(&self, checks: &[SafetyCheck]) -> bool {
        checks.iter().any(|c| {
            matches!(
                c,
                SafetyCheck::Violation {
                    action: SafetyAction::Halt,
                    ..
                }
            )
        })
    }

    pub fn has_any_violation(&self, checks: &[SafetyCheck]) -> bool {
        checks
            .iter()
            .any(|c| matches!(c, SafetyCheck::Violation { .. }))
    }
}
