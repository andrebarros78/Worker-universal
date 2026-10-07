$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$fail = 0

function Run-F02Step {
    param(
        [string]$Name,
        [scriptblock]$Command
    )
    Write-Output "=== F02_$Name ==="
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Output "FAILED_F02_STEP=$Name EXIT=$LASTEXITCODE"
        $script:fail = 1
    }
}

$env:RUSTUP_HOME = 'C:\Users\andre\.rustup'
$env:CARGO_HOME = 'C:\Users\andre\.cargo'
$env:CARGO_INCREMENTAL = '0'
$env:CARGO_TARGET_DIR = 'C:\ProgramData\SentinelX\workspace\tma-build\foundation-rust'
$env:PATH = "C:\Users\andre\.cargo\bin;$env:PATH"

$cargo = 'C:\Users\andre\.cargo\bin\cargo.exe'
$go = 'C:\Program Files\Go\bin\go.exe'
$gofmt = 'C:\Program Files\Go\bin\gofmt.exe'
$node = 'C:\Program Files\Volta\node.exe'
$python = Join-Path $root '.venv\Scripts\python.exe'

Push-Location (Join-Path $root 'foundation-rust')
Run-F02Step 'RUST_FMT' { & $cargo fmt --check }
Run-F02Step 'RUST_CLIPPY' { & $cargo clippy --all-targets -- -D warnings }
Run-F02Step 'RUST_TEST' { & $cargo test }
Run-F02Step 'RUST_SELF_TEST' { & $cargo run --quiet -- self-test }
Pop-Location

Push-Location (Join-Path $root 'adapter-sdk\go')
Run-F02Step 'GO_FMT' {
    $bad = Get-ChildItem -Recurse -Filter '*.go' | ForEach-Object { & $gofmt -l $_.FullName }
    if ($bad) {
        $bad | Write-Output
        exit 3
    }
}
Run-F02Step 'GO_VET' { & $go vet ./... }
Run-F02Step 'GO_TEST' { & $go test ./... }
Pop-Location

Run-F02Step 'TYPESCRIPT_SDK_TEST' {
    & $node --test (Join-Path $root 'adapter-sdk\typescript\sdk.test.ts')
}

Push-Location (Join-Path $root 'adapter-sdk\python')
Run-F02Step 'PYTHON_SDK_TEST' { & $python -m unittest -v test_sdk.py }
Pop-Location

Run-F02Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f02_contracts.py') }
Run-F02Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f02_map.py') }
Run-F02Step 'CORE_PURITY' { & $python (Join-Path $root 'scripts\verify_core_purity.py') }

if ($fail -eq 0) {
    Write-Output 'F02_VERIFY=PASS'
} else {
    Write-Output 'F02_VERIFY=FAIL'
}
exit $fail
