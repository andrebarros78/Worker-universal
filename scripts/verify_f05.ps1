$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$core = Join-Path $root 'core-rust'
$python = Join-Path $root '.venv\Scripts\python.exe'
$cargo = 'C:\Users\andre\.cargo\bin\cargo.exe'
$gcc = 'C:\msys64\ucrt64\bin\gcc.exe'
$env:PATH = 'C:\msys64\ucrt64\bin;' + $env:PATH
$previousCargoTargetDir = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\core-rust'
$fail = 0

function Run-F05Step {
    param([string]$Name, [scriptblock]$Command)
    Write-Output "=== F05_$Name ==="
    try {
        & $Command
        if ($LASTEXITCODE -ne 0) { Write-Output "FAILED_F05_STEP=$Name EXIT=$LASTEXITCODE"; $script:fail = 1 }
    } catch {
        Write-Output "FAILED_F05_STEP=$Name EXCEPTION=$($_.Exception.Message)"
        $script:fail = 1
    }
}

Run-F05Step 'DEPENDENCIES' {
    if (-not (Test-Path -LiteralPath $gcc)) { throw "GCC_NOT_INSTALLED=$gcc" }
    & $gcc --version | Select-Object -First 1
    & $cargo --version
}
Run-F05Step 'RUST_FMT' { Push-Location $core; try { & $cargo fmt --check } finally { Pop-Location } }
Run-F05Step 'RUST_CLIPPY' { Push-Location $core; try { & $cargo clippy --all-targets -- -D warnings } finally { Pop-Location } }
Run-F05Step 'RUST_TEST' { Push-Location $core; try { & $cargo test } finally { Pop-Location } }
Run-F05Step 'BUILD_PROBE' { Push-Location $core; try { & $cargo build --bin tma-durable-probe } finally { Pop-Location } }
Run-F05Step 'PROCESS_RECOVERY' { & (Join-Path $root 'scripts\verify_f05_recovery.ps1') }
Run-F05Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f05_map.py') }
Run-F05Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f05_purity.py') }

if ($fail -eq 0) { Write-Output 'F05_VERIFY=PASS' } else { Write-Output 'F05_VERIFY=FAIL' }
if ($null -eq $previousCargoTargetDir) {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
} else {
    $env:CARGO_TARGET_DIR = $previousCargoTargetDir
}
exit $fail
