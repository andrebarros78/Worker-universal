$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$validation = Join-Path $root 'validation-rust'
$core = Join-Path $root 'core-rust'
$python = Join-Path $root '.venv\Scripts\python.exe'
$cargo = 'C:\Users\andre\.cargo\bin\cargo.exe'
$gcc = 'C:\msys64\ucrt64\bin\gcc.exe'
$previousCargoTargetDir = $env:CARGO_TARGET_DIR
$previousRustupToolchain = $env:RUSTUP_TOOLCHAIN
$env:RUSTUP_TOOLCHAIN = 'stable-x86_64-pc-windows-gnu'
$env:PATH = 'C:\msys64\ucrt64\bin;' + $env:PATH
$fail = 0

function Run-F07Step {
    param([string]$Name, [scriptblock]$Command)
    Write-Output "=== F07_$Name ==="
    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Output "FAILED_F07_STEP=$Name EXIT=$LASTEXITCODE"
            $script:fail = 1
        }
    } catch {
        Write-Output "FAILED_F07_STEP=$Name EXCEPTION=$($_.Exception.Message)"
        $script:fail = 1
    }
}

Run-F07Step 'DEPENDENCIES' {
    if (-not (Test-Path -LiteralPath $gcc)) { throw "GCC_NOT_INSTALLED=$gcc" }
    & $cargo --version
    & $gcc --version | Select-Object -First 1
}

Run-F07Step 'R05_01_CORE_GATE' {
    $env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\core-rust'
    Push-Location $core
    try {
        & $cargo test succeeded_requires_independent_matching_validation_receipt
        if ($LASTEXITCODE -ne 0) { return }
        & $cargo test validation_receipts_are_append_only
    } finally {
        Pop-Location
    }
}

$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\validation-rust'
Run-F07Step 'FMT' { Push-Location $validation; try { & $cargo fmt --check } finally { Pop-Location } }
Run-F07Step 'CLIPPY' { Push-Location $validation; try { & $cargo clippy --all-targets -- -D warnings } finally { Pop-Location } }
Run-F07Step 'TEST' { Push-Location $validation; try { & $cargo test } finally { Pop-Location } }
Run-F07Step 'SELF_TEST' { Push-Location $validation; try { & $cargo run --quiet --bin tma-validation -- self-test } finally { Pop-Location } }
Run-F07Step 'FAILURE_MATRIX' { Push-Location $validation; try { & $cargo run --quiet --bin tma-validation -- failure-matrix } finally { Pop-Location } }
Run-F07Step 'BUILD_RECOVERY_PROBE' { Push-Location $validation; try { & $cargo build --bin tma-validation-recovery-probe } finally { Pop-Location } }
Run-F07Step 'PROCESS_RESTART' { & (Join-Path $root 'scripts\verify_f07_restart.ps1') }
Run-F07Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f07_contracts.py') }
Run-F07Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f07_map.py') }
Run-F07Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f07_purity.py') }

if ($fail -eq 0) { Write-Output 'F07_VERIFY=PASS' } else { Write-Output 'F07_VERIFY=FAIL' }

if ($null -eq $previousCargoTargetDir) {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
} else {
    $env:CARGO_TARGET_DIR = $previousCargoTargetDir
}
if ($null -eq $previousRustupToolchain) {
    Remove-Item Env:RUSTUP_TOOLCHAIN -ErrorAction SilentlyContinue
} else {
    $env:RUSTUP_TOOLCHAIN = $previousRustupToolchain
}
exit $fail
