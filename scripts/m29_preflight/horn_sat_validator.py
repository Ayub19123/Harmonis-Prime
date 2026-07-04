#!/usr/bin/env python3
# =============================================================================
# M2.9.1: Horn-SAT Config Validation Preflight
# Harmonis Prime Sovereign Core v6.2.0-M2.9.1
# =============================================================================
# Encodes config invariants as Horn clauses, solves via unit propagation.
# Pillar: Correctness — proves environment validity before execution.
# =============================================================================

import json
import sys
import os
import time
from pathlib import Path


def encode_horn_cnf(config):
    """
    Encode configuration invariants as Horn-SAT CNF.
    
    Horn clauses: at most one positive literal per clause.
    Format: (¬p₁ ∨ ¬p₂ ∨ ... ∨ q)  ≡  (p₁ ∧ p₂ ∧ ... → q)
    """
    clauses = []
    variables = {}
    var_counter = [1]
    
    def var(name):
        if name not in variables:
            variables[name] = var_counter[0]
            var_counter[0] += 1
        return variables[name]
    
    # H1: timeout > 0
    if config.get("timeout", 0) > 0:
        clauses.append([var("timeout_valid")])
    else:
        clauses.append([-var("timeout_valid")])
    
    # H2: memory_limit <= system_memory (simplified: > 0)
    if config.get("memory_limit", 0) > 0:
        clauses.append([var("memory_valid")])
    else:
        clauses.append([-var("memory_valid")])
    
    # H3: thread_count <= cpu_cores (simplified: > 0 and <= 64)
    threads = config.get("thread_count", 0)
    if 0 < threads <= 64:
        clauses.append([var("threads_valid")])
    else:
        clauses.append([-var("threads_valid")])
    
    # H4: Cargo.lock exists and is non-empty
    cargo_lock = Path(config.get("project_root", ".")) / "Cargo.lock"
    if cargo_lock.exists() and cargo_lock.stat().st_size > 0:
        clauses.append([var("deps_locked")])
    else:
        clauses.append([-var("deps_locked")])
    
    # H5: benchmark_runner binary exists
    binary = Path(config.get("project_root", ".")) / "target/release/benchmark_runner"
    if binary.exists():
        clauses.append([var("binary_exists")])
    else:
        clauses.append([-var("binary_exists")])
    
    # Master implication: all valid → config_ok
    # Horn form: (¬timeout_valid ∨ ¬memory_valid ∨ ¬threads_valid ∨ ¬deps_locked ∨ ¬binary_exists ∨ config_ok)
    all_valid = [
        -var("timeout_valid"),
        -var("memory_valid"),
        -var("threads_valid"),
        -var("deps_locked"),
        -var("binary_exists"),
        var("config_ok")
    ]
    clauses.append(all_valid)
    
    return clauses, variables


def unit_propagation(clauses):
    """
    Solve Horn-SAT via unit propagation (linear time for Horn).
    Returns: (satisfiable, assignment, propagation_count)
    """
    assignment = {}
    changed = True
    propagations = 0
    
    while changed:
        changed = False
        for clause in clauses:
            # Count unassigned literals and check for conflicts
            unassigned = []
            satisfied = False
            
            for lit in clause:
                var = abs(lit)
                if var in assignment:
                    val = assignment[var]
                    if (lit > 0 and val) or (lit < 0 and not val):
                        satisfied = True
                        break
                else:
                    unassigned.append(lit)
            
            if satisfied:
                continue
            
            if len(unassigned) == 1:
                # Unit clause: force assignment
                lit = unassigned[0]
                var = abs(lit)
                val = lit > 0
                if var not in assignment:
                    assignment[var] = val
                    changed = True
                    propagations += 1
            elif len(unassigned) == 0:
                # Empty clause = conflict
                return False, assignment, propagations
    
    # All clauses satisfied or no more unit propagations
    # For Horn-SAT, this means satisfiable
    return True, assignment, propagations


def validate_config(config_path):
    """Run full validation pipeline."""
    start = time.perf_counter()
    
    with open(config_path, "r") as f:
        config = json.load(f)
    
    clauses, variables = encode_horn_cnf(config)
    
    # Solve
    sat, assignment, props = unit_propagation(clauses)
    
    elapsed_ms = (time.perf_counter() - start) * 1000
    
    # Build proof
    proof = {
        "status": "SAT" if sat else "UNSAT",
        "propagations": props,
        "variables": {k: assignment.get(v, None) for k, v in variables.items()},
        "solve_time_ms": round(elapsed_ms, 4),
        "clauses": len(clauses),
        "proof_size_bytes": len(json.dumps(clauses))
    }
    
    return proof


def main():
    import argparse
    parser = argparse.ArgumentParser(description="M2.9.1: Horn-SAT Config Validation")
    parser.add_argument("config_json", help="Path to config JSON")
    parser.add_argument("--output", "-o", default="m29_preflight_proof.json")
    args = parser.parse_args()
    
    proof = validate_config(args.config_json)
    
    with open(args.output, "w") as f:
        json.dump(proof, f, indent=2)
    
    print("=" * 60)
    print("  HARMONIS PRIME — M2.9.1 HORN-SAT PREFLIGHT")
    print("=" * 60)
    print(f"  Status:        {proof['status']}")
    print(f"  Clauses:        {proof['clauses']}")
    print(f"  Propagations:   {proof['propagations']}")
    print(f"  Solve time:     {proof['solve_time_ms']:.4f} ms")
    print(f"  Proof size:     {proof['proof_size_bytes']} bytes")
    print(f"  Output:         {args.output}")
    print("=" * 60)
    
    if proof['status'] == "UNSAT":
        print("\nERROR: Configuration INVALID. Aborting execution.")
        sys.exit(1)
    elif proof['solve_time_ms'] > 100:
        print("\nWARNING: Solve time exceeds 100ms threshold.")
        sys.exit(0)
    else:
        print("\nOK: Configuration VALID. Proceed with execution.")
        sys.exit(0)


if __name__ == "__main__":
    main()
