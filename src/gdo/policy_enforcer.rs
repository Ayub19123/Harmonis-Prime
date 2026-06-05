use crate::rso::policy::{OptimizationAction, SubstrateState};
use crate::tsg::decision_bounds::{BoundedDecision, DecisionEngine};
use crate::tsg::governance_policy::{GovernanceError, GovernancePolicy};

#[derive(Debug, Clone)]
pub struct EnforcementResult {
    pub approved: bool,
    pub bounded_decision: Option<BoundedDecision>,
    pub violation: Option<GovernanceError>,
    pub constraint_hash: String,
    pub evaluated_at: u64,
}

pub struct PolicyEnforcer {
    pub decision_engine: DecisionEngine,
    pub current_depth: u32,
}

impl PolicyEnforcer {
    pub fn new(policy: GovernancePolicy) -> Self {
        Self {
            decision_engine: DecisionEngine::new(policy),
            current_depth: 0,
        }
    }

    pub fn enforce(
        &mut self,
        action: &OptimizationAction,
        state: &SubstrateState,
    ) -> EnforcementResult {
        let evaluated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        match self
            .decision_engine
            .evaluate_action(action, state, self.current_depth)
        {
            Ok(bounded) => {
                self.current_depth += 1;
                EnforcementResult {
                    approved: true,
                    bounded_decision: Some(bounded),
                    violation: None,
                    constraint_hash: format!("{:016x}", evaluated_at.wrapping_mul(31)),
                    evaluated_at,
                }
            }
            Err(err) => EnforcementResult {
                approved: false,
                bounded_decision: None,
                violation: Some(err),
                constraint_hash: format!("{:016x}", evaluated_at.wrapping_mul(17)),
                evaluated_at,
            },
        }
    }

    pub fn reset_depth(&mut self) {
        self.current_depth = 0;
    }

    pub fn get_depth(&self) -> u32 {
        self.current_depth
    }
}
