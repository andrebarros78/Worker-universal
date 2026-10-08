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
- proof: docs\PROOF_F03_VISION_OCR.md;
- tag: v0.3.0-vision-ocr.

### F04 — ISOLATED_BROWSER_COMPUTER_WORKER — CLOSED
- map: BRC-001..BRC-040;
- proof: docs\PROOF_F04_BROWSER_COMPUTER.md;
- tag: v0.4.0-isolated-browser-worker.

Remediation discovered during F05:
- R04-01 deadline-limited timeout classification corrected without weakening deadlines;
- 5 repeated legacy worker runs PASS;
- F04_VERIFY=PASS;
- proof: docs\REMEDIATION_F04R1_DEADLINE_CLASSIFICATION.md.

### F05 — DURABLE_MISSION_RUNTIME — CLOSED
- map: DUR-001..DUR-045;
- canonical mission persistence is implemented in Rust;
- SQLite uses WAL + synchronous FULL;
- durable mission projection + queue;
- append-only SHA-256 event chain;
- append-only snapshots/checkpoints;
- mission idempotency keys;
- result idempotency;
- leases + monotonic fencing tokens;
- stale/wrong-owner/expired result rejection;
- terminal queue cleanup;
- deterministic restart reconciliation;
- 16-way claim race yields one owner;
- real process kill/reopen proof for planned/running/validating;
- Running resumes Running;
- Validating resumes Validating;
- checkpoint survives process death;
- fresh worker receives a newer fencing token;
- SQLite integrity and ledger chain verified;
- 19 Rust tests PASS;
- F05_VERIFY=PASS;
- full BASELINE_VERIFY=PASS;
- proof: docs\PROOF_F05_DURABLE_RUNTIME.md;
- release tag: v0.5.0-durable-runtime.

## Current phase

F06 — PLANNER_ROUTER_KNOWLEDGE — IN_PROGRESS

Objective:
Convert a high-level mission into a bounded executable plan and choose the best available capability.

Required deliverables:
- mission classifier;
- planner interface;
- capability requirements;
- Router scoring;
- deterministic fallback policy;
- Knowledge/Research interface;
- LLM provider abstraction;
- local/no-LLM fallback for deterministic mission classes.

Exit gate:
- no provider lock-in;
- API/MCP/browser selection fixtures;
- provider outage fallback;
- plan bounded by deadline/cost/policy;
- deterministic task works without LLM/API;
- F06 proof + clean commit + state advancement.

## Next phase

F07 — INDEPENDENT_VALIDATION_RECOVERY — PLANNED

## Terminal phase

F13 — PERSONAL_PRODUCTION_V1_0

Terminal label: MISSION_PROVEN.
