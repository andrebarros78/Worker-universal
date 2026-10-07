# Executive Project — TIMED-MISSION-AGENT

## 1. Executive objective

Build a Universal Mission Worker with a clean deterministic core and replaceable execution capabilities, progressing through explicit phases from the verified POLYGLOT_V0_2 baseline to PERSONAL_PRODUCTION_V1_0.

## 2. Architectural stack

### Deterministic authority — Rust
Owns mission state, deadlines, idempotency, fencing, result gate and invariant enforcement.

### Runtime supervision — Go
Owns worker lifecycle, process health, restart budget, concurrency resources and service operation.

### Web execution — TypeScript / Node / Playwright
Owns DOM observation, page actions, forms, downloads/uploads, session-aware browser execution and web evidence.

### Intelligence — Python
Owns OCR, document intelligence, computer vision, ML/LLM adapters and experimental perception algorithms.

### Availability — Elixir / Erlang OTP
Owns optional outer supervision and future multi-node availability once the single-host runtime proves stable.

### Formal proof — Ada / SPARK
Owns small selected invariants only after GNATprove is available. It is not in the mandatory boot path.

### Integration contracts — JSON Schema
All cross-runtime requests/results use versioned schemas.

## 3. Infrastructure services

These are deterministic services, not "agents":

- Capability Registry
- Adapter Registry/SDK
- Persistent State
- Durable Queue
- Evidence Ledger
- Artifact Store
- Session Manager
- Secret Vault interface
- Policy/Eligibility Engine
- Telemetry/Metrics
- Replay Engine
- Simulation Harness
- Watchdog/Health
- Economic Ledger

## 4. Dependency policy

### Mandatory local dependencies for baseline
- Rust toolchain
- Go toolchain
- Node
- Python
- SQLite

### Optional at runtime
- external APIs
- MCP servers/connectors
- remote LLM providers
- remote OCR/vision providers
- PostgreSQL
- Redis/message broker
- Elixir distributed nodes
- cloud VM

The system must start and run deterministic/self-test capabilities without any external API key.

## 5. Decision hierarchy

For each required action the Planner/Router selects the best registered capability according to:

1. capability correctness;
2. deterministic/direct integration availability;
3. expected latency;
4. reliability history;
5. economic cost;
6. recovery options.

Preferred execution order where equivalent:

API/native/MCP → browser DOM → isolated computer/UI.

## 6. Pre-registration gate

When a work source has onboarding:

DISCOVER REQUIREMENTS
→ ELIGIBILITY
→ REQUIRED DATA READY
→ TRAIN/SIMULATE
→ BENCHMARK
→ BLOCKER CLASSIFICATION
→ REGISTRATION_READY

No production adapter is enabled merely because registration succeeded.

## 7. Post-registration gate

ACCOUNT_CONFIRMED
→ CONFIGURATION_VALID
→ PERMISSIONS_VALID
→ SYNTHETIC MISSION
→ RESULT VALIDATION
→ FAILURE/RECOVERY TEST
→ PRODUCTION_READY

## 8. Promotion gates

Every new capability/adapter has four states:

EXPERIMENTAL → TESTED → QUALIFIED → PRODUCTION

Promotion requires evidence, not code presence.

## 9. Concurrency model

Concurrency evolves in controlled steps:

1 → 4 → 8 → 16 workers/channels.

At each level test:

- deadlock;
- race;
- duplicate submission;
- lost result;
- stale result;
- state corruption;
- ledger ordering;
- lease/fencing behavior;
- recovery after worker death.

## 10. Persistence model

Single-host default:

SQLite WAL + append-only event journal + snapshots/checkpoints.

Scale-out migration target:

PostgreSQL/durable distributed queue behind the same repository interfaces.

No database technology is allowed to leak into mission-domain contracts.

## 11. Release governance

A phase may be marked CLOSED only with:

- deliverables complete;
- focused tests PASS;
- full applicable regression PASS;
- evidence artifact;
- Git commit;
- canonical state update;
- next phase pointer.

A release tag identifies a proven baseline, not merely a feature milestone.

## 12. End condition

PERSONAL_PRODUCTION_V1_0 is achieved when:

- supported mission classes run end-to-end;
- browser/API/MCP/vision capabilities are selected dynamically;
- durable state survives worker/process restart;
- duplicate/stale execution is prevented;
- validation is independent;
- recovery is proven by fault injection;
- economic reporting works from observed data;
- adapters can be added without changing Rust core;
- one-command health/proof gate passes;
- canonical state and operational runbook permit another AI/operator to continue without prior chat context.
