# Ensures host bridge (and optional max-reach deps) are ready. Idempotent.
param(
    [switch]$MaxReach,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$StartScript = Join-Path $RepoRoot "scripts\start-host-bridge.ps1"
$EnvFile = Join-Path $RepoRoot ".env"

function Get-EnvValue([string]$Key) {
    if (-not (Test-Path $EnvFile)) { return $null }
    foreach ($line in Get-Content $EnvFile) {
        if ($line -match "^\s*$Key=(.*)$") { return $Matches[1].Trim() }
    }
    return $null
}

function Write-Status([string]$Message) {
    if (-not $Quiet) { Write-Host $Message }
}

$DataRoot = if ($env:IGY6_DATA_ROOT) { $env:IGY6_DATA_ROOT } else { Get-EnvValue "IGY6_DATA_ROOT" }
if (-not $DataRoot) { $DataRoot = Join-Path (Split-Path $RepoRoot -Parent) "IGY6_Data" }
$DataRoot = $DataRoot -replace "/", "\"
$OpsDir = Join-Path $DataRoot "ops"
$Port = if ($env:IGY6_HOST_BRIDGE_PORT) { $env:IGY6_HOST_BRIDGE_PORT } else { "8765" }

New-Item -ItemType Directory -Force -Path $OpsDir | Out-Null
$env:IGY6_DATA_ROOT = $DataRoot

$StartAgentScript = Join-Path $RepoRoot "scripts\start-ensure-agent.ps1"
if (Test-Path $StartAgentScript) {
    & $StartAgentScript -Quiet
}

function Test-BridgeListening {
    try {
        $client = New-Object System.Net.Sockets.TcpClient
        $async = $client.BeginConnect("127.0.0.1", [int]$Port, $null, $null)
        $ok = $async.AsyncWaitHandle.WaitOne(1500, $false)
        if ($ok -and $client.Connected) {
            $client.Close()
            return $true
        }
        $client.Close()
    } catch {
        return $false
    }
    return $false
}

if (-not (Test-BridgeListening)) {
    Write-Status "Host bridge not listening on 127.0.0.1:$Port - starting..."
    & $StartScript
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $deadline) {
        if (Test-BridgeListening) { break }
        Start-Sleep -Milliseconds 500
    }
    if (-not (Test-BridgeListening)) {
        Write-Error "Host bridge failed to start on port $Port"
        exit 1
    }
    Write-Status "Host bridge is listening on 127.0.0.1:$Port"
} else {
    Write-Status "Host bridge already listening on 127.0.0.1:$Port"
}

if ($MaxReach) {
    $WebDir = Join-Path $RepoRoot "apps\web"
    Push-Location $WebDir
    try {
        if (-not (Test-Path "node_modules\playwright")) {
            Write-Status "Installing Playwright npm package..."
            & npm install --no-fund --no-audit
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
        Write-Status "Ensuring Playwright browsers (chromium + msedge)..."
        & npx playwright install chromium msedge
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    } finally {
        Pop-Location
    }
}

Write-Status "Host bridge ensure complete."