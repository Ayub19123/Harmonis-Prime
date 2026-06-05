use crate::fv::tla_spec::TlaSpec;

/// Result of invariant checking
#[derive(Debug, Clone)]
pub struct InvariantResult {
    pub invariant_name: String,
    pub holds: bool,
    pub counterexample: Option<Vec<String>>,
    pub checked_states: u64,
    pub violated_at: Option<String>,
}

/// Core invariant checker for TLA+ specifications
pub struct InvariantChecker {
    pub checked_invariants: Vec<InvariantResult>,
    pub total_states_explored: u64,
}

impl InvariantChecker {
    pub fn new() -> Self {
        Self {
            checked_invariants: Vec::new(),
            total_states_explored: 0,
        }
    }

    /// Check Raft consensus safety: No two committed entries conflict
    pub fn check_raft_safety(
        &mut self,
        _spec: &TlaSpec,
        node_logs: &[Vec<String>],
    ) -> InvariantResult {
        let mut result = InvariantResult {
            invariant_name: "RaftConsensusSafety".to_string(),
            holds: true,
            counterexample: None,
            checked_states: 0,
            violated_at: None,
        };

        for (i, log_i) in node_logs.iter().enumerate() {
            for (j, log_j) in node_logs.iter().enumerate() {
                if i >= j {
                    continue;
                }

                for (entry_idx, entry_i) in log_i.iter().enumerate() {
                    if entry_idx < log_j.len() {
                        let entry_j = &log_j[entry_idx];
                        if entry_i != entry_j {
                            result.holds = false;
                            result.violated_at =
                                Some(format!("node_{}/node_{} at index {}", i, j, entry_idx));
                            result.counterexample = Some(vec![
                                format!("node_{}[{}] = {}", i, entry_idx, entry_i),
                                format!("node_{}[{}] = {}", j, entry_idx, entry_j),
                            ]);
                            self.checked_invariants.push(result.clone());
                            return result;
                        }
                    }
                }
                result.checked_states += 1;
            }
        }

        self.total_states_explored += result.checked_states;
        self.checked_invariants.push(result.clone());
        result
    }

    /// Check actor liveness: Every failed actor is eventually restarted
    pub fn check_actor_liveness(
        &mut self,
        actor_states: &[String],
        restart_log: &[(usize, u64)],
    ) -> InvariantResult {
        let mut result = InvariantResult {
            invariant_name: "ActorLiveness".to_string(),
            holds: true,
            counterexample: None,
            checked_states: actor_states.len() as u64,
            violated_at: None,
        };

        for (idx, state) in actor_states.iter().enumerate() {
            if state == "failed" {
                let was_restarted = restart_log.iter().any(|(actor_idx, _)| *actor_idx == idx);
                if !was_restarted {
                    result.holds = false;
                    result.violated_at = Some(format!("actor_{}", idx));
                    result.counterexample = Some(vec![
                        format!("actor_{} state = failed", idx),
                        "No restart recorded in restart_log".to_string(),
                    ]);
                    self.checked_invariants.push(result.clone());
                    return result;
                }
            }
        }

        self.total_states_explored += result.checked_states;
        self.checked_invariants.push(result.clone());
        result
    }

    /// Check causal consistency: POM operations preserve happens-before
    pub fn check_causal_consistency(
        &mut self,
        operations: &[(String, u64, u64)],
    ) -> InvariantResult {
        let mut result = InvariantResult {
            invariant_name: "CausalConsistency".to_string(),
            holds: true,
            counterexample: None,
            checked_states: operations.len() as u64,
            violated_at: None,
        };

        for i in 0..operations.len() {
            for j in (i + 1)..operations.len() {
                let (op_i, ts_i, dep_i) = &operations[i];
                let (op_j, ts_j, dep_j) = &operations[j];

                // If op_i happens-before op_j (dep_j references op_i), then ts_i < ts_j
                if *dep_j == *ts_i && *ts_j <= *ts_i {
                    result.holds = false;
                    result.violated_at = Some(format!("{} / {}", op_i, op_j));
                    result.counterexample = Some(vec![
                        format!("{} ts={} dep={}", op_i, ts_i, dep_i),
                        format!(
                            "{} ts={} dep={} (violates happens-before)",
                            op_j, ts_j, dep_j
                        ),
                    ]);
                    self.checked_invariants.push(result.clone());
                    return result;
                }
            }
        }

        self.total_states_explored += result.checked_states;
        self.checked_invariants.push(result.clone());
        result
    }

    /// Check governance enforcement: No action violates policy bounds
    pub fn check_governance_enforcement(
        &mut self,
        actions: &[(String, f64, f64)],
        cpu_limit: f64,
        mem_limit: f64,
    ) -> InvariantResult {
        let mut result = InvariantResult {
            invariant_name: "GovernanceEnforcement".to_string(),
            holds: true,
            counterexample: None,
            checked_states: actions.len() as u64,
            violated_at: None,
        };

        for (action_name, cpu, mem) in actions {
            if *cpu > cpu_limit || *mem > mem_limit {
                result.holds = false;
                result.violated_at = Some(action_name.clone());
                result.counterexample = Some(vec![
                    format!("Action: {} CPU={:.1}% MEM={:.1}MB", action_name, cpu, mem),
                    format!("Limits: CPU={:.1}% MEM={:.1}MB", cpu_limit, mem_limit),
                ]);
                self.checked_invariants.push(result.clone());
                return result;
            }
        }

        self.total_states_explored += result.checked_states;
        self.checked_invariants.push(result.clone());
        result
    }

    pub fn get_summary(&self) -> (usize, usize, u64) {
        let passed = self.checked_invariants.iter().filter(|r| r.holds).count();
        let failed = self.checked_invariants.iter().filter(|r| !r.holds).count();
        (passed, failed, self.total_states_explored)
    }
}
