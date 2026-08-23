# IGY6 Easy Installer for Windows (PowerShell)
# Builds the compiled 'igy6.exe' executable (Rust CLI) and installs it for easy use.
# Running 'igy6' (or 'igy6 start') will start the full Docker stack (detached) and open your browser to the UI.
# Requires: Rust (cargo via rustup), Docker Desktop (with Compose v2), PowerShell.
# DIFF-268: installs/notes local media extraction tools; worker image carries the full set.

param(
    [switch] $Restore,
    [string] $Backup = "latest"
)
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

function Get-EnvFile {
    return (Join-Path $RepoRoot ".env")
}

function Get-BackupBase {
    $envFile = Get-EnvFile
    $dataRoot = $null
    if (Test-Path $envFile) {
        $lines = Get-Content -LiteralPath $envFile -ErrorAction SilentlyContinue
        $match = $lines | Where-Object { $_ -match '^\s*(?:export\s+)?IGY6_DATA_ROOT\s*=' } | Select-Object -Last 1
        if ($match) {
            $val = ($match -replace '^\s*(?:export\s+)?IGY6_DATA_ROOT\s*=\s*', '').Trim('"')
            if ($val) { $dataRoot = $val }
        }
    }
    if ($dataRoot) {
        return (Join-Path $dataRoot "ops\env-backups")
    } else {
        return (Join-Path $RepoRoot ".igy6-backups\env")
    }
}

function Get-BackupsSorted {
    param([string] $Base)
    if (-not (Test-Path $Base)) { return @() }
    return Get-ChildItem -LiteralPath $Base -Filter 'env-*.bak' -File -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending
}

function Restore-EnvBackup {
    param([string] $Which = "latest")
    $base = Get-BackupBase
    New-Item -ItemType Directory -Force -Path $base | Out-Null
    $chosen = $null
    if ([string]::IsNullOrWhiteSpace($Which) -or $Which -eq "latest") {
        $chosen = (Get-BackupsSorted -Base $base | Select-Object -First 1)
        if (-not $chosen) {
            Write-Host "ERROR: No backups found in $base"
            exit 1
        }
    } else {
        if (Test-Path -LiteralPath $Which) {
            $chosen = Get-Item -LiteralPath $Which
        } else {
            $candidate = Join-Path $base $Which
            if (Test-Path -LiteralPath $candidate) {
                $chosen = Get-Item -LiteralPath $candidate
            }
        }
        if (-not $chosen) {
            Write-Host "ERROR: Backup not found: $Which"
            exit 1
        }
    }
    Write-Host "Restoring from backup: $($chosen.FullName)"
    $envFile = Get-EnvFile
    if (Test-Path -LiteralPath $envFile) {
        $ts = Get-Date -AsUTC -Format "yyyyMMddTHHmmssZ"
        $safety = Join-Path $base ("env-{0}-pre-restore.bak" -f $ts)
        Copy-Item -Force -LiteralPath $envFile -Destination $safety
        Write-Host "Safety backup written: $safety"
    }
    $tmp = "$envFile.restore.$PID"
    Copy-Item -Force -LiteralPath $chosen.FullName -Destination $tmp
    Move-Item -Force -LiteralPath $tmp -Destination $envFile
    Write-Host "Restore complete: $envFile"
    Write-Host "Note: Restart the IGY6 stack to apply environment changes."
}

Write-Host "=== IGY6 Windows Installer ==="
Write-Host "Repo: $RepoRoot"

# Windows-native restore mode
if ($Restore.IsPresent) {
    Restore-EnvBackup -Which $Backup
    exit 0
}

# Check prerequisites
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: cargo (Rust) not found."
    Write-Host "Install Rust: winget install Rustlang.Rustup or https://rustup.rs/"
    exit 1
}

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "NOTE: docker not found. You can still build/install the IGY6 CLI now."
    Write-Host "      To run the stack later, install Docker Desktop (Compose v2)."
}

function Install-MediaTools {
    Write-Host "Installing local media extraction tools when possible (PDF/OCR/ffmpeg/whisper)..."
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install -e --id Gyan.FFmpeg --accept-package-agreements --accept-source-agreements 2>$null
        winget install -e --id UB-Mannheim.TesseractOCR --accept-package-agreements --accept-source-agreements 2>$null
        winget install -e --id oschwartz10612.Poppler --accept-package-agreements --accept-source-agreements 2>$null
    } elseif (Get-Command choco -ErrorAction SilentlyContinue) {
        choco install ffmpeg tesseract poppler -y 2>$null
    } else {
        Write-Host "NOTE: winget/choco not found. Worker Docker image still installs media tools on rebuild."
    }
    if (Get-Command pip -ErrorAction SilentlyContinue) {
        pip install --user openai-whisper 2>$null
    } elseif (Get-Command pip3 -ErrorAction SilentlyContinue) {
        pip3 install --user openai-whisper 2>$null
    }
    Write-Host "Media tool check (host):"
    foreach ($tool in @("pdftotext", "tesseract", "ffmpeg", "whisper")) {
        if (Get-Command $tool -ErrorAction SilentlyContinue) {
            Write-Host "  $tool : ok"
        } else {
            Write-Host "  $tool : missing on host (worker image provides it after rebuild)"
        }
    }
}

Install-MediaTools

# Build the executable
Write-Host "Building igy6 CLI (release)..."
Push-Location $RepoRoot
cargo build -p igy6-cli --release
Pop-Location

$Binary = Join-Path $RepoRoot "target\release\igy6.exe"
if (-not (Test-Path $Binary)) {
    Write-Host "ERROR: Build failed, binary not found at $Binary"
    exit 1
}

# Install location (user-local, no admin needed)
$InstallDir = Join-Path $env:LOCALAPPDATA "IGY6\bin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -Force $Binary (Join-Path $InstallDir "igy6.exe")

Write-Host "Installed igy6.exe to $InstallDir"

# Add to user PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to user PATH (restart terminal or logoff/logon to take effect)"
}

# Set IGY6_REPO so the binary can find the source repo from anywhere (important for installed .exe)
[Environment]::SetEnvironmentVariable("IGY6_REPO", $RepoRoot, "User")
Write-Host "Set IGY6_REPO=$RepoRoot (the binary will use this to locate the stack)"

Write-Host ""
Write-Host "=== Installation complete ==="
Write-Host "Health summary:"
Write-Host ("  cargo  : " + ($(Get-Command cargo -ErrorAction SilentlyContinue) ? "ok" : "missing"))
Write-Host ("  docker : " + ($(Get-Command docker -ErrorAction SilentlyContinue) ? "ok" : "missing"))
Write-Host "Restart your terminal (or open a new PowerShell window)."
Write-Host ""
Write-Host "To run:"
Write-Host "  igy6                 # Starts the full stack (detached) + opens browser to the UI"
Write-Host "  igy6 start           # Same"
Write-Host "  igy6 --help"
Write-Host "  igy6 stop"
Write-Host ""
Write-Host "Note: Keep this repo directory ($RepoRoot). Docker Desktop must be running."
Write-Host "Rebuild worker so media tools are in the image:"
Write-Host "  docker compose -f infra/docker-compose.yml build worker"
Write-Host "First run bootstraps .env; set a program password in the UI (Settings → User & Security)."
Write-Host "igy6 auto-picks WEB_PORT/APP_PORT if 3000/8000 are busy and verifies the IGY6 UI before opening the browser."
Write-Host "The .exe is at: $InstallDir\igy6.exe"
Write-Host ""
Write-Host "Optional local LLM (Ollama):"
Write-Host "  pwsh scripts/ollama-local-setup.ps1 -Check"
Write-Host "  pwsh scripts/ollama-local-setup.ps1 -Install -Model qwen2.5-coder:7b"
Write-Host "If Ollama is already running, igy6 start auto-enables LLM_PROVIDER=ollama."
Write-Host ""
Write-Host "Full working guide: docs/WORKING.md"
Write-Host "Profile options: docs/config/PROFILES.md"
Write-Host "Config reference: docs/config/ENV_REFERENCE.md"
