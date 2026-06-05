use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub max_optimization_depth: u32,
    pub safety_envelope_active: bool,
    pub strict_audit_logging: bool,
    pub resource_limit_cpu_percent: f64,
    pub resource_limit_memory_mb: u64,
    pub max_latency_ms: u64,
    pub allowed_optimization_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PolicyEvaluation {
    pub approved: bool,
    pub violation_reason: Option<String>,
    pub constraint_hash: String,
    pub evaluated_at: u64,
}

#[derive(Debug, Clone)]
pub enum GovernanceError {
    SafetyEnvelopeDisabled,
    PolicyViolation {
        reason: String,
    },
    ResourceLimitExceeded {
        resource: String,
        current: f64,
        limit: f64,
    },
    UnauthorizedAction {
        action: String,
    },
    CausalDriftDetected {
        drift_score: f64,
        threshold: f64,
    },
}

impl GovernancePolicy {
    pub fn sovereign_default() -> Self {
        Self {
            max_optimization_depth: 10,
            safety_envelope_active: true,
            strict_audit_logging: true,
            resource_limit_cpu_percent: 80.0,
            resource_limit_memory_mb: 8192,
            max_latency_ms: 100,
            allowed_optimization_actions: vec![
                String::from("RebalanceQuorum"),
                String::from("ThrottleInboundTraffic"),
                String::from("ShiftClusterLeader"),
                String::from("AcknowledgeCausalDrift"),
            ],
        }
    }

    pub fn evaluate_optimization_request(
        &self,
        action_key: &str,
        current_depth: u32,
        risk_score: f64,
    ) -> Result<PolicyEvaluation, GovernanceError> {
        if !self.safety_envelope_active {
            return Err(GovernanceError::SafetyEnvelopeDisabled);
        }

        if current_depth > self.max_optimization_depth {
            return Err(GovernanceError::PolicyViolation {
                reason: format!(
                    "Depth {} exceeds max {}",
                    current_depth, self.max_optimization_depth
                ),
            });
        }

        if !self
            .allowed_optimization_actions
            .contains(&action_key.to_string())
        {
            return Err(GovernanceError::UnauthorizedAction {
                action: action_key.to_string(),
            });
        }

        if risk_score > 1.0 {
            return Err(GovernanceError::CausalDriftDetected {
                drift_score: risk_score,
                threshold: 1.0,
            });
        }

        let constraint_hash = format!("{:016x}", self.hash_state());
        Ok(PolicyEvaluation {
            approved: true,
            violation_reason: None,
            constraint_hash,
            evaluated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    pub fn hash_state(&self) -> u64 {
        let mut hash: u64 = 0;
        hash = hash.wrapping_add(self.max_optimization_depth as u64);
        hash = hash.wrapping_add(if self.safety_envelope_active { 1 } else { 0 });
        hash = hash.wrapping_add(self.resource_limit_memory_mb);
        hash
    }

    pub fn enable_safety_envelope(&mut self) {
        self.safety_envelope_active = true;
    }

    pub fn disable_safety_envelope(&mut self) {
        self.safety_envelope_active = false;
    }
}
