param(
    [Parameter(Mandatory = $true)]
    [string]$Url
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$nodeScript = Join-Path $repoRoot "scripts\harvest-browser-cookies.mjs"

if (-not (Test-Path $nodeScript)) {
    Write-Output '{"cookie":null,"authorization":null}'
    exit 0
}

$node = Get-Command node -ErrorAction SilentlyContinue
if (-not $node) {
    Write-Output '{"cookie":null,"authorization":null}'
    exit 0
}

Push-Location $repoRoot
try {
    & node $nodeScript $Url
} finally {
    Pop-Location
}