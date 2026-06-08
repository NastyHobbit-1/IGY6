# Starts the local-only IGY6 host bridge on 127.0.0.1 (Windows).
param(
    [switch]$Foreground,
    [switch]$Rebuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$EnvFile = Join-Path $RepoRoot ".env"

function Get-EnvValue([string]$Key) {
    if (-not (Test-Path $EnvFile)) { return $null }
    foreach ($line in Get-Content $EnvFile) {
        if ($line -match "^\s*$Key=(.*)$") { return $Matches[1].Trim() }
    }
    return $null
}

$DataRoot = if ($env:IGY6_DATA_ROOT) { $env:IGY6_DATA_ROOT } else { Get-EnvValue "IGY6_DATA_ROOT" }
if (-not $DataRoot) { $DataRoot = Join-Path (Split-Path $RepoRoot -Parent) "IGY6_Data" }
$DataRoot = $DataRoot -replace "/", "\"

$OpsDir = Join-Path $DataRoot "ops"
$TokenFile = if ($env:IGY6_HOST_BRIDGE_TOKEN_FILE) { $env:IGY6_HOST_BRIDGE_TOKEN_FILE } else { Join-Path $OpsDir "host-bridge.token" }
$PidFile = Join-Path $OpsDir "host-bridge.pid"
$LogFile = Join-Path $OpsDir "host-bridge.log"
$Port = if ($env:IGY6_HOST_BRIDGE_PORT) { $env:IGY6_HOST_BRIDGE_PORT } else { "8765" }
$StartAgentScript = Join-Path $RepoRoot "scripts\start-ensure-agent.ps1"

New-Item -ItemType Directory -Force -Path $OpsDir | Out-Null

if (-not (Test-Path $TokenFile)) {
    $bytes = New-Object byte[] 32
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $token = [BitConverter]::ToString($bytes).Replace("-", "").ToLower()
    Set-Content -Path $TokenFile -Value $token -NoNewline
    Write-Host "Created token file: $TokenFile"
}

function Stop-RunningBridge {
    $stopped = $false
    if (Test-Path $PidFile) {
        $existingPid = Get-Content $PidFile -ErrorAction SilentlyContinue
        if ($existingPid -and (Get-Process -Id $existingPid -ErrorAction SilentlyContinue)) {
            Write-Host "Stopping host bridge (PID $existingPid)..."
            Stop-Process -Id $existingPid -Force -ErrorAction SilentlyContinue
            $stopped = $true
        }
        Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
    }
    Get-Process -Name "igy6-host-bridge" -ErrorAction SilentlyContinue | ForEach-Object {
        Write-Host "Stopping host bridge (PID $($_.Id))..."
        Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue
        $stopped = $true
    }
    if ($stopped) {
        Start-Sleep -Seconds 1
    }
}

function Start-EnsureAgentIfNeeded {
    if (-not (Test-Path $StartAgentScript)) {
        Write-Warning "Ensure agent starter not found: $StartAgentScript"
        return
    }
    & $StartAgentScript
}

if ($Rebuild) {
    Stop-RunningBridge
} elseif (Test-Path $PidFile) {
    $existingPid = Get-Content $PidFile -ErrorAction SilentlyContinue
    if ($existingPid -and (Get-Process -Id $existingPid -ErrorAction SilentlyContinue)) {
        Write-Host "Host bridge already running (PID $existingPid). Use -Rebuild to replace it."
        Start-EnsureAgentIfNeeded
        if (Test-Path $StartAgentScript) {
            & $StartAgentScript -RegisterLogonTask -Quiet
        }
        exit 0
    }
}

$env:IGY6_DATA_ROOT = $DataRoot
$bridgeArgs = @(
    "--host", "127.0.0.1",
    "--port", $Port,
    "--repo-root", $RepoRoot,
    "--token-file", $TokenFile
)
$Binary = Join-Path $RepoRoot "target\debug\igy6-host-bridge.exe"

Write-Host "Starting IGY6 host bridge on 127.0.0.1:$Port"
Write-Host "Token file: $TokenFile"
Write-Host "Data root: $DataRoot"

Push-Location $RepoRoot
try {
    if ($Rebuild -or -not (Test-Path $Binary)) {
        Write-Host "Building host bridge..."
        & cargo build -p igy6-host-bridge
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    if ($Foreground) {
        & $Binary @bridgeArgs
    } else {
        $proc = Start-Process -FilePath $Binary -ArgumentList $bridgeArgs -WorkingDirectory $RepoRoot -WindowStyle Hidden -PassThru
        Set-Content -Path $PidFile -Value $proc.Id
        Write-Host "Host bridge started (PID $($proc.Id))."
    }

    if (-not $Foreground) {
        Start-EnsureAgentIfNeeded
        if (Test-Path $StartAgentScript) {
            & $StartAgentScript -RegisterLogonTask -Quiet
        }
    }
} finally {
    Pop-Location
}