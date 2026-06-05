use crate::governance::gdo::{DirectiveOutcome, GlobalDirectiveOptimizer, ResourceAllocation};
use crate::governance::tsg::{SafetyCheck, TrustSafetyGuard};
use crate::hal::atomic_boot::HardwareBindings;
use crate::hal::fingerprint::HardwareFingerprint;

#[derive(Debug, Clone, PartialEq)]
pub struct GovernancePolicy {
    pub tsg: TrustSafetyGuard,
    pub gdo: GlobalDirectiveOptimizer,
    pub policy_id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyEnforcementResult {
    Compliant {
        allocation: ResourceAllocation,
        audit: Vec<String>,
    },
    Throttled {
        allocation: ResourceAllocation,
        factor: f64,
        audit: Vec<String>,
    },
    EmergencyHalt {
        reason: String,
        audit: Vec<String>,
    },
}

impl GovernancePolicy {
    pub fn production() -> Self {
        Self {
            tsg: TrustSafetyGuard::production(),
            gdo: GlobalDirectiveOptimizer::production(),
            policy_id: "TSG-GDO-v1.0".to_string(),
            version: "BRICK-39B".to_string(),
        }
    }

    pub fn enforce(
        &self,
        fingerprint: &HardwareFingerprint,
        bindings: &HardwareBindings,
    ) -> PolicyEnforcementResult {
        let mut audit = Vec::new();
        audit.push(format!(
            "POLICY {} enforcing on fingerprint {}",
            self.policy_id, fingerprint.fingerprint_id
        ));
        audit.push(format!(
            "TSG thermal ceiling: {}C",
            self.tsg.thermal_ceiling_celsius
        ));
        audit.push(format!("GDO throttle factor: {}", self.gdo.throttle_factor));

        // Step 1: TSG evaluation
        let safety_checks = self.tsg.evaluate(fingerprint, bindings);
        let violations = safety_checks
            .iter()
            .filter(|c| matches!(c, SafetyCheck::Violation { .. }))
            .count();
        let warnings = safety_checks
            .iter()
            .filter(|c| matches!(c, SafetyCheck::Warning { .. }))
            .count();

        audit.push(format!(
            "TSG evaluation: {} violations, {} warnings",
            violations, warnings
        ));

        // Step 2: GDO optimization
        let directive = self.gdo.optimize(&safety_checks, bindings);

        match directive {
            DirectiveOutcome::Proceed { allocation } => {
                audit.push("GDO: Proceed with full allocation".to_string());
                PolicyEnforcementResult::Compliant { allocation, audit }
            }
            DirectiveOutcome::Throttle {
                allocation,
                factor,
                reason: _,
            } => {
                audit.push(format!("GDO: Throttle applied at factor {}", factor));
                PolicyEnforcementResult::Throttled {
                    allocation,
                    factor,
                    audit,
                }
            }
            DirectiveOutcome::EmergencyHalt {
                reason,
                audit_log: gdo_audit,
            } => {
                audit.extend(gdo_audit);
                audit.push(format!("GDO: EMERGENCY HALT -- {}", reason));
                PolicyEnforcementResult::EmergencyHalt { reason, audit }
            }
        }
    }

    pub fn is_emergency_halt(&self, result: &PolicyEnforcementResult) -> bool {
        matches!(result, PolicyEnforcementResult::EmergencyHalt { .. })
    }
}
