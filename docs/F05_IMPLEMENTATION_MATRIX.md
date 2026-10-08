# F05 Implementation Matrix

| Requirements | Implementation | Proof |
|---|---|---|
| DUR-001..006 | core-rust/src/durable.rs | Rust durable tests |
| DUR-007..010 | append-only events, checksum chain, snapshots/checkpoints | ledger + recovery tests |
| DUR-011..013 | idempotency_key uniqueness and replay contract | tests |
| DUR-014..019 | lease/fencing ownership model | tests + race |
| DUR-020..026 | durable transition/result path | tests |
| DUR-027..033 | reconcile + tma-durable-probe + kill/reboot drill | verify_f05_recovery.ps1 |
| DUR-034..035 | integrity_check + verify_ledger | gate |
| DUR-036..039 | uniqueness + BEGIN IMMEDIATE + 16-way race | tests |
| DUR-040..041 | SQLite-only vendor-neutral persistence | purity gate |
| DUR-042 | ReconcileReport | tests |
| DUR-043..044 | core-rust/src/bin/durable_probe.rs | process drill |
| DUR-045 | scripts/verify_f05.ps1 | dedicated gate |
