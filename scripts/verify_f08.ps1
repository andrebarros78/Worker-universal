$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$qualification = Join-Path $root 'qualification-rust'
$foundation = Join-Path $root 'foundation-rust'
$python = Join-Path $root '.venv\Scripts\python.exe'
$cargo = 'C:\Users\andre\.cargo\bin\cargo.exe'
$gcc = 'C:\msys64\ucrt64\bin\gcc.exe'
$previousCargoTargetDir = $env:CARGO_TARGET_DIR
$env:PATH = 'C:\msys64\ucrt64\bin;' + $env:PATH
$fail = 0

function Run-F08Step {
    param([string]$Name, [scriptblock]$Command)
    Write-Output "=== F08_$Name ==="
    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Output "FAILED_F08_STEP=$Name EXIT=$LASTEXITCODE"
            $script:fail = 1
        }
    } catch {
        Write-Output "FAILED_F08_STEP=$Name EXCEPTION=$($_.Exception.Message)"
        $script:fail = 1
    }
}

Run-F08Step 'DEPENDENCIES' {
    if (-not (Test-Path -LiteralPath $gcc)) { throw "GCC_NOT_INSTALLED=$gcc" }
    & $cargo --version
    & $gcc --version | Select-Object -First 1
}

$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\foundation-rust'
Run-F08Step 'R02_01_PROMOTION_GATE' {
    Push-Location $foundation
    try {
        & $cargo test promotion_cannot_skip_forward_states
        if ($LASTEXITCODE -ne 0) { return }
        & $cargo test promotion_demotion_is_allowed_for_safety
    } finally { Pop-Location }
}

$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\qualification-rust'
Run-F08Step 'FMT' { Push-Location $qualification; try { & $cargo fmt --check } finally { Pop-Location } }
Run-F08Step 'CLIPPY' { Push-Location $qualification; try { & $cargo clippy --all-targets -- -D warnings } finally { Pop-Location } }
Run-F08Step 'TEST' { Push-Location $qualification; try { & $cargo test } finally { Pop-Location } }
Run-F08Step 'REPRODUCIBILITY_5X' {
    Push-Location $qualification
    try { & $cargo test synthetic_harness_is_reproducible_five_times -- --exact } finally { Pop-Location }
}
Run-F08Step 'TIMED_DEADLINE' {
    Push-Location $qualification
    try { & $cargo test wall_clock_harness_enforces_case_deadline -- --exact } finally { Pop-Location }
}
Run-F08Step 'SELF_TEST' {
    Push-Location $qualification
    try { & $cargo run --quiet --bin tma-qualification -- self-test } finally { Pop-Location }
}
Run-F08Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f08_contracts.py') }
Run-F08Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f08_map.py') }
Run-F08Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f08_purity.py') }

if ($fail -eq 0) { Write-Output 'F08_VERIFY=PASS' } else { Write-Output 'F08_VERIFY=FAIL' }

if ($null -eq $previousCargoTargetDir) {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
} else {
    $env:CARGO_TARGET_DIR = $previousCargoTargetDir
}
exit $fail
