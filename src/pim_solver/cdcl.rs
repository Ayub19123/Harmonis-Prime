use std::collections::VecDeque;

/// Trail entry recording assignment.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct TrailEntry {
    var: usize,
    value: bool,
    decision_level: usize,
    reason: Option<usize>,
}

/// Watched literal state for a clause.
#[derive(Debug, Clone)]
struct WatchedClause {
    literals: Vec<i32>,
    watch_a: usize,
    watch_b: usize,
}

/// CDCL Solver state.
pub struct CdclSolver {
    num_vars: usize,
    clauses: Vec<WatchedClause>,
    learned_clauses: Vec<WatchedClause>,
    assignment: Vec<Option<bool>>,
    trail: Vec<TrailEntry>,
    decision_level: usize,
    conflict_count: u64,
    propagate_queue: VecDeque<usize>,
    // M2.5.6: VSIDS heuristic fields
    activity: Vec<f64>,               // Per-variable activity score
    saved_phase: Vec<Option<bool>>,   // Phase saving: last assigned polarity
    var_decay: f64,                   // Activity decay factor
    // M2.5.7: Adaptive restart fields
    restart_count: u64,               // Total restarts performed
    conflicts_since_restart: u64,     // Conflicts since last restart
    luby_index: usize,                // Current position in Luby sequence
}

/// Solver result.
#[derive(Debug, Clone, PartialEq)]
pub enum SolveResult {
    Sat(Vec<bool>),
    Unsat,
}

impl CdclSolver {
    /// Build solver from DIMACS instance.
    pub fn from_dimacs(instance: &crate::pim_solver::dimacs::DimacsInstance) -> Self {
        let mut watched_clauses = Vec::new();
        for clause in &instance.clauses {
            if clause.is_empty() {
                continue; // Empty clause handled at solve time
            }
            let watch_a = 0;
            let watch_b = if clause.len() > 1 { 1 } else { 0 };
            watched_clauses.push(WatchedClause {
                literals: clause.clone(),
                watch_a,
                watch_b,
            });
        }

        let mut assignment = vec![None; instance.num_vars + 1];
        let mut propagate_queue = VecDeque::new();

        // Enforce unit clauses at level 0
        for (_ci, lit) in watched_clauses.iter().enumerate() {
            if lit.literals.len() == 1 {
                let var = lit.literals[0].abs() as usize;
                let val = lit.literals[0] > 0;
                assignment[var] = Some(val);
                propagate_queue.push_back(var);
            }
        }

        CdclSolver {
            num_vars: instance.num_vars,
            clauses: watched_clauses,
            learned_clauses: Vec::new(),
            assignment,
            trail: Vec::new(),
            decision_level: 0,
            conflict_count: 0,
            propagate_queue,
            // M2.5.6: VSIDS initialization
            activity: vec![0.0; instance.num_vars + 1],
            saved_phase: vec![None; instance.num_vars + 1],
            var_decay: 0.95,
            // M2.5.7: Restart initialization
            restart_count: 0,
            conflicts_since_restart: 0,
            luby_index: 0,
        }
    }

    /// Evaluate a literal under current assignment.
    fn eval(&self, lit: i32) -> Option<bool> {
        let var = lit.abs() as usize;
        self.assignment[var].map(|v| if lit > 0 { v } else { !v })
    }

    /// Assign a variable and record on trail.
    fn assign(&mut self, var: usize, value: bool, reason: Option<usize>) {
        self.assignment[var] = Some(value);
        self.trail.push(TrailEntry {
            var,
            value,
            decision_level: self.decision_level,
            reason,
        });
        self.propagate_queue.push_back(var);
        // M2.5.6: Save phase for phase saving
        self.saved_phase[var] = Some(value);
    }

    /// Unit propagation with two-watched literals.
    fn unit_propagate(&mut self) -> Option<usize> {
        while let Some(var) = self.propagate_queue.pop_front() {
            let num_clauses = self.clauses.len() + self.learned_clauses.len();
            for ci in 0..num_clauses {
                let (w_a_lit, w_b_lit, watch_a_idx, watch_b_idx, lits) = if ci < self.clauses.len() {
                    let c = &self.clauses[ci];
                    (c.literals[c.watch_a], c.literals[c.watch_b], c.watch_a, c.watch_b, &c.literals as &[i32])
                } else {
                    let c = &self.learned_clauses[ci - self.clauses.len()];
                    (c.literals[c.watch_a], c.literals[c.watch_b], c.watch_a, c.watch_b, &c.literals as &[i32])
                };

                // Step 1: If either watch true, clause satisfied
                if self.eval(w_a_lit) == Some(true) || self.eval(w_b_lit) == Some(true) {
                    continue;
                }

                // Step 2: Determine which watch became false
                let mut to_update = None;
                if w_a_lit.abs() as usize == var && self.eval(w_a_lit) == Some(false) {
                    to_update = Some(0usize);
                } else if w_b_lit.abs() as usize == var && self.eval(w_b_lit) == Some(false) {
                    to_update = Some(1usize);
                }
                if to_update.is_none() {
                    continue;
                }

                let other_lit = if to_update.unwrap() == 0 { w_b_lit } else { w_a_lit };

                // Step 3: If other watch true, clause satisfied
                if self.eval(other_lit) == Some(true) {
                    continue;
                }

                // Step 4: Search for new watch
                let mut new_watch = None;
                for (idx, &lit) in lits.iter().enumerate() {
                    if idx == watch_a_idx || idx == watch_b_idx {
                        continue;
                    }
                    match self.eval(lit) {
                        None | Some(true) => {
                            new_watch = Some(idx);
                            break;
                        }
                        Some(false) => continue,
                    }
                }

                // Step 5: Apply mutable update
                if let Some(nw) = new_watch {
                    let clause = if ci < self.clauses.len() {
                        &mut self.clauses[ci]
                    } else {
                        &mut self.learned_clauses[ci - self.clauses.len()]
                    };
                    if to_update.unwrap() == 0 {
                        clause.watch_a = nw;
                    } else {
                        clause.watch_b = nw;
                    }
                } else {
                    // No new watch — unit or conflict
                    if self.eval(other_lit) == Some(false) {
                        return Some(ci);
                    } else if self.eval(other_lit).is_none() {
                        let unit_var = other_lit.abs() as usize;
                        let unit_val = other_lit > 0;
                        if self.assignment[unit_var].is_none() {
                            self.assign(unit_var, unit_val, Some(ci));
                        }
                    }
                }
            }
        }
        None
    }

    /// 1-UIP conflict analysis with correct backjump level.
    fn analyze_conflict(&mut self, conflict_ci: usize) -> (Vec<i32>, usize) {
        // CRITICAL FIX: Any conflict at decision level 0 means UNSAT
        if self.decision_level == 0 {
            return (Vec::new(), 0);
        }

        let conflict_clause = if conflict_ci < self.clauses.len() {
            &self.clauses[conflict_ci].literals.clone()
        } else {
            &self.learned_clauses[conflict_ci - self.clauses.len()].literals.clone()
        };

        let mut learned = conflict_clause.clone();
        let mut current_level_count = learned.iter().filter(|&&lit| {
            let var = lit.abs() as usize;
            self.trail.iter().find(|e| e.var == var).map_or(false, |e| {
                e.decision_level == self.decision_level
            })
        }).count();

        let mut idx = self.trail.len();
        while current_level_count > 1 && idx > 0 {
            idx -= 1;
            let entry = self.trail[idx];
            if entry.decision_level != self.decision_level {
                continue;
            }
            if entry.reason.is_none() {
                continue; // Decision variable, not propagated
            }

            // Find the literal in learned clause that matches this variable
            let var = entry.var;
            if !learned.iter().any(|&lit| lit.abs() as usize == var) {
                continue;
            }

            // Resolve with reason clause
            let reason_ci = entry.reason.unwrap();
            let reason_clause = if reason_ci < self.clauses.len() {
                self.clauses[reason_ci].literals.clone()
            } else {
                self.learned_clauses[reason_ci - self.clauses.len()].literals.clone()
            };

            // Resolution: learned = learned ∪ reason \ {var, -var}
            let mut new_learned = Vec::new();
            for &lit in &learned {
                if lit.abs() as usize != var {
                    new_learned.push(lit);
                }
            }
            for &lit in &reason_clause {
                if lit.abs() as usize != var && !new_learned.contains(&lit) {
                    new_learned.push(lit);
                }
            }
            learned = new_learned;

            current_level_count = learned.iter().filter(|&&lit| {
                let v = lit.abs() as usize;
                self.trail.iter().find(|e| e.var == v).map_or(false, |e| {
                    e.decision_level == self.decision_level
                })
            }).count();
        }

        // Compute backjump level: highest decision level in learned clause below current
        let mut backjump_level = 0;
        for &lit in &learned {
            let var = lit.abs() as usize;
            if let Some(entry) = self.trail.iter().find(|e| e.var == var) {
                if entry.decision_level < self.decision_level && entry.decision_level > backjump_level {
                    backjump_level = entry.decision_level;
                }
            }
        }

        // M2.5.6: Bump activity for variables in learned clause
        for &lit in &learned {
            let var = lit.abs() as usize;
            self.activity[var] += 1.0;
        }
        // M2.5.6: Decay all activities
        for a in self.activity.iter_mut().skip(1) {
            *a *= self.var_decay;
        }

        (learned, backjump_level)
    }

    /// Backjump to target decision level.
    fn backjump(&mut self, target_level: usize) {
        while let Some(entry) = self.trail.last() {
            if entry.decision_level <= target_level {
                break;
            }
            let var = entry.var;
            self.assignment[var] = None;
            self.trail.pop();
        }
        self.decision_level = target_level;
        self.propagate_queue.clear();
    }

    /// Enqueue unit clauses from learned clauses at level 0.
    fn enqueue_unit_clauses(&mut self) {
        let unit_clauses: Vec<(usize, i32)> = self.learned_clauses.iter().enumerate()
            .filter(|(_, c)| c.literals.len() == 1)
            .map(|(ci, c)| (ci + self.clauses.len(), c.literals[0]))
            .collect();
        for (ci, lit) in unit_clauses {
            let var = lit.abs() as usize;
            if self.assignment[var].is_none() {
                let val = lit > 0;
                self.assign(var, val, Some(ci));
            }
        }
    }

    // M2.5.6: VSIDS heuristic methods

    /// VSIDS variable selection: highest activity unassigned variable.
    /// Tie-break by variable index for determinism.
    /// Phase saving: use last assigned polarity if available.
    fn pick_branch_var(&self) -> Option<(usize, bool)> {
        let mut best_var: Option<usize> = None;
        let mut best_score: f64 = -1.0;

        for v in 1..=self.num_vars {
            if self.assignment[v].is_none() {
                let score = self.activity[v];
                if score > best_score || (score == best_score && best_var.map_or(true, |bv| v < bv)) {
                    best_score = score;
                    best_var = Some(v);
                }
            }
        }

        best_var.map(|v| (v, self.saved_phase[v].unwrap_or(true)))
    }

    // M2.5.7: Adaptive restart methods

    /// Luby sequence for restart scheduling.
    /// Returns the n-th Luby number (0-indexed).
    /// Sequence: 1, 1, 2, 1, 1, 2, 4, 1, 1, 2, 1, 1, 2, 4, 8, ...
    fn luby(x: usize) -> u64 {
        let mut size = 1;
        let mut seq = 0;
        while size < x + 1 {
            size = 2 * size + 1;
            seq += 1;
        }
        let mut x = x;
        while size - 1 != x {
            size = (size - 1) >> 1;
            seq -= 1;
            x = x % size;
        }
        1u64 << seq
    }

    /// Compute current restart threshold from Luby sequence.
    /// Scaled by 100 conflicts per unit.
    fn restart_threshold(&self) -> u64 {
        Self::luby(self.luby_index) * 100
    }

    /// Perform a restart: clear trail, reset decision level, keep learned clauses and activities.
    fn restart(&mut self) {
        self.restart_count += 1;
        self.conflicts_since_restart = 0;
        self.luby_index += 1;

        // Clear all search state
        for v in 1..=self.num_vars {
            self.assignment[v] = None;
        }
        self.trail.clear();
        self.propagate_queue.clear();
        self.decision_level = 0;

        // --- FIX: Collect unit literals first to avoid borrow conflict ---
        let mut unit_lits: Vec<i32> = Vec::new();
        for clause in &self.clauses {
            if clause.literals.len() == 1 {
                unit_lits.push(clause.literals[0]);
            }
        }

        // Assign original unit clauses at level 0
        for lit in unit_lits {
            let var = lit.abs() as usize;
            if self.assignment[var].is_none() {
                let val = lit > 0;
                self.assign(var, val, None);
            }
        }

        // Re-enforce learned unit clauses at level 0
        self.enqueue_unit_clauses();
    }

    /// Main CDCL solve loop.
    pub fn solve(&mut self) -> SolveResult {
        // Enqueue initial unit clauses before propagation
        self.enqueue_unit_clauses();

        // Initial propagation at level 0
        if let Some(_ci) = self.unit_propagate() {
            return SolveResult::Unsat;
        }

        loop {
            // Check if all variables assigned
            if self.trail.len() == self.num_vars {
                let model = (1..=self.num_vars)
                    .map(|v| self.assignment[v].unwrap_or(false))
                    .collect();
                return SolveResult::Sat(model);
            }

            // Make a decision using VSIDS + phase saving
            let (var, phase) = match self.pick_branch_var() {
                Some(vp) => vp,
                None => {
                    let model = (1..=self.num_vars)
                        .map(|v| self.assignment[v].unwrap_or(false))
                        .collect();
                    return SolveResult::Sat(model);
                }
            };

            self.decision_level += 1;
            self.assign(var, phase, None);

            // Propagate after decision
            if let Some(ci) = self.unit_propagate() {
                self.conflict_count += 1;
                self.conflicts_since_restart += 1;
                let (learned, backjump_level) = self.analyze_conflict(ci);

                if learned.is_empty() {
                    return SolveResult::Unsat;
                }

                // Add learned clause
                let w_a = 0;
                let w_b = if learned.len() > 1 { 1 } else { 0 };
                self.learned_clauses.push(WatchedClause {
                    literals: learned,
                    watch_a: w_a,
                    watch_b: w_b,
                });

                // Backjump
                self.backjump(backjump_level);

                // Enqueue unit literal from learned clause
                let learned_ci = self.clauses.len() + self.learned_clauses.len() - 1;
                let learned_clause = if learned_ci < self.clauses.len() {
                    &self.clauses[learned_ci]
                } else {
                    &self.learned_clauses[learned_ci - self.clauses.len()]
                };
                if learned_clause.literals.len() == 1 {
                    let lit = learned_clause.literals[0];
                    let unit_var = lit.abs() as usize;
                    if self.assignment[unit_var].is_none() {
                        let unit_val = lit > 0;
                        self.assign(unit_var, unit_val, Some(learned_ci));
                    }
                }

                // Propagate learned unit and check for level-0 conflict
                if let Some(ci2) = self.unit_propagate() {
                    self.conflict_count += 1;
                    self.conflicts_since_restart += 1;
                    let (learned2, _backjump2) = self.analyze_conflict(ci2);

                    if self.decision_level == 0 {
                        return SolveResult::Unsat;
                    }

                    if learned2.is_empty() {
                        return SolveResult::Unsat;
                    }

                    // Add recursive learned clause and continue
                    let w_a2 = 0;
                    let w_b2 = if learned2.len() > 1 { 1 } else { 0 };
                    self.learned_clauses.push(WatchedClause {
                        literals: learned2,
                        watch_a: w_a2,
                        watch_b: w_b2,
                    });
                }

                // M2.5.7: Check if restart is needed
                if self.conflicts_since_restart >= self.restart_threshold() {
                    self.restart();
                    // After restart, propagate any unit clauses
                    if let Some(_ci) = self.unit_propagate() {
                        return SolveResult::Unsat;
                    }
                }
            }
        }
    }
}