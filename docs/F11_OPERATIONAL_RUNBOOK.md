# F11 Operational Runbook — Availability and Concurrency

Canonical root: D:\TIMED-MISSION-AGENT
Target: PC Vendas / Windows

## Commands

- Dedicated proof: scripts/verify_f11.ps1.
- Full regression: scripts/verify_baseline.ps1.
- Supervised worker health: from supervisor-go, go run ./cmd/tma-supervisor serve-health 0.
- The health command prints a loopback-only URL. GET /healthz and /readyz; expect HTTP 200 for normal idle/operational pool.
- Independent Rust matrix: run availability-rust binary with matrix <disposable-db-file> <1|4|8|16>.
- Continuous soak: run availability-rust binary with soak 120. Temporary databases are created in the system temporary directory and removed by the harness.
- F05 process restart drill: scripts/verify_f05_recovery.ps1.
- Go data race test: from supervisor-go, go test -race ./supervisor.
- Rust formatting/lint: from availability-rust, cargo fmt --check and cargo clippy --all-targets -- -D warnings.

## Control and failure rules

- The F05 DurableStore is the single mission lease, checkpoint, fencing and queue authority.
- A claim is not a validated mission completion. F07 independent evidence validation is still mandatory for durable Succeeded.
- Go workers cap concurrent runs; a full queue rejects with ErrBackpressure, and a repeated task ID rejects with ErrDuplicateTask.
- Resource-scoped mutexes prevent simultaneous use of the same resource key.
- Workers restarted within the configured attempt budget and respect cancellation. A stalled operation is exposed through watchdog_stalled / HTTP 503.
- Watchdog health detection is not a claim of forced termination of uncooperative in-process code.
- For crashed external durable workers, use F05 lease expiration/reconcile/reclaim and verify the fencing token increment and checkpoint.
- Audit the hash-chain ledger and SQLite integrity before concluding recovery.
- Never rerun a terminal F05 mission or bypass the F07 validation receipt requirement.
- No external platform workflow or financial transaction is performed in F11.

## Interpreting proof

The matrix tests synthetic queued missions on Windows using real SQLite concurrency. A 120-second soak shows short-run stability, not 24/7 uptime. Longer observation belongs to operational qualification before production. Optional Elixir/OTP external supervision is not enabled by F11.
