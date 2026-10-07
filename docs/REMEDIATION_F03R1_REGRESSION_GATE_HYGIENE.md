# Remediation F03R1 — Regression Gate Hygiene

Status: CLOSED
Discovered during: F04
References: F03 VISION_OCR_DOCUMENT_INTELLIGENCE

## Finding

The F03 benchmark gate regenerated benchmarks/f03_benchmark_report.json on every full regression run. Accuracy and validation were deterministic, but latency values are intentionally measured from the current run and therefore vary. This made a successful regression gate mutate a tracked F03 proof artifact.

## Correction

- run_f03_benchmark.py now accepts TMA_F03_REPORT_PATH.
- verify_f03_artifacts.py validates the same override path.
- verify_f03.ps1 writes runtime benchmark output outside the repository under C:\ProgramData\SentinelX\workspace\tma-f03-runtime.
- The canonical tracked F03 report remains the release evidence sealed with v0.3.0-vision-ocr.
- Running the benchmark script directly without the override still writes the canonical default path when deliberate regeneration is required.

## Proof required

A full baseline run must leave benchmarks/f03_benchmark_report.json byte-identical to HEAD.
