$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$planner = Join-Path $root 'planner-rust'
$python = Join-Path $root '.venv\Scripts\python.exe'
$cargo = 'C:\Users\andre\.cargo\bin\cargo.exe'
$gcc = 'C:\msys64\ucrt64\bin\gcc.exe'
$previousCargoTargetDir = $env:CARGO_TARGET_DIR
$env:PATH = 'C:\msys64\ucrt64\bin;' + $env:PATH
$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\planner-rust'
$fail = 0

function Run-F06Step {
    param([string]$Name, [scriptblock]$Command)
    Write-Output "=== F06_$Name ==="
    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Output "FAILED_F06_STEP=$Name EXIT=$LASTEXITCODE"
            $script:fail = 1
        }
    } catch {
        Write-Output "FAILED_F06_STEP=$Name EXCEPTION=$($_.Exception.Message)"
        $script:fail = 1
    }
}

Run-F06Step 'DEPENDENCIES' {
    if (-not (Test-Path -LiteralPath $gcc)) { throw "GCC_NOT_INSTALLED=$gcc" }
    & $cargo --version
    & $gcc --version | Select-Object -First 1
}
Run-F06Step 'FMT' { Push-Location $planner; try { & $cargo fmt --check } finally { Pop-Location } }
Run-F06Step 'CLIPPY' { Push-Location $planner; try { & $cargo clippy --all-targets -- -D warnings } finally { Pop-Location } }
Run-F06Step 'TEST' { Push-Location $planner; try { & $cargo test } finally { Pop-Location } }
Run-F06Step 'SELF_TEST' { Push-Location $planner; try { & $cargo run --quiet --bin tma-planner -- self-test } finally { Pop-Location } }
Run-F06Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f06_contracts.py') }
Run-F06Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f06_map.py') }
Run-F06Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f06_purity.py') }

if ($fail -eq 0) { Write-Output 'F06_VERIFY=PASS' } else { Write-Output 'F06_VERIFY=FAIL' }

if ($null -eq $previousCargoTargetDir) {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
} else {
    $env:CARGO_TARGET_DIR = $previousCargoTargetDir
}
exit $fail
