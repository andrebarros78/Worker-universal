$ErrorActionPreference='Continue'
$root=Split-Path -Parent $PSScriptRoot
$python=Join-Path $root '.venv\Scripts\python.exe'
$cargo='C:\Users\andre\.cargo\bin\cargo.exe'
$env:PATH='C:\msys64\ucrt64\bin;'+$env:PATH
$env:TMA_F12_TEST_ROOT='C:\ProgramData\SentinelX\workspace'
$oldCargo=$env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR='C:\ProgramData\SentinelX\workspace\tma-build\security-rust'
$fail=0
function Run-F12Step {
 param([string]$Name,[scriptblock]$Cmd)
 Write-Output "=== F12_$Name ==="
 try {
  & $Cmd
  if ($LASTEXITCODE -ne 0) {
   Write-Output "FAILED_F12_STEP=$Name EXIT=$LASTEXITCODE"
   $script:fail=1
  }
 } catch {
  Write-Output "FAILED_F12_STEP=$Name ERROR=$($_.Exception.GetType().Name)"
  $script:fail=1
 }
}
Run-F12Step 'SECURITY_TESTS' {
 & $python -m unittest discover -s (Join-Path $root 'security-runtime') -p 'test_security.py' -q
}
Run-F12Step 'RUST_FMT' {
 Push-Location (Join-Path $root 'security-rust')
 try { & $cargo fmt --check } finally { Pop-Location }
}
Run-F12Step 'RUST_CLIPPY' {
 Push-Location (Join-Path $root 'security-rust')
 try { & $cargo clippy --all-targets -- -D warnings } finally { Pop-Location }
}
Run-F12Step 'RUST_BUILD_TEST' {
 Push-Location (Join-Path $root 'security-rust')
 try { & $cargo test; if($LASTEXITCODE -ne 0){throw 'TEST_FAILED'}; & $cargo build } finally { Pop-Location }
}
Run-F12Step 'F05_OFFLINE_BACKUP_RESTORE' {
 & $python (Join-Path $root 'security-runtime\f12_offline_restore.py')
}
Run-F12Step 'DEPENDENCY_INVENTORY' {
 & $python (Join-Path $root 'scripts\verify_f12_inventory.py')
}
Run-F12Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f12_contracts.py') }
Run-F12Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f12_map.py') }
Run-F12Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f12_purity.py') }
Write-Output '=== F12_FORMAL_TOOLCHAIN ==='
$gnat=Get-Command gnatprove.exe -ErrorAction SilentlyContinue
if($gnat){
 Push-Location (Join-Path $root 'formal-ada')
 try {
  & $gnat.Source -P tma_formal.gpr --level=2
  if($LASTEXITCODE -ne 0){Write-Output 'FAILED_F12_STEP=GNATPROVE';$fail=1}
  else{Write-Output 'F12_SPARK_STATUS=PROVEN_BY_GNATPROVE'}
 }finally{Pop-Location}
}else{
 Write-Output 'F12_SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED'
}
if($null -eq $oldCargo){Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue}else{$env:CARGO_TARGET_DIR=$oldCargo}
if($fail -eq 0){Write-Output 'F12_VERIFY=PASS'}else{Write-Output 'F12_VERIFY=FAIL'}
exit $fail
