$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
if (Test-Path -LiteralPath 'C:\msys64\ucrt64\bin\gcc.exe') { $env:PATH = 'C:\msys64\ucrt64\bin;' + $env:PATH }
$fail = 0
$previousPycachePrefix = $env:PYTHONPYCACHEPREFIX
$baselineRunId = [guid]::NewGuid().ToString('N')
$baselinePycacheRoot = 'C:\ProgramData\SentinelX\workspace\tma-pycache'
$baselinePycacheDir = Join-Path $baselinePycacheRoot ("baseline-" + $baselineRunId)
New-Item -ItemType Directory -Force -Path $baselinePycacheDir | Out-Null
$env:PYTHONPYCACHEPREFIX = $baselinePycacheDir

function Run-Step {
    param(
        [string]$Name,
        [scriptblock]$Command
    )
    Write-Output "=== $Name ==="
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Output "FAILED_STEP=$Name EXIT=$LASTEXITCODE"
        $script:fail = 1
    }
}

$env:RUSTUP_HOME = 'C:\Users\andre\.rustup'
$env:CARGO_HOME = 'C:\Users\andre\.cargo'
$env:PATH = "C:\Users\andre\.cargo\bin;C:\Program Files\Erlang OTP\bin;C:\Program Files\Elixir\bin;$env:PATH"

$cargo = 'C:\Users\andre\.cargo\bin\cargo.exe'
$go = 'C:\Program Files\Go\bin\go.exe'
$node = 'C:\Program Files\Volta\node.exe'
$python = Join-Path $root '.venv\Scripts\python.exe'
$mix = 'C:\Program Files\Elixir\bin\mix.bat'

$previousCargoTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\core-rust'
Push-Location (Join-Path $root 'core-rust')
Run-Step 'RUST_FMT' { & $cargo fmt --check }
Run-Step 'RUST_CLIPPY' { & $cargo clippy --all-targets -- -D warnings }
Run-Step 'RUST_TEST' { & $cargo test }
Run-Step 'RUST_SELF_TEST' { & $cargo run --quiet --bin tma-core -- self-test }
Pop-Location
if ($null -eq $previousCargoTargetDir) {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
} else {
    $env:CARGO_TARGET_DIR = $previousCargoTargetDir
}

Push-Location (Join-Path $root 'supervisor-go')
Run-Step 'GO_FMT' {
    $unformatted = & $go fmt ./...
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    if ($unformatted) {
        Write-Output $unformatted
        Write-Output 'GO_FMT_MODIFIED_FILES'
        exit 3
    }
}
Run-Step 'GO_VET' { & $go vet ./... }
Run-Step 'GO_TEST' { & $go test ./... }
Run-Step 'GO_SELF_TEST' { & $go run ./cmd/tma-supervisor self-test }
Pop-Location

Push-Location (Join-Path $root 'web-worker')
Run-Step 'NODE_TS_TEST' { & $node --test test/worker.test.ts }
Run-Step 'NODE_SELF_TEST' { & $node src/worker.ts self-test }
Pop-Location

$env:PYTHONPATH = Join-Path $root 'src'
Run-Step 'PYTHON_COMPILE' { & $python -m compileall -q (Join-Path $root 'src') }
Run-Step 'PYTHON_TEST' { & $python -m unittest discover -s (Join-Path $root 'tests') -v }
Run-Step 'CONTRACT_TEST' { & $python (Join-Path $root 'scripts\verify_contracts.py') }
Run-Step 'CONTINUITY_TEST' { & $python (Join-Path $root 'scripts\verify_continuity.py') }
Run-Step 'F02_FOUNDATION' { & (Join-Path $root 'scripts\verify_f02.ps1') }
Run-Step 'F03_VISION_OCR' { & (Join-Path $root 'scripts\verify_f03.ps1') }
Run-Step 'F04_BROWSER_COMPUTER' { & (Join-Path $root 'scripts\verify_f04.ps1') }
Run-Step 'F05_DURABLE_RUNTIME' { & (Join-Path $root 'scripts\verify_f05.ps1') }
Run-Step 'F06_PLANNER_ROUTER_KNOWLEDGE' { & (Join-Path $root 'scripts\verify_f06.ps1') }
Run-Step 'F07_INDEPENDENT_VALIDATION_RECOVERY' { & (Join-Path $root 'scripts\verify_f07.ps1') }
Run-Step 'F08_TRAINING_BENCHMARK_QUALIFICATION' { & (Join-Path $root 'scripts\verify_f08.ps1') }
Run-Step 'F09_PLATFORM_ADAPTERS' { & (Join-Path $root 'scripts\verify_f09.ps1') }
Run-Step 'F10_ECONOMIC_CONTROLLER_SCHEDULER' { & (Join-Path $root 'scripts\verify_f10.ps1') }
Run-Step 'F11_AVAILABILITY_CONCURRENCY_HARDENING' { & (Join-Path $root 'scripts\verify_f11.ps1') }
Run-Step 'F12_SECURITY_FORMAL_OPERATIONAL_HARDENING' { & (Join-Path $root 'scripts\verify_f12.ps1') }
Run-Step 'F13_ENGINEERING_CANDIDATE' { & (Join-Path $root 'scripts\verify_f13_engineering.ps1') }

Push-Location (Join-Path $root 'availability-elixir')
Run-Step 'ELIXIR_FORMAT' { & $mix format --check-formatted }
Run-Step 'ELIXIR_TEST' { & $mix test }
Pop-Location

$gnatprove = Get-Command gnatprove.exe -ErrorAction SilentlyContinue
if ($gnatprove) {
    Push-Location (Join-Path $root 'formal-ada')
    Run-Step 'SPARK_PROOF' { & $gnatprove.Source -P tma_formal.gpr --level=2 }
    Pop-Location
} else {
    Write-Output '=== SPARK_PROOF ==='
    Write-Output 'SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED'
}

if ($fail -eq 0) {
    Write-Output 'BASELINE_VERIFY=PASS'
} else {
    Write-Output 'BASELINE_VERIFY=FAIL'
}
if ($null -eq $previousPycachePrefix) {
    Remove-Item Env:PYTHONPYCACHEPREFIX -ErrorAction SilentlyContinue
} else {
    $env:PYTHONPYCACHEPREFIX = $previousPycachePrefix
}
Remove-Item -LiteralPath $baselinePycacheDir -Recurse -Force -ErrorAction SilentlyContinue
exit $fail
