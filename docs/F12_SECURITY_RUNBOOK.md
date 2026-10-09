# F12 — Security and Disaster Recovery Runbook

Environment: PC Vendas. Root: D:\TIMED-MISSION-AGENT.

## Authority boundaries

F05 controls durable state, fencing and reconciliation; F07 controls independent result validation; F08 controls promotion; F10 controls economic accounting; F11 controls concurrency. F12 adds explicit operational security controls without replacing any prior authority.

## CLI operations

Run using the project Python .venv. The operational entrypoint is security-runtime/cli.py.

- validate-config --root ROOT --config FILE: reject inline secret fields and enforce non-resolving env://TMA_ secret references.
- backup --root ROOT --source LIVE_DB --destination NEW_BACKUP_DIR: online SQLite snapshot, integrity check, HMAC-signed manifest and private directory ACL.
- restore --root ROOT --snapshot SNAPSHOT_DIR --destination NEW_RESTORE_DIR: verify signature/hash/integrity and restore separately; never overwrite.
- audit-record --root ROOT --db AUDIT_DB --event ID --actor ACTOR --operation ACTION --outcome OUTCOME --resource RESOURCE --at MILLISECONDS.
- audit-verify --root ROOT --db AUDIT_DB: verify append-only SHA256 hash chain.
- inventory-create --root ROOT --destination NEW_JSON and inventory-verify --root ROOT --source JSON: sign and verify allowlisted dependency lockfiles.

Signing key is read only from TMA_F12_SIGNING_KEY_HEX, a 32-byte-minimum hex-encoded key supplied in the trusted process environment. Never put it in a command argument, Git, log or snapshot. Store and rotate production keys in an external vault under a separate operating policy; F12 does not create a vault or migrate real secrets.

## Controlled recovery

1. Stop admission of new tasks via existing F11 supervision. Never delete WAL/SHM from a live SQLite database.
2. Capture online snapshot into new ACL-private directory within an approved root, with SHA256 and HMAC manifest.
3. Verify manifest, SQLite integrity, event chain and audit records.
4. Restore only into a new isolated directory, retaining the original.
5. Run tma-security-verify verify against the restored F05 database. Verify ledger events, snapshots, queue, SQLite integrity and required PRAGMAs.
6. Allow F05 fencing/reconciliation to reclaim only eligible, nonterminal work. F07 independent validation is still required for Succeeded.
7. If corruption or missing dependencies are detected, fail closed and preserve originals for analysis.

## Security and operating limitations

Paths are confined to the explicitly approved root; symlinks/traversal rejected. NTFS ACLs on newly created backup directories disable inheritance and grant only the executing identity, SYSTEM and Administrators. The audit trail accepts constrained metadata only, not raw credentials; 16 simultaneous audit writes were tested.

The real F05 database was restored in an offline synthetic integration test, not from paid-provider production. A combined process-kill-with-online-backup test was blocked before execution and was not counted. The existing independent F05 kill/recovery drill was proven in F11 and remains in the regression. These do not prove a production failover or multi-day uptime.

GNATprove remains unavailable: source-level SPARK code is not a formal proof. When GNATprove is installed, the F12 gate runs it and requires success.
