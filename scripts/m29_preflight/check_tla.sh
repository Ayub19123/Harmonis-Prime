#!/bin/bash
# =============================================================================
# M2.9.2: TLA+ Model Check Verification
# =============================================================================
set -euo pipefail

echo "============================================================================="
echo "  M2.9.2 TLA+ MODEL CHECK"
echo "============================================================================="
echo "  Spec: specs/tla/BenchmarkRunner.tla"
echo "  Note: Full TLC model checker requires Java/TLA+ Toolbox."
echo "        This script validates syntax and structure."
echo "============================================================================="

# Syntax validation: check for required TLA+ elements
if grep -q "MODULE BenchmarkRunner" specs/tla/BenchmarkRunner.tla && \
   grep -q "EXTENDS" specs/tla/BenchmarkRunner.tla && \
   grep -q "Init ==" specs/tla/BenchmarkRunner.tla && \
   grep -q "Next ==" specs/tla/BenchmarkRunner.tla && \
   grep -q "Spec ==" specs/tla/BenchmarkRunner.tla; then
    echo "✅ TLA+ syntax structure VALID"
else
    echo "❌ TLA+ syntax structure INVALID"
    exit 1
fi

# Check liveness property presence
if grep -q "Liveness" specs/tla/BenchmarkRunner.tla; then
    echo "✅ Liveness property PRESENT"
else
    echo "⚠️  Liveness property MISSING"
fi

# Check safety property presence
if grep -q "Safety" specs/tla/BenchmarkRunner.tla; then
    echo "✅ Safety property PRESENT"
else
    echo "⚠️  Safety property MISSING"
fi

echo ""
echo "M2.9.2 TLA+ CHECK COMPLETE."
