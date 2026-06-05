use crate::fv::invariant_checker::{InvariantChecker, InvariantResult};
use crate::fv::tla_spec::{TlaAction, TlaSpec};

/// Represents a state in the model checking graph
#[derive(Debug, Clone)]
pub struct ModelState {
    pub state_id: u64,
    pub variable_values: Vec<(String, String)>,
    pub outgoing_actions: Vec<String>,
}

/// Bounded model checker for TLA+ specifications
pub struct ModelChecker {
    pub max_depth: u32,
    pub state_queue: Vec<ModelState>,
    pub visited_states: Vec<ModelState>,
    pub invariant_checker: InvariantChecker,
    pub deadlock_detected: bool,
    pub deadlock_trace: Option<Vec<String>>,
}

impl ModelChecker {
    pub fn new(max_depth: u32) -> Self {
        Self {
            max_depth,
            state_queue: Vec::new(),
            visited_states: Vec::new(),
            invariant_checker: InvariantChecker::new(),
            deadlock_detected: false,
            deadlock_trace: None,
        }
    }

    /// Initialize model checking from initial state
    pub fn initialize(&mut self, spec: &TlaSpec) {
        let initial_values: Vec<(String, String)> = spec
            .variables
            .iter()
            .map(|v| (v.name.clone(), v.current_value.clone()))
            .collect();

        let initial_state = ModelState {
            state_id: 0,
            variable_values: initial_values,
            outgoing_actions: spec
                .actions
                .iter()
                .filter(|a| a.enabled)
                .map(|a| a.name.clone())
                .collect(),
        };

        self.state_queue.push(initial_state.clone());
        self.visited_states.push(initial_state);
    }

    /// Perform bounded BFS model checking
    pub fn check_bounded(&mut self, spec: &TlaSpec, depth: u32) -> Vec<InvariantResult> {
        let results = Vec::new();
        let mut current_depth = 0;

        while !self.state_queue.is_empty() && current_depth < depth.min(self.max_depth) {
            let state_count = self.state_queue.len();

            for _ in 0..state_count {
                if let Some(current) = self.state_queue.pop() {
                    // Check for deadlock: no enabled actions
                    if current.outgoing_actions.is_empty() && current_depth < self.max_depth {
                        self.deadlock_detected = true;
                        let trace: Vec<String> = self
                            .visited_states
                            .iter()
                            .map(|s| format!("S{}", s.state_id))
                            .collect();
                        self.deadlock_trace = Some(trace);
                    }

                    // Explore each enabled action
                    for action_name in &current.outgoing_actions {
                        if let Some(action) = spec.actions.iter().find(|a| &a.name == action_name) {
                            let next_state = self.apply_action(&current, action);
                            if !self.is_visited(&next_state) {
                                self.state_queue.push(next_state.clone());
                                self.visited_states.push(next_state);
                            }
                        }
                    }
                }
            }
            current_depth += 1;
        }

        results
    }

    /// Apply a TLA+ action to generate next state
    fn apply_action(&self, state: &ModelState, action: &TlaAction) -> ModelState {
        let mut new_values = state.variable_values.clone();

        for post in &action.postconditions {
            if post == "entry_committed" {
                for (name, val) in &mut new_values {
                    if name.contains("_log") && val == "empty" {
                        *val = "committed".to_string();
                    }
                }
            }
            if post == "actor_restarted" {
                for (name, val) in &mut new_values {
                    if name.contains("_state") && val == "failed" {
                        *val = "restarted".to_string();
                    }
                }
            }
            if post == "message_processed" {
                for (name, val) in &mut new_values {
                    if name.contains("_state") && val == "idle" {
                        *val = "processing".to_string();
                    }
                }
            }
        }

        ModelState {
            state_id: self.visited_states.len() as u64,
            variable_values: new_values,
            outgoing_actions: action.postconditions.clone(),
        }
    }

    /// Check if state has been visited
    fn is_visited(&self, state: &ModelState) -> bool {
        self.visited_states
            .iter()
            .any(|s| s.variable_values == state.variable_values)
    }

    /// Generate model checking report
    pub fn generate_report(&self) -> String {
        let (passed, failed, total) = self.invariant_checker.get_summary();

        let mut report = String::from("BRICK-30 MODEL CHECKING REPORT\n");
        report.push_str("==============================\n\n");
        report.push_str(&format!("States explored: {}\n", self.visited_states.len()));
        report.push_str(&format!("Max depth: {}\n", self.max_depth));
        report.push_str(&format!("Deadlock detected: {}\n", self.deadlock_detected));

        if let Some(trace) = &self.deadlock_trace {
            report.push_str(&format!("Deadlock trace: {:?}\n", trace));
        }

        report.push_str(&format!("\nInvariants checked: {}\n", passed + failed));
        report.push_str(&format!("  Passed: {}\n", passed));
        report.push_str(&format!("  Failed: {}\n", failed));
        report.push_str(&format!("  Total states in invariant checks: {}\n", total));

        report
    }
}
