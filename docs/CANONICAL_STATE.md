# CANONICAL STATE

Project: TIMED-MISSION-AGENT
Canonical root: `D:\TIMED-MISSION-AGENT`

## Sovereign objective

Generate dependable personal income by turning eligible automatable work into completed, validated missions with minimal operator burden. Reliability and economic usefulness are first-class product requirements.

## Canonical entry

Every new AI/chat/operator starts at:

`00_START_HERE.md`

Machine-readable execution pointer:

`state\EXECUTION_STATE.json`

## Closed baselines

### F00 — POLYGLOT_V0_2 — CLOSED
- commit: `b2e0d814acefcc8dbc31323d77e5e9671aa7b15c`
- tag: `v0.2.0-polyglot-baseline`
- proof: `docs\PROOF_POLYGLOT_V0_2.md`

### F01 — CONCEPT_EXECUTIVE_CONTINUITY_BASELINE — CLOSED
- tag: `v0.2.1-executive-baseline`
- proof: `docs\PROOF_F01_EXECUTIVE_CONTINUITY.md`

### F02 — CAPABILITY_FOUNDATION — CLOSED
- complete conceptual map: CAP-001..CAP-045;
- provider-neutral Rust foundation;
- Registry/Discovery/Resolver/Lifecycle/Policy;
- Secrets/Artifacts/Sessions;
- Tool Bus + invocation/result/error contracts;
- Adapter SDK surfaces for Go/TypeScript/Python;
- providerless/offline boot proven;
- Rust core purity proven;
- `F02_VERIFY=PASS`;
- full `BASELINE_VERIFY=PASS`;
- proof: `docs\PROOF_F02_CAPABILITY_FOUNDATION.md`;
- release tag: `v0.2.2-capability-foundation`.

## Current phase

`F03 — VISION_OCR_DOCUMENT_INTELLIGENCE — IN_PROGRESS`

Objective:
Build image ingestion, OCR/vision ensemble and provider interfaces, document/invoice extraction, field reconciliation, confidence calibration, selective re-read and a benchmark corpus.

Required deliverables:
- image ingestion and normalization;
- OCR provider interface through F02 capabilities;
- local OCR adapter;
- optional remote vision adapter interface;
- invoice/document field schema;
- multi-source field conflict resolver;
- confidence calibration;
- selective re-read for uncertain fields;
- versioned labeled benchmark corpus;
- accuracy/latency/reliability report.

Exit gate:
- benchmark corpus versioned;
- field accuracy threshold defined and met;
- p50/p95/p99 measured;
- invalid CNPJ/date/arithmetic caught;
- calibration evaluated;
- provider outage/fallback tested;
- no OCR/LLM provider becomes mission authority;
- full baseline remains PASS;
- F03 proof + clean commit + state advancement exist.

## Next phase

`F04 — ISOLATED_BROWSER_COMPUTER_WORKER — PLANNED`

## Terminal phase

`F13 — PERSONAL_PRODUCTION_V1_0`

Terminal label: `MISSION_PROVEN`.
