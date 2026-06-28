//! M2.6: SAT Solver CLI Binary — Competition Entry Point
//!
//! Usage: cargo run --bin sat_solver <input.cnf> [--proof <file.drat>] [--mem-profile] [--clause-db-stats] [--save-checkpoint <file>] [--load-checkpoint <file>]
//!
//! Exit codes:
//!   10 = SATISFIABLE
//!   20 = UNSATISFIABLE
//!   1  = ERROR (parse failure, I/O error, etc.)

use sovereign_core::pim_solver::{CdclSolver, DimacsInstance, SolveResult};

fn print_usage(program: &str) {
    eprintln!("Usage: {} <input.cnf> [--proof <file.drat>] [--mem-profile] [--clause-db-stats] [--save-checkpoint <file>] [--load-checkpoint <file>]", program);
    eprintln!("  --proof <file.drat>        Write DRAT proof trace for UNSAT instances");
    eprintln!("  --mem-profile              Print memory telemetry after solving");
    eprintln!("  --clause-db-stats          Print clause database statistics");
    eprintln!("  --save-checkpoint <file>   Save solver state snapshot before solving");
    eprintln!(
        "  --load-checkpoint <file>   Load solver state from checkpoint (bypasses CNF parsing)"
    );
    eprintln!("Exit codes: 10 = SAT, 20 = UNSAT, 1 = ERROR");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];

    if args.len() < 2 {
        print_usage(program);
        std::process::exit(1);
    }

    let mut cnf_path: Option<String> = None;
    let mut proof_path: Option<String> = None;
    let mut save_checkpoint: Option<String> = None;
    let mut load_checkpoint: Option<String> = None;
    let mut mem_profile = false;
    let mut clause_db_stats = false;

    let mut i = 1;
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
            "--save-checkpoint" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("ERROR: --save-checkpoint requires a file path");
                    print_usage(program);
                    std::process::exit(1);
                }
                save_checkpoint = Some(args[i].clone());
            }
            "--load-checkpoint" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("ERROR: --load-checkpoint requires a file path");
                    print_usage(program);
                    std::process::exit(1);
                }
                load_checkpoint = Some(args[i].clone());
            }
            "--mem-profile" => mem_profile = true,
            "--clause-db-stats" => clause_db_stats = true,
            other => {
                if other.starts_with("--") {
                    eprintln!("ERROR: Unknown flag: {}", other);
                    print_usage(program);
                    std::process::exit(1);
                } else if cnf_path.is_none() {
                    cnf_path = Some(other.to_string());
                } else {
                    eprintln!(
                        "ERROR: Multiple input targets specified: {} and {}",
                        cnf_path.as_ref().unwrap(),
                        other
                    );
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    // Instantiate solver: load checkpoint or parse DIMACS
    let mut solver = if let Some(ref path) = load_checkpoint {
        match CdclSolver::load_checkpoint(path) {
            Ok(s) => {
                eprintln!("c Checkpoint loaded from: {}", path);
                s
            }
            Err(e) => {
                eprintln!("ERROR: Failed to load checkpoint {}: {}", path, e);
                std::process::exit(1);
            }
        }
    } else {
        if cnf_path.is_none() {
            eprintln!("ERROR: Input <input.cnf> path required when not loading a checkpoint.");
            print_usage(program);
            std::process::exit(1);
        }
        let p = cnf_path.unwrap();
        let instance = match DimacsInstance::parse(&p) {
            Ok(inst) => inst,
            Err(e) => {
                eprintln!("ERROR: Failed to parse {}: {:?}", p, e);
                std::process::exit(1);
            }
        };
        CdclSolver::from_dimacs(&instance)
    };

    // Save checkpoint before solving if requested
    if let Some(ref path) = save_checkpoint {
        if let Err(e) = solver.save_checkpoint(path) {
            eprintln!("ERROR: Failed to save checkpoint {}: {}", path, e);
            std::process::exit(1);
        }
        eprintln!("c Checkpoint saved to: {}", path);
    }

    // Solve
    let result = solver.solve();

    match result {
        SolveResult::Sat(model) => {
            println!("s SATISFIABLE");
            print!("v");
            for (var_idx, &value) in model.iter().enumerate() {
                let lit = if value {
                    (var_idx + 1) as i32
                } else {
                    -((var_idx + 1) as i32)
                };
                print!(" {}", lit);
            }
            println!(" 0");

            if mem_profile || clause_db_stats {
                print_telemetry(&solver, mem_profile, clause_db_stats);
            }

            std::process::exit(10);
        }
        SolveResult::Unsat => {
            println!("s UNSATISFIABLE");

            if let Some(ref path) = proof_path {
                if let Err(e) = solver.write_proof(path) {
                    eprintln!("ERROR: Failed to write proof to {}: {}", path, e);
                    std::process::exit(1);
                }
                eprintln!("c Proof written to: {}", path);
            }

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
        eprintln!(
            "c Learned clauses:       {}",
            telemetry.learned_clause_count
        );
        eprintln!("c Conflict rate:         {:.4}", telemetry.conflict_rate);
    }
    if mem_profile {
        eprintln!("c Decisions:             {}", telemetry.decision_count);
        eprintln!("c Propagations:          {}", telemetry.propagation_count);
        eprintln!("c Restarts:              {}", telemetry.restart_count);
        eprintln!("c Reductions:            {}", telemetry.reduction_count);
        eprintln!("c Memory pressure (MB):  {}", telemetry.memory_pressure_mb);
    }
}
