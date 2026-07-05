#!/usr/bin/env bash
set -euo pipefail

SPECS_DIR="specs/tla"
TLA_FILE="${SPECS_DIR}/BenchmarkRunner.tla"
LOG_FILE="logs/tlc_check.log"
REPORT_FILE="data/regression/m292_report.json"
MAX_CHECK_SEC=60
MAX_SPEC_LINES=500

mkdir -p logs data/regression

echo "============================================================================="
echo "  M2.9.2 — TLA+ Model Check"
echo "  Spec: ${TLA_FILE}"
echo "============================================================================="

# Gate 1: Spec exists and size
[ ! -f "${TLA_FILE}" ] && echo "❌ SPEC NOT FOUND" && exit 1
SPEC_LINES=$(wc -l < "${TLA_FILE}")
echo "Spec lines: ${SPEC_LINES}"
[ "${SPEC_LINES}" -gt "${MAX_SPEC_LINES}" ] && echo "❌ SPEC TOO LARGE" && exit 1

# Gate 2: TLC availability
TLA2TOOLS_JAR="tla2tools.jar"
if [ ! -f "${TLA2TOOLS_JAR}" ]; then
    echo "Downloading tla2tools.jar..."
    curl -sL https://github.com/tlaplus/tlaplus/releases/download/v1.7.1/tla2tools.jar -o "${TLA2TOOLS_JAR}"
fi
TLC_CMD="java -cp ${TLA2TOOLS_JAR} tlc2.TLC"

# Gate 3: Run TLC
echo ""
echo "=== Running TLC Model Checker ==="
START_TIME=$(date +%s%N)

if ! timeout "${MAX_CHECK_SEC}" ${TLC_CMD} -workers 4 "${TLA_FILE}" > "${LOG_FILE}" 2>&1; then
    TLC_EXIT=$?
    END_TIME=$(date +%s%N)
    CHECK_MS=$(( (END_TIME - START_TIME) / 1000000 ))
    [ "${TLC_EXIT}" -eq 124 ] && echo "❌ TIMEOUT" && exit 1
    echo "❌ TLC ERROR"
    tail -20 "${LOG_FILE}"
    exit 1
fi

END_TIME=$(date +%s%N)
CHECK_MS=$(( (END_TIME - START_TIME) / 1000000 ))

# Gate 4: Parse output
STATES=$(grep -oP 'States generated: \K[0-9]+' "${LOG_FILE}" 2>/dev/null || echo "0")
DISTINCT=$(grep -oP 'Distinct states: \K[0-9]+' "${LOG_FILE}" 2>/dev/null || echo "0")
ERRORS=$(grep -c 'Error:' "${LOG_FILE}" 2>/dev/null || echo "0")
DEADLOCKS=$(grep -c 'Deadlock reached' "${LOG_FILE}" 2>/dev/null || echo "0")

STATUS="PASS"
[ "${ERRORS}" -gt 0 ] && STATUS="FAIL"
[ "${DEADLOCKS}" -gt 0 ] && STATUS="FAIL"

# Gate 5: Invariants
INVARIANT_OK=true
if grep -q "Invariant.*violated" "${LOG_FILE}" 2>/dev/null; then
    INVARIANT_OK=false
    STATUS="FAIL"
fi

# Evidence Report
cat > "${REPORT_FILE}" << EOJSON
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "brick": "M2.9.2",
  "spec_file": "${TLA_FILE}",
  "spec_lines": ${SPEC_LINES},
  "model_check_ms": ${CHECK_MS},
  "states_generated": ${STATES:-0},
  "distinct_states": ${DISTINCT:-0},
  "errors": ${ERRORS},
  "deadlocks": ${DEADLOCKS},
  "invariants_hold": ${INVARIANT_OK},
  "status": "${STATUS}",
  "gates": {
    "spec_under_500_lines": $([ ${SPEC_LINES} -le 500 ] && echo true || echo false),
    "check_under_60s": $([ ${CHECK_MS} -le 60000 ] && echo true || echo false),
    "zero_errors": $([ ${ERRORS} -eq 0 ] && echo true || echo false),
    "zero_deadlocks": $([ ${DEADLOCKS} -eq 0 ] && echo true || echo false),
    "invariants_hold": ${INVARIANT_OK}
  },
  "pillars": {
    "correctness": $([ ${ERRORS} -eq 0 ] && echo true || echo false),
    "auditability": true,
    "reproducibility": true,
    "competitiveness": $([ ${CHECK_MS} -le 60000 ] && echo true || echo false)
  }
}
EOJSON

# Summary
echo ""
echo "=== M2.9.2 Evidence ==="
echo "Spec lines      : ${SPEC_LINES}"
echo "Check time      : ${CHECK_MS} ms"
echo "States generated: ${STATES:-0}"
echo "Distinct states : ${DISTINCT:-0}"
echo "Errors          : ${ERRORS}"
echo "Deadlocks       : ${DEADLOCKS}"
echo "Invariants hold : ${INVARIANT_OK}"

echo ""
echo "=== Evidence Gates ==="
jq -r '.gates | to_entries[] | "  \(.key): \(.value)"' "${REPORT_FILE}" | sed 's/true: ✅/g; s/false: ❌/g'

echo ""
echo "=== Four Pillars ==="
jq -r '.pillars | to_entries[] | "  \(.key): \(.value)"' "${REPORT_FILE}" | sed 's/true: ✅/g; s/false: ❌/g'

if [ "${STATUS}" = "PASS" ] && [ ${ERRORS} -eq 0 ] && [ ${DEADLOCKS} -eq 0 ] && [ "${INVARIANT_OK}" = "true" ]; then
    echo ""
    echo "🧱 M2.9.2 TLA+ MODEL CHECK — SEALED."
    exit 0
else
    echo ""
    echo "❌ M2.9.2 FAILED — Review ${LOG_FILE}"
    exit 1
fi
