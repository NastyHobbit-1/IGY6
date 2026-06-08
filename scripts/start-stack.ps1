# Starts host bridge + ensure agent + Docker Compose stack (Windows).
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$ComposeFile = Join-Path $RepoRoot "infra\docker-compose.yml"
$EnvFile = Join-Path $RepoRoot ".env"

Write-Host "Starting host bridge and ensure agent..."
& (Join-Path $RepoRoot "scripts\start-host-bridge.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Starting Docker Compose stack..."
Push-Location $RepoRoot
try {
    docker compose -f $ComposeFile --env-file $EnvFile up -d
    exit $LASTEXITCODE
} finally {
    Pop-Location
}