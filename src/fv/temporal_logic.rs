use std::collections::HashMap;

/// Temporal logic formula types (LTL/CTL)
#[derive(Debug, Clone)]
pub enum TemporalFormula {
    True,
    False,
    Atomic(String),
    Not(Box<TemporalFormula>),
    And(Box<TemporalFormula>, Box<TemporalFormula>),
    Or(Box<TemporalFormula>, Box<TemporalFormula>),
    Implies(Box<TemporalFormula>, Box<TemporalFormula>),
    Next(Box<TemporalFormula>),
    Eventually(Box<TemporalFormula>),
    Always(Box<TemporalFormula>),
    Until(Box<TemporalFormula>, Box<TemporalFormula>),
}

/// Evaluates temporal logic formulas over state traces
pub struct TemporalLogicEvaluator {
    pub trace: Vec<HashMap<String, String>>,
    pub current_index: usize,
}

impl TemporalLogicEvaluator {
    pub fn new(trace: Vec<HashMap<String, String>>) -> Self {
        Self {
            trace,
            current_index: 0,
        }
    }

    /// Evaluate a formula at the current position in the trace
    pub fn evaluate(&self, formula: &TemporalFormula) -> bool {
        self.evaluate_at(formula, self.current_index)
    }

    fn evaluate_at(&self, formula: &TemporalFormula, index: usize) -> bool {
        match formula {
            TemporalFormula::True => true,
            TemporalFormula::False => false,
            TemporalFormula::Atomic(prop) => {
                if index < self.trace.len() {
                    self.trace[index]
                        .get(prop)
                        .map(|v| v == "true")
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            TemporalFormula::Not(f) => !self.evaluate_at(f, index),
            TemporalFormula::And(f1, f2) => {
                self.evaluate_at(f1, index) && self.evaluate_at(f2, index)
            }
            TemporalFormula::Or(f1, f2) => {
                self.evaluate_at(f1, index) || self.evaluate_at(f2, index)
            }
            TemporalFormula::Implies(f1, f2) => {
                !self.evaluate_at(f1, index) || self.evaluate_at(f2, index)
            }
            TemporalFormula::Next(f) => {
                if index + 1 < self.trace.len() {
                    self.evaluate_at(f, index + 1)
                } else {
                    false // No next state
                }
            }
            TemporalFormula::Eventually(f) => {
                for i in index..self.trace.len() {
                    if self.evaluate_at(f, i) {
                        return true;
                    }
                }
                false
            }
            TemporalFormula::Always(f) => {
                for i in index..self.trace.len() {
                    if !self.evaluate_at(f, i) {
                        return false;
                    }
                }
                true
            }
            TemporalFormula::Until(f1, f2) => {
                for i in index..self.trace.len() {
                    if self.evaluate_at(f2, i) {
                        return true;
                    }
                    if !self.evaluate_at(f1, i) {
                        return false;
                    }
                }
                false
            }
        }
    }

    /// Check liveness: Something good eventually happens
    pub fn check_liveness(&self, good_property: &str) -> bool {
        let formula = TemporalFormula::Eventually(Box::new(TemporalFormula::Atomic(
            good_property.to_string(),
        )));
        self.evaluate(&formula)
    }

    /// Check safety: Nothing bad ever happens
    pub fn check_safety(&self, bad_property: &str) -> bool {
        let formula = TemporalFormula::Always(Box::new(TemporalFormula::Not(Box::new(
            TemporalFormula::Atomic(bad_property.to_string()),
        ))));
        self.evaluate(&formula)
    }

    /// Verify response property: Every request is eventually acknowledged
    pub fn check_response(&self, request: &str, acknowledge: &str) -> bool {
        let formula = TemporalFormula::Always(Box::new(TemporalFormula::Implies(
            Box::new(TemporalFormula::Atomic(request.to_string())),
            Box::new(TemporalFormula::Eventually(Box::new(
                TemporalFormula::Atomic(acknowledge.to_string()),
            ))),
        )));
        self.evaluate(&formula)
    }
}
