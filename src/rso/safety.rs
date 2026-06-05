use crate::rso::policy::OptimizationAction;

#[derive(Debug, Clone)]
pub struct SafetyViolation {
    pub rule: String,
    pub action: String,
    pub severity: u8,
}

pub struct SafetyEnvelope;

impl SafetyEnvelope {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, action: &OptimizationAction) -> Result<(), SafetyViolation> {
        match action {
            OptimizationAction::RebalanceQuorum { .. } => Ok(()),
            OptimizationAction::ThrottleInboundTraffic { rate_limit, .. } => {
                if *rate_limit == 0 {
                    Err(SafetyViolation {
                        rule: String::from("THROTTLE_NONZERO"),
                        action: format!("{:?}", action),
                        severity: 9,
                    })
                } else {
                    Ok(())
                }
            }
            OptimizationAction::ShiftClusterLeader { .. } => Ok(()),
            OptimizationAction::AcknowledgeCausalDrift { delta, .. } => {
                if delta.abs() > 1.0 {
                    Err(SafetyViolation {
                        rule: String::from("DRIFT_BOUNDED"),
                        action: format!("{:?}", action),
                        severity: 7,
                    })
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn validate_batch(
        &self,
        actions: &[OptimizationAction],
    ) -> Vec<Result<(), SafetyViolation>> {
        actions.iter().map(|a| self.validate(a)).collect()
    }
}
