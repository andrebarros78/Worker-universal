$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$probe = 'C:\ProgramData\SentinelX\workspace\tma-build\validation-rust\debug\tma-validation-recovery-probe.exe'
$runtimeRoot = 'C:\ProgramData\SentinelX\workspace\tma-f07-recovery'
$runId = [guid]::NewGuid().ToString('N')
$runtime = Join-Path $runtimeRoot $runId
$db = Join-Path $runtime 'recovery.sqlite3'
$ready = Join-Path $runtime 'recovery.ready'
$process = $null

if (-not (Test-Path -LiteralPath $probe)) {
    throw "validation recovery probe not built: $probe"
}

New-Item -ItemType Directory -Force -Path $runtime | Out-Null
Write-Output "F07_RECOVERY_RUNTIME=$runId"

try {
    $process = Start-Process -FilePath $probe -ArgumentList @('hold', $db, $ready) -PassThru -WindowStyle Hidden
    $seen = $false
    for ($attempt = 0; $attempt -lt 100; $attempt++) {
        if (Test-Path -LiteralPath $ready) {
            $seen = $true
            break
        }
        $process.Refresh()
        if ($process.HasExited) {
            break
        }
        Start-Sleep -Milliseconds 50
    }
    if (-not $seen) {
        throw 'validation recovery probe failed before ready'
    }

    Get-Content -LiteralPath $ready
    Stop-Process -Id $process.Id -Force
    Wait-Process -Id $process.Id -ErrorAction SilentlyContinue
    Write-Output "F07_WORKER_KILLED=PASS pid=$($process.Id)"

    & $probe recover $db
    if ($LASTEXITCODE -ne 0) {
        throw "validation recovery process restart failed: $LASTEXITCODE"
    }
} finally {
    if ($null -ne $process) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
                Wait-Process -Id $process.Id -ErrorAction SilentlyContinue
            }
        } catch {
        }
    }
    Remove-Item -LiteralPath $runtime -Recurse -Force -ErrorAction SilentlyContinue
}
