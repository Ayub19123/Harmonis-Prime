//! BRICK-51 Layer 1: Goal Registry
//! Global goal propagation and convergence tracking
//! CMF-513: 95% of goals converge within 5 seconds

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Goal {
    pub id: String,
    pub priority: f64,
    pub converged: bool,
    pub convergence_time_ms: u64,
}

pub struct GoalRegistry {
    goals: HashMap<String, Goal>,
    converged_count: u64,
    total_count: u64,
}

impl GoalRegistry {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            converged_count: 0,
            total_count: 0,
        }
    }

    pub fn propose(&mut self, id: &str, priority: f64) {
        let goal = Goal {
            id: id.to_string(),
            priority,
            converged: false,
            convergence_time_ms: 0,
        };
        self.goals.insert(id.to_string(), goal);
        self.total_count += 1;
    }

    pub fn converge(&mut self, id: &str, time_ms: u64) -> bool {
        if let Some(goal) = self.goals.get_mut(id) {
            goal.converged = true;
            goal.convergence_time_ms = time_ms;
            self.converged_count += 1;
            true
        } else {
            false
        }
    }

    pub fn convergence_rate(&self) -> f64 {
        if self.total_count == 0 {
            return 1.0;
        }
        self.converged_count as f64 / self.total_count as f64
    }

    pub fn fast_convergence_rate(&self, threshold_ms: u64) -> f64 {
        let fast = self
            .goals
            .values()
            .filter(|g| g.converged && g.convergence_time_ms <= threshold_ms)
            .count() as u64;
        if self.total_count == 0 {
            return 1.0;
        }
        fast as f64 / self.total_count as f64
    }

    pub fn stats(&self) -> (u64, u64, f64, f64) {
        (
            self.total_count,
            self.converged_count,
            self.convergence_rate(),
            self.fast_convergence_rate(5000),
        )
    }
}
