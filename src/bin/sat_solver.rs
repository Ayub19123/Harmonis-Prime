//! M2.6: SAT Solver CLI Binary — Competition Entry Point
//!
//! Usage: cargo run --bin sat_solver <input.cnf> [--proof <file.drat>] [--mem-profile] [--clause-db-stats]
//!
//! Exit codes:
//!   10 = SATISFIABLE
//!   20 = UNSATISFIABLE
//!   1  = ERROR (parse failure, I/O error, etc.)

use sovereign_core::pim_solver::{CdclSolver, DimacsInstance, SolveResult};

fn print_usage(program: &str) {
    eprintln!("Usage: {} <input.cnf> [--proof <file.drat>] [--mem-profile] [--clause-db-stats]", program);
    eprintln!("  --proof <file.drat>    Write DRAT proof trace for UNSAT instances");
    eprintln!("  --mem-profile          Print memory telemetry after solving");
    eprintln!("  --clause-db-stats      Print clause database statistics");
    eprintln!("Exit codes: 10 = SAT, 20 = UNSAT, 1 = ERROR");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];

    if args.len() < 2 {
        print_usage(program);
        std::process::exit(1);
    }

    let cnf_path = &args[1];
    let mut proof_path: Option<String> = None;
    let mut mem_profile = false;
    let mut clause_db_stats = false;

    // Parse optional flags
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--proof" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("ERROR: --proof requires a file path");
                    print_usage(program);
                    std::process::exit(1);
                }
                proof_path = Some(args[i].clone());
            }
            "--mem-profile" => mem_profile = true,
            "--clause-db-stats" => clause_db_stats = true,
            other => {
                eprintln!("ERROR: Unknown flag: {}", other);
                print_usage(program);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Parse DIMACS CNF
    let instance = match DimacsInstance::parse(cnf_path) {
        Ok(inst) => inst,
        Err(e) => {
            eprintln!("ERROR: Failed to parse {}: {:?}", cnf_path, e);
            std::process::exit(1);
        }
    };

    // Solve
    let mut solver = CdclSolver::from_dimacs(&instance);
    let result = solver.solve();

    match result {
        SolveResult::Sat(model) => {
            println!("s SATISFIABLE");
            // Output model in DIMACS format: v <assignments> 0
            print!("v");
            for (var_idx, &value) in model.iter().enumerate() {
                let lit = if value { (var_idx + 1) as i32 } else { -((var_idx + 1) as i32) };
                print!(" {}", lit);
            }
            println!(" 0");

            // Telemetry output
            if mem_profile || clause_db_stats {
                print_telemetry(&solver, mem_profile, clause_db_stats);
            }

            std::process::exit(10);
        }
        SolveResult::Unsat => {
            println!("s UNSATISFIABLE");

            // Write DRAT proof if requested
            if let Some(path) = proof_path {
                if let Err(e) = solver.write_proof(&path) {
                    eprintln!("ERROR: Failed to write proof to {}: {}", path, e);
                    std::process::exit(1);
                }
                eprintln!("c Proof written to: {}", path);
            }

            // Telemetry output
            if mem_profile || clause_db_stats {
                print_telemetry(&solver, mem_profile, clause_db_stats);
            }

            std::process::exit(20);
        }
    }
}

fn print_telemetry(solver: &CdclSolver, mem_profile: bool, clause_db_stats: bool) {
    let telemetry = solver.telemetry();

    if clause_db_stats {
        eprintln!("c Clause DB size:        {}", telemetry.clause_db_size);
        eprintln!("c Learned clauses:       {}", telemetry.learned_clause_count);
        eprintln!("c Conflict rate:         {:.4}", telemetry.conflict_rate);
        eprintln!("c Decisions:             {}", telemetry.decision_count);
        eprintln!("c Propagations:          {}", telemetry.propagation_count);
        eprintln!("c Restarts:              {}", telemetry.restart_count);
        eprintln!("c Reductions:            {}", telemetry.reduction_count);
    }

    if mem_profile {
        eprintln!("c Memory pressure (MB):  {}", telemetry.memory_pressure_mb);
    }
}
