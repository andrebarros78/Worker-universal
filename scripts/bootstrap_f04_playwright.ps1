param(
    [switch]$SkipBrowserInstall
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$web = Join-Path $root 'web-worker'
$deps = 'C:\ProgramData\SentinelX\workspace\tma-web-worker-deps'
$browserStore = 'C:\ProgramData\SentinelX\workspace\tma-playwright-browsers'
$node = 'C:\Program Files\Volta\node.exe'
$npm = 'C:\Program Files\Volta\npm.cmd'
$link = Join-Path $web 'node_modules'

New-Item -ItemType Directory -Force -Path $deps | Out-Null
Copy-Item -LiteralPath (Join-Path $web 'package.json') -Destination (Join-Path $deps 'package.json') -Force
Copy-Item -LiteralPath (Join-Path $web 'package-lock.json') -Destination (Join-Path $deps 'package-lock.json') -Force

Push-Location $deps
try {
    & $npm ci --ignore-scripts --no-audit --no-fund
    if ($LASTEXITCODE -ne 0) {
        throw "npm ci failed: $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

if (Test-Path -LiteralPath $link) {
    $item = Get-Item -LiteralPath $link -Force
    if ($item.LinkType -eq 'Junction') {
        cmd /c rmdir "$link" | Out-Null
    } else {
        Remove-Item -LiteralPath $link -Recurse -Force
    }
}
New-Item -ItemType Junction -Path $link -Target (Join-Path $deps 'node_modules') | Out-Null

$env:PLAYWRIGHT_BROWSERS_PATH = $browserStore
if (-not $SkipBrowserInstall) {
    & $node (Join-Path $deps 'node_modules\playwright\cli.js') install chromium
    if ($LASTEXITCODE -ne 0) {
        throw "playwright browser install failed: $LASTEXITCODE"
    }
}

Push-Location $web
try {
    & $node -e "const p=require('playwright'); console.log('PLAYWRIGHT_READY='+Boolean(p.chromium))"
    if ($LASTEXITCODE -ne 0) {
        throw "playwright import failed: $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Write-Output "F04_BOOTSTRAP=PASS"
Write-Output "DEPS=$deps"
Write-Output "BROWSERS=$browserStore"
Write-Output "NODE_MODULES_JUNCTION=$link"
