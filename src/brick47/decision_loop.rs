//! BRICK-47 Pillar 3: Self-Healing Decision Loop (SHDL)
//! Detect -> Diagnose -> Simulate -> Propose -> Validate -> Apply -> Verify
//! Benchmark: MTTR < 5s, >= 90% auto-remediation success

use crate::brick47::types::{AuditEntry, RemediationAction, SimulationResult};
use std::collections::VecDeque;
use std::time::Instant;

/// Stage in the self-healing pipeline
#[derive(Clone, Debug, PartialEq)]
pub enum HealStage {
    Detected,
    Diagnosed,
    Simulated,
    Proposed,
    Validated,
    Applied,
    Verified,
    Failed(String),
}

/// SelfHealingDecisionLoop: End-to-end autonomous remediation pipeline
pub struct SelfHealingDecisionLoop {
    audit_log: VecDeque<AuditEntry>,
    max_audit_entries: usize,
    total_detections: u64,
    successful_heals: u64,
    failed_heals: u64,
    total_mttr_ms: f64,
}

impl SelfHealingDecisionLoop {
    pub fn new(max_audit: usize) -> Self {
        Self {
            audit_log: VecDeque::with_capacity(max_audit),
            max_audit_entries: max_audit,
            total_detections: 0,
            successful_heals: 0,
            failed_heals: 0,
            total_mttr_ms: 0.0,
        }
    }

    /// Stage 1: Detect anomaly
    pub fn detect(&mut self, anomaly_id: &str, layer: &str, severity: f64) -> Instant {
        self.total_detections += 1;
        let entry = AuditEntry::new(
            "detect",
            &format!(
                "anomaly={} layer={} severity={:.3}",
                anomaly_id, layer, severity
            ),
            true,
        );
        self.push_audit(entry);
        Instant::now()
    }

    /// Stage 2: Diagnose Ã¢â‚¬â€ identify root cause
    pub fn diagnose(&mut self, root_cause: &str, confidence: f64) -> bool {
        let success = confidence >= 0.85;
        let entry = AuditEntry::new(
            "diagnose",
            &format!("root_cause={} confidence={:.3}", root_cause, confidence),
            success,
        );
        self.push_audit(entry);
        success
    }

    /// Stage 3: Simulate Ã¢â‚¬â€ counterfactual check (delegates to pillar 7)
    pub fn simulate(&mut self, result: &SimulationResult) -> bool {
        let entry = AuditEntry::new(
            "simulate",
            &format!(
                "action={} approved={} reason={}",
                result.action_id, result.approved, result.reason
            ),
            result.approved,
        );
        self.push_audit(entry);
        result.approved
    }

    /// Stage 4: Propose remediation
    pub fn propose(&mut self, action: &RemediationAction) -> bool {
        let safe = action.estimated_impact_ms < 5000.0;
        let entry = AuditEntry::new(
            "propose",
            &format!(
                "action={} impact={:.1}ms rollback_steps={}",
                action.action_id,
                action.estimated_impact_ms,
                action.rollback_steps.len()
            ),
            safe,
        );
        self.push_audit(entry);
        safe
    }

    /// Stage 5: Validate against governance constraints
    pub fn validate(&mut self, stability_ok: bool, latency_ok: bool, coverage_ok: bool) -> bool {
        let valid = stability_ok && latency_ok && coverage_ok;
        let entry = AuditEntry::new(
            "validate",
            &format!(
                "stability={} latency={} coverage={}",
                stability_ok, latency_ok, coverage_ok
            ),
            valid,
        );
        self.push_audit(entry);
        valid
    }

    /// Stage 6: Apply fix
    pub fn apply(&mut self, action_id: &str) -> Instant {
        let entry = AuditEntry::new("apply", &format!("action={} executed", action_id), true);
        self.push_audit(entry);
        Instant::now()
    }

    /// Stage 7: Verify recovery
    pub fn verify(
        &mut self,
        detect_time: Instant,
        apply_time: Instant,
        health_restored: bool,
    ) -> bool {
        let mttr_ms = apply_time.duration_since(detect_time).as_secs_f64() * 1000.0;
        self.total_mttr_ms += mttr_ms;

        if health_restored && mttr_ms < 5000.0 {
            self.successful_heals += 1;
        } else {
            self.failed_heals += 1;
        }

        let entry = AuditEntry::new(
            "verify",
            &format!("mttr={:.1}ms health_restored={}", mttr_ms, health_restored),
            health_restored && mttr_ms < 5000.0,
        );
        self.push_audit(entry);
        health_restored && mttr_ms < 5000.0
    }

    /// Full pipeline: detect -> diagnose -> simulate -> propose -> validate -> apply -> verify
    pub fn execute_full(
        &mut self,
        anomaly_id: &str,
        layer: &str,
        severity: f64,
        root_cause: &str,
        root_confidence: f64,
        simulation: &SimulationResult,
        action: &RemediationAction,
        stability_ok: bool,
        latency_ok: bool,
        coverage_ok: bool,
        health_restored: bool,
    ) -> (bool, f64) {
        let t0 = self.detect(anomaly_id, layer, severity);

        if !self.diagnose(root_cause, root_confidence) {
            return (false, 0.0);
        }

        if !self.simulate(simulation) {
            return (false, 0.0);
        }

        if !self.propose(action) {
            return (false, 0.0);
        }

        if !self.validate(stability_ok, latency_ok, coverage_ok) {
            return (false, 0.0);
        }

        let t_apply = self.apply(&action.action_id);
        let success = self.verify(t0, t_apply, health_restored);

        (success, t_apply.duration_since(t0).as_secs_f64() * 1000.0)
    }

    fn push_audit(&mut self, entry: AuditEntry) {
        if self.audit_log.len() >= self.max_audit_entries {
            self.audit_log.pop_front();
        }
        self.audit_log.push_back(entry);
    }

    pub fn audit_trail(&self) -> &VecDeque<AuditEntry> {
        &self.audit_log
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.successful_heals + self.failed_heals;
        if total == 0 {
            return 0.0;
        }
        self.successful_heals as f64 / total as f64
    }

    pub fn avg_mttr_ms(&self) -> f64 {
        let total = self.successful_heals + self.failed_heals;
        if total == 0 {
            return 0.0;
        }
        self.total_mttr_ms / total as f64
    }

    pub fn stats(&self) -> (u64, u64, u64, f64) {
        (
            self.total_detections,
            self.successful_heals,
            self.failed_heals,
            self.avg_mttr_ms(),
        )
    }
}
