$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$fail = 0

function Run-F04Step {
    param([string]$Name, [scriptblock]$Command)
    Write-Output "=== F04_$Name ==="
    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Output "FAILED_F04_STEP=$Name EXIT=$LASTEXITCODE"
            $script:fail = 1
        }
    } catch {
        Write-Output "FAILED_F04_STEP=$Name EXCEPTION=$($_.Exception.Message)"
        $script:fail = 1
    }
}

$node = 'C:\Program Files\Volta\node.exe'
$npm = 'C:\Program Files\Volta\npm.cmd'
$python = Join-Path $root '.venv\Scripts\python.exe'
$web = Join-Path $root 'web-worker'
$env:PLAYWRIGHT_BROWSERS_PATH = 'C:\ProgramData\SentinelX\workspace\tma-playwright-browsers'

Run-F04Step 'DEPENDENCIES' {
    Push-Location $web
    try {
        & $node --version
        & $npm --version
        & $npm ls --depth=0
        if (-not (Test-Path -LiteralPath $env:PLAYWRIGHT_BROWSERS_PATH)) {
            throw 'PLAYWRIGHT_BROWSER_STORE_MISSING'
        }
    } finally {
        Pop-Location
    }
}

Run-F04Step 'NODE_TESTS' {
    Push-Location $web
    try {
        & $node --test test/worker.test.ts test/browser_worker.test.ts test/computer_contract.test.ts
    } finally {
        Pop-Location
    }
}

Run-F04Step 'SELF_TEST' {
    Push-Location $web
    try {
        & $node src/worker.ts self-test
    } finally {
        Pop-Location
    }
}

Run-F04Step 'ISOLATION_PROBE' {
    Push-Location $web
    try {
        & $node scripts/isolation_probe.ts
    } finally {
        Pop-Location
    }
}

Run-F04Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f04_contracts.py') }
Run-F04Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f04_map.py') }
Run-F04Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f04_purity.py') }

if ($fail -eq 0) {
    Write-Output 'F04_VERIFY=PASS'
} else {
    Write-Output 'F04_VERIFY=FAIL'
}
exit $fail
