# Proof — F05 Durable Mission Runtime

Date: 2026-10-07
Host: PC Vendas
Canonical root: D:\TIMED-MISSION-AGENT

## Objective

Make canonical mission execution survive worker, process, supervisor and machine-restart boundaries without losing queue/state, accepting duplicate results, accepting stale workers, or corrupting causal history.

## Authority

Mission truth remains in Rust.

F05 extends tma-core with a durable SQLite projection and append-only causal journal. Python EvidenceLedger remains evidence infrastructure and is not mission-state authority.

## Requirement map

- docs/F05_DURABLE_RUNTIME_MAP.md
- DUR-001..DUR-045
- docs/F05_IMPLEMENTATION_MATRIX.md

Observed:
F05_MAP_OK requirements=45 mapped=45

## Durable storage

Implemented in:
core-rust/src/durable.rs

Storage:
- SQLite;
- journal_mode=WAL;
- synchronous=FULL;
- foreign_keys=ON;
- 10-second busy timeout.

Tables:
- missions — current durable projection;
- queue — durable work queue;
- leases — one active lease per mission;
- mission_events — append-only causal journal;
- mission_snapshots — append-only state/checkpoint snapshots;
- results — result idempotency and acceptance/rejection ledger.

Append-only triggers reject UPDATE/DELETE on mission_events and mission_snapshots.

## Journal integrity

Each mission event contains:
- per-mission ordinal;
- previous event hash;
- SHA-256 event hash;
- timestamp;
- kind;
- payload.

verify_ledger() replays the hash chain and validates snapshot references.

A test deliberately removes the update trigger inside an isolated temporary database, tampers with event payload, and proves the verifier detects corruption.

SQLite PRAGMA integrity_check is also required to return ok.

## Idempotency

Mission creation requires a unique idempotency_key.

Behavior:
- same key + identical mission specification → Existing;
- same key + different mission data → IdempotencyConflict.

Result submission requires a unique result_id.

Behavior:
- first valid result → accepted once;
- exact replay → Duplicate without a second transition/event;
- same result_id with changed identity/payload → ResultIdConflict.

## Leases and fencing

Claiming a mission:
- uses TransactionBehavior::Immediate;
- selects one eligible durable queue item;
- issues a monotonically increasing per-mission fencing token;
- creates/replaces the single lease row.

Lease mutation requires:
- matching worker owner;
- matching fencing token;
- non-expired lease.

Expired or replaced workers cannot commit state/checkpoints/results.

## Durable recovery

Nonterminal state is never reset during reconciliation.

If a worker dies:
- expired lease is removed;
- missing queue row is recreated;
- Running remains Running;
- Validating remains Validating;
- checkpoint remains intact;
- next worker receives a strictly newer fencing token.

Terminal missions are removed from the queue.

## Concurrency proof

Rust test:
sixteen_way_claim_race_has_one_active_owner

Observed:
- 16 concurrent claimers;
- exactly 1 ClaimOutcome::Claimed;
- one active lease due SQLite IMMEDIATE transaction serialization.

This is a focused F05 lease-race proof, not the later full F11 concurrency-hardening campaign.

## Real process-kill recovery drill

Executable:
core-rust/src/bin/durable_probe.rs

Gate:
scripts/verify_f05_recovery.ps1

The gate launches a real durable worker process, waits until the requested state is durably persisted, force-kills the process, starts a fresh process, reopens the same SQLite database, reconciles, claims the mission and verifies journal/database integrity.

Observed:

### Planned

READY state=planned token=0

After process kill/reopen:

RECOVERED state=Running token=1 checkpoint= expired_leases=0 events=4 snapshots=3 integrity=ok

F05_RECOVERY_PLANNED=PASS

### Running

READY state=running token=1

After process kill/reopen:

RECOVERED state=Running token=2 checkpoint=checkpoint=running expired_leases=1 events=7 snapshots=4 integrity=ok

F05_RECOVERY_RUNNING=PASS

### Validating

READY state=validating token=1

After process kill/reopen:

RECOVERED state=Validating token=2 checkpoint=checkpoint=validating expired_leases=1 events=8 snapshots=5 integrity=ok

F05_RECOVERY_VALIDATING=PASS

Overall:

F05_PROCESS_RECOVERY=PASS

This proves process/supervisor replacement and a simulated machine-reboot boundary because recovery occurs in a new process from durable disk state only.

## Focused Rust verification

tma-core version:
0.5.0

Dependencies:
- rusqlite 0.32.1 with bundled SQLite;
- sha2 0.10.9.

Native build dependency:
- MSYS2 20260611;
- UCRT64 GCC 16.2.0.

The GCC path is project-scoped for the build gates; no global Rust architecture change was made.

Observed focused tests:
- 19 Rust tests PASS;
- cargo fmt --check PASS;
- cargo clippy --all-targets -- -D warnings PASS;
- durable probe build PASS.

## F05 dedicated gate

scripts/verify_f05.ps1

Observed:

F05_MAP_OK requirements=45 mapped=45
F05_PURITY_OK storage=sqlite_wal broker_dependencies=0 provider_dependencies=0 transaction=immediate
F05_VERIFY=PASS

## Build-output hardening discovered during F05

The original core-rust target directory on D: produced intermittent Windows filesystem errors during large Rust builds.

Correction:
- official core build output moved to:
  C:\ProgramData\SentinelX\workspace\tma-build\core-rust
- source remains under:
  D:\TIMED-MISSION-AGENT\core-rust
- tma-core self-test now explicitly selects --bin tma-core because F05 added tma-durable-probe.

This is build-output isolation only. It does not create a second source/project root.

## Full product regression before closure

Observed with F05 still active:

CONTINUITY_OK current=F05 next=F06 terminal=F13 root=D:\TIMED-MISSION-AGENT

F05_RECOVERY_PLANNED=PASS
F05_RECOVERY_RUNNING=PASS
F05_RECOVERY_VALIDATING=PASS
F05_PROCESS_RECOVERY=PASS
F05_MAP_OK requirements=45 mapped=45
F05_PURITY_OK storage=sqlite_wal broker_dependencies=0 provider_dependencies=0 transaction=immediate
F05_VERIFY=PASS
BASELINE_VERIFY=PASS

SPARK remains accurately reported:
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

## Safety/non-goals

F05 contains no:
- Redis;
- RabbitMQ;
- Kafka;
- provider SDK;
- LLM dependency;
- browser dependency;
- platform-specific business logic.

SQLite is the local persistence adapter for the personal single-host product stage.

## Final post-advancement regression

Observed after the canonical pointer advanced to F06:

CONTINUITY_OK current=F06 next=F07 terminal=F13 root=D:\TIMED-MISSION-AGENT

F04_VERIFY=PASS

F05_RECOVERY_PLANNED=PASS
F05_RECOVERY_RUNNING=PASS
F05_RECOVERY_VALIDATING=PASS
F05_PROCESS_RECOVERY=PASS
F05_MAP_OK requirements=45 mapped=45
F05_PURITY_OK storage=sqlite_wal broker_dependencies=0 provider_dependencies=0 transaction=immediate
F05_VERIFY=PASS

SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

BASELINE_VERIFY=PASS

## Closure decision

F05 exit criteria are satisfied. The post-advancement full regression passed.

Canonical advancement:
- F05 → CLOSED
- F06 PLANNER_ROUTER_KNOWLEDGE → IN_PROGRESS
- F07 INDEPENDENT_VALIDATION_RECOVERY → PLANNED

Release tag:
v0.5.0-durable-runtime
