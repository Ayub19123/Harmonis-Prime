use crate::cre::causal_graph::CausalGraph;
use crate::cre::counterfactual::{CounterfactualEngine, Intervention};
use crate::rso::policy::{OptimizationAction, StructuralOptimizationEngine, SubstrateState};
use crate::rso::reward::{RewardFunction, RewardSignal};

#[derive(Debug, Clone)]
pub struct OptimizationEpoch {
    pub epoch_id: u64,
    pub action_taken: OptimizationAction,
    pub reward_received: RewardSignal,
    pub state_before: SubstrateState,
    pub state_after: SubstrateState,
}

pub struct RecursiveOptimizer {
    pub policy_engine: StructuralOptimizationEngine,
    pub reward_function: RewardFunction,
    pub counterfactual_engine: CounterfactualEngine,
    pub epoch_history: Vec<OptimizationEpoch>,
    pub max_epochs: usize,
}

impl RecursiveOptimizer {
    pub fn new(
        policy_engine: StructuralOptimizationEngine,
        reward_function: RewardFunction,
        max_epochs: usize,
    ) -> Self {
        Self {
            policy_engine,
            reward_function,
            counterfactual_engine: CounterfactualEngine::new(),
            epoch_history: Vec::new(),
            max_epochs: max_epochs.max(1),
        }
    }

    pub fn run_optimization_cycle(
        &mut self,
        current_state: &SubstrateState,
        available_actions: &[OptimizationAction],
        world: &CausalGraph,
    ) -> Option<OptimizationEpoch> {
        if self.epoch_history.len() >= self.max_epochs {
            return None;
        }

        let (action, _confidence) = self
            .policy_engine
            .observe_and_predict(current_state, available_actions);

        let intervention = Intervention {
            target_node: String::from("system"),
            forced_value: 0.5,
            description: format!("{:?}", std::mem::discriminant(&action)),
        };

        let outcome = self
            .counterfactual_engine
            .evaluate(world, &intervention, "risk");

        let reward = self.reward_function.compute_from_counterfactual(
            current_state,
            &intervention,
            &outcome,
        );

        let next_state = SubstrateState {
            active_nodes: current_state.active_nodes.clone(),
            live_probabilities: current_state.live_probabilities.clone(),
            operational_efficiency: current_state.operational_efficiency + reward.value,
        };

        self.policy_engine
            .learn_weights(current_state, &action, reward.value, &next_state);

        let epoch = OptimizationEpoch {
            epoch_id: self.epoch_history.len() as u64,
            action_taken: action,
            reward_received: reward,
            state_before: current_state.clone(),
            state_after: next_state.clone(),
        };

        self.epoch_history.push(epoch.clone());
        Some(epoch)
    }

    pub fn convergence_score(&self) -> f64 {
        if self.epoch_history.len() < 2 {
            return 0.0;
        }
        let recent: Vec<f64> = self
            .epoch_history
            .iter()
            .rev()
            .take(5)
            .map(|e| e.reward_received.value)
            .collect();
        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        1.0 - variance.sqrt().clamp(0.0, 1.0)
    }
}
