# IGY6 Easy Installer for Windows (PowerShell)
# IGY6 Easy Installer for Windows (PowerShell)
# Builds the compiled 'igy6.exe' executable (Rust CLI) and installs it for easy use.
# Running 'igy6' (or 'igy6 start') will start the full Docker stack (detached) and open your browser to the UI.
# Requires: Rust (cargo via rustup), Docker Desktop (with Compose v2), PowerShell.

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host "=== IGY6 Windows Installer ==="
Write-Host "Repo: $RepoRoot"

# Check prerequisites
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: cargo (Rust) not found."
    Write-Host "Install Rust: winget install Rustlang.Rustup or https://rustup.rs/"
    exit 1
}

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: docker not found. Please install Docker Desktop."
    exit 1
}

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
Write-Host "Restart your terminal (or open a new PowerShell window)."
Write-Host ""
Write-Host "To run:"
Write-Host "  igy6                 # Starts the full stack (detached) + opens browser to the UI"
Write-Host "  igy6 start           # Same"
Write-Host "  igy6 --help"
Write-Host "  igy6 stop"
Write-Host ""
Write-Host "Note: Keep this repo directory ($RepoRoot). Docker Desktop must be running."
Write-Host "First run bootstraps .env with password 'ThatDog123' etc."
Write-Host "igy6 auto-picks WEB_PORT/APP_PORT if 3000/8000 are busy and verifies the IGY6 UI before opening the browser."
Write-Host "The .exe is at: $InstallDir\igy6.exe"
Write-Host ""
Write-Host "Optional local LLM (Ollama):"
Write-Host "  pwsh scripts/ollama-local-setup.ps1 -Check"
Write-Host "  pwsh scripts/ollama-local-setup.ps1 -Install -Model qwen2.5-coder:7b"
Write-Host "If Ollama is already running, igy6 start auto-enables LLM_PROVIDER=ollama."
Write-Host ""
Write-Host "Full working guide: docs/WORKING.md"
