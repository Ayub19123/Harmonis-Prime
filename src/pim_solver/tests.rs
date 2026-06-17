//! SET-7A: PIM 3-SAT Solver Invariant Tests
//! 
//! Invariants:
//!   - test_clause_evaluation_parallel: O(1) physical bound for fixed crossbar
//!   - test_energy_minimization_converges: global minimum detection (E=0)
//!   - test_crossbar_area_scaling: O(m*n) memory footprint
//!   - test_programming_time_bound: O(m) programming time

use crate::pim_solver::solver::*;

// --- Invariant 1: Parallel Evaluation (Physical O(1) for Fixed Crossbar) ---

#[test]
fn test_clause_evaluation_parallel() {
    let config = CrossbarConfig::new(50, 20).unwrap();
    let mut solver = PimSolver::new(config);
    
    // Program 50 clauses
    for i in 0..50 {
        solver.add_clause(Clause::new(&[(i % 20, false)])).unwrap();
    }
    
    let assignment = vec![VariableAssignment::True; 20];
    
    // Evaluation should be single-pass regardless of clause count
    // (simulated: we verify all clauses are evaluated)
    let state = solver.evaluate_parallel(&assignment);
    assert_eq!(state.satisfied_clauses, 50);
    assert_eq!(state.violated_clauses, 0);
    assert_eq!(state.total_energy, 0);
}

// --- Invariant 2: Energy Minimization Converges ---

#[test]
fn test_energy_minimization_converges() {
    let config = CrossbarConfig::new(3, 2).unwrap();
    let mut solver = PimSolver::new(config);
    
    // Simple 2-SAT: (x0 OR x1) AND (NOT(x0) OR x1)
    solver.add_clause(Clause::new(&[(0, false), (1, false)])).unwrap();
    solver.add_clause(Clause::new(&[(0, true), (1, false)])).unwrap();
    
    // Brute force to find satisfying assignment
    let solution = solver.brute_force_solve();
    assert!(solution.is_some(), "Formula should be satisfiable");
    
    let assignment = solution.unwrap();
    let state = solver.evaluate_parallel(&assignment);
    assert_eq!(state.total_energy, 0, "Global minimum (E=0) must be reachable");
}

#[test]
fn test_unsatisfiable_formula_high_energy() {
    let config = CrossbarConfig::new(2, 1).unwrap();
    let mut solver = PimSolver::new(config);
    
    // x0 AND NOT(x0) — unsatisfiable
    solver.add_clause(Clause::new(&[(0, false)])).unwrap();
    solver.add_clause(Clause::new(&[(0, true)])).unwrap();
    
    let assignment = vec![VariableAssignment::True];
    let state = solver.evaluate_parallel(&assignment);
    assert_eq!(state.total_energy, 1, "One clause always violated");
    
    let assignment2 = vec![VariableAssignment::False];
    let state2 = solver.evaluate_parallel(&assignment2);
    assert_eq!(state2.total_energy, 1, "One clause always violated");
}

// --- Invariant 3: Crossbar Area Scaling ---

#[test]
fn test_crossbar_area_scaling() {
    // Verify O(m*n) scaling
    let config_small = CrossbarConfig::new(10, 10).unwrap();
    let config_large = CrossbarConfig::new(100, 100).unwrap();
    
    assert_eq!(config_small.area_units(), 100);
    assert_eq!(config_large.area_units(), 10000);
    
    // 10x increase in each dimension = 100x area
    assert_eq!(
        config_large.area_units() / config_small.area_units(),
        100
    );
}

// --- Invariant 4: Programming Time Bound ---

#[test]
fn test_programming_time_bound() {
    let config = CrossbarConfig::new(100, 50).unwrap();
    let mut solver = PimSolver::new(config);
    
    // Programming: O(m) — one step per clause
    for i in 0..100 {
        solver.add_clause(Clause::new(&[(i % 50, false)])).unwrap();
    }
    
    // Programming steps must equal clause count
    assert_eq!(solver.config.programming_steps(), 100);
    assert_eq!(solver.clauses.len(), 100);
}

#[test]
fn test_crossbar_capacity_enforced() {
    let config = CrossbarConfig::new(2, 5).unwrap();
    let mut solver = PimSolver::new(config);
    
    solver.add_clause(Clause::new(&[(0, false)])).unwrap();
    solver.add_clause(Clause::new(&[(1, false)])).unwrap();
    
    // Third clause should fail — crossbar full
    let result = solver.add_clause(Clause::new(&[(2, false)]));
    assert!(result.is_err(), "Crossbar capacity must be enforced");
}