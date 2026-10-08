# F05 — Durable Mission Runtime Complete Map

Phase authority: F05
Status: IN_PROGRESS

Goal: preserve canonical mission truth across worker/process/machine fault boundaries without duplicate execution, stale writes, lost queue entries, or journal corruption.

| ID | Requirement | F05 deliverable | Proof |
|---|---|---|---|
| DUR-001 | Canonical durable store in Rust | DurableStore inside tma-core | Rust tests |
| DUR-002 | SQLite local persistence | SQLite database file | reopen tests |
| DUR-003 | WAL mode | journal_mode=WAL | pragma gate |
| DUR-004 | Strong commit durability | synchronous=FULL | pragma gate |
| DUR-005 | Persistent mission repository | missions projection table | reopen tests |
| DUR-006 | Durable queue | queue table | restart/reconcile tests |
| DUR-007 | Append-only event journal | mission_events + no-update/delete triggers | immutability tests |
| DUR-008 | Event checksum chain | SHA-256 previous-hash chain | ledger verification |
| DUR-009 | Deterministic snapshots | mission_snapshots | recovery test |
| DUR-010 | Checkpoint persistence | checkpoint payload + snapshot | kill/reopen test |
| DUR-011 | Idempotency key | unique idempotency key | duplicate create test |
| DUR-012 | Idempotent create replay | same key/payload returns existing mission | test |
| DUR-013 | Idempotency conflict rejection | same key/different mission data rejected | test |
| DUR-014 | Lease acquisition | owner + expiry | test |
| DUR-015 | Lease renewal | same owner/token extends expiry | test |
| DUR-016 | Lease expiry | expired lease is reclaimable | test |
| DUR-017 | Monotonic fencing tokens | per-mission fence sequence | race test |
| DUR-018 | Stale fencing rejection | old token cannot mutate | stale-result test |
| DUR-019 | Worker ownership check | wrong owner cannot mutate | test |
| DUR-020 | Durable state transition | transactional projection + event + snapshot | test |
| DUR-021 | Core state-machine authority | transition validated by tma-core allowed() | test |
| DUR-022 | Result idempotency | unique result_id | duplicate-result test |
| DUR-023 | Duplicate result rejection | second result does not create second transition | test |
| DUR-024 | Terminal result hash | accepted terminal result recorded | test |
| DUR-025 | Late/stale result rejection | expired/replaced lease cannot commit | test |
| DUR-026 | Queue completion cleanup | terminal mission leaves queue | test |
| DUR-027 | Restart reconciliation | nonterminal missions restored to queue | reopen test |
| DUR-028 | Expired lease cleanup | reconcile removes expired leases | test |
| DUR-029 | Running-state recovery | running mission reclaimable after lease expiry | kill drill |
| DUR-030 | Validating-state recovery | validating mission reclaimable after lease expiry | kill drill |
| DUR-031 | Planned-state recovery | planned mission survives worker absence | kill/reopen drill |
| DUR-032 | Simulated reboot boundary | close process, reopen DB, reconcile, resume | recovery drill |
| DUR-033 | Supervisor restart boundary | durable worker process killed/restarted | recovery drill |
| DUR-034 | SQLite integrity check | PRAGMA integrity_check=ok | gate |
| DUR-035 | Ledger consistency check | event chain + snapshot references valid | gate |
| DUR-036 | Queue uniqueness | one queue row per mission | schema/test |
| DUR-037 | Single active lease | one lease row per mission | schema/race test |
| DUR-038 | Lease race | concurrent claimers yield one active owner | 16-thread test |
| DUR-039 | Transaction serialization | BEGIN IMMEDIATE claim/update | concurrency test |
| DUR-040 | No external broker dependency | no Redis/RabbitMQ/Kafka required | purity gate |
| DUR-041 | Provider/vendor neutrality | durable core has no API/LLM/browser provider dependency | purity gate |
| DUR-042 | Recovery report | reconcile returns recovery counts | test |
| DUR-043 | Durable runtime CLI probe | tma-durable-probe | process-kill drill |
| DUR-044 | Process kill at meaningful states | planned/running/validating | PowerShell drill |
| DUR-045 | One-command phase gate | verify_f05.ps1 | F05_VERIFY=PASS |

## Storage model

SQLite is the first personal single-host persistence adapter. It is local infrastructure, not product/domain authority.

The authoritative current state remains the Rust mission state machine. The SQLite mission row is its durable projection, while mission_events is the append-only causal history.

## Non-goals

F05 does not implement planner reasoning, platform-specific business logic, distributed multi-host consensus, Redis/RabbitMQ/Kafka, or economic scheduling.
