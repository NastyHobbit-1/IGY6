# IGY6 runtime verification script.
# Requires PowerShell 7+.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [scriptblock] $Command
    )

    Write-Host ""
    Write-Host "==> $Name"
    & $Command
}

$requiredRootFiles = @(
    'Cargo.toml',
    'infra/docker-compose.yml',
    'apps/web/package.json',
    '.env.example'
)

foreach ($path in $requiredRootFiles) {
    if (-not (Test-Path -LiteralPath $path)) {
        Write-Error "This script must be run from the IGY6 repository root. Missing required path: $path"
        exit 1
    }
}

$envFile = $null
if (Test-Path -LiteralPath '.env.test') {
    $envFile = '.env.test'
} elseif (Test-Path -LiteralPath '.env') {
    $envFile = '.env'
} else {
    Write-Error 'No .env.test or .env file found. Create one from .env.example and configure IGY6_DATA_ROOT.'
    exit 1
}

Write-Host "Using $envFile"

try {
    Invoke-Step 'Rust workspace tests' {
        cargo test --workspace
    }

    Invoke-Step 'Web dependency install' {
        npm --prefix apps/web install
    }

    Invoke-Step 'Web audit' {
        npm --prefix apps/web audit
    }

    Invoke-Step 'Web typecheck' {
        npm --prefix apps/web run typecheck
    }

    Invoke-Step 'Web production build' {
        npm --prefix apps/web run build
    }

    Invoke-Step 'Docker web build' {
        docker compose -f infra/docker-compose.yml --env-file $envFile build web
    }

    Invoke-Step 'Docker web start' {
        docker compose -f infra/docker-compose.yml --env-file $envFile up -d web
    }

    Start-Sleep -Seconds 20

    Invoke-Step 'Docker service status' {
        docker compose -f infra/docker-compose.yml --env-file $envFile ps
    }

    Invoke-Step 'API live health' {
        Invoke-RestMethod -Uri 'http://127.0.0.1:18000/health/live' -Method Get | Out-Null
    }

    Invoke-Step 'API ready health' {
        Invoke-RestMethod -Uri 'http://127.0.0.1:18000/health/ready' -Method Get | Out-Null
    }

    Invoke-Step 'Web HTTP status' {
        $response = Invoke-WebRequest -Uri 'http://127.0.0.1:13000' -UseBasicParsing
        if ($response.StatusCode -ne 200) {
            throw "Expected web HTTP 200 but received $($response.StatusCode)"
        }
    }
}
finally {
    $tsBuildInfo = 'apps/web/tsconfig.tsbuildinfo'
    if (Test-Path -LiteralPath $tsBuildInfo) {
        Remove-Item -LiteralPath $tsBuildInfo -Force
    }
}

Write-Host ''
Write-Host 'Runtime verification passed successfully.'
exit 0
