#!/usr/bin/env python3
# M2.7.16: Cactus Plot Generator
# X-axis: Number of instances solved, Y-axis: Cumulative time

import json, sys, argparse
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

def generate_cactus(results, output_path, title="Harmonis Prime Cactus Plot"):
    times = []
    for r in results:
        status = str(r.get("status", "UNKNOWN")).upper()
        if status in ("SAT", "UNSAT", "SOLVED", "TRUE", "FALSE"):
            times.append(float(r.get("time", 0.0)))
    if not times:
        print("WARNING: No solved instances. Creating empty plot.", file=sys.stderr)
        times = [0.0]
    times.sort()
    cumulative = [sum(times[:i+1]) for i in range(len(times))]
    x = list(range(1, len(times) + 1))
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.plot(x, cumulative, marker="o", linestyle="-", linewidth=2.5, markersize=5,
            color="#1f77b4", label="Harmonis Prime")
    ax.set_xlabel("Number of Instances Solved", fontsize=13)
    ax.set_ylabel("Cumulative Time (seconds)", fontsize=13)
    ax.set_title(title, fontsize=15, fontweight="bold")
    ax.grid(True, alpha=0.3, linestyle="--")
    ax.legend(loc="lower right", fontsize=11)
    ax.text(0.02, 0.98, f"Total solved: {len(x)}", transform=ax.transAxes,
            fontsize=10, verticalalignment="top",
            bbox=dict(boxstyle="round", facecolor="wheat", alpha=0.5))
    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches="tight")
    print(f"Cactus plot saved: {output_path}")
    print(f"Instances plotted: {len(x)}")

def main():
    parser = argparse.ArgumentParser(description="Generate cactus plots")
    parser.add_argument("results_json")
    parser.add_argument("--output", "-o", default="cactus_plot.png")
    parser.add_argument("--title", default="Harmonis Prime Cactus Plot")
    args = parser.parse_args()
    with open(args.results_json) as fh:
        data = json.load(fh)
    results = data.get("results", data) if isinstance(data, dict) else data
    if not isinstance(results, list):
        print("ERROR: results must be a list", file=sys.stderr)
        sys.exit(1)
    generate_cactus(results, args.output, args.title)

if __name__ == "__main__":
    main()
