$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$go='C:\Program Files\Go\bin\go.exe'
$runtime=Join-Path 'C:\ProgramData\SentinelX\workspace\tma-f11-health' ([guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $runtime | Out-Null
$bin=Join-Path $runtime 'tma-supervisor.exe'
$out=Join-Path $runtime 'stdout.txt'
$err=Join-Path $runtime 'stderr.txt'
$proc=$null
try {
  Push-Location (Join-Path $root 'supervisor-go')
  try { & $go build -o $bin ./cmd/tma-supervisor } finally {Pop-Location}
  if($LASTEXITCODE -ne 0){throw 'F11_HEALTH_BINARY_BUILD_FAILED'}
  $proc=Start-Process -FilePath $bin -ArgumentList @('serve-health','0') -PassThru -WindowStyle Hidden -RedirectStandardOutput $out -RedirectStandardError $err
  $url=$null
  for($i=0;$i -lt 100;$i++){
    $proc.Refresh()
    if($proc.HasExited){throw 'F11_HEALTH_SERVER_EXITED'}
    if(Test-Path -LiteralPath $out){
      $match=Select-String -LiteralPath $out -Pattern 'F11_HEALTH_LISTEN=(http://127\.0\.0\.1:\d+)' | Select-Object -First 1
      if($match){$url=$match.Matches[0].Groups[1].Value;break}
    }
    Start-Sleep -Milliseconds 60
  }
  if(-not $url){throw 'F11_HEALTH_SERVER_DID_NOT_START'}
  $health=Invoke-WebRequest -Uri "$url/healthz" -UseBasicParsing -TimeoutSec 3
  $ready=Invoke-WebRequest -Uri "$url/readyz" -UseBasicParsing -TimeoutSec 3
  if($health.StatusCode -ne 200 -or $health.Content.Trim() -ne 'healthy') {throw 'F11_HEALTHZ_UNHEALTHY'}
  if($ready.StatusCode -ne 200 -or $ready.Content.Trim() -ne 'true') {throw 'F11_READYZ_NOT_READY'}
  Write-Output "F11_HEALTH_HTTP=PASS healthz=200 readyz=200 loopback_only=true"
} finally {
  if($proc){
    $proc.Refresh()
    if(-not $proc.HasExited){
      Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
      Wait-Process -Id $proc.Id -ErrorAction SilentlyContinue
    }
  }
  Remove-Item -LiteralPath $runtime -Recurse -Force -ErrorAction SilentlyContinue
}
