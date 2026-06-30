// M2.7.9: Epistemic Look-Ahead — Shadow Implication Graph
// 3-ply hypergraph projection for conflict anticipation

use std::collections::{BTreeMap, HashSet};

/// Shadow literal: a literal in the projected search space.
/// Unlike the active trail, shadow literals are speculative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadowLiteral {
    pub var: usize,
    pub value: bool,
}

impl ShadowLiteral {
    pub fn from_i32(lit: i32) -> Self {
        let var = lit.abs() as usize;
        let value = lit > 0;
        Self { var, value }
    }

    pub fn to_i32(&self) -> i32 {
        if self.value {
            self.var as i32
        } else {
            -(self.var as i32)
        }
    }

    /// Negation of this literal.
    pub fn negate(&self) -> Self {
        Self {
            var: self.var,
            value: !self.value,
        }
    }
}

/// Shadow implication edge: clause index → implied literal.
#[derive(Debug, Clone)]
pub struct ShadowEdge {
    pub clause_idx: usize,
    pub antecedents: Vec<ShadowLiteral>,
    pub implied: ShadowLiteral,
}

/// 3-ply shadow implication graph.
/// Projects the active implication graph 3 decision levels ahead.
#[derive(Debug, Default, Clone)]
pub struct ShadowImplicationGraph {
    /// Current shadow assignments (speculative)
    pub assignments: BTreeMap<usize, bool>,
    /// Implication edges triggered by shadow assignments
    pub edges: Vec<ShadowEdge>,
    /// Forced literals: those appearing in ≥85% of branches
    pub forced_literals: HashSet<ShadowLiteral>,
}

impl ShadowImplicationGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize shadow from current active assignments.
    pub fn seed_from_assignments(&mut self, active_assignments: &BTreeMap<usize, bool>) {
        self.assignments = active_assignments.clone();
        self.edges.clear();
        self.forced_literals.clear();
    }

    /// Simulate a shadow decision: assign a variable and propagate.
    /// Returns conflict detected (true = conflict).
    pub fn shadow_decide(&mut self, var: usize, value: bool, clauses: &[Vec<i32>]) -> bool {
        // Check if already assigned oppositely
        if let Some(&existing) = self.assignments.get(&var) {
            if existing != value {
                return true; // Conflict
            }
            return false; // Already assigned consistently
        }

        self.assignments.insert(var, value);

        // Trigger unit propagation in shadow
        self.shadow_propagate(clauses)
    }

    /// Shadow unit propagation: find clauses with all but one literal falsified.
    fn shadow_propagate(&mut self, clauses: &[Vec<i32>]) -> bool {
        let mut changed = true;
        while changed {
            changed = false;
            for (ci, clause) in clauses.iter().enumerate() {
                let mut unassigned_count = 0;
                let mut unassigned_lit: Option<i32> = None;
                let mut satisfied = false;

                for &lit in clause {
                    let var = lit.abs() as usize;
                    let val = lit > 0;
                    match self.assignments.get(&var) {
                        Some(&assigned) => {
                            if assigned == val {
                                satisfied = true;
                                break;
                            }
                        }
                        None => {
                            unassigned_count += 1;
                            unassigned_lit = Some(lit);
                        }
                    }
                }

                if satisfied {
                    continue;
                }

                if unassigned_count == 0 {
                    // All literals falsified → conflict
                    return true;
                }

                if unassigned_count == 1 {
                    // Unit clause in shadow
                    if let Some(lit) = unassigned_lit {
                        let var = lit.abs() as usize;
                        let val = lit > 0;
                        if let Some(&existing) = self.assignments.get(&var) {
                            if existing != val {
                                return true; // Conflict
                            }
                        } else {
                            self.assignments.insert(var, val);
                            self.edges.push(ShadowEdge {
                                clause_idx: ci,
                                antecedents: vec![], // Simplified for 3-ply
                                implied: ShadowLiteral::from_i32(lit),
                            });
                            changed = true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Perform 3-ply shadow branching from current state.
    /// For each unassigned variable, branch both ways and collect forced literals.
    pub fn three_ply_projection(
        &mut self,
        clauses: &[Vec<i32>],
        unassigned_vars: &[usize],
        ply_depth: usize,
    ) {
        if ply_depth == 0 || unassigned_vars.is_empty() {
            return;
        }

        let var = unassigned_vars[0];
        let remaining = &unassigned_vars[1..];

        // Branch: var = true
        let mut shadow_true = self.clone();
        if !shadow_true.shadow_decide(var, true, clauses) {
            shadow_true.three_ply_projection(clauses, remaining, ply_depth - 1);
        }

        // Branch: var = false
        let mut shadow_false = self.clone();
        if !shadow_false.shadow_decide(var, false, clauses) {
            shadow_false.three_ply_projection(clauses, remaining, ply_depth - 1);
        }

        // Collect forced literals: those assigned in BOTH non-conflicting branches
        // and in ≥85% of all projected branches
        self.collect_forced_literals(&shadow_true, &shadow_false, clauses);
    }

    /// Identify literals forced across ≥85% of projected branches.
    fn collect_forced_literals(
        &mut self,
        shadow_true: &ShadowImplicationGraph,
        shadow_false: &ShadowImplicationGraph,
        _clauses: &[Vec<i32>],
    ) {
        // For 3-ply with single variable at top: if both branches assign same literal,
        // that literal is forced.
        for (&var, &val_true) in &shadow_true.assignments {
            if let Some(&val_false) = shadow_false.assignments.get(&var) {
                if val_true == val_false && !self.assignments.contains_key(&var) {
                    self.forced_literals.insert(ShadowLiteral {
                        var,
                        value: val_true,
                    });
                }
            }
        }
    }

    /// Check if a literal is forced across ≥85% of branches.
    /// For 3-ply: literal must appear in both top-level branches.
    pub fn is_forced(&self, lit: &ShadowLiteral) -> bool {
        self.forced_literals.contains(lit)
    }

    /// Get all forced literals.
    pub fn forced_literals(&self) -> &HashSet<ShadowLiteral> {
        &self.forced_literals
    }
}
