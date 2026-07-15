#!/usr/bin/env python3
# =============================================================================
# M2.7.17: Regression Intelligence — 12% EMA Anomaly Detection
# Harmonis Prime Sovereign Core v6.2.0-M2.7.17
# =============================================================================
# Detects performance regression when current PAR-2 exceeds 12% of EMA baseline.
# Pillar: Competitiveness — prevents silent performance degradation
# =============================================================================

import json
import sys
import argparse
import os
from pathlib import Path


def load_historical(baseline_file):
    """Load historical PAR-2 scores from baseline JSON."""
    if not os.path.exists(baseline_file):
        return []
    with open(baseline_file, "r") as f:
        data = json.load(f)
    return data.get("history", [])


def compute_ema(values, alpha=0.3):
    """
    Compute Exponential Moving Average.
    alpha=0.3 means 30% weight on latest, 70% on history.
    """
    if not values:
        return None
    ema = values[0]
    for v in values[1:]:
        ema = alpha * v + (1 - alpha) * ema
    return ema


def detect_regression(current_par2, baseline_file, threshold=0.12):
    """
    Detect if current PAR-2 exceeds EMA baseline by threshold (default 12%).
    
    Returns:
        dict with regression status, EMA, deviation, and recommendation
    """
    history = load_historical(baseline_file)
    
    # If no history, establish baseline
    if len(history) < 3:
        return {
            "status": "BASELINE_ESTABLISHING",
            "current_par2": current_par2,
            "ema_baseline": None,
            "deviation": None,
            "deviation_percent": None,
            "threshold": threshold,
            "recommendation": "Need 3+ data points for EMA. Current score recorded as baseline."
        }
    
    # Extract PAR-2 values
    par2_values = [entry.get("par2_score", 0.0) for entry in history]
    ema = compute_ema(par2_values)
    
    if ema is None or ema == 0:
        return {
            "status": "ERROR",
            "current_par2": current_par2,
            "ema_baseline": ema,
            "deviation": None,
            "deviation_percent": None,
            "threshold": threshold,
            "recommendation": "EMA is zero or undefined. Check baseline data."
        }
    
    deviation = (current_par2 - ema) / ema
    is_regression = deviation > threshold
    
    status = "REGRESSION_DETECTED" if is_regression else "HEALTHY"
    
    return {
        "status": status,
        "current_par2": round(current_par2, 4),
        "ema_baseline": round(ema, 4),
        "deviation": round(deviation, 4),
        "threshold": threshold,
        "deviation_percent": round(deviation * 100, 2),
        "recommendation": (
            f"ALERT: PAR-2 degraded {deviation*100:.1f}% above EMA baseline. "
            f"Investigate recent commits." if is_regression else
            f"HEALTHY: PAR-2 within {deviation*100:.1f}% of EMA baseline."
        )
    }


def update_baseline(current_par2, baseline_file, commit_hash=None):
    """Append current score to historical baseline."""
    history = load_historical(baseline_file)
    entry = {
        "par2_score": current_par2,
        "commit": commit_hash or "unknown",
        "timestamp": str(__import__('datetime').datetime.now())
    }
    history.append(entry)
    
    # Keep last 20 entries to prevent unbounded growth
    history = history[-20:]
    
    data = {"history": history, "last_updated": str(__import__('datetime').datetime.now())}
    
    os.makedirs(os.path.dirname(baseline_file) or ".", exist_ok=True)
    with open(baseline_file, "w") as f:
        json.dump(data, f, indent=2)
    
    print(f"Baseline updated: {baseline_file} ({len(history)} entries)")


def main():
    parser = argparse.ArgumentParser(
        description="M2.7.17: Regression Intelligence — 12% EMA Anomaly Detection"
    )
    parser.add_argument("par2_json", help="Path to PAR-2 score JSON from M2.7.16")
    parser.add_argument(
        "--baseline", "-b", default="baselines/par2_history.json",
        help="Historical baseline file"
    )
    parser.add_argument(
        "--threshold", "-t", type=float, default=0.12,
        help="Regression threshold (default: 0.12 = 12%)"
    )
    parser.add_argument(
        "--commit", "-c", default=None,
        help="Git commit hash for this run"
    )
    parser.add_argument(
        "--update", action="store_true",
        help="Update baseline with current score after detection"
    )
    args = parser.parse_args()
    
    # Load current PAR-2 score
    with open(args.par2_json, "r") as f:
        score_data = json.load(f)
    
    current_par2 = score_data.get("par2_score", 0.0)
    
    # Detect regression
    result = detect_regression(current_par2, args.baseline, args.threshold)
    
    # Update baseline if requested
    if args.update:
        update_baseline(current_par2, args.baseline, args.commit)
    
    # Output report
    print("=" * 60)
    print("  HARMONIS PRIME — M2.7.17 REGRESSION INTELLIGENCE")
    print("=" * 60)
    print(f"  Status:             {result['status']}")
    print(f"  Current PAR-2:      {result['current_par2']:.4f}")
    print(f"  EMA Baseline:       {result['ema_baseline'] if result['ema_baseline'] else 'N/A (establishing)'}")
    print(f"  Deviation:          {result['deviation_percent'] if result['deviation_percent'] is not None else 'N/A'}%")
    print(f"  Threshold:          {result['threshold']*100:.0f}%")
    print(f"  Recommendation:     {result['recommendation']}")
    print("=" * 60)
    
    # Exit code: 1 if regression detected (fails CI), 0 otherwise
    if result["status"] == "REGRESSION_DETECTED":
        print("\nERROR: Performance regression detected. Failing CI.")
        sys.exit(1)
    elif result["status"] == "ERROR":
        print("\nWARNING: Baseline error. Continuing with caution.")
        sys.exit(0)
    else:
        print("\nOK: Performance within acceptable bounds.")
        sys.exit(0)


if __name__ == "__main__":
    main()
