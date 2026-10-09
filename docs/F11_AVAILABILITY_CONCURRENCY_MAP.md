# F11 — Availability and Concurrency: Requirements

Status: IN_PROGRESS
All claims require measured evidence, not simulated performance scores.
| ID | Requirement | Implementation/proof |
|---|---|---|
| AVC-001 | Independent availability crate | availability-rust |
| AVC-002 | 1-worker matrix | matrix_1 |
| AVC-003 | 4-worker matrix | matrix_4 |
| AVC-004 | 8-worker matrix | matrix_8 |
| AVC-005 | 16-worker matrix | matrix_16 |
| AVC-006 | Real F05 SQLite queue | DurableStore |
| AVC-007 | Atomic claim | claim_next |
| AVC-008 | Durable checkpoint | save_checkpoint |
| AVC-009 | Unique claims | BTreeSet |
| AVC-010 | Duplicate detection | duplicates |
| AVC-011 | Lost task detection | lost |
| AVC-012 | Fencing token | mission.fence_seq |
| AVC-013 | Event-chain validation | verify_ledger |
| AVC-014 | SQLite integrity | integrity_check |
| AVC-015 | Deadlock detection | finite queue drain |
| AVC-016 | Timed concurrency results | elapsed_ms |
| AVC-017 | Long-running soak harness | soak |
| AVC-018 | Soak all tiers | 1/4/8/16 |
| AVC-019 | Disposable DBs per run | tmp db |
| AVC-020 | No provider API in harness | purity |
| AVC-021 | Go supervised worker pool | supervisor/pool.go |
| AVC-022 | Max 16 concurrency | NewPool |
| AVC-023 | Queue backpressure | ErrBackpressure |
| AVC-024 | Duplicate task admission rejected | ErrDuplicateTask |
| AVC-025 | Controlled restarts | RunWithRestart |
| AVC-026 | Context cancellation | Stop |
| AVC-027 | Resource-key mutex | resource locks |
| AVC-028 | Health state | Health |
| AVC-029 | Stalled worker watchdog | watchdog_stalled |
| AVC-030 | HTTP /healthz | Handler |
| AVC-031 | HTTP /readyz | Handler |
| AVC-032 | Loopback-only listener | serve-health |
| AVC-033 | Graceful signal shutdown | signal.NotifyContext |
| AVC-034 | Go race detector | go test -race |
| AVC-035 | Go compiler/vet | go vet |
| AVC-036 | Go matrix test | TestF11ConcurrencyMatrix |
| AVC-037 | Go lock test | TestF11ResourceLockSerializesSameKey |
| AVC-038 | Go backpressure test | TestF11BackpressureAndHealthEndpoints |
| AVC-039 | Go watchdog/cancel test | TestF11WatchdogStallAndCancellation |
| AVC-040 | Go restart budget test | TestF11BoundedRestart |
| AVC-041 | Go 16-parallel submissions | TestF11ConcurrentSubmissionsWithoutDataRace |
| AVC-042 | Go duplicate task test | TestF11DuplicateSubmissionRejected |
| AVC-043 | Real process kill/recovery | F05 recovery script |
| AVC-044 | Planned recovery | F05 recovered planned |
| AVC-045 | Running recovery | F05 recovered running |
| AVC-046 | Validating recovery | F05 recovered validating |
| AVC-047 | F07 retained independent success | previous validated gate |
| AVC-048 | F10 retained concurrency ledger | prior regression |
| AVC-049 | JSON health contract | schema/fixture |
| AVC-050 | JSON matrix report contract | schema/fixture |
| AVC-051 | Purity scanner | verify_f11_purity.py |
| AVC-052 | Requirement map | verify_f11_map.py |
| AVC-053 | Dedicated gate | verify_f11.ps1 |
| AVC-054 | Global regression | verify_baseline.ps1 |
| AVC-055 | Canonical continuation to F12 | continuity |
| AVC-056 | Git release and proof | docs + tag |
