# CANONICAL STATE

Project: TIMED-MISSION-AGENT
Canonical root: `D:\TIMED-MISSION-AGENT`
GitHub: `https://github.com/andrebarros78/Worker-universal.git`

## Sovereign objective

Generate dependable personal income by turning eligible automatable work into completed, validated missions with minimal operator burden. Reliability and economic usefulness are first-class product requirements.

## Canonical entry

Every new AI/chat/operator starts at:

`00_START_HERE.md`

Machine-readable execution pointer:

`state\EXECUTION_STATE.json`

## Closed baselines

### F00 — POLYGLOT_V0_2 — CLOSED
- tag: `v0.2.0-polyglot-baseline`
- proof: `docs\PROOF_POLYGLOT_V0_2.md`

### F01 — CONCEPT_EXECUTIVE_CONTINUITY_BASELINE — CLOSED
- tag: `v0.2.1-executive-baseline`
- proof: `docs\PROOF_F01_EXECUTIVE_CONTINUITY.md`

### F02 — CAPABILITY_FOUNDATION — CLOSED
- tag: `v0.2.2-capability-foundation`
- proof: `docs\PROOF_F02_CAPABILITY_FOUNDATION.md`

### F03 — VISION_OCR_DOCUMENT_INTELLIGENCE — CLOSED
- complete requirement map: VIS-001..VIS-035;
- local Tesseract OCR provider;
- provider-neutral OCR/document interfaces;
- image preprocessing + selective reread;
- multi-source field reconciliation;
- confidence calibration;
- deterministic CNPJ/date/currency/arithmetic validation;
- versioned 16-case / 4-variant corpus;
- 112/112 benchmark fields correct;
- validation 16/16;
- final observed p99 1427.96 ms;
- calibration ECE 0.0496 → 0.0250;
- 4 selective-reread cases;
- `F03_VERIFY=PASS`;
- full `BASELINE_VERIFY=PASS`;
- proof: `docs\PROOF_F03_VISION_OCR.md`;
- release tag: `v0.3.0-vision-ocr`.

## Current phase

`F04 — ISOLATED_BROWSER_COMPUTER_WORKER — IN_PROGRESS`

Objective:
Build isolated browser/computer execution without taking over the operator's main desktop.

Required deliverables:
- Playwright browser process;
- isolated browser profiles;
- DOM observation;
- semantic element locator;
- click/type/form/upload/download;
- screenshot/DOM evidence;
- session persistence;
- layout-drift recovery;
- Computer Worker contract for isolated Windows/VM session.

Exit gate:
- synthetic sites/forms;
- upload/download proof;
- browser crash recovery;
- session restore;
- moved-element recovery;
- deadline enforcement;
- operator desktop remains independent;
- full baseline remains PASS;
- F04 proof + clean commit + state advancement exist.

## Next phase

`F05 — DURABLE_MISSION_RUNTIME — PLANNED`

## Terminal phase

`F13 — PERSONAL_PRODUCTION_V1_0`

Terminal label: `MISSION_PROVEN`.
