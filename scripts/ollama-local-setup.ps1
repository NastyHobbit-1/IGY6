# IGY6 optional local Ollama setup (Windows)
# Check-only by default. Use -Install to pull a model and write .env LLM settings.

param(
    [switch]$Check,
    [switch]$Install,
    [string]$Model = "qwen2.5-coder:7b"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path | Split-Path -Parent
$EnvFile = Join-Path $RepoRoot ".env"
$OllamaUrl = "http://127.0.0.1:11434"

function Test-Ollama {
    try {
        $r = Invoke-WebRequest -Uri "$OllamaUrl/api/tags" -UseBasicParsing -TimeoutSec 5
        return $r.StatusCode -eq 200
    } catch {
        return $false
    }
}

function Get-OllamaModels {
    try {
        $json = Invoke-RestMethod -Uri "$OllamaUrl/api/tags" -TimeoutSec 10
        return @($json.models | ForEach-Object { $_.name })
    } catch {
        return @()
    }
}

Write-Host "=== IGY6 Ollama setup (Windows) ==="

if (-not (Test-Ollama)) {
    Write-Host "Ollama API not reachable at $OllamaUrl"
    Write-Host "Install from https://ollama.com/download then run: ollama pull $Model"
    exit 1
}

Write-Host "Ollama is running at $OllamaUrl"
$models = Get-OllamaModels
if ($models.Count -gt 0) {
    Write-Host "Installed models: $($models -join ', ')"
} else {
    Write-Host "No models installed yet."
}

if ($Check -and -not $Install) {
    exit 0
}

if ($Install) {
    if (-not (Get-Command ollama -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: ollama CLI not found in PATH."
        exit 1
    }
    if ($models -notcontains $Model) {
        Write-Host "Pulling $Model ..."
        ollama pull $Model
    }
    if (-not (Test-Path $EnvFile)) {
        Copy-Item (Join-Path $RepoRoot ".env.example") $EnvFile
    }
    $content = Get-Content $EnvFile -Raw
    if ($content -match "(?m)^LLM_PROVIDER=.*$") {
        $content = $content -replace "(?m)^LLM_PROVIDER=.*$", "LLM_PROVIDER=ollama"
    } else {
        $content += "`nLLM_PROVIDER=ollama`n"
    }
    if ($content -match "(?m)^OLLAMA_BASE_URL=.*$") {
        $content = $content -replace "(?m)^OLLAMA_BASE_URL=.*$", "OLLAMA_BASE_URL=http://host.docker.internal:11434"
    } else {
        $content += "OLLAMA_BASE_URL=http://host.docker.internal:11434`n"
    }
    if ($content -match "(?m)^OLLAMA_MODEL=.*$") {
        $content = $content -replace "(?m)^OLLAMA_MODEL=.*$", "OLLAMA_MODEL=$Model"
    } else {
        $content += "OLLAMA_MODEL=$Model`n"
    }
    Set-Content -Path $EnvFile -Value $content -NoNewline
    Write-Host "Updated $EnvFile with LLM_PROVIDER=ollama and OLLAMA_MODEL=$Model"
    Write-Host "Restart stack: igy6 stop && igy6 start"
}