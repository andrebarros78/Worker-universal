$ErrorActionPreference = 'Continue'
$root=Split-Path -Parent $PSScriptRoot
$python=Join-Path $root '.venv\Scripts\python.exe'
$env:PLAYWRIGHT_BROWSERS_PATH='C:\ProgramData\SentinelX\workspace\tma-playwright-browsers'
$fail=0
function Run-F09Step{
 param([string]$Name,[scriptblock]$Cmd)
 Write-Output "=== F09_$Name ==="
 try{
  & $Cmd
  if($LASTEXITCODE -ne 0){Write-Output "FAILED_F09_STEP=$Name EXIT=$LASTEXITCODE";$script:fail=1}
 } catch {
  Write-Output "FAILED_F09_STEP=$Name ERROR=$($_.Exception.Message)"
  $script:fail=1
 }
}
Run-F09Step 'BROWSER_DEPENDENCY' {
 if(-not (Test-Path "$env:PLAYWRIGHT_BROWSERS_PATH\chromium_headless_shell-1248")){throw "F04_BROWSER_BINARIES_MISSING"}
 & node --version
}
Run-F09Step 'F04_INTEGRATION_TESTS' {
 Push-Location (Join-Path $root 'platform-adapters')
 try{ & node --test test/platform.test.ts test/sdk.test.ts } finally{Pop-Location}
}
Run-F09Step 'SELF_TEST' {
 Push-Location (Join-Path $root 'platform-adapters')
 try{ & node src/self_test.ts } finally{Pop-Location}
}
Run-F09Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f09_contracts.py') }
Run-F09Step 'MAP' { & $python (Join-Path $root 'scripts\verify_f09_map.py') }
Run-F09Step 'PURITY' { & $python (Join-Path $root 'scripts\verify_f09_purity.py') }
if($fail -eq 0){Write-Output 'F09_VERIFY=PASS'}else{Write-Output 'F09_VERIFY=FAIL'}
exit $fail
