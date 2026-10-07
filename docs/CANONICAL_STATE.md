# CANONICAL STATE

Project: TIMED-MISSION-AGENT
Canonical root: D:\TIMED-MISSION-AGENT
GitHub: https://github.com/andrebarros78/Worker-universal.git

## Sovereign objective

Generate dependable personal income by turning eligible automatable work into completed, validated missions with minimal operator burden. Reliability and economic usefulness are first-class product requirements.

## Canonical entry

Every new AI/chat/operator starts at:

00_START_HERE.md

Machine-readable execution pointer:

state\EXECUTION_STATE.json

## Closed baselines

### F00 — POLYGLOT_V0_2 — CLOSED
- tag: v0.2.0-polyglot-baseline
- proof: docs\PROOF_POLYGLOT_V0_2.md

### F01 — CONCEPT_EXECUTIVE_CONTINUITY_BASELINE — CLOSED
- tag: v0.2.1-executive-baseline
- proof: docs\PROOF_F01_EXECUTIVE_CONTINUITY.md

### F02 — CAPABILITY_FOUNDATION — CLOSED
- tag: v0.2.2-capability-foundation
- proof: docs\PROOF_F02_CAPABILITY_FOUNDATION.md

### F03 — VISION_OCR_DOCUMENT_INTELLIGENCE — CLOSED
- map: VIS-001..VIS-035;
- local Tesseract provider with provider-neutral document authority;
- canonical benchmark: 16 cases / 112 fields / 100% accuracy and validation;
- proof: docs\PROOF_F03_VISION_OCR.md;
- tag: v0.3.0-vision-ocr.

Remediation discovered later:
- R03-01 regression benchmark output is now non-mutating;
- proof: docs\REMEDIATION_F03R1_REGRESSION_GATE_HYGIENE.md.

### F04 — ISOLATED_BROWSER_COMPUTER_WORKER — CLOSED
- map: BRC-001..BRC-040;
- Playwright 1.64.0;
- Playwright-managed Chromium;
- headless mode enforced;
- per-mission BrowserContext isolation;
- semantic DOM locator strategy;
- form fill/click/upload/download;
- DOM/screenshot/download/error evidence;
- storageState session persistence;
- real browser crash recovery;
- replay-safe retry policy;
- unsafe action no-replay;
- moved-element recovery;
- deadline containment;
- download path containment;
- Computer Worker dedicated-session/VM contract;
- isolation proof: browser session 0, Explorer session 1, sharedOperatorSession=false;
- 14 focused Node tests PASS;
- F04_VERIFY=PASS;
- full baseline required and proven at closure;
- proof: docs\PROOF_F04_BROWSER_COMPUTER.md;
- release tag: v0.4.0-isolated-browser-worker.

## Current phase

F05 — DURABLE_MISSION_RUNTIME — IN_PROGRESS

Objective:
Make mission execution survive worker/process and machine restart boundaries without losing canonical state or accepting duplicate/stale results.

Required deliverables:
- persistent mission repository;
- durable queue;
- append-only event journal;
- checkpoints/snapshots;
- idempotency keys;
- leases and fencing tokens;
- worker result deduplication;
- stale-result rejection;
- restart reconciliation;
- reboot-boundary recovery proof.

Exit gate:
- worker killed during each meaningful mission state;
- supervisor restart;
- simulated/reproducible machine restart boundary;
- duplicate result injection;
- lease expiry/race;
- ledger consistency;
- full baseline PASS;
- F05 proof + clean commit + state advancement.

## Next phase

F06 — PLANNER_ROUTER_KNOWLEDGE — PLANNED

## Terminal phase

F13 — PERSONAL_PRODUCTION_V1_0

Terminal label: MISSION_PROVEN.
