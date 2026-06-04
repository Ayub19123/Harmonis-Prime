use crate::governance::tsg::{SafetyAction, SafetyCheck};
use crate::hal::atomic_boot::HardwareBindings;

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDirectiveOptimizer {
    pub compute_fairness_enabled: bool,
    pub resource_throttle_enabled: bool,
    pub emergency_shutdown_enabled: bool,
    pub priority_enforcement_enabled: bool,
    pub max_concurrent_tasks: usize,
    pub throttle_factor: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectiveOutcome {
    Proceed {
        allocation: ResourceAllocation,
    },
    Throttle {
        allocation: ResourceAllocation,
        factor: f64,
        reason: String,
    },
    EmergencyHalt {
        reason: String,
        audit_log: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceAllocation {
    pub cpu_threads: usize,
    pub memory_bytes: u64,
    pub gpu_devices: usize,
    pub telemetry_rate_hz: u64,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

impl GlobalDirectiveOptimizer {
    pub fn production() -> Self {
        Self {
            compute_fairness_enabled: true,
            resource_throttle_enabled: true,
            emergency_shutdown_enabled: true,
            priority_enforcement_enabled: true,
            max_concurrent_tasks: 16,
            throttle_factor: 0.75,
        }
    }

    pub fn optimize(
        &self,
        checks: &[SafetyCheck],
        bindings: &HardwareBindings,
    ) -> DirectiveOutcome {
        let mut audit = Vec::new();
        audit.push(format!("GDO evaluating {} safety checks", checks.len()));

        // Critical violations = emergency halt
        if checks.iter().any(|c| {
            matches!(
                c,
                SafetyCheck::Violation {
                    action: SafetyAction::Halt,
                    ..
                }
            )
        }) {
            let reasons: Vec<String> = checks
                .iter()
                .filter(|c| {
                    matches!(
                        c,
                        SafetyCheck::Violation {
                            action: SafetyAction::Halt,
                            ..
                        }
                    )
                })
                .map(|c| format!("{:?}", c))
                .collect();
            audit.push(format!(
                "CRITICAL: Emergency halt triggered: {}",
                reasons.join("; ")
            ));
            return DirectiveOutcome::EmergencyHalt {
                reason: reasons.join("; "),
                audit_log: audit,
            };
        }

        // Throttle violations
        let throttle_violations: Vec<&SafetyCheck> = checks
            .iter()
            .filter(|c| {
                matches!(
                    c,
                    SafetyCheck::Violation {
                        action: SafetyAction::Throttle,
                        ..
                    }
                )
            })
            .collect();

        if !throttle_violations.is_empty() && self.resource_throttle_enabled {
            let throttled_cpu = (bindings.cpu_threads as f64 * self.throttle_factor) as usize;
            let throttled_mem = (bindings.memory_pool_bytes as f64 * self.throttle_factor) as u64;

            audit.push(format!(
                "THROTTLE: Reducing CPU from {} to {}",
                bindings.cpu_threads, throttled_cpu
            ));
            audit.push(format!(
                "THROTTLE: Reducing memory from {} to {}",
                bindings.memory_pool_bytes, throttled_mem
            ));

            return DirectiveOutcome::Throttle {
                allocation: ResourceAllocation {
                    cpu_threads: throttled_cpu.max(1),
                    memory_bytes: throttled_mem.max(1_000_000_000),
                    gpu_devices: bindings.gpu_devices.len(),
                    telemetry_rate_hz: 500,
                    priority: TaskPriority::Normal,
                },
                factor: self.throttle_factor,
                reason: "TSG violation: thermal/memory ceiling exceeded".to_string(),
            };
        }

        // Warnings
        let warnings: Vec<&SafetyCheck> = checks
            .iter()
            .filter(|c| matches!(c, SafetyCheck::Warning { .. }))
            .collect();

        if !warnings.is_empty() {
            audit.push(format!("WARN: {} safety warnings active", warnings.len()));
            return DirectiveOutcome::Proceed {
                allocation: ResourceAllocation {
                    cpu_threads: bindings.cpu_threads,
                    memory_bytes: bindings.memory_pool_bytes,
                    gpu_devices: bindings.gpu_devices.len(),
                    telemetry_rate_hz: 1000,
                    priority: TaskPriority::High,
                },
            };
        }

        // Normal operation
        audit.push("PASS: All safety checks clear".to_string());
        DirectiveOutcome::Proceed {
            allocation: ResourceAllocation {
                cpu_threads: bindings.cpu_threads,
                memory_bytes: bindings.memory_pool_bytes,
                gpu_devices: bindings.gpu_devices.len(),
                telemetry_rate_hz: 1000,
                priority: TaskPriority::Normal,
            },
        }
    }

    pub fn enforce_priority(&self, priority: TaskPriority, allocation: &mut ResourceAllocation) {
        if !self.priority_enforcement_enabled {
            return;
        }
        match priority {
            TaskPriority::Critical => {
                allocation.telemetry_rate_hz = 2000;
            }
            TaskPriority::High => {
                allocation.telemetry_rate_hz = 1000;
            }
            TaskPriority::Normal => {
                allocation.telemetry_rate_hz = 500;
            }
            TaskPriority::Low | TaskPriority::Background => {
                allocation.cpu_threads = (allocation.cpu_threads as f64 * 0.5) as usize;
                allocation.memory_bytes = (allocation.memory_bytes as f64 * 0.5) as u64;
                allocation.telemetry_rate_hz = 100;
            }
        }
    }
}
