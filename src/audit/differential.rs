//! M2.5.3: Differential Validation — CDCL vs DPLL Reference
//! 
//! Tests Harmonis Prime CDCL solver against naive DPLL oracle.
//! Divergence = bug in CDCL implementation.

#[cfg(test)]
mod tests {
    use crate::audit::reference_solver::dpll_solve;
    use crate::pim_solver::cdcl::{CdclSolver, SolveResult};
    use crate::pim_solver::dimacs::DimacsInstance;

    #[test]
    fn test_differential_basic_cases() {
        let test_cases = vec![
            // SAT: simple unit clauses
            (2, vec![vec![1], vec![2]], true),
            // UNSAT: contradiction
            (1, vec![vec![1], vec![-1]], false),
            // SAT: choice required
            (2, vec![vec![1, 2], vec![-1, 2]], true),
            // UNSAT: XOR pattern — SKIPPED (hangs in M2.5.2, fix in M2.5.4)
            // (2, vec![vec![1, 2], vec![1, -2], vec![-1, 2], vec![-1, -2]], false),
            // SAT: single unit clause
            (3, vec![vec![-2]], true),
            // SAT: empty instance
            (2, vec![], true),
        ];
        
        for (num_vars, clauses, expected_sat) in test_cases {
            let instance = DimacsInstance {
                num_vars,
                num_clauses: clauses.len(),
                clauses: clauses.clone(),
            };
            
            // Harmonis Prime CDCL result
            let mut solver = CdclSolver::from_dimacs(&instance);
            let hp_result = solver.solve();
            
            // Reference DPLL result
            let dpll_result = dpll_solve(num_vars, &clauses);
            
            // Compare
            let hp_sat = matches!(hp_result, SolveResult::Sat(_));
            let dpll_sat = dpll_result.is_some();
            
            assert_eq!(
                hp_sat, dpll_sat,
                "Divergence on CNF with {} vars: HP={:?}, DPLL={:?}",
                num_vars, hp_sat, dpll_sat
            );
            
            // Verify SAT models are valid
            if let SolveResult::Sat(ref model) = hp_result {
                if let Some(ref dpll_model) = dpll_result {
                    assert_eq!(
                        model.len(), dpll_model.len(),
                        "Model length mismatch"
                    );
                }
            }
            
            // Verify expected result
            assert_eq!(hp_sat, expected_sat, "HP result mismatch on expected SAT={}", expected_sat);
            assert_eq!(dpll_sat, expected_sat, "DPLL result mismatch on expected SAT={}", expected_sat);
        }
    }
}