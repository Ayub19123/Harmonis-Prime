//! BRICK-47 Pillar 4: Learning Governance Layer
//! Constraint enforcement: stability, false positives, coverage; rollback on violation
//! Benchmark: No degradation of any KPI

use crate::brick47::types::AuditEntry;
use std::collections::VecDeque;

/// GovernanceConstraint: Hard limit on system health metrics
#[derive(Clone, Debug)]
pub struct GovernanceConstraint {
    pub metric_name: String,
    pub min_value: f64,
    pub max_value: f64,
}

impl GovernanceConstraint {
    pub fn new(name: &str, min: f64, max: f64) -> Self {
        Self {
            metric_name: name.to_string(),
            min_value: min,
            max_value: max,
        }
    }

    pub fn check(&self, value: f64) -> bool {
        value >= self.min_value && value <= self.max_value
    }
}

/// LearningGovernanceLayer: Safety rail for all self-learning updates
pub struct LearningGovernanceLayer {
    constraints: Vec<GovernanceConstraint>,
    violation_log: VecDeque<AuditEntry>,
    max_violations: usize,
    approved_updates: u64,
    rejected_updates: u64,
    rollback_count: u64,
}

impl LearningGovernanceLayer {
    pub fn production_default() -> Self {
        Self {
            constraints: vec![
                GovernanceConstraint::new("stability", 0.95, 1.0),
                GovernanceConstraint::new("false_positive_rate", 0.0, 0.02),
                GovernanceConstraint::new("observability_coverage", 0.99, 1.0),
                GovernanceConstraint::new("recovery_speed_ms", 0.0, 5000.0),
                GovernanceConstraint::new("availability", 0.9995, 1.0),
            ],
            violation_log: VecDeque::with_capacity(1000),
            max_violations: 1000,
            approved_updates: 0,
            rejected_updates: 0,
            rollback_count: 0,
        }
    }

    /// Evaluate a proposed update against all constraints
    pub fn evaluate(&mut self, metrics: &[(String, f64)]) -> (bool, Vec<String>) {
        let mut violations = Vec::new();

        for (name, value) in metrics {
            if let Some(constraint) = self.constraints.iter().find(|c| c.metric_name == *name) {
                if !constraint.check(*value) {
                    violations.push(format!(
                        "{}={:.5} violates constraint [{:.3}, {:.3}]",
                        name, value, constraint.min_value, constraint.max_value
                    ));
                }
            }
        }

        let approved = violations.is_empty();

        if approved {
            self.approved_updates += 1;
        } else {
            self.rejected_updates += 1;
            self.rollback_count += 1;
            let entry = AuditEntry::new("governance_reject", &violations.join("; "), false);
            self.push_violation(entry);
        }

        (approved, violations)
    }

    /// Post-update verification: detect silent degradation
    pub fn verify_post_update(
        &mut self,
        before: &[(String, f64)],
        after: &[(String, f64)],
    ) -> bool {
        let mut regressions = Vec::new();

        for (name, after_val) in after {
            if let Some((_, before_val)) = before.iter().find(|(n, _)| n == name) {
                // Check if metric degraded (lower is worse for most, except FPR)
                let degraded = match name.as_str() {
                    "false_positive_rate" | "recovery_speed_ms" => after_val > before_val,
                    _ => after_val < before_val,
                };

                if degraded {
                    regressions.push(format!(
                        "{} degraded: {:.5} -> {:.5}",
                        name, before_val, after_val
                    ));
                }
            }
        }

        if !regressions.is_empty() {
            self.rollback_count += 1;
            let entry = AuditEntry::new("governance_rollback", &regressions.join("; "), false);
            self.push_violation(entry);
            false
        } else {
            true
        }
    }

    fn push_violation(&mut self, entry: AuditEntry) {
        if self.violation_log.len() >= self.max_violations {
            self.violation_log.pop_front();
        }
        self.violation_log.push_back(entry);
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.approved_updates,
            self.rejected_updates,
            self.rollback_count,
        )
    }

    pub fn violation_log(&self) -> &VecDeque<AuditEntry> {
        &self.violation_log
    }
}
