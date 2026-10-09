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

## Proven Planner/Router/Knowledge

F06 PLANNER_ROUTER_KNOWLEDGE
- map: PLN-001..PLN-050;
- separate tma-planner Rust orchestration crate;
- deterministic mission classifier and execution DAG;
- F02 CapabilityRegistry/Policy integration;
- deterministic provider preference/fallback/tie-break;
- API/MCP/browser/vision routing fixtures PASS;
- provider-outage and missing-dependency handling PASS;
- local/no-LLM deterministic fallback PASS;
- Knowledge/Research and Reasoner abstractions;
- stable SHA-256 plan fingerprint;
- F05 durable checkpoint/restart integration PASS;
- 16 focused Rust tests PASS;
- F06_CONTRACTS_OK fixtures=4;
- F06_MAP_OK requirements=50 mapped=50;
- F06_PURITY_OK;
- F06_VERIFY=PASS;
- full product gate: BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F06_PLANNER_ROUTER_KNOWLEDGE.md;
- tag target: v0.5.1-planner-router-knowledge.

## Proven Independent Validation + Recovery

F07 INDEPENDENT_VALIDATION_RECOVERY
- map: VAL-001..VAL-055;
- separate tma-validation Rust crate;
- executor success treated as untrusted input;
- capability-specific independent proof strategies;
- evidence completeness/hash/size/confidence/deadline gates;
- F02 ErrorClass reused exactly: 16/16 classes covered;
- deterministic bounded RecoveryAgent;
- retry/deadline/replay-safe/side-effect hard gates;
- recovery fallback and bounded backoff;
- stable validation/recovery SHA-256 fingerprints;
- F05 durable recovery checkpoint/restart integration;
- R05-01 validated-success gate: Succeeded requires independent append-only validation receipt;
- tma-core current suite: 21 PASS / 0 FAIL;
- 19 focused F07 Rust tests PASS;
- real recovery process kill/restart PASS with fencing 1→2;
- F07_CONTRACTS_OK fixtures=4;
- F07_MAP_OK requirements=55 mapped=55;
- F07_PURITY_OK;
- F07_VERIFY=PASS;
- full product gate: BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F07_VALIDATION_RECOVERY.md;
- tag target: v0.5.2-independent-validation-recovery.

## Proven Training/Benchmark/Qualification Gates

F08 TRAINING_BENCHMARK_QUALIFICATION_GATES
- map: QUA-001..QUA-060;
- qualification-rust v0.6.0;
- training and benchmark modes;
- deterministic synthetic benchmark harness;
- real wall-clock deadline harness;
- success/accuracy/F07-validation/p95/recovery thresholds;
- p50/p95/p99 scorecards;
- dataset/version/seed binding;
- 5-run reproducibility PASS;
- PRE_REGISTRATION → REGISTERED → POST_REGISTRATION → PRODUCTION_READY lifecycle;
- sequential promotion Experimental → Tested → Qualified → Production;
- R02-01 direct promotion skip closed;
- benchmark samples bound to F07 ValidationReport;
- 19 focused Rust tests PASS;
- F08_CONTRACTS_OK fixtures=4;
- F08_MAP_OK requirements=60 mapped=60;
- F08_PURITY_OK;
- F08_VERIFY=PASS;
- full product gate: BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F08_TRAINING_BENCHMARK_QUALIFICATION.md;
- tag target: v0.6.0-training-qualification.

## Proven Initial Platform Adapters

F09 PLATFORM_ADAPTERS
- generic_web, generic_form, generic_invoice, homeocta, 99freelas;
- F02 Adapter SDK integration and real F04 headless browser;
- 5/5 synthetic executions and 5/5 crash/recovery drills PASS;
- F03 OCR / F06 research capability delegation;
- F07-compatible DOM/screenshot integrity checks;
- F08 Experimental/Configured quarantine;
- live external requests, assessment and proposal submission denied;
- 8 focused Node tests PASS;
- F09_CONTRACTS_OK fixtures=8;
- F09_MAP_OK requirements=50 mapped=50;
- F09_PURITY_OK;
- F09_VERIFY=PASS;
- full BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F09_PLATFORM_ADAPTERS.md;
- live provider integration NOT claimed.

## Proven Economic Controller and Scheduler

F10 ECONOMIC_CONTROLLER_SCHEDULER
- provider-neutral economic-rust v0.7.0;
- BRL cents fixed-point expected value, margin and profit/hour;
- F02 Production/Operational/Healthy capability gate;
- F05 terminal mission success + F07 independent validation binding;
- observed success probability from F05 states;
- SQLite append-only WAL ledger and SHA-256 hash chain;
- one earning per mission; one entry per external settlement reference;
- payment, verified earned revenue and cost kept separate;
- 16 concurrent idempotent cost writes PASS;
- 23 focused Rust tests PASS;
- F10_CONTRACTS_OK fixtures=5;
- F10_MAP_OK requirements=55 mapped=55;
- F10_PURITY_OK;
- F10_VERIFY=PASS;
- BASELINE_VERIFY=PASS;
- proof: docs/PROOF_F10_ECONOMIC_CONTROLLER_SCHEDULER.md;
- no actual paid work or bank reconciliation claimed.

## Proven Availability and Concurrency Hardening

F11 AVAILABILITY_CONCURRENCY_HARDENING
- availability-rust v0.8.0; real F05 durable SQLite claim + fencing + checkpoint;
- 1/4/8/16 concurrency PASS with no duplicate or lost claim;
- Go bounded pool, backpressure, idempotent task admission and resource locks;
- Go watchdog and HTTP loopback healthz/readyz PASS;
- Go race detector PASS;
- actual process kill/restart: planned/running/validating PASS;
- 30-second gate soak: 608 missions;
- extended 120-second soak: 100 cycles, 3200 missions, 4 tiers, integrity OK;
- F11_CONTRACTS_OK fixtures=2;
- F11_MAP_OK requirements=56 mapped=56;
- F11_PURITY_OK;
- F11_VERIFY=PASS;
- full BASELINE_VERIFY=PASS;
- runbook: docs/F11_OPERATIONAL_RUNBOOK.md;
- proof: docs/PROOF_F11_AVAILABILITY_CONCURRENCY_HARDENING.md;
- no 24/7 guarantee or multi-day soak claimed.

## Proven Security and Operational Recovery

F12 SECURITY_FORMAL_OPERATIONAL_HARDENING
- security-runtime with non-resolving secret references and audit metadata controls;
- append-only hash-chained SQLite audit with 16 concurrent writers;
- private Windows NTFS directory ACL on test backups;
- online SQLite backup with HMAC-SHA256 and streaming SHA256 checks;
- fail-closed restore into a new directory; overwrite, corruption and traversal denied;
- security-rust F05 journal verifier;
- 32 focused Python tests PASS;
- 4-worker/64-mission actual F05 persisted SQLite offline backup/restore PASS;
- signed inventory of 13 dependency and build manifests;
- F12_CONTRACTS_OK fixtures=4;
- F12_MAP_OK requirements=60 mapped=60;
- F12_PURITY_OK;
- F12_VERIFY=PASS and BASELINE_VERIFY=PASS;
- GNATPROVE_NOT_INSTALLED; formal proof NOT claimed;
- combined kill-and-backup drill blocked before execution, NOT claimed;
- docs/F12_SECURITY_RUNBOOK.md and docs/PROOF_F12_SECURITY_FORMAL_OPERATIONAL_HARDENING.md.

## Active construction lane

F13 PERSONAL_PRODUCTION_V1_0 — terminal acceptance IN_PROGRESS.

MISSION_PROVEN is not yet an achieved state.

## Formal proof status

Ada/SPARK sources exist, but GNATprove is not installed. Status remains:
SOURCE_READY_NOT_PROVEN.
