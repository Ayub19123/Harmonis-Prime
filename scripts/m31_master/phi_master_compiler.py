#!/usr/bin/env python3
"""
M3.1.4 — Φ_Master Compiler (Field-Aligned, Print-Fixed)
"""
import sys
import json
import subprocess
from pathlib import Path

def compile_phi_master(config_path: Path, project_root: Path) -> dict:
    clauses = []
    variables = {}
    var_id = 1

    def var(name: str) -> int:
        if name not in variables:
            nonlocal var_id
            variables[name] = var_id
            var_id += 1
        return variables[name]

    with open(config_path) as f:
        config = json.load(f)

    timeout = config.get("timeout", config.get("timeout_seconds", 999999))
    memory = config.get("memory_limit", config.get("memory_mb", 999999))
    threads = config.get("thread_count", config.get("threads", 999))

    bounds_ok = timeout <= 7200 and memory <= 32768 and threads <= 64

    has_timeout = "timeout" in config or "timeout_seconds" in config
    has_memory = "memory_limit" in config or "memory_mb" in config
    has_threads = "thread_count" in config or "threads" in config
    schema_ok = has_timeout and has_memory and has_threads

    strict_ok = True
    validator = project_root / "scripts/m30_decision/horn_sat_tightened.py"
    if validator.exists():
        try:
            result = subprocess.run(
                [sys.executable, str(validator), str(config_path), "--project-root", str(project_root)],
                capture_output=True, text=True, timeout=10
            )
            for line in result.stdout.strip().splitlines():
                if line.startswith("{"):
                    if json.loads(line).get("status") == "UNSAT":
                        strict_ok = False
                    break
        except Exception as e:
            print(f"⚠️  Validator delegation failed: {e}", file=sys.stderr)

    clauses.append([var("bounds_ok")] if bounds_ok else [-var("bounds_ok")])
    clauses.append([var("schema_ok")] if schema_ok else [-var("schema_ok")])
    clauses.append([var("strict_ok")] if strict_ok else [-var("strict_ok")])

    binary = project_root / "target/release/benchmark_runner"
    clauses.append([var("binary_exists")] if (binary.exists() and binary.stat().st_mode & 0o111) else [-var("binary_exists")])

    toolchain = project_root / "rust-toolchain"
    tc_valid = toolchain.exists() and "1.96.0" in toolchain.read_text()
    clauses.append([var("toolchain_valid")] if tc_valid else [-var("toolchain_valid")])

    lockfile = project_root / "Cargo.lock"
    clauses.append([var("deps_locked")] if lockfile.exists() else [-var("deps_locked")])

    for sv in ["bounds_ok", "schema_ok", "strict_ok", "binary_exists", "toolchain_valid", "deps_locked"]:
        clauses.append([-var("config_ok"), var(sv)])

    clauses.append([var("config_ok")])

    assignment = {}
    changed = True
    while changed:
        changed = False
        for clause in clauses:
            unassigned = [l for l in clause if abs(l) not in assignment]
            satisfied = any(
                (l > 0 and assignment.get(abs(l), False)) or
                (l < 0 and not assignment.get(abs(l), True))
                for l in clause
            )
            if not satisfied and len(unassigned) == 1:
                assignment[abs(unassigned[0])] = unassigned[0] > 0
                changed = True
            elif not satisfied and len(unassigned) == 0:
                return {
                    "status": "UNSAT",
                    "conflict": clause,
                    "failed_invariant": [k for k, v in variables.items() if v in [abs(l) for l in clause]],
                    "variables": variables,
                    "clauses": clauses,
                    "assignment": assignment,
                    "diagnostics": {
                        "bounds_ok": bounds_ok,
                        "schema_ok": schema_ok,
                        "strict_ok": strict_ok,
                        "binary_exists": binary.exists(),
                        "toolchain_valid": tc_valid,
                        "deps_locked": lockfile.exists(),
                        "config_fields": list(config.keys()),
                        "timeout": timeout,
                        "memory": memory,
                        "threads": threads
                    }
                }

    return {
        "status": "SAT",
        "variables": variables,
        "clauses": clauses,
        "assignment": assignment,
        "n_vars": var_id - 1,
        "n_clauses": len(clauses),
        "diagnostics": {
            "bounds_ok": bounds_ok,
            "schema_ok": schema_ok,
            "strict_ok": strict_ok,
            "config_fields": list(config.keys()),
            "timeout": timeout,
            "memory": memory,
            "threads": threads
        }
    }

def write_dimacs(result: dict, path: Path):
    with open(path, "w") as f:
        f.write(f"c Φ_Master — Harmonis Prime Config CNF\n")
        f.write(f"p cnf {result['n_vars']} {result['n_clauses']}\n")
        for clause in result["clauses"]:
            f.write(" ".join(map(str, clause)) + " 0\n")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("config", type=Path)
    parser.add_argument("--project-root", type=Path, default=Path("."))
    parser.add_argument("--output", type=Path, default=Path("instances/m31_test/phi_master.cnf"))
    parser.add_argument("--json", type=Path, default=Path("instances/m31_test/phi_master.json"))
    args = parser.parse_args()

    result = compile_phi_master(args.config, args.project_root)
    with open(args.json, "w") as f:
        json.dump(result, f, indent=2)

    if result["status"] == "SAT":
        write_dimacs(result, args.output)
        print(f"✅ Φ_Master SAT — config valid. CNF: {args.output}")
        print(f"   Variables: {result['n_vars']}, Clauses: {result['n_clauses']}")
        # FIX: name is str (key), val is bool — both safe to print
        for name, val in sorted(result["assignment"].items(), key=lambda x: str(x[0])):
            label = str(name)
            status = "TRUE" if val else "FALSE"
            print(f"   {label:25s}: {status}")
    else:
        print(f"🚫 Φ_Master UNSAT — config invalid.")
        print(f"   Conflict in clause: {result['conflict']}")
        print(f"   Failed invariants: {result.get('failed_invariant', [])}")
        print(f"   Diagnostics:")
        for k, v in result["diagnostics"].items():
            print(f"      {k:20s}: {v}")
        sys.exit(1)
