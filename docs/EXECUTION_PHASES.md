# Detailed Execution Phases

## F00 — Polyglot deterministic baseline — CLOSED

Purpose: establish the technology split and prove every mandatory runtime layer independently.

Already proven under tag `v0.2.0-polyglot-baseline`.

---

## F01 — Concept and executive continuity baseline

### Objective
Make the product independent of the originating chat/AI.

### Deliverables
- START HERE entrypoint.
- AGENTS rules.
- Project Concept.
- Executive Project.
- Detailed phases.
- Phase Ledger.
- machine-readable execution state.
- continuity verification script.
- updated canonical state.

### Exit gate
A fresh process can determine root, objective, current phase, next phase, baseline commit and proof files without chat context. All baseline tests remain green. Clean Git commit and tag required.

---

## F02 — Capability foundation

### Objective
Create the stable extension plane without contaminating Rust core.

Complete requirement authority: `docs/F02_CAPABILITY_MAP.md` (CAP-001..CAP-045).
Implementation/proof mapping: `docs/F02_IMPLEMENTATION_MATRIX.md`.

### Deliverables
- Capability Registry.
- Adapter SDK and versioned contracts.
- capability health/status model.
- Policy/Eligibility Engine interface.
- SecretProvider/Vault interface.
- Artifact Store interface.
- Session Manager interface.
- Tool Bus supporting native/API/MCP/browser/AI capability descriptors.
- local implementations sufficient for offline tests.

### Exit tests
- registry add/remove/health/fallback;
- incompatible contract rejection;
- missing optional provider does not prevent boot;
- secrets never serialized in mission/evidence;
- deterministic capability selection fixtures.

---

## F03 — Vision/OCR and document intelligence

### Objective
Make timed document tasks accurate and measurable.

Complete requirement authority: `docs/F03_VISION_OCR_MAP.md` (VIS-001..VIS-035).
Implementation/proof mapping: `docs/F03_IMPLEMENTATION_MATRIX.md`.

### Deliverables
- image ingestion;
- preprocessing;
- OCR provider interface;
- local OCR adapter;
- optional remote vision adapter;
- invoice field schema;
- multi-source field reconciliation;
- confidence calibration;
- selective re-read of uncertain fields;
- labeled benchmark corpus.

### Exit tests
- benchmark dataset versioned;
- field-level accuracy threshold defined and met;
- p50/p95/p99 measured;
- invalid CNPJ/date/math caught;
- confidence calibration evaluated;
- no single OCR engine is mission authority.

---

## F04 — Isolated Browser/Computer Worker

### Objective
Execute web/GUI work without taking the operator desktop.

Complete requirement authority: `docs/F04_BROWSER_COMPUTER_MAP.md` (BRC-001..BRC-040).
Implementation/proof mapping: `docs/F04_IMPLEMENTATION_MATRIX.md`.

### Deliverables
- Playwright browser process;
- isolated browser profiles;
- DOM observation;
- semantic element locator;
- click/type/form/upload/download;
- screenshot/DOM evidence;
- session persistence;
- layout-drift recovery;
- Computer Worker contract for isolated Windows/VM session.

### Exit tests
- synthetic sites/forms;
- download/upload proof;
- browser crash recovery;
- session restore;
- moved-element recovery;
- deadline enforcement;
- operator desktop remains independent.

---

## F05 — Durable mission runtime

### Objective
Make mission state survive process and machine faults.

Complete requirement authority: `docs/F05_DURABLE_RUNTIME_MAP.md` (DUR-001..DUR-045).
Implementation/proof mapping: `docs/F05_IMPLEMENTATION_MATRIX.md`.

### Deliverables
- persistent mission repository;
- durable queue;
- append-only events;
- checkpoints;
- idempotency keys;
- leases and fencing tokens;
- worker result deduplication;
- stale-result rejection;
- restart/reboot reconciliation.

### Exit tests
- kill worker during each state;
- restart supervisor;
- simulated machine reboot boundary;
- duplicate result injection;
- lease expiry/race;
- ledger consistency.

---

## F06 — Planner/Router/Knowledge

### Objective
Convert a high-level mission into a bounded executable plan and choose the best capability.

Complete requirement authority: `docs/F06_PLANNER_ROUTER_KNOWLEDGE_MAP.md` (PLN-001..PLN-050).
Implementation/proof mapping: `docs/F06_IMPLEMENTATION_MATRIX.md`.

### Deliverables
- mission classifier;
- planner interface;
- capability requirements;
- Router scoring;
- deterministic fallback policy;
- Knowledge/Research interface;
- LLM provider abstraction;
- local/no-LLM fallback for deterministic mission classes.

### Exit tests
- no provider lock-in;
- API/MCP/browser selection fixtures;
- provider outage fallback;
- plan bounded by deadline/cost/policy;
- deterministic task works without LLM/API.

---

## F07 — Independent Validation and Recovery

### Objective
Never confuse "executor returned success" with real success.

Complete requirement authority: `docs/F07_VALIDATION_RECOVERY_MAP.md` (VAL-001..VAL-055).
Implementation/proof mapping: `docs/F07_IMPLEMENTATION_MATRIX.md`.

### Deliverables
- Validation Agent;
- proof strategies per capability;
- failure taxonomy;
- Recovery Agent;
- bounded retry budget;
- recovery state transitions;
- evidence completeness gate.

### Exit tests
Inject every named failure class and prove expected recovery/terminal state.

---

## F08 — Training, benchmark and qualification gates

### Objective
Use the production execution engine in controlled training/benchmark flows before release.

### Deliverables
- timed mission harness;
- training mode;
- benchmark mode;
- PRE-REGISTRATION state machine;
- POST-REGISTRATION state machine;
- promotion criteria;
- capability scorecards.

### Exit tests
- reproducible timed runs;
- accuracy/latency thresholds;
- synthetic registration lifecycle;
- recovery before PRODUCTION_READY;
- no adapter skips promotion gate.

---

## F09 — Platform adapters

### Objective
Connect real work sources without coupling them to the core.

### Deliverables
- generic_web;
- generic_form;
- generic_invoice;
- initial HomeOcta adapter;
- initial 99Freelas adapter;
- adapter configuration schema;
- adapter-specific validation/evidence rules.

### Exit tests
Each adapter passes contract, simulation, failure-recovery and eligibility/policy gates independently.

---

## F10 — Economic Controller and Scheduler

### Objective
Optimize for dependable net income, not raw task count.

### Deliverables
- opportunity record;
- revenue/cost ledger;
- observed success probability;
- expected-value calculation;
- priority queue;
- concurrency allocation;
- stop-loss/minimum-margin configuration;
- income dashboard/report.

### Exit tests
- deterministic accounting;
- no double counting;
- scheduler fixtures;
- cost spikes alter ranking;
- negative-value mission can be rejected before expensive execution.

---

## F11 — Availability and concurrency hardening

### Objective
Prove continuous operation under concurrent and failure conditions.

### Deliverables
- watchdogs;
- health endpoints;
- Go supervisor hardening;
- optional Elixir/OTP outer supervision;
- multi-worker orchestration;
- resource locks;
- backpressure;
- soak test harness.

### Exit tests
1 → 4 → 8 → 16 concurrency, plus deadlock/race/duplicate/loss/corruption tests and long-duration soak.

---

## F12 — Security, formal and operational hardening

### Objective
Reduce operational fragility before production release.

### Deliverables
- secret isolation;
- least-privilege worker identities where practical;
- audit trail;
- dependency inventory;
- backup/restore;
- disaster recovery runbook;
- selected SPARK proofs once GNATprove is installed;
- release integrity checks.

### Exit tests
- secret non-disclosure;
- backup restore;
- corrupted-state recovery;
- dependency loss drills;
- formal proof evidence where toolchain exists.

---

## F13 — PERSONAL_PRODUCTION_V1_0

### Objective
Deliver the personal production system.

### Required proof
- supported mission runs end-to-end;
- mission survives worker death;
- result independently validated;
- deadline enforced;
- evidence persisted;
- economic result reported;
- restart/recovery proven;
- new adapter does not modify Rust core;
- one-command verification PASS;
- operational runbook complete;
- canonical state terminal or points to post-v1 maintenance backlog.

Terminal label: `MISSION_PROVEN`.
