# Proof — F03 Vision/OCR & Document Intelligence

Date: 2026-10-07
Host: PC Vendas
Canonical root: `D:\TIMED-MISSION-AGENT`

## Objective

Prove a provider-neutral local Vision/OCR and document-intelligence layer with measurable accuracy, deterministic validation, confidence calibration, selective reread and reproducible benchmark evidence.

## Delivered

- image ingestion, hashing and dimension validation;
- EXIF-safe grayscale/autocontrast preprocessing;
- default and reread preprocessing profiles;
- provider-neutral `OCRProvider` and `RemoteVisionProvider` protocols;
- local Tesseract 5.4.0 adapter;
- line-level OCR confidence capture;
- invoice field parsing and canonical labels;
- multi-provider/multi-profile candidate reconciliation;
- monotonic confidence calibration;
- calibration ECE measurement;
- selective reread triggered by low confidence;
- selective reread triggered by deterministic validation failure;
- deterministic validation for CNPJ, date, currency and invoice arithmetic;
- invalid-field confidence penalty;
- unavailable-provider fallback;
- provider-neutral document engine;
- versioned synthetic invoice corpus;
- clean/low-contrast/mild-blur/compressed image variants;
- deterministic corpus regeneration;
- real OCR benchmark with accuracy and latency thresholds;
- F02 capability and adapter contract fixtures;
- F03 one-command verification gate.

## Tooling

Local OCR:
- Tesseract OCR `5.4.0.20240606`

Python:
- Pillow `11.3.0`

No remote OCR/LLM/API is required for F03 operation.

## Requirement map

- `docs/F03_VISION_OCR_MAP.md`
- `VIS-001..VIS-035`
- `docs/F03_IMPLEMENTATION_MATRIX.md`

Observed:
`F03_ARTIFACTS_OK requirements=35 cases=16 variants=4 accuracy=1.0000 validation=1.0000`

## Benchmark corpus

Dataset:
`benchmarks/f03_corpus`

Version:
`1.0.0`

Cases:
16 synthetic invoices.

Variants:
- clean;
- low_contrast;
- mild_blur;
- compressed.

Determinism proof:
`F03_CORPUS_DETERMINISTIC files=18`

## Final observed benchmark

From the full product verification run:

`F03_BENCHMARK cases=16 fields=112 accuracy=1.0000 validation=1.0000 p50_ms=329.21 p95_ms=1427.96 p99_ms=1427.96 raw_ece=0.0496 calibrated_ece=0.0250 reread_cases=4`

Thresholds:
- field accuracy >= 0.97 — PASS;
- validation pass rate >= 0.95 — PASS;
- p99 <= 5000 ms — PASS;
- calibration non-regression — PASS;
- selective reread exercised — PASS.

## Focused verification

`scripts\verify_f03.ps1`

Observed:
`F03_VERIFY=PASS`

Focused tests:
- 10 F03 document-intelligence tests PASS;
- 6 validator tests PASS;
- real Tesseract capability adapter exercised on corpus.

Provider purity:
`F03_PURITY_OK document_engine=provider_neutral remote_extension=present`

Contracts:
`F03_CONTRACTS_OK capability=vision.ocr.tesseract adapter=vision.ocr.tesseract`

## Full product regression

`scripts\verify_baseline.ps1`

Observed:
`BASELINE_VERIFY=PASS`

Python full suite:
22 tests PASS.

Other retained gates also passed:
- Rust deterministic core;
- Go supervisor;
- TypeScript worker;
- F02 Capability Foundation;
- Elixir availability layer.

SPARK remains accurately reported as:
`SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED`.

## Authority rule

Tesseract is a proven local provider, not mission authority.

The document engine depends on the OCR provider protocol, performs candidate reconciliation and uses deterministic domain validation before accepting extracted fields.

## Closure decision

F03 exit criteria are satisfied.

Canonical advancement:
- F03 → CLOSED
- F04 ISOLATED_BROWSER_COMPUTER_WORKER → IN_PROGRESS
- F05 DURABLE_MISSION_RUNTIME → PLANNED

Release tag target:
`v0.3.0-vision-ocr`
