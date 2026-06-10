# IGY6 Runtime Verification Script
# PowerShell 7+
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Test-Path '.env.test') -and -not (Test-Path '.env')) {
    Write-Error 'No .env or .env.test found. Please create one from .env.example and configure IGY6_DATA_ROOT.'
    exit 1
}

$envFile = if (Test-Path '.env.test') { '.env.test' } else { '.env' }
Write-Host "Using $envFile"

# Core checks
cargo test --workspace

npm --prefix apps/web install
npm --prefix apps/web audit
npm --prefix apps/web run typecheck
npm --prefix apps/web run build

docker compose -f infra/docker-compose.yml --env-file $envFile build web
docker compose -f infra/docker-compose.yml --env-file $envFile up -d web
Start-Sleep -Seconds 15
docker compose -f infra/docker-compose.yml --env-file $envFile ps

# Health checks
Invoke-RestMethod -Uri 'http://127.0.0.1:18000/health/live' -Method Get | Out-Null
Invoke-RestMethod -Uri 'http://127.0.0.1:18000/health/ready' -Method Get | Out-Null

# Clean build artifact
if (Test-Path 'apps/web/tsconfig.tsbuildinfo') { Remove-Item 'apps/web/tsconfig.tsbuildinfo' -Force }

Write-Host 'Runtime verification passed successfully.'
exit 0