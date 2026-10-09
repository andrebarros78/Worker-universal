$ErrorActionPreference='Stop'
$r=Split-Path -Parent $PSScriptRoot
$py=Join-Path $r '.venv\Scripts\python.exe'
$c='C:\Users\andre\.cargo\bin\cargo.exe'
$env:PATH='C:\msys64\ucrt64\bin;'+$env:PATH
$previous=$env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR='C:\ProgramData\SentinelX\workspace\tma-build\production-rust'
$work='C:\ProgramData\SentinelX\workspace\tma-f13-release'
New-Item -ItemType Directory -Force -Path $work | Out-Null
$target=Join-Path $work ([guid]::NewGuid().ToString('N'))
$fail=0
function Run-F13 {
 param([string]$Name,[scriptblock]$Action)
 Write-Output "=== F13_$Name ==="
 try {
  & $Action
  if($LASTEXITCODE -ne 0){Write-Output "FAILED_F13_STEP=$Name EXIT=$LASTEXITCODE";$script:fail=1}
 }catch{
  Write-Output "FAILED_F13_STEP=$Name ERROR=$($_.Exception.GetType().Name)"
  $script:fail=1
 }
}
try {
 Run-F13 'FMT' {Push-Location (Join-Path $r 'production-rust');try{& $c fmt --check}finally{Pop-Location}}
 Run-F13 'CLIPPY' {Push-Location (Join-Path $r 'production-rust');try{& $c clippy --all-targets -- -D warnings}finally{Pop-Location}}
 Run-F13 'TESTS' {Push-Location (Join-Path $r 'production-rust');try{& $c test}finally{Pop-Location}}
 Run-F13 'BUILD' {Push-Location (Join-Path $r 'production-rust');try{& $c build --bin tma-production}finally{Pop-Location}}
 $bin=Join-Path $env:CARGO_TARGET_DIR 'debug\tma-production.exe'
 Run-F13 'LOCAL_E2E' {
  & $bin rehearsal $target (Join-Path $r 'examples\f13_local_input.txt')
 }
 if($fail -eq 0) {
  Run-F13 'ARTIFACT_EVIDENCE' {
   & $py (Join-Path $r 'scripts\verify_f13_artifact.py') $target (Join-Path $r 'examples\f13_local_input.txt')
  }
  Run-F13 'DUPLICATE_TARGET_DENIED' {
   & $py (Join-Path $r 'scripts\verify_f13_duplicate.py') $bin $target (Join-Path $r 'examples\f13_local_input.txt')
  }
 }
 Run-F13 'OPERATOR_ROLE_SEPARATION' { & (Join-Path $r 'scripts\verify_f13_operator_control.ps1') }
 Run-F13 'GATE_MAP' { & $py (Join-Path $r 'scripts\verify_f13_map.py') }
 Run-F13 'SAFETY_POLICY' { & $py (Join-Path $r 'scripts\verify_f13_purity.py') }
 if($fail -eq 0){
  Write-Output 'F13_ENGINEERING_VERIFY=PASS'
  Write-Output 'F13_PRODUCTION_ACCEPTANCE=BLOCKED EXTERNAL_ADAPTER_NOT_QUALIFIED'
  Write-Output 'F13_MISSION_PROVEN=PENDING'
 }else {
  Write-Output 'F13_ENGINEERING_VERIFY=FAIL'
 }
}finally{
 if(Test-Path $target){Remove-Item -LiteralPath $target -Recurse -Force -ErrorAction SilentlyContinue}
 if($null -eq $previous){Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue}else{$env:CARGO_TARGET_DIR=$previous}
}
exit $fail
