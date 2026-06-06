#!/usr/bin/env pwsh
# Harmonis Prime – Reproducible Benchmark Run Script
# Version: v7.1.0-BRICK51.3-ABSOLUTE-ZERO
# Purpose: One-command reproducibility check. NOT certification. NOT validation.
#          This is execution help for external engineers.

Write-Host "🧱 HARMONIS PRIME – REPRODUCIBLE BENCHMARK RUN" -ForegroundColor Cyan
Write-Host "   Tag: v7.1.0-BRICK51.3-ABSOLUTE-ZERO" -ForegroundColor Gray
Write-Host "   Commit: $((git rev-parse HEAD).Substring(0, 8))" -ForegroundColor Gray
Write-Host "   Hardware: SIMULATED (consumer laptop, stock config)" -ForegroundColor Gray
Write-Host "   Energy: NOT MEASURED (honest null)" -ForegroundColor Gray
Write-Host ""

# 1. Check Rust toolchain
$rustcVersion = rustc --version
Write-Host "🔧 Rust toolchain: $rustcVersion" -ForegroundColor Gray
if ($rustcVersion -notmatch "1.96.0") {
    Write-Host "⚠️  WARNING: Rust version mismatch. Expected 1.96.0" -ForegroundColor Yellow
    Write-Host "   Install: rustup install 1.96.0 && rustup default 1.96.0" -ForegroundColor Yellow
}

# 2. Verify clean build
Write-Host "`n📦 Verifying clean build (0 errors, 0 warnings)..." -ForegroundColor Cyan
cargo check --bin benchmark 2>&1 | Select-String "error" | ForEach-Object { Write-Host "❌ BUILD ERROR: $_" -ForegroundColor Red; exit 1 }
cargo check --bin benchmark_consensus 2>&1 | Select-String "error" | ForEach-Object { Write-Host "❌ BUILD ERROR: $_" -ForegroundColor Red; exit 1 }
Write-Host "✅ Build check passed" -ForegroundColor Green

# 3. Run graph benchmark (10K iterations)
Write-Host "`n📊 Running graph benchmark (10K iterations)..." -ForegroundColor Cyan
cargo run --release --bin benchmark -- 10000 0x51C3_2026_0613

# 4. Run consensus simulation benchmark (10K iterations)
Write-Host "`n⚙️ Running consensus simulation benchmark (10K iterations)..." -ForegroundColor Cyan
cargo run --release --bin benchmark_consensus -- 10000 0x51C3_2026_0613

# 5. Run test suite
Write-Host "`n🧪 Running test suite..." -ForegroundColor Cyan
cargo test

# 6. Run security audit
Write-Host "`n🔒 Running security audit..." -ForegroundColor Cyan
cargo audit

Write-Host "`n🧱 BENCHMARK RUN COMPLETE" -ForegroundColor Green
Write-Host "   Output files:" -ForegroundColor Gray
Write-Host "   - metrics.json (graph benchmark telemetry)" -ForegroundColor Gray
Write-Host "   - metrics_consensus.json (consensus simulation telemetry)" -ForegroundColor Gray
Write-Host "   - LIMITATIONS.md (honest measurement boundaries)" -ForegroundColor Gray
Write-Host "   - LIMITATIONS_CONSENSUS.md (consensus-specific limitations)" -ForegroundColor Gray
Write-Host ""
Write-Host "   NEXT STEPS:" -ForegroundColor Gray
Write-Host "   1. Verify metrics.json contains 10,000 entries" -ForegroundColor Gray
Write-Host "   2. Compare outputs with EXPECTED_OUTPUTS.md" -ForegroundColor Gray
Write-Host "   3. Report results to REPRODUCTION_LOG.md" -ForegroundColor Gray
Write-Host ""
Write-Host "   SOVEREIGN PRINCIPLE: Claims = Artifacts. Nothing more. Nothing less." -ForegroundColor Gray
