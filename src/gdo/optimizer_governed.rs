use crate::cre::causal_graph::CausalGraph;
use crate::gdo::policy_enforcer::{EnforcementResult, PolicyEnforcer};
use crate::gdo::resource_governor::{ResourceGovernor, ResourceMetrics};
use crate::rso::optimizer::{OptimizationEpoch, RecursiveOptimizer};
use crate::rso::policy::{OptimizationAction, SubstrateState};
use crate::rso::reward::RewardFunction;
use crate::tsg::audit_logger::AuditLogger;
use crate::tsg::governance_policy::GovernancePolicy;

#[derive(Debug, Clone)]
pub struct GovernedEpoch {
    pub epoch: OptimizationEpoch,
    pub enforcement: EnforcementResult,
    pub throttled: bool,
    pub audit_entry_id: String,
}

pub struct GovernedOptimizer {
    pub optimizer: RecursiveOptimizer,
    pub enforcer: PolicyEnforcer,
    pub governor: ResourceGovernor,
    pub audit_logger: AuditLogger,
    pub total_throttled: u64,
    pub total_violations: u64,
}

impl GovernedOptimizer {
    pub fn new(
        policy: GovernancePolicy,
        reward_function: RewardFunction,
        max_epochs: usize,
        cpu_limit: f64,
        memory_limit_mb: u64,
        latency_limit_ms: u64,
    ) -> Self {
        let optimizer = RecursiveOptimizer::new(
            crate::rso::policy::StructuralOptimizationEngine::new(0.01, 0.95),
            reward_function,
            max_epochs,
        );

        Self {
            optimizer,
            enforcer: PolicyEnforcer::new(policy),
            governor: ResourceGovernor::new(cpu_limit, memory_limit_mb, latency_limit_ms),
            audit_logger: AuditLogger::new(true),
            total_throttled: 0,
            total_violations: 0,
        }
    }

    pub fn run_governed_cycle(
        &mut self,
        current_state: &SubstrateState,
        available_actions: &[OptimizationAction],
        world: &CausalGraph,
        metrics: &ResourceMetrics,
    ) -> Option<GovernedEpoch> {
        // Step 1: Resource governance check
        let throttle = self.governor.evaluate_resources(metrics);
        if throttle.should_throttle {
            self.total_throttled += 1;
            return Some(GovernedEpoch {
                epoch: OptimizationEpoch {
                    epoch_id: self.optimizer.epoch_history.len() as u64,
                    action_taken: OptimizationAction::AcknowledgeCausalDrift {
                        source_node: String::from("governor"),
                        delta: 0.0,
                    },
                    reward_received: crate::rso::reward::RewardSignal {
                        value: 0.0,
                        confidence: 1.0,
                        source: String::from("throttle"),
                    },
                    state_before: current_state.clone(),
                    state_after: current_state.clone(),
                },
                enforcement: EnforcementResult {
                    approved: false,
                    bounded_decision: None,
                    violation: None,
                    constraint_hash: format!("{:016x}", metrics.cpu_usage_percent as u64),
                    evaluated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                },
                throttled: true,
                audit_entry_id: format!("throttle_{}", self.total_throttled),
            });
        }

        // Step 2: Policy enforcement
        let action = available_actions.first().cloned().unwrap_or(
            OptimizationAction::AcknowledgeCausalDrift {
                source_node: String::from("default"),
                delta: 0.0,
            },
        );

        let enforcement = self.enforcer.enforce(&action, current_state);

        if !enforcement.approved {
            self.total_violations += 1;
            return None;
        }

        // Step 3: Execute optimized cycle
        let epoch =
            self.optimizer
                .run_optimization_cycle(current_state, available_actions, world)?;

        // Step 4: Audit
        let audit_entry = self.audit_logger.log(
            format!("epoch_{}", epoch.epoch_id),
            format!("{:?}", std::mem::discriminant(&epoch.action_taken)),
            true,
            enforcement.constraint_hash.clone(),
        );

        Some(GovernedEpoch {
            epoch,
            enforcement,
            throttled: false,
            audit_entry_id: format!("audit_{}", audit_entry.timestamp),
        })
    }

    pub fn convergence_score(&self) -> f64 {
        self.optimizer.convergence_score()
    }

    pub fn get_stats(&self) -> (u64, u64, usize) {
        (
            self.total_throttled,
            self.total_violations,
            self.audit_logger.entry_count(),
        )
    }
}
