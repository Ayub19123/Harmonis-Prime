# ============================================================
# Harmonis Prime — Pre-Commit Gate (Industrial Standard)
# ============================================================
# Run before every commit. If it fails, do NOT commit.
# ============================================================

$ErrorActionPreference = "Stop"

Write-Host "`n🔍 HARMONIS PRIME PRE-COMMIT GATE" -ForegroundColor Cyan

# 1. Compile check
Write-Host "`n[1/3] Compile check..." -ForegroundColor Yellow
cargo check --all-targets 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "❌ COMPILE FAILED" -ForegroundColor Red; exit 1 }
Write-Host "✅ Compile: PASS" -ForegroundColor Green

# 2. Test check
Write-Host "`n[2/3] Test suite..." -ForegroundColor Yellow
cargo test --quiet 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "❌ TESTS FAILED" -ForegroundColor Red; exit 1 }
Write-Host "✅ Tests: PASS" -ForegroundColor Green

# 3. Security audit
Write-Host "`n[3/3] Security audit..." -ForegroundColor Yellow
cargo audit 2>&1 | Select-String "vulnerabilities found" | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "❌ VULNERABILITIES FOUND" -ForegroundColor Red; exit 1 }
Write-Host "✅ Audit: CLEAN" -ForegroundColor Green

Write-Host "`n🧱 GATE PASSED. Absolute industrial standard. Proceed." -ForegroundColor Green
