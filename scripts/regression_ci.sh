#!/bin/bash
# =============================================================================
# M2.7.17: Regression Intelligence CI Orchestration
# Harmonis Prime Sovereign Core v6.2.0-M2.7.17
# =============================================================================
# Runs after benchmark job. Detects regression, generates per-class report.
# Pillar: Competitiveness — automated performance governance
# =============================================================================

set -euo pipefail

BENCHMARK_RESULTS="${1:-./benchmark_output/benchmark_results.json}"
PAR2_JSON="${2:-./benchmark_output/par2_score.json}"
OUTPUT_DIR="${3:-./benchmark_output}"
COMMIT_HASH="${4:-$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')}"

mkdir -p "$OUTPUT_DIR"

echo "============================================================================="
echo "  M2.7.17 REGRESSION INTELLIGENCE"
echo "============================================================================="
echo "  Commit:      $COMMIT_HASH"
echo "  Results:     $BENCHMARK_RESULTS"
echo "  PAR-2:       $PAR2_JSON"
echo "  Output:      $OUTPUT_DIR"
echo "============================================================================="

# Step 1: Per-class reporting
echo ""
echo "=== PER-CLASS REPORT ==="
python3 scripts/per_class_report.py "$BENCHMARK_RESULTS" --output "$OUTPUT_DIR/per_class_report.json"

# Step 2: Regression detection with baseline update
echo ""
echo "=== REGRESSION DETECTION ==="
python3 scripts/regression_detect.py "$PAR2_JSON" \
    --baseline "baselines/par2_history.json" \
    --threshold 0.12 \
    --commit "$COMMIT_HASH" \
    --update

echo ""
echo "============================================================================="
echo "  M2.7.17 REGRESSION INTELLIGENCE COMPLETE"
echo "============================================================================="
echo "  Per-class:   $OUTPUT_DIR/per_class_report.json"
echo "  Baseline:    baselines/par2_history.json"
echo "============================================================================="
