# CANONICAL STATE

Project: TIMED-MISSION-AGENT
Canonical root: D:\TIMED-MISSION-AGENT
Baseline: POLYGLOT_V0_2

## Sovereign objective

Generate dependable personal income by turning eligible automatable work into completed, validated missions with minimal operator burden. Reliability and economic usefulness are first-class product requirements.

## Architecture authority

- Rust: canonical deterministic state/SLA/invariants.
- Go: process/worker supervision.
- TypeScript/Node: web execution adapter.
- Python: OCR/vision/document extraction.
- Elixir/Erlang OTP: high-availability supervision layer.
- Ada/SPARK: formal invariant module.
- JSON Schema: inter-process contract.

## Verified toolchains on PC Vendas

- Python 3.14.4
- Node 24.18.0 / npm 11.16.0
- Go 1.27.0
- Rust 1.99.0; project target stable-x86_64-pc-windows-gnu
- Elixir 1.20.4 with Erlang/OTP 29.1.1
- GNATprove/SPARK: NOT INSTALLED; source-only status

## Baseline proof

`scripts\verify_baseline.ps1` observed `BASELINE_VERIFY=PASS`.

Verified:
- Rust format + Clippy + 6 tests + self-test
- Go format + vet + tests + self-test
- TypeScript 3 tests + self-test
- Python compile + 11 tests
- JSON contract gate
- Elixir format + 2 tests
- SPARK source present; formal proof explicitly NOT VERIFIED

Detailed evidence: `docs\PROOF_POLYGLOT_V0_2.md`.

## Next phase

VISION_OCR_V0_3:
image ingestion, OCR/vision ensemble, field conflict resolution, calibration and invoice benchmark corpus.

Do not create a parallel project root. Continue from this file and Git history.
