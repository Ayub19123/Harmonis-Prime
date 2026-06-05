use crate::governance::policy::{GovernancePolicy, PolicyEnforcementResult};
use crate::hal::atomic_boot::HardwareBindings;
use crate::hal::fingerprint::HardwareFingerprint;
use crate::runtime::flow_runtime::FlowState;
use crate::runtime::telemetry_loop::TelemetryFrame;

/// GovernanceLock: Organic decision engine
/// Continuously evaluates and adapts to environmental factors
pub struct GovernanceLock {
    policy: GovernancePolicy,
    violation_count: u64,
    adaptation_count: u64,
    locked: bool,
}

impl GovernanceLock {
    pub fn new() -> Self {
        Self {
            policy: GovernancePolicy::production(),
            violation_count: 0,
            adaptation_count: 0,
            locked: false,
        }
    }

    /// Lock Governance Layer — activate continuous observation
    pub fn lock(
        &mut self,
        fingerprint: &HardwareFingerprint,
        bindings: &HardwareBindings,
    ) -> PolicyEnforcementResult {
        println!("GOVERNANCE: Locking continuous observation...");
        self.locked = true;

        let result = self.policy.enforce(fingerprint, bindings);

        match &result {
            PolicyEnforcementResult::Compliant { .. } => {
                println!("GOVERNANCE: Layer locked — COMPLIANT");
            }
            PolicyEnforcementResult::Throttled { factor, .. } => {
                self.adaptation_count += 1;
                println!(
                    "GOVERNANCE: Layer locked — THROTTLED at {:.0}%",
                    factor * 100.0
                );
            }
            PolicyEnforcementResult::EmergencyHalt { reason, .. } => {
                self.violation_count += 1;
                println!("GOVERNANCE: Layer locked — EMERGENCY HALT: {}", reason);
            }
        }

        result
    }

    /// Adapt to telemetry frame — real-time environmental response
    pub fn adapt(&mut self, frame: &TelemetryFrame) -> String {
        if !self.locked {
            return "GOVERNANCE: Not locked".to_string();
        }

        if frame.thermal_reading > self.policy.tsg.thermal_ceiling_celsius {
            self.violation_count += 1;
            self.adaptation_count += 1;
            return format!(
                "GOVERNANCE: ADAPT — thermal breach {:.1}C > {:.1}C",
                frame.thermal_reading, self.policy.tsg.thermal_ceiling_celsius
            );
        }

        if frame.memory_utilization > 90.0 {
            self.adaptation_count += 1;
            return "GOVERNANCE: ADAPT — memory pressure reducing allocation".to_string();
        }

        if matches!(frame.flow_state, FlowState::BottleneckDetected) {
            self.adaptation_count += 1;
            return "GOVERNANCE: ADAPT — bottleneck recovery initiated".to_string();
        }

        "GOVERNANCE: STABLE — no adaptation required".to_string()
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.violation_count, self.adaptation_count)
    }
}
