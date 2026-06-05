use crate::rso::policy::{OptimizationAction, SubstrateState};
use crate::tsg::governance_policy::{GovernanceError, GovernancePolicy};

#[derive(Debug, Clone)]
pub struct DecisionBounds {
    pub cpu_bound: f64,
    pub memory_bound: u64,
    pub latency_bound: u64,
    pub action_whitelist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BoundedDecision {
    pub action: OptimizationAction,
    pub bounds_applied: DecisionBounds,
    pub pre_approved: bool,
}

pub struct DecisionEngine {
    pub policy: GovernancePolicy,
}

impl DecisionEngine {
    pub fn new(policy: GovernancePolicy) -> Self {
        Self { policy }
    }

    pub fn calculate_bounds(&self, state: &SubstrateState) -> DecisionBounds {
        let efficiency_factor = state.operational_efficiency.clamp(0.0, 1.0);
        DecisionBounds {
            cpu_bound: self.policy.resource_limit_cpu_percent * efficiency_factor,
            memory_bound: (self.policy.resource_limit_memory_mb as f64 * efficiency_factor) as u64,
            latency_bound: self.policy.max_latency_ms,
            action_whitelist: self.policy.allowed_optimization_actions.clone(),
        }
    }

    pub fn evaluate_action(
        &self,
        action: &OptimizationAction,
        state: &SubstrateState,
        current_depth: u32,
    ) -> Result<BoundedDecision, GovernanceError> {
        let action_key = match action {
            OptimizationAction::RebalanceQuorum { .. } => "RebalanceQuorum",
            OptimizationAction::ThrottleInboundTraffic { .. } => "ThrottleInboundTraffic",
            OptimizationAction::ShiftClusterLeader { .. } => "ShiftClusterLeader",
            OptimizationAction::AcknowledgeCausalDrift { .. } => "AcknowledgeCausalDrift",
        };

        let bounds = self.calculate_bounds(state);
        let risk_score = 1.0 - state.operational_efficiency;

        let evaluation =
            self.policy
                .evaluate_optimization_request(action_key, current_depth, risk_score)?;

        Ok(BoundedDecision {
            action: action.clone(),
            bounds_applied: bounds,
            pre_approved: evaluation.approved,
        })
    }

    pub fn evaluate_action_batch(
        &self,
        actions: &[OptimizationAction],
        state: &SubstrateState,
        current_depth: u32,
    ) -> Vec<Result<BoundedDecision, GovernanceError>> {
        actions
            .iter()
            .map(|a| self.evaluate_action(a, state, current_depth))
            .collect()
    }
}
