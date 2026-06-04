use crate::cre::causal_graph::CausalGraph;
use crate::cre::counterfactual::{Intervention, OutcomeDelta};
use crate::rso::policy::SubstrateState;

#[derive(Debug, Clone)]
pub struct RewardSignal {
    pub value: f64,
    pub confidence: f64,
    pub source: String,
}

pub struct RewardFunction {
    pub baseline_efficiency: f64,
    pub risk_aversion: f64,
}

impl RewardFunction {
    pub fn new(baseline_efficiency: f64, risk_aversion: f64) -> Self {
        Self {
            baseline_efficiency: baseline_efficiency.clamp(0.0, 1.0),
            risk_aversion: risk_aversion.clamp(0.0, 1.0),
        }
    }

    pub fn compute_from_counterfactual(
        &self,
        baseline: &SubstrateState,
        intervention: &Intervention,
        outcome: &OutcomeDelta,
    ) -> RewardSignal {
        let efficiency_delta = baseline.operational_efficiency - (-outcome.risk_delta);
        let reward_value =
            efficiency_delta * (1.0 - self.risk_aversion) + outcome.confidence * self.risk_aversion;

        RewardSignal {
            value: reward_value.clamp(-1.0, 1.0),
            confidence: outcome.confidence,
            source: format!("counterfactual:{}", intervention.target_node),
        }
    }

    pub fn compute_from_causal_graph(
        &self,
        _graph: &CausalGraph,
        state: &SubstrateState,
    ) -> RewardSignal {
        let avg_probability: f64 = state.live_probabilities.values().sum::<f64>()
            / state.live_probabilities.len().max(1) as f64;
        let reward_value =
            (avg_probability - self.baseline_efficiency) * (1.0 - self.risk_aversion);

        RewardSignal {
            value: reward_value.clamp(-1.0, 1.0),
            confidence: avg_probability,
            source: String::from("causal_graph"),
        }
    }
}
