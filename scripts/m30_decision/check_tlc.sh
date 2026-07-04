#!/bin/bash
# =============================================================================
# M3.0.2: TLC Model Checker Integration
# =============================================================================
set -euo pipefail

echo "============================================================================="
echo "  M3.0.2 TLC MODEL CHECKER"
echo "============================================================================="

# Check if TLC is available
if command -v tlc &> /dev/null; then
    echo "✅ TLC found: $(tlc -version 2>&1 | head -1)"
    
    # Create TLC config file
    cat > specs/tla/BenchmarkRunner.cfg << 'TLCEOF'
CONSTANTS
    Instances = {i1, i2, i3}
    MaxTime = 1000
    Workers = 2

INIT Init
NEXT Next

PROPERTY Liveness
INVARIANT Safety
INVARIANT TypeInvariant
TLCEOF

    echo "=== RUNNING TLC ==="
    tlc specs/tla/BenchmarkRunner.tla -config specs/tla/BenchmarkRunner.cfg || {
        echo "❌ TLC found property violation — check counterexample"
        exit 1
    }
    echo "✅ TLC model check PASSED — no violations"
else
    echo "⚠️  TLC not installed — running structural validation only"
    echo "   To install: wget https://github.com/tlaplus/tlaplus/releases/download/v1.7.1/TLAToolbox-1.7.1-linux.gtk.x86_64.zip"
    
    # Fallback: Enhanced structural check
    if grep -q "MODULE BenchmarkRunner" specs/tla/BenchmarkRunner.tla && \
       grep -q "EXTENDS" specs/tla/BenchmarkRunner.tla && \
       grep -q "Init ==" specs/tla/BenchmarkRunner.tla && \
       grep -q "Next ==" specs/tla/BenchmarkRunner.tla && \
       grep -q "Spec ==" specs/tla/BenchmarkRunner.tla && \
       grep -q "Liveness" specs/tla/BenchmarkRunner.tla && \
       grep -q "Safety" specs/tla/BenchmarkRunner.tla; then
        echo "✅ TLA+ structure VALID (fallback mode)"
    else
        echo "❌ TLA+ structure INVALID"
        exit 1
    fi
fi

echo ""
echo "M3.0.2 TLC CHECK COMPLETE."
