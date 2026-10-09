# Proof — F12 Security, Formal and Operational Hardening

Date: 2026-10-08
Host: PC Vendas
Root: D:\TIMED-MISSION-AGENT

## Scope and implementation

- Python-stdlib security-runtime: non-resolving secret references, constrained audit records, append-only SHA256 audit chain, ACL private backup directories, SQLite online backup with HMAC-SHA256 manifest, separate restore with hash/SQLite integrity checks, safe-root path confinement, dependency inventory and signed release metadata.
- security-runtime/cli.py: explicitly invoked operational tools; no automatic interception of all existing adapters is claimed.
- security-rust v0.9.0: independent F05 DurableStore restored journal, snapshots and queue verifier.
- Security unit and integration fixtures are synthetic; no production secrets, financial operations or provider logins were used.

## Dedicated gate observations

- Python tests: 32 PASS.
- Cargo fmt/clippy -D warnings and Rust compile/test PASS; 2 Rust negative tests reject missing/empty restored databases.
- Offline real F05 SQLite integration: 4 workers, 64 synthetic missions, online signed backup and separate restore, Rust ledger verified before and after; original database preserved.
- 16 concurrent audit writers PASS; audit tampering detected.
- Windows ACL private-directory test PASS.
- Backup corruption, missing signature, wrong signing key, symlink/traversal and overwrite denial PASS.
- F12_RELEASE_INVENTORY=PASS dependencies=13, HMAC-SHA256.
- F12_CONTRACTS_OK fixtures=4.
- F12_MAP_OK requirements=60 mapped=60.
- F12_PURITY_OK no_provider_sdks=1 secrets_reference_only=1 audit=append_only backup=signed restored_f05=rust_verified.
- F12_VERIFY=PASS.

## Formal status

F12_SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

No GNATprove-generated proof exists. The missing toolchain is an explicit limitation, not a successful formal verification.

## Drill boundary

A new combined process-hold, forced-stop, online-backup-and-restore invocation was blocked by execution security before running and was not repeated. The independently executed offline F05 backup/restore passed. Existing F05 real kill/recovery drills remain proven under F11.

## Baseline with F12 current

CONTINUITY_OK current=F12 next=F13 terminal=F13 root=D:\TIMED-MISSION-AGENT
F12_VERIFY=PASS
BASELINE_VERIFY=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

## Baseline after advancing to F13

CONTINUITY_OK current=F13 next=NONE terminal=F13 root=D:\TIMED-MISSION-AGENT
F12_VERIFY=PASS
BASELINE_VERIFY=PASS
SPARK_STATUS=SOURCE_READY_NOT_PROVEN GNATPROVE_NOT_INSTALLED

## Release seal

F12 CLOSED. F13 PERSONAL_PRODUCTION_V1_0 IN_PROGRESS; terminal acceptance remains pending. Release tag: v0.9.0-security-operational.
