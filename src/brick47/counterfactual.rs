//! BRICK-47 Pillar 7: Counterfactual Simulation Engine
//! Shadow environment Ã¢â‚¬â€ "what-if" simulation before applying fixes
//! Benchmark: Zero unsafe self-fixes

use crate::brick47::types::{RemediationAction, SimulationResult};
use std::collections::HashMap;

/// SimulationScenario: Pre-defined chaos scenario for shadow testing
#[derive(Clone, Debug)]
pub struct SimulationScenario {
    pub name: String,
    pub stability_before: f64,
    pub latency_before_ms: f64,
    pub coverage_before: f64,
    pub expected_stability_delta: f64,
    pub expected_latency_delta_ms: f64,
    pub expected_coverage_delta: f64,
    pub cascade_risk: f64,
}

impl SimulationScenario {
    pub fn new(name: &str, stability: f64, latency_ms: f64, coverage: f64) -> Self {
        Self {
            name: name.to_string(),
            stability_before: stability,
            latency_before_ms: latency_ms,
            coverage_before: coverage,
            expected_stability_delta: 0.0,
            expected_latency_delta_ms: 0.0,
            expected_coverage_delta: 0.0,
            cascade_risk: 0.0,
        }
    }

    pub fn with_deltas(mut self, stab: f64, lat: f64, cov: f64, risk: f64) -> Self {
        self.expected_stability_delta = stab;
        self.expected_latency_delta_ms = lat;
        self.expected_coverage_delta = cov;
        self.cascade_risk = risk.clamp(0.0, 1.0);
        self
    }
}

/// CounterfactualSimulationEngine: Shadow environment for safe fix validation
pub struct CounterfactualSimulationEngine {
    scenarios: HashMap<String, SimulationScenario>,
    simulation_count: u64,
    approved_count: u64,
    rejected_count: u64,
}

impl CounterfactualSimulationEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            scenarios: HashMap::new(),
            simulation_count: 0,
            approved_count: 0,
            rejected_count: 0,
        };
        engine.register_default_scenarios();
        engine
    }

    fn register_default_scenarios(&mut self) {
        // Node crash scenario
        self.scenarios.insert(
            "node_crash".to_string(),
            SimulationScenario::new("node_crash", 0.98, 50.0, 1.0)
                .with_deltas(-0.02, 200.0, 0.0, 0.15),
        );
        // Memory leak scenario
        self.scenarios.insert(
            "memory_leak".to_string(),
            SimulationScenario::new("memory_leak", 0.95, 100.0, 1.0)
                .with_deltas(-0.05, 500.0, -0.01, 0.25),
        );
        // Network partition scenario
        self.scenarios.insert(
            "network_partition".to_string(),
            SimulationScenario::new("network_partition", 0.97, 80.0, 1.0)
                .with_deltas(-0.03, 300.0, 0.0, 0.20),
        );
        // Consensus split scenario
        self.scenarios.insert(
            "consensus_split".to_string(),
            SimulationScenario::new("consensus_split", 0.96, 120.0, 1.0)
                .with_deltas(-0.04, 400.0, -0.02, 0.30),
        );
        // Cache corruption scenario
        self.scenarios.insert(
            "cache_corruption".to_string(),
            SimulationScenario::new("cache_corruption", 0.99, 30.0, 1.0)
                .with_deltas(-0.01, 100.0, 0.0, 0.10),
        );
    }

    /// Register custom scenario
    pub fn register_scenario(&mut self, scenario: SimulationScenario) {
        self.scenarios.insert(scenario.name.clone(), scenario);
    }

    /// Simulate a remediation action against a scenario
    pub fn simulate(
        &mut self,
        action: &RemediationAction,
        scenario_name: &str,
    ) -> SimulationResult {
        self.simulation_count += 1;

        let scenario = match self.scenarios.get(scenario_name) {
            Some(s) => s.clone(),
            None => {
                self.rejected_count += 1;
                return SimulationResult::rejected(
                    &action.action_id,
                    &format!("Unknown scenario: {}", scenario_name),
                );
            }
        };

        // Invariant checks
        let projected_stability = scenario.stability_before + scenario.expected_stability_delta;
        let projected_latency = scenario.latency_before_ms + scenario.expected_latency_delta_ms;
        let projected_coverage = scenario.coverage_before + scenario.expected_coverage_delta;

        // Hard reject conditions
        if projected_stability < 0.90 {
            self.rejected_count += 1;
            return SimulationResult::rejected(
                &action.action_id,
                &format!(
                    "Projected stability {:.3} < 0.90 threshold",
                    projected_stability
                ),
            );
        }

        if projected_latency > 5000.0 {
            self.rejected_count += 1;
            return SimulationResult::rejected(
                &action.action_id,
                &format!(
                    "Projected latency {:.1}ms > 5000ms threshold",
                    projected_latency
                ),
            );
        }

        if projected_coverage < 0.95 {
            self.rejected_count += 1;
            return SimulationResult::rejected(
                &action.action_id,
                &format!(
                    "Projected coverage {:.3} < 0.95 threshold",
                    projected_coverage
                ),
            );
        }

        if scenario.cascade_risk > 0.35 {
            self.rejected_count += 1;
            return SimulationResult::rejected(
                &action.action_id,
                &format!("Cascade risk {:.3} > 0.35 threshold", scenario.cascade_risk),
            );
        }

        self.approved_count += 1;
        SimulationResult::approved(
            &action.action_id,
            scenario.expected_stability_delta,
            scenario.expected_latency_delta_ms,
            scenario.cascade_risk,
        )
    }

    /// Batch simulate across all registered scenarios
    pub fn simulate_all(&mut self, action: &RemediationAction) -> Vec<SimulationResult> {
        let names: Vec<String> = self.scenarios.keys().cloned().collect();
        names
            .iter()
            .map(|name| self.simulate(action, name))
            .collect()
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (
            self.simulation_count,
            self.approved_count,
            self.rejected_count,
        )
    }

    pub fn approval_rate(&self) -> f64 {
        if self.simulation_count == 0 {
            return 0.0;
        }
        self.approved_count as f64 / self.simulation_count as f64
    }
}
