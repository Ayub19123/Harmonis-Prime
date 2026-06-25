//! M2.5.3: Embedded Reference DPLL Solver
//! 
//! NOT CLAIMED: This is a naive recursive DPLL, not optimized.
//! NOT CLAIMED: This replaces MiniSat (it does not — MiniSat is production-grade).
//! 
//! PURPOSE: Provide a trusted, embeddable oracle for differential testing.
//! If Harmonis Prime (CDCL) disagrees with Reference (DPLL), one is wrong.
//! 
//! HONEST CONSTRAINT: Reference solver is O(2^n) — only for small test CNFs (<50 vars).

/// Solve CNF using naive recursive DPLL. Returns Some(model) if SAT, None if UNSAT.
pub fn dpll_solve(num_vars: usize, clauses: &[Vec<i32>]) -> Option<Vec<bool>> {
    let mut assignment = vec![None; num_vars + 1];
    dpll_recursive(&mut assignment, clauses, 1)
}

fn dpll_recursive(
    assignment: &mut Vec<Option<bool>>,
    clauses: &[Vec<i32>],
    start_var: usize,
) -> Option<Vec<bool>> {
    // Check for conflicts
    for clause in clauses {
        if is_clause_false(clause, assignment) {
            return None;
        }
    }
    
    // Check if all clauses satisfied
    if clauses.iter().all(|c| is_clause_satisfied(c, assignment)) {
        return Some((1..assignment.len()).map(|v| assignment[v].unwrap_or(false)).collect());
    }
    
    // Find next unassigned variable
    let var = (start_var..assignment.len()).find(|&v| assignment[v].is_none())?;
    
    // Try true
    assignment[var] = Some(true);
    if let Some(model) = dpll_recursive(assignment, clauses, var + 1) {
        return Some(model);
    }
    
    // Try false
    assignment[var] = Some(false);
    if let Some(model) = dpll_recursive(assignment, clauses, var + 1) {
        return Some(model);
    }
    
    // Backtrack
    assignment[var] = None;
    None
}

fn is_clause_satisfied(clause: &[i32], assignment: &[Option<bool>]) -> bool {
    clause.iter().any(|&lit| {
        let var = lit.abs() as usize;
        match assignment[var] {
            Some(v) if lit > 0 => v,
            Some(v) => !v,
            None => false,
        }
    })
}

fn is_clause_false(clause: &[i32], assignment: &[Option<bool>]) -> bool {
    clause.iter().all(|&lit| {
        let var = lit.abs() as usize;
        match assignment[var] {
            Some(v) if lit > 0 => !v,
            Some(v) => v,
            None => false,
        }
    })
}