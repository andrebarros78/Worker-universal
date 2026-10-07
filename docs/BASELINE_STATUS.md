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
- tag: v0.3.0-vision-ocr.
- R03-01 fixed regression-report mutation without reopening F03.

## Proven Browser/Computer foundation

F04 ISOLATED_BROWSER_COMPUTER_WORKER
- map: BRC-001..BRC-040;
- Playwright 1.64.0 + Playwright-managed Chromium;
- 14 focused Node tests PASS;
- form/upload/download/evidence PASS;
- session persistence and real crash recovery PASS;
- unsafe-step no-replay PASS;
- layout drift PASS;
- deadline containment PASS;
- operator isolation: browser session 0 vs Explorer session 1;
- Computer Worker dedicated-session contract PASS;
- F04_VERIFY=PASS;
- full product gate: BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F04_BROWSER_COMPUTER.md;
- tag: v0.4.0-isolated-browser-worker.

## Active construction lane

F05 DURABLE_MISSION_RUNTIME

No other phase is authorized as the main construction lane until F05 closes.

## Formal proof status

Ada/SPARK sources exist, but GNATprove is not installed. Status remains:
SOURCE_READY_NOT_PROVEN.
