#!/bin/bash
# =============================================================================
# M2.9.1: Config Validation Preflight Orchestration
# =============================================================================
set -euo pipefail

CONFIG="${1:-instances/m29_test/config_valid.json}"
OUTPUT="${2:-benchmark_output/m29_preflight_proof.json}"

echo "============================================================================="
echo "  M2.9.1 HORN-SAT CONFIG VALIDATION PREFLIGHT"
echo "============================================================================="
echo "  Config:  $CONFIG"
echo "  Output:  $OUTPUT"
echo "============================================================================="

python3 scripts/m29_preflight/horn_sat_validator.py "$CONFIG" --output "$OUTPUT"

echo ""
echo "M2.9.1 PREFLIGHT COMPLETE."
