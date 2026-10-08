$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$probe = 'C:\ProgramData\SentinelX\workspace\tma-build\core-rust\debug\tma-durable-probe.exe'
$runtime = 'C:\ProgramData\SentinelX\workspace\tma-f05-recovery'

if (-not (Test-Path -LiteralPath $probe)) { throw "durable probe not built: $probe" }
New-Item -ItemType Directory -Force -Path $runtime | Out-Null

foreach ($state in @('planned','running','validating')) {
    Write-Output "=== F05_KILL_RECOVER_$($state.ToUpper()) ==="
    $db = Join-Path $runtime "$state.sqlite3"
    $ready = Join-Path $runtime "$state.ready"
    foreach ($path in @($db, "$db-wal", "$db-shm", $ready)) { Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue }

    $process = Start-Process -FilePath $probe -ArgumentList @('hold', $db, $state, $ready) -PassThru -WindowStyle Hidden
    $seen = $false
    for ($attempt = 0; $attempt -lt 100; $attempt++) {
        if (Test-Path -LiteralPath $ready) { $seen = $true; break }
        $process.Refresh()
        if ($process.HasExited) { break }
        Start-Sleep -Milliseconds 50
    }

    if (-not $seen) {
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
        throw "worker failed before ready state=$state"
    }

    Get-Content -LiteralPath $ready
    Stop-Process -Id $process.Id -Force
    Wait-Process -Id $process.Id -ErrorAction SilentlyContinue

    $recovery = & $probe recover $db $state 2>&1
    if ($LASTEXITCODE -ne 0) { throw "recovery failed state=$state output=$($recovery -join ' ')" }
    $recovery | Write-Output
    Write-Output "F05_RECOVERY_$($state.ToUpper())=PASS"
}

Write-Output 'F05_PROCESS_RECOVERY=PASS'
