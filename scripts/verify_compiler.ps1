# Verify Compiler -- Check rustc version and hash.
# HONEST SCOPE (M1.5): Build environment verification.
# This is a build script, NOT a Rust test.

param(
    [string]$ExpectedVersion = "1.78.0"
)

Write-Host "=== Harmonis Prime Compiler Verification ===" -ForegroundColor Cyan

# Check rustc exists
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rustc) {
    Write-Error "rustc not found in PATH"
    exit 1
}

# Get version
$versionOutput = rustc --version
Write-Host "rustc: $versionOutput"

# Check version match (major.minor only)
$versionMatch = $versionOutput -match "rustc\s+(\d+\.\d+)"
if (-not $versionMatch) {
    Write-Error "Could not parse rustc version"
    exit 1
}

$actualVersion = $Matches[1]
if ($actualVersion -ne $ExpectedVersion) {
    Write-Warning "Version mismatch: expected $ExpectedVersion, got $actualVersion"
} else {
    Write-Host "Version check PASSED" -ForegroundColor Green
}

# Get verbose version info
$verbose = rustc --version --verbose
Write-Host "`nVerbose info:`n$verbose"

# Hash the rustc binary (placeholder for reproducible builds)
$rustcPath = $rustc.Source
$hash = (Get-FileHash $rustcPath -Algorithm SHA256).Hash
Write-Host "`nrustc binary SHA-256: $hash"
Write-Host "NOTE: Store this hash for reproducible build verification."

Write-Host "`nCompiler verification COMPLETE" -ForegroundColor Green