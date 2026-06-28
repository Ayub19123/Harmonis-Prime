//! SET-7A: PIM Crossbar Simulation - Energy-Based k-SAT Evaluation
//!
//! Phase 1 (Software): Simulated crossbar with deterministic evaluation
//! Phase 2 (Silicon): Physical PIM array with analog threshold detection
//!
//! Physical Parallelism Principle:
//!   All clauses evaluated simultaneously via Kirchhoff's Current Law.
//!   For fixed crossbar: O(1) evaluation time vs clause count.
//!   Area cost: O(m*n). Programming cost: O(m). This is physics, not magic.

/// Boolean variable assignment: true, false, or unassigned
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VariableAssignment {
    True,
    False,
    Unassigned,
}

/// A single clause in k-SAT: disjunction of literals
/// Each literal is (variable_index, is_negated)
#[derive(Debug, Clone, PartialEq)]
pub struct Clause {
    pub literals: Vec<(usize, bool)>, // (var_index, negated)
}

impl Clause {
    /// Create a new clause from literal tuples
    pub fn new(literals: &[(usize, bool)]) -> Self {
        Self {
            literals: literals.to_vec(),
        }
    }

    /// Evaluate clause under given assignment
    /// Returns true if at least one literal is satisfied
    pub fn evaluate(&self, assignment: &[VariableAssignment]) -> bool {
        self.literals.iter().any(|(var_idx, negated)| {
            match assignment.get(*var_idx) {
                Some(VariableAssignment::True) => !negated,
                Some(VariableAssignment::False) => *negated,
                _ => false, // Unassigned = not satisfied
            }
        })
    }

    /// Penalty function: 0 if satisfied, 1 if violated
    /// Used in energy minimization formulation
    pub fn penalty(&self, assignment: &[VariableAssignment]) -> u64 {
        if self.evaluate(assignment) {
            0
        } else {
            1
        }
    }
}

/// Crossbar configuration: physical dimensions and constraints
#[derive(Debug, Clone, PartialEq)]
pub struct CrossbarConfig {
    pub rows: usize,          // m = number of clauses
    pub columns: usize,       // n = number of variables
    pub max_clauses: usize,   // physical limit
    pub max_variables: usize, // physical limit
}

impl CrossbarConfig {
    /// Create crossbar config with validation
    pub fn new(rows: usize, columns: usize) -> Result<Self, &'static str> {
        if rows == 0 || columns == 0 {
            return Err("Crossbar dimensions must be positive");
        }
        Ok(Self {
            rows,
            columns,
            max_clauses: rows,
            max_variables: columns,
        })
    }

    /// Physical area estimate: proportional to m * n
    pub fn area_units(&self) -> usize {
        self.rows * self.columns
    }

    /// Programming time estimate: O(m) - one row per clause
    pub fn programming_steps(&self) -> usize {
        self.rows
    }
}

/// Energy state of the PIM system
/// E(v) = sum of clause penalties (Ising/QUBO formulation)
#[derive(Debug, Clone, PartialEq)]
pub struct EnergyState {
    pub total_energy: u64, // sum of all clause penalties
    pub satisfied_clauses: usize,
    pub violated_clauses: usize,
    pub assignment: Vec<VariableAssignment>,
}

/// PIM SAT Solver - simulated crossbar evaluation
#[derive(Debug, Clone)]
pub struct PimSolver {
    pub config: CrossbarConfig,
    pub clauses: Vec<Clause>,
}

impl PimSolver {
    /// Create solver with given crossbar dimensions
    pub fn new(config: CrossbarConfig) -> Self {
        let capacity = config.rows;
        Self {
            config,
            clauses: Vec::with_capacity(capacity),
        }
    }
    /// Add a clause to the crossbar (programming phase: O(1) per clause)
    pub fn add_clause(&mut self, clause: Clause) -> Result<(), &'static str> {
        if self.clauses.len() >= self.config.max_clauses {
            return Err("Crossbar capacity exceeded");
        }
        if clause
            .literals
            .iter()
            .any(|(idx, _)| *idx >= self.config.columns)
        {
            return Err("Variable index exceeds crossbar width");
        }
        self.clauses.push(clause);
        Ok(())
    }

    /// Evaluate ALL clauses simultaneously (simulated physical parallelism)
    /// For fixed crossbar: O(1) with respect to clause count
    /// In software simulation: we iterate, but the algorithmic complexity
    /// of the physical operation is O(1) for fixed hardware
    pub fn evaluate_parallel(&self, assignment: &[VariableAssignment]) -> EnergyState {
        let mut total_energy: u64 = 0;
        let mut satisfied = 0;
        let mut violated = 0;

        // Physical parallelism simulation:
        // In real PIM, all rows evaluate simultaneously via KCL
        for clause in &self.clauses {
            let penalty = clause.penalty(assignment);
            total_energy += penalty;
            if penalty == 0 {
                satisfied += 1;
            } else {
                violated += 1;
            }
        }

        EnergyState {
            total_energy,
            satisfied_clauses: satisfied,
            violated_clauses: violated,
            assignment: assignment.to_vec(),
        }
    }

    /// Check if formula is satisfied (energy = 0)
    pub fn is_satisfied(&self, assignment: &[VariableAssignment]) -> bool {
        self.evaluate_parallel(assignment).total_energy == 0
    }

    /// Brute-force search for satisfying assignment (exponential - for validation only)
    /// This is the classical algorithm, NOT the PIM physical operation
    pub fn brute_force_solve(&self) -> Option<Vec<VariableAssignment>> {
        let n = self.config.columns;
        if n > 20 {
            return None; // Too large for brute force
        }

        // Try all 2^n assignments
        for bits in 0..(1usize << n) {
            let mut assignment = vec![VariableAssignment::Unassigned; n];
            for i in 0..n {
                assignment[i] = if (bits >> i) & 1 == 1 {
                    VariableAssignment::True
                } else {
                    VariableAssignment::False
                };
            }

            if self.is_satisfied(&assignment) {
                return Some(assignment);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clause_evaluation_basic() {
        // Clause: x0 OR NOT(x1)
        let clause = Clause::new(&[(0, false), (1, true)]);

        // x0=true, x1=false: satisfied (x0 is true)
        let assignment = vec![VariableAssignment::True, VariableAssignment::False];
        assert!(clause.evaluate(&assignment));
        assert_eq!(clause.penalty(&assignment), 0);

        // x0=false, x1=true: satisfied (NOT(x1) is false, but x0 is false... wait)
        // x0=false, x1=true: NOT(x1)=false, x0=false -> clause is FALSE
        let assignment2 = vec![VariableAssignment::False, VariableAssignment::True];
        assert!(!clause.evaluate(&assignment2));
        assert_eq!(clause.penalty(&assignment2), 1);
    }

    #[test]
    fn test_crossbar_area_scaling() {
        let config = CrossbarConfig::new(100, 50).unwrap();
        assert_eq!(config.area_units(), 5000); // O(m*n)
        assert_eq!(config.programming_steps(), 100); // O(m)
    }

    #[test]
    fn test_solver_parallel_evaluation() {
        let config = CrossbarConfig::new(3, 2).unwrap();
        let mut solver = PimSolver::new(config);

        // (x0 OR x1) AND (NOT(x0) OR x1) AND (x0 OR NOT(x1))
        solver
            .add_clause(Clause::new(&[(0, false), (1, false)]))
            .unwrap();
        solver
            .add_clause(Clause::new(&[(0, true), (1, false)]))
            .unwrap();
        solver
            .add_clause(Clause::new(&[(0, false), (1, true)]))
            .unwrap();

        // Test x0=true, x1=true
        let assignment = vec![VariableAssignment::True, VariableAssignment::True];
        let state = solver.evaluate_parallel(&assignment);

        // Clause 1: true OR true = true (penalty 0)
        // Clause 2: false OR true = true (penalty 0)
        // Clause 3: true OR false = true (penalty 0)
        assert_eq!(state.total_energy, 0);
        assert_eq!(state.satisfied_clauses, 3);
        assert!(solver.is_satisfied(&assignment));
    }
}
