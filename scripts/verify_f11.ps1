$ErrorActionPreference='Continue'
$root=Split-Path -Parent $PSScriptRoot
$go='C:\Program Files\Go\bin\go.exe'
$cargo='C:\Users\andre\.cargo\bin\cargo.exe'
$python=Join-Path $root '.venv\Scripts\python.exe'
$env:PATH='C:\msys64\ucrt64\bin;'+$env:PATH
$priorCargo=$env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR='C:\ProgramData\SentinelX\workspace\tma-build\availability-rust'
$fail=0
function Run-F11Step{
 param([string]$Name,[scriptblock]$Cmd)
 Write-Output "=== F11_$Name ==="
 try{
   & $Cmd
   if($LASTEXITCODE -ne 0){Write-Output "FAILED_F11_STEP=$Name EXIT=$LASTEXITCODE";$script:fail=1}
 }catch{
   Write-Output "FAILED_F11_STEP=$Name ERROR=$($_.Exception.Message)"
   $script:fail=1
 }
}
Run-F11Step 'RUST_FMT' {Push-Location (Join-Path $root 'availability-rust');try{& $cargo fmt --check}finally{Pop-Location}}
Run-F11Step 'RUST_CLIPPY' {Push-Location (Join-Path $root 'availability-rust');try{& $cargo clippy --all-targets -- -D warnings}finally{Pop-Location}}
Run-F11Step 'RUST_TEST' {Push-Location (Join-Path $root 'availability-rust');try{& $cargo test}finally{Pop-Location}}
$runner='C:\ProgramData\SentinelX\workspace\tma-build\availability-rust\debug\tma-availability.exe'
Run-F11Step 'BUILD_RUNNER' {Push-Location (Join-Path $root 'availability-rust');try{& $cargo build --bin tma-availability}finally{Pop-Location}}
Run-F11Step 'CONCURRENCY_MATRIX' {
 $runtime=Join-Path 'C:\ProgramData\SentinelX\workspace\tma-f11-matrix' ([guid]::NewGuid().ToString('N'))
 New-Item -ItemType Directory -Force -Path $runtime | Out-Null
 try{
  foreach($w in @(1,4,8,16)){
   $db=Join-Path $runtime ("f11-$w.sqlite3")
   & $runner matrix $db $w
   if($LASTEXITCODE -ne 0){throw "matrix failed at tier $w"}
  }
  Write-Output 'F11_MATRIX=PASS tiers=1,4,8,16'
 }finally{Remove-Item -LiteralPath $runtime -Recurse -Force -ErrorAction SilentlyContinue}
}
Run-F11Step 'GO_FMT' {
 Push-Location (Join-Path $root 'supervisor-go')
 try{
  $unformatted=& $go fmt ./...
  if($unformatted){Write-Output $unformatted;throw 'GO_FMT_MODIFIED_FILES'}
 }finally{Pop-Location}
}
Run-F11Step 'GO_VET' {Push-Location (Join-Path $root 'supervisor-go');try{& $go vet ./...}finally{Pop-Location}}
Run-F11Step 'GO_TEST' {Push-Location (Join-Path $root 'supervisor-go');try{& $go test ./... -count=1 -timeout=120s}finally{Pop-Location}}
Run-F11Step 'GO_RACE' {
 Push-Location (Join-Path $root 'supervisor-go')
 try {
  $env:CGO_ENABLED='1'
  & $go test -race ./supervisor -count=1 -timeout=120s
 }finally{Pop-Location}
}
Run-F11Step 'HTTP_HEALTH' { & (Join-Path $root 'scripts\verify_f11_health.ps1') }
Run-F11Step 'KILL_RESTART_RECOVERY' { & (Join-Path $root 'scripts\verify_f05_recovery.ps1') }
Run-F11Step 'SOAK_30S' {
 & $runner soak 30
}
Run-F11Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f11_contracts.py') }
Run-F11Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f11_map.py') }
Run-F11Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f11_purity.py') }
if($null -eq $priorCargo){Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue}else{$env:CARGO_TARGET_DIR=$priorCargo}
if($fail -eq 0){Write-Output 'F11_VERIFY=PASS'}else{Write-Output 'F11_VERIFY=FAIL'}
exit $fail
