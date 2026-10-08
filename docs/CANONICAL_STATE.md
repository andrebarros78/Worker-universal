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

### F06 — PLANNER_ROUTER_KNOWLEDGE — CLOSED
- map: PLN-001..PLN-050;
- tma-planner Rust orchestration crate;
- deterministic MissionClassifier + Planner;
- bounded execution DAG with cycle/dependency/budget validation;
- routing through F02 CapabilityRegistry + PolicyEngine;
- health/readiness/dependency/promotion/reliability/cost/latency filtering;
- provider preference and deterministic fallback;
- API/MCP/browser/vision fixtures;
- local/no-LLM deterministic fallback;
- Knowledge/Research + Reasoner provider abstractions;
- stable SHA-256 plan fingerprint;
- F05 durable plan checkpoint/restart integration;
- 16 focused Rust tests PASS;
- F06_CONTRACTS_OK;
- F06_MAP_OK 50/50;
- F06_PURITY_OK;
- F06_VERIFY=PASS;
- full BASELINE_VERIFY=PASS;
- proof: docs\PROOF_F06_PLANNER_ROUTER_KNOWLEDGE.md;
- release tag: v0.5.1-planner-router-knowledge.

### F07 — INDEPENDENT_VALIDATION_RECOVERY — CLOSED
- map: VAL-001..VAL-055;
- tma-validation Rust crate;
- executor result treated as untrusted input;
- independent evidence completeness/hash/confidence/deadline validation;
- proof strategies for Document/Vision, Browser, Research, Computer and deterministic capabilities;
- 16/16 F02 ErrorClass recovery matrix;
- bounded retry/deadline/replay-safe/side-effect policy;
- recovery checkpoints persisted through F05;
- real process kill/restart recovery PASS with fencing advancement;
- R05-01: Succeeded now requires independent append-only validation receipt;
- tma-core 0.5.2 current suite: 21 PASS;
- 19 focused F07 tests PASS;
- F07_CONTRACTS_OK;
- F07_MAP_OK 55/55;
- F07_PURITY_OK;
- F07_VERIFY=PASS;
- full BASELINE_VERIFY=PASS;
- proof: docs\PROOF_F07_VALIDATION_RECOVERY.md;
- release tag: v0.5.2-independent-validation-recovery.

Post-release remediation:
- R07-01 concurrent verification isolation is CLOSED;
- F03 canonical corpus gate is serialized;
- F03/F05/F07 temporary verification state is invocation-isolated;
- proof: docs\REMEDIATION_F07R1_CONCURRENT_VERIFICATION_ISOLATION.md.

### F08 — TRAINING_BENCHMARK_QUALIFICATION_GATES — CLOSED
- map: QUA-001..QUA-060;
- qualification-rust v0.6.0;
- Training + Benchmark modes;
- deterministic synthetic and wall-clock harnesses;
- success/accuracy/validation/p95/recovery qualification thresholds;
- deterministic scorecards and receipts;
- synthetic PRE_REGISTRATION → REGISTERED → POST_REGISTRATION → PRODUCTION_READY lifecycle;
- sequential capability promotion gate;
- R02-01 closes direct promotion skip;
- benchmark samples bound to F07 ValidationReport;
- 19 focused Rust tests PASS;
- F08_CONTRACTS_OK;
- F08_MAP_OK 60/60;
- F08_PURITY_OK;
- F08_VERIFY=PASS;
- full BASELINE_VERIFY=PASS;
- proof: docs\PROOF_F08_TRAINING_BENCHMARK_QUALIFICATION.md;
- release tag: v0.6.0-training-qualification.

### F09 — PLATFORM_ADAPTERS — CLOSED
- five initial adapters and F02 Adapter SDK;
- F04 isolated browser and F03/F06 capability delegation;
- F07-compatible SHA-256 evidence and F08 quarantine;
- 5 synthetic Chromium runs PASS; 5 process crash recoveries PASS;
- 8 focused Node tests PASS; contracts=8; map=50/50;
- F09_VERIFY=PASS and BASELINE_VERIFY=PASS;
- initial simulation-only scope, no live platform integration claimed;
- proof: docs/PROOF_F09_PLATFORM_ADAPTERS.md;
- release: v0.6.1-platform-adapters.

### F10 — ECONOMIC_CONTROLLER_SCHEDULER — CLOSED
- economic-rust v0.7.0, fixed-point BRL-cent accounting;
- durable SQLite WAL/FULL ledger, immutable hash chain and idempotent events;
- F05 Succeeded + F07 validation required for verified earnings;
- F02 Production/Operational capability gate for scheduler;
- profitability, daily budget, margin, stop-loss, deadline, locks and slots;
- accrued earnings/cash/receivables report; external settlement not independently reconciled;
- 23 focused tests PASS; 16 concurrent idempotent writers PASS;
- F10_CONTRACTS_OK fixtures=5;
- F10_MAP_OK 55/55;
- F10_PURITY_OK;
- F10_VERIFY=PASS;
- BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F10_ECONOMIC_CONTROLLER_SCHEDULER.md;
- tag: v0.7.0-economic-controller.

## Current phase

F11 — AVAILABILITY_CONCURRENCY_HARDENING — IN_PROGRESS

Objective:
Prove availability, restart/recovery and safe scaling through 1 → 4 → 8 → 16 concurrent workers.

Required deliverables:
- controlled concurrency matrix and availability benchmark;
- deadlock, lost update, duplicate dispatch and corruption checks;
- restart/crash/soak and recovery drills;
- resource contention and queue recovery;
- no mutation of prior-phase proofs.

Exit gate:
- concurrency tiers 1/4/8/16 PASS;
- failure injection and recovery PASS;
- independent ledger checks PASS;
- full baseline and F11 proof, release, state advancement.

## Next phase

F12 — SECURITY_FORMAL_OPERATIONAL_HARDENING — PLANNED

## Terminal phase

F13 — PERSONAL_PRODUCTION_V1_0

Terminal label: MISSION_PROVEN.
