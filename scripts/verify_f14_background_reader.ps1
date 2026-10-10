$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$package=Join-Path $root 'web-worker\extension'
$node=Join-Path $root 'web-worker\runtime\win-x64\node.exe'
if(!(Test-Path $node -PathType Leaf)){throw 'F14_PORTABLE_NODE_MISSING'}
$files=@(Get-ChildItem -LiteralPath $package -File -ErrorAction Stop | Select-Object -ExpandProperty Name | Sort-Object)
$expected=@('background.js','content.js','identity.js','manifest.json')
if(@(Compare-Object -ReferenceObject $expected -DifferenceObject $files).Count -ne 0){
 Write-Output 'F14_EXTENSION_PACKAGE_INVALID'
 exit 1
}
$manifest=Get-Content -LiteralPath (Join-Path $package 'manifest.json') -Raw|ConvertFrom-Json
$hosts=@($manifest.host_permissions|Sort-Object)
if($manifest.manifest_version -ne 3 -or
   @($manifest.permissions|Sort-Object) -join ',' -ne 'alarms,scripting' -or
   $hosts -join ',' -ne 'http://127.0.0.1:18764/*,https://www.homeocta.com/*'){
 Write-Output 'F14_EXTENSION_SCOPE_INVALID'
 exit 2
}
Push-Location $root
try{
 & $node --check (Join-Path $package 'content.js')
 if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 & $node --check (Join-Path $package 'background.js')
 if($LASTEXITCODE -ne 0){exit $LASTEXITCODE}
 & $node --test web-worker/test/portable_browser_bridge.test.ts web-worker/test/edge_extension_reader.test.ts web-worker/test/edge_extension_browser.test.ts web-worker/test/course_mission.test.ts web-worker/test/course_mission_exit_contract.test.ts web-worker/test/course_memory_multi.test.ts
 $result=$LASTEXITCODE
 if($result -ne 0){Write-Output 'F14_BRIDGE_ENGINEERING=FAIL';exit $result}
 Write-Output 'F14_BRIDGE_ENGINEERING=PASS'
 Write-Output 'F14_BROWSER_ISOLATED_E2E=PASS'
 Write-Output 'F14_EXTENSION_INSTALLED_IN_OPERATOR_EDGE=NOT_PROVEN'
 Write-Output 'F14_AUTHENTICATED_COURSE_CAPTURE=NOT_PROVEN'
 Write-Output 'F14_MISSION_PROVEN=false'
}finally{Pop-Location}
