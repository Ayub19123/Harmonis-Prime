# Generate SBOM Placeholder -- Software Bill of Materials.
# HONEST SCOPE (M1.5): Build artifact inventory.
# This is a build script, NOT a Rust test.

param(
    [string]$OutputPath = "sbom.json"
)

Write-Host "=== Harmonis Prime SBOM Generation ===" -ForegroundColor Cyan

# Read Cargo.toml for dependencies
$cargoPath = "Cargo.toml"
if (-not (Test-Path $cargoPath)) {
    Write-Error "Cargo.toml not found"
    exit 1
}

$cargo = Get-Content $cargoPath -Raw

# Extract crate name and version
$nameMatch = $cargo -match 'name\s*=\s*"([^"]+)"'
$versionMatch = $cargo -match 'version\s*=\s*"([^"]+)"'

$crateName = if ($nameMatch) { $Matches[1] } else { "unknown" }
$crateVersion = if ($versionMatch) { $Matches[1] } else { "unknown" }

# Extract dependencies (simple regex)
$deps = @()
$depMatches = [regex]::Matches($cargo, '^\s*([a-zA-Z0-9_-]+)\s*=\s*["{]', 'Multiline')
foreach ($m in $depMatches) {
    $depName = $m.Groups[1].Value
    if ($depName -notin @("name", "version", "edition", "authors", "license", "description")) {
        $deps += @{ name = $depName; version = "unknown"; source = "crates.io" }
    }
}

# Build SBOM
$sbom = @{
    specVersion = "1.4"
    serialNumber = "urn:uuid:$([guid]::NewGuid().ToString())"
    version = 1
    metadata = @{
        timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
        tools = @(@{ vendor = "Harmonis Prime"; name = "placeholder-sbom"; version = "0.1.0" })
        component = @{
            type = "application"
            name = $crateName
            version = $crateVersion
            purl = "pkg:cargo/$crateName@$crateVersion"
        }
    }
    components = $deps
}

# Write JSON
$sbom | ConvertTo-Json -Depth 10 | Out-File $OutputPath -Encoding utf8
Write-Host "SBOM written to: $OutputPath" -ForegroundColor Green
Write-Host "Components found: $($deps.Count)" -ForegroundColor Cyan
Write-Host "NOTE: This is a PLACEHOLDER SBOM. Full SBOM requires cargo-cyclonedx." -ForegroundColor Yellow