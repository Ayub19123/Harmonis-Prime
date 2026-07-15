#!/bin/bash
# =============================================================================
# M2.9: Bounded Research Spike — Master Orchestration
# Harmonis Prime Sovereign Core v6.2.0-M2.9
# =============================================================================
set -euo pipefail

OUTPUT_DIR="benchmark_output"
mkdir -p "$OUTPUT_DIR"

echo "============================================================================="
echo "  M2.9 BOUNDED RESEARCH SPIKE"
echo "  Harmonis Prime v6.2.0-M2.9"
echo "============================================================================="

echo ""
echo "=== M2.9.1: HORN-SAT CONFIG VALIDATION ==="
python3 scripts/m29_preflight/horn_sat_validator.py \
    instances/m29_test/config_valid.json \
    --output "$OUTPUT_DIR/m29_1_valid_proof.json"

echo ""
echo "=== M2.9.2: TLA+ MODEL CHECK ==="
bash scripts/m29_preflight/check_tla.sh

echo ""
echo "=== M2.9.3: BFT CNF TESTNET ==="
python3 scripts/m29_preflight/bft_cnf_generator.py \
    --nodes 5 --faults 1 \
    --output instances/m29_test/bft_consensus.cnf

echo ""
echo "============================================================================="
echo "  M2.9 RESEARCH SPIKE COMPLETE"
echo "============================================================================="
echo "  Reports:"
echo "    - $OUTPUT_DIR/m29_1_valid_proof.json"
echo "    - $OUTPUT_DIR/m29_bft_report.json"
echo "  Artifacts:"
echo "    - instances/m29_test/bft_consensus.cnf"
echo "============================================================================="
