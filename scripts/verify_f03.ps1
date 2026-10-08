$ErrorActionPreference = 'Continue'
$root = Split-Path -Parent $PSScriptRoot
$fail = 0
$f03GateMutex = [System.Threading.Mutex]::new($false, 'Global\TMA_F03_VERIFY_GATE')
$f03GateAcquired = $false
try {
    $f03GateAcquired = $f03GateMutex.WaitOne([TimeSpan]::FromMinutes(10))
} catch [System.Threading.AbandonedMutexException] {
    $f03GateAcquired = $true
}
if (-not $f03GateAcquired) {
    throw 'F03_GATE_MUTEX_TIMEOUT'
}
Write-Output 'F03_GATE_MUTEX=ACQUIRED'

function Run-F03Step {
    param([string]$Name, [scriptblock]$Command)
    Write-Output "=== F03_$Name ==="
    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Output "FAILED_F03_STEP=$Name EXIT=$LASTEXITCODE"
            $script:fail = 1
        }
    } catch {
        Write-Output "FAILED_F03_STEP=$Name EXCEPTION=$($_.Exception.Message)"
        $script:fail = 1
    }
}

$python = Join-Path $root '.venv\Scripts\python.exe'
$env:PYTHONPATH = Join-Path $root 'src'
$tesseract = 'C:\Program Files\Tesseract-OCR\tesseract.exe'
$previousF03ReportPath = $env:TMA_F03_REPORT_PATH
$previousPycachePrefix = $env:PYTHONPYCACHEPREFIX
$f03RunId = [guid]::NewGuid().ToString('N')
$f03RuntimeRoot = 'C:\ProgramData\SentinelX\workspace\tma-f03-runtime'
$f03RuntimeDir = Join-Path $f03RuntimeRoot $f03RunId
New-Item -ItemType Directory -Force -Path $f03RuntimeDir | Out-Null
$env:TMA_F03_REPORT_PATH = Join-Path $f03RuntimeDir 'benchmark-report.json'
$env:PYTHONPYCACHEPREFIX = Join-Path $f03RuntimeDir 'pycache'
Write-Output ("F03_RUNTIME=" + $f03RunId)

Run-F03Step 'DEPENDENCIES' {
    if (-not (Test-Path -LiteralPath $tesseract)) {
        throw 'TESSERACT_NOT_INSTALLED'
    }
    & $tesseract --version | Select-Object -First 1
    if ($LASTEXITCODE -ne 0) { throw "tesseract version failed: $LASTEXITCODE" }
    & $python -c "import PIL; print('PILLOW='+PIL.__version__)"
}

Run-F03Step 'COMPILE' {
    Push-Location $root
    try {
        & $python -m compileall -q src tests scripts
    } finally {
        Pop-Location
    }
}

Run-F03Step 'UNIT' {
    Push-Location $root
    try {
        & $python -m unittest discover -s tests -p 'test_f03_document_intelligence.py' -v
    } finally {
        Pop-Location
    }
}

Run-F03Step 'CORPUS_REGENERATE' {
    Push-Location $root
    try {
        & $python scripts\generate_f03_corpus.py
    } finally {
        Pop-Location
    }
}

Run-F03Step 'CORPUS_DETERMINISM' { & $python (Join-Path $root 'scripts\verify_f03_corpus_determinism.py') }

Run-F03Step 'BENCHMARK' {
    Push-Location $root
    try {
        & $python scripts\run_f03_benchmark.py
    } finally {
        Pop-Location
    }
}

Run-F03Step 'ARTIFACTS' { & $python (Join-Path $root 'scripts\verify_f03_artifacts.py') }
Run-F03Step 'CONTRACTS' { & $python (Join-Path $root 'scripts\verify_f03_contracts.py') }
Run-F03Step 'PROVIDER_PURITY' { & $python (Join-Path $root 'scripts\verify_f03_purity.py') }

Run-F03Step 'VALIDATOR_REGRESSION' {
    Push-Location $root
    try {
        & $python -m unittest discover -s tests -p 'test_validators.py' -v
    } finally {
        Pop-Location
    }
}

if ($fail -eq 0) {
    Write-Output 'F03_VERIFY=PASS'
} else {
    Write-Output 'F03_VERIFY=FAIL'
}

if ($null -eq $previousF03ReportPath) {
    Remove-Item Env:TMA_F03_REPORT_PATH -ErrorAction SilentlyContinue
} else {
    $env:TMA_F03_REPORT_PATH = $previousF03ReportPath
}
if ($null -eq $previousPycachePrefix) {
    Remove-Item Env:PYTHONPYCACHEPREFIX -ErrorAction SilentlyContinue
} else {
    $env:PYTHONPYCACHEPREFIX = $previousPycachePrefix
}
Remove-Item -LiteralPath $f03RuntimeDir -Recurse -Force -ErrorAction SilentlyContinue
if ($f03GateAcquired) {
    $f03GateMutex.ReleaseMutex()
}
$f03GateMutex.Dispose()
Write-Output 'F03_GATE_MUTEX=RELEASED'
exit $fail
