$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
Push-Location (Join-Path $root 'operator-control')
try {
  & node --test test/roles.test.ts
  if($LASTEXITCODE -ne 0){Write-Output 'F13_ROLE_BOUNDARIES=FAIL';exit $LASTEXITCODE}
  Write-Output 'F13_ROLE_BOUNDARIES=PASS tests=9 commercial_policy_not_in_engineering=true operator_decision_separate=true'
} finally {
  Pop-Location
}
