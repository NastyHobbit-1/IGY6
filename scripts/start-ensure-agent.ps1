# Starts the local host-bridge ensure agent (127.0.0.1) if not already running.
param(
    [switch]$RegisterLogonTask,
    [switch]$Quiet
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

function Write-Status([string]$Message) {
    if (-not $Quiet) { Write-Host $Message }
}

$DataRoot = if ($env:IGY6_DATA_ROOT) { $env:IGY6_DATA_ROOT } else { Get-EnvValue "IGY6_DATA_ROOT" }
if (-not $DataRoot) { $DataRoot = Join-Path (Split-Path $RepoRoot -Parent) "IGY6_Data" }
$DataRoot = $DataRoot -replace "/", "\"
$OpsDir = Join-Path $DataRoot "ops"
$AgentPidFile = Join-Path $OpsDir "host-bridge-agent.pid"
$AgentPort = if ($env:IGY6_HOST_BRIDGE_AGENT_PORT) { $env:IGY6_HOST_BRIDGE_AGENT_PORT } else { "8770" }
$AgentScript = Join-Path $RepoRoot "apps\web\scripts\host-bridge-agent.mjs"

New-Item -ItemType Directory -Force -Path $OpsDir | Out-Null
$env:IGY6_DATA_ROOT = $DataRoot

function Test-AgentListening {
    try {
        $client = New-Object System.Net.Sockets.TcpClient
        $async = $client.BeginConnect("127.0.0.1", [int]$AgentPort, $null, $null)
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

function Start-EnsureAgentProcess {
    if (-not (Test-Path $AgentScript)) {
        Write-Warning "Ensure agent script not found: $AgentScript"
        return $false
    }
    $nodeCmd = Get-Command node -ErrorAction SilentlyContinue
    $node = if ($nodeCmd) { $nodeCmd.Source } else { $null }
    if (-not $node) {
        Write-Warning "Node.js not found on PATH; max reach auto-ensure may not work until agent is started manually."
        return $false
    }
    $agentProc = Start-Process -FilePath $node -ArgumentList @($AgentScript) -WorkingDirectory $RepoRoot -WindowStyle Hidden -PassThru
    Set-Content -Path $AgentPidFile -Value $agentProc.Id
    Write-Status "Host bridge ensure agent started (PID $($agentProc.Id)) on 127.0.0.1:$AgentPort"
    return $true
}

function Register-EnsureAgentLogonTask {
    $taskName = "IGY6-HostBridgeEnsureAgent"
    $self = Join-Path $RepoRoot "scripts\start-ensure-agent.ps1"
    $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$self`" -Quiet"
    $trigger = New-ScheduledTaskTrigger -AtLogOn
    $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Limited
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
    Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
    Write-Status "Registered logon task: $taskName"
}

if ($RegisterLogonTask) {
    Register-EnsureAgentLogonTask
}

if (Test-AgentListening) {
    Write-Status "Host bridge ensure agent already listening on 127.0.0.1:$AgentPort"
    exit 0
}

if (Test-Path $AgentPidFile) {
    $agentPid = Get-Content $AgentPidFile -ErrorAction SilentlyContinue
    if ($agentPid -and (Get-Process -Id $agentPid -ErrorAction SilentlyContinue)) {
        $deadline = (Get-Date).AddSeconds(10)
        while ((Get-Date) -lt $deadline) {
            if (Test-AgentListening) { exit 0 }
            Start-Sleep -Milliseconds 300
        }
    }
}

if (-not (Start-EnsureAgentProcess)) {
    exit 1
}

$deadline = (Get-Date).AddSeconds(15)
while ((Get-Date) -lt $deadline) {
    if (Test-AgentListening) { exit 0 }
    Start-Sleep -Milliseconds 300
}

Write-Error "Ensure agent failed to listen on 127.0.0.1:$AgentPort"
exit 1