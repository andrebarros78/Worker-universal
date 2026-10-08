# Baseline Status

## Proven technical baseline

POLYGLOT_V0_2
- tag: v0.2.0-polyglot-baseline
- proof: docs/PROOF_POLYGLOT_V0_2.md

## Proven continuity baseline

F01 CONCEPT_EXECUTIVE_CONTINUITY_BASELINE
- tag: v0.2.1-executive-baseline
- proof: docs/PROOF_F01_EXECUTIVE_CONTINUITY.md

## Proven capability foundation

F02 CAPABILITY_FOUNDATION
- map: CAP-001..CAP-045;
- dedicated gate: F02_VERIFY=PASS;
- tag: v0.2.2-capability-foundation.

## Proven Vision/OCR foundation

F03 VISION_OCR_DOCUMENT_INTELLIGENCE
- map: VIS-001..VIS-035;
- canonical release benchmark: 16 cases / 112 fields / accuracy 1.0000 / validation 1.0000;
- dedicated gate: F03_VERIFY=PASS;
- tag: v0.3.0-vision-ocr;
- R03-01 fixed regression-report mutation without reopening F03.

## Proven Browser/Computer foundation

F04 ISOLATED_BROWSER_COMPUTER_WORKER
- map: BRC-001..BRC-040;
- Playwright 1.64.0 + Playwright-managed Chromium;
- 14 focused Node tests PASS;
- process/session isolation PASS;
- F04_VERIFY=PASS;
- tag: v0.4.0-isolated-browser-worker.
- R04-01 deadline-limited timeout classification fixed; 5 repeated legacy runs + F04_VERIFY PASS.

## Proven Durable Mission Runtime

F05 DURABLE_MISSION_RUNTIME
- map: DUR-001..DUR-045;
- Rust authority + SQLite WAL/FULL;
- append-only event hash chain;
- durable queue, snapshots and checkpoints;
- idempotent mission/result contracts;
- lease ownership and monotonic fencing;
- stale-result rejection;
- 16-way lease race: one active owner;
- real process-kill recovery in planned/running/validating;
- restart/reboot reconciliation PASS;
- 19 Rust tests PASS;
- F05_VERIFY=PASS;
- full product gate: BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F05_DURABLE_RUNTIME.md;
- tag: v0.5.0-durable-runtime.

## Active construction lane

F06 PLANNER_ROUTER_KNOWLEDGE

No other phase is authorized as the main construction lane until F06 closes.

## Formal proof status

Ada/SPARK sources exist, but GNATprove is not installed. Status remains:
SOURCE_READY_NOT_PROVEN.
