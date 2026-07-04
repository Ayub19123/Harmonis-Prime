#!/usr/bin/env python3
# M2.7.16: PAR-2 Score Calculator
# Formula: PAR-2 = (Σ solved_time + Σ unsolved(2 × timeout)) / N

import json, sys, argparse

def compute_par2(results, timeout):
    total = 0.0
    solved = 0
    unsolved = 0
    for r in results:
        status = str(r.get("status", "UNKNOWN")).upper()
        t = float(r.get("time", timeout))
        if status in ("SAT", "UNSAT", "SOLVED", "TRUE", "FALSE"):
            total += t
            solved += 1
        else:
            total += 2.0 * timeout
            unsolved += 1
    n = len(results)
    par2 = total / n if n > 0 else 0.0
    return {
        "par2_score": round(par2, 4),
        "solved": solved,
        "unsolved": unsolved,
        "total_instances": n,
        "timeout": timeout,
        "formula": "PAR-2 = (sum(solved_times) + 2*timeout*unsolved) / N"
    }

def main():
    parser = argparse.ArgumentParser(description="Compute PAR-2 scores")
    parser.add_argument("results_json")
    parser.add_argument("--timeout", type=float, default=300.0)
    parser.add_argument("--output", "-o", default="par2_score.json")
    args = parser.parse_args()
    with open(args.results_json) as fh:
        data = json.load(fh)
    results = data.get("results", data) if isinstance(data, dict) else data
    if not isinstance(results, list):
        print("ERROR: results must be a list", file=sys.stderr)
        sys.exit(1)
    score = compute_par2(results, args.timeout)
    with open(args.output, "w") as fh:
        json.dump(score, fh, indent=2)
    print("=" * 50)
    print("  HARMONIS PRIME — PAR-2 SCORE REPORT")
    print("=" * 50)
    print(f"  PAR-2 Score:        {score['par2_score']:.4f}")
    print(f"  Solved:             {score['solved']} / {score['total_instances']}")
    print(f"  Unsolved:           {score['unsolved']} / {score['total_instances']}")
    print(f"  Timeout:            {score['timeout']}s")
    print(f"  Output:             {args.output}")
    print("=" * 50)

if __name__ == "__main__":
    main()
