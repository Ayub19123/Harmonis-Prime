//! BRICK-50 Pillar 5: Dynamic Test-Time Scaling

#[derive(Clone)]
pub struct HypothesisBranch {
    pub id: String,
    pub confidence: f64,
    pub depth: usize,
    pub verified: bool,
}

pub struct TestTimeScaler {
    branches: Vec<HypothesisBranch>,
    max_depth: usize,
    backtracks: u64,
    verified_solutions: u64,
}

impl TestTimeScaler {
    pub fn new(max_depth: usize) -> Self {
        Self {
            branches: Vec::new(),
            max_depth: max_depth.max(1),
            backtracks: 0,
            verified_solutions: 0,
        }
    }

    pub fn explore(&mut self, problem_id: &str, initial_confidence: f64) -> Vec<HypothesisBranch> {
        let mut new_branches = Vec::new();
        let branch_count = (self.max_depth * 2).max(3);
        for i in 0..branch_count {
            let confidence = initial_confidence * (1.0 - (i as f64 * 0.05)).clamp(0.1, 1.0);
            let branch = HypothesisBranch {
                id: format!("{}_branch_{}", problem_id, i),
                confidence,
                depth: i + 1,
                verified: false,
            };
            new_branches.push(branch.clone());
            self.branches.push(branch);
        }
        new_branches
    }

    pub fn verify_and_refine(&mut self, branch_id: &str) -> bool {
        if let Some(branch) = self.branches.iter_mut().find(|b| b.id == branch_id) {
            if branch.confidence >= 0.95 {
                branch.verified = true;
                self.verified_solutions += 1;
                true
            } else {
                self.backtracks += 1;
                branch.confidence = (branch.confidence + 0.1).min(1.0);
                false
            }
        } else {
            false
        }
    }

    pub fn coverage(&self) -> f64 {
        if self.branches.is_empty() {
            return 0.0;
        }
        let verified = self.branches.iter().filter(|b| b.verified).count() as f64;
        verified / self.branches.len() as f64
    }

    pub fn stats(&self) -> (u64, u64, f64) {
        (self.backtracks, self.verified_solutions, self.coverage())
    }
}
