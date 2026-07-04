#!/usr/bin/env python3
# =============================================================================
# M2.7.17: Per-Class Benchmark Reporting
# Harmonis Prime Sovereign Core v6.2.0-M2.7.17
# =============================================================================
# Classifies benchmark instances by family (SAT/UNSAT, small/medium/large)
# and reports per-class PAR-2 scores.
# Pillar: Auditability — granular performance visibility
# =============================================================================

import json
import argparse
from collections import defaultdict


def classify_instance(instance_name, time, status):
    """
    Classify instance by problem characteristics.
    """
    name = instance_name.lower()
    
    # SAT vs UNSAT classification
    if status in ("UNSAT", "FALSE"):
        family = "UNSAT"
    elif status in ("SAT", "TRUE"):
        family = "SAT"
    else:
        family = "UNKNOWN"
    
    # Size classification based on solve time (proxy for difficulty)
    if time < 1.0:
        size = "FAST"
    elif time < 10.0:
        size = "MEDIUM"
    else:
        size = "HARD"
    
    return f"{family}_{size}"


def generate_per_class_report(results):
    """
    Generate per-class PAR-2 report.
    """
    classes = defaultdict(lambda: {"times": [], "solved": 0, "total": 0})
    
    for r in results:
        cls = classify_instance(
            r.get("instance", "unknown"),
            r.get("time", 0.0),
            r.get("status", "UNKNOWN")
        )
        classes[cls]["times"].append(r.get("time", 0.0))
        classes[cls]["total"] += 1
        if str(r.get("status", "")).upper() in ("SAT", "UNSAT", "SOLVED", "TRUE", "FALSE"):
            classes[cls]["solved"] += 1
    
    report = {}
    for cls, data in sorted(classes.items()):
        n = data["total"]
        solved = data["solved"]
        times = data["times"]
        avg_time = sum(times) / len(times) if times else 0.0
        par2 = sum(times) / n if n > 0 else 0.0
        
        report[cls] = {
            "instances": n,
            "solved": solved,
            "unsolved": n - solved,
            "solve_rate": round(solved / n * 100, 2) if n > 0 else 0.0,
            "avg_time_sec": round(avg_time, 4),
            "par2_contribution": round(par2, 4)
        }
    
    return report


def main():
    parser = argparse.ArgumentParser(description="M2.7.17: Per-class benchmark reporting")
    parser.add_argument("results_json", help="Path to benchmark results JSON")
    parser.add_argument("--output", "-o", default="per_class_report.json")
    args = parser.parse_args()
    
    with open(args.results_json, "r") as f:
        data = json.load(f)
    
    results = data.get("results", data) if isinstance(data, dict) else data
    if not isinstance(results, list):
        print("ERROR: results must be a list", file=sys.stderr)
        exit(1)
    
    report = generate_per_class_report(results)
    
    with open(args.output, "w") as f:
        json.dump(report, f, indent=2)
    
    print("=" * 60)
    print("  HARMONIS PRIME — M2.7.17 PER-CLASS REPORT")
    print("=" * 60)
    for cls, metrics in report.items():
        print(f"  {cls:20s} | {metrics['solved']:3d}/{metrics['instances']:3d} solved | "
              f"PAR-2: {metrics['par2_contribution']:.4f} | "
              f"Avg: {metrics['avg_time_sec']:.4f}s")
    print("=" * 60)
    print(f"Report saved: {args.output}")


if __name__ == "__main__":
    main()
