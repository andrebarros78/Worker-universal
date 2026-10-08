$ErrorActionPreference='Continue'
$root=Split-Path -Parent $PSScriptRoot
$economic=Join-Path $root 'economic-rust'
$python=Join-Path $root '.venv\Scripts\python.exe'
$cargo='C:\Users\andre\.cargo\bin\cargo.exe'
$env:PATH='C:\msys64\ucrt64\bin;'+$env:PATH
$previousCargo=$env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR='C:\ProgramData\SentinelX\workspace\tma-build\economic-rust'
$fail=0
function Run-F10Step{
 param([string]$Name,[scriptblock]$Cmd)
 Write-Output "=== F10_$Name ==="
 try{
  & $Cmd
  if($LASTEXITCODE -ne 0){Write-Output "FAILED_F10_STEP=$Name EXIT=$LASTEXITCODE";$script:fail=1}
 }catch{
  Write-Output "FAILED_F10_STEP=$Name ERROR=$($_.Exception.Message)"
  $script:fail=1
 }
}
Run-F10Step 'FMT' {Push-Location $economic;try{& $cargo fmt --check}finally{Pop-Location}}
Run-F10Step 'CLIPPY' {Push-Location $economic;try{& $cargo clippy --all-targets -- -D warnings}finally{Pop-Location}}
Run-F10Step 'TESTS' {Push-Location $economic;try{& $cargo test}finally{Pop-Location}}
Run-F10Step 'CONCURRENT_IDEMPOTENCE' {
 Push-Location $economic
 try{& $cargo test sixteen_parallel_writers_cannot_duplicate_single_cost_event}
 finally{Pop-Location}
}
Run-F10Step 'SELF_TEST' {
 Push-Location $economic
 try{& $cargo run --quiet --bin tma-economic -- self-test}
 finally{Pop-Location}
}
Run-F10Step 'CONTRACTS' {& $python (Join-Path $root 'scripts\verify_f10_contracts.py')}
Run-F10Step 'MAP' {& $python (Join-Path $root 'scripts\verify_f10_map.py')}
Run-F10Step 'PURITY' {& $python (Join-Path $root 'scripts\verify_f10_purity.py')}
if($null -eq $previousCargo){Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue}else{$env:CARGO_TARGET_DIR=$previousCargo}
if($fail -eq 0){Write-Output 'F10_VERIFY=PASS'}else{Write-Output 'F10_VERIFY=FAIL'}
exit $fail
