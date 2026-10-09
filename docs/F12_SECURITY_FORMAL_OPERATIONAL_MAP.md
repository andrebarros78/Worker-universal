# F12 Security, Formal and Operational Hardening — Requirement Map

Scope: PC Vendas, D:\TIMED-MISSION-AGENT.
No secret values, bank credentials or external productive platform actions are used in F12.

| ID | Requirement | Implementation / proof |
|---|---|---|
| SEC-001 | Separate security runtime | security-runtime/security.py |
| SEC-002 | Python stdlib-only operations | purity gate |
| SEC-003 | Deny plaintext passwords | validate_secret_refs |
| SEC-004 | Deny plaintext API keys | validate_secret_refs |
| SEC-005 | Non-resolving env://TMA_ references | validate_secret_refs |
| SEC-006 | Reject invalid secret references | security tests |
| SEC-007 | Reject URL-embedded credentials | validate_secret_refs |
| SEC-008 | Do not echo signing keys | CLI |
| SEC-009 | Signing key from environment | CLI |
| SEC-010 | Require 32-byte HMAC signing key | hmac_sign |
| SEC-011 | Reject unsafe audit metadata | safe_identity |
| SEC-012 | Reject known token-shaped audit identifiers | safe_identity |
| SEC-013 | Append-only audit SQLite | AuditTrail |
| SEC-014 | Duplicate audit event idempotence | AuditTrail.record |
| SEC-015 | Conflicting event rejection | AuditTrail.record |
| SEC-016 | SHA256 audit hash chain | AuditTrail.verify |
| SEC-017 | 16 concurrent audit writers | unittest |
| SEC-018 | Audit tamper detection | unittest |
| SEC-019 | Least-privilege backup directory ACL | create_private_dir |
| SEC-020 | Windows user/SYSTEM/Admin restricted access | Windows icacls |
| SEC-021 | Root path allowlist | safe_rooted |
| SEC-022 | Deny parent traversal/outside root | safe_rooted |
| SEC-023 | Deny symlink redirection | safe_rooted |
| SEC-024 | SQLite online backup | backup_sqlite |
| SEC-025 | Backup integrity validation | sqlite_check |
| SEC-026 | HMAC-signed backup manifest | backup_sqlite |
| SEC-027 | Streaming sha256/size checks | hash_file |
| SEC-028 | Database size ceiling | MAX_DATABASE_BYTES |
| SEC-029 | Backup overwrite refused | backup_sqlite |
| SEC-030 | Restore separate target only | restore_sqlite |
| SEC-031 | Restore signature verification | restore_sqlite |
| SEC-032 | Restore hash verification | restore_sqlite |
| SEC-033 | Restore corruption rejection | restore_sqlite |
| SEC-034 | Restore filesystem durability | fsync |
| SEC-035 | Dependency inventory allowlist | DEPENDENCY_FILES |
| SEC-036 | Missing dependency manifests fail closed | dependency_inventory |
| SEC-037 | Signed release manifest | signed_release_inventory |
| SEC-038 | Dependency drift detected | verify_release_inventory |
| SEC-039 | Real F05 fixture created by F11 SQLite runtime | f12_offline_restore.py |
| SEC-040 | Rust F05 journal verification before backup | security-rust verifier |
| SEC-041 | Rust F05 journal verification after restore | security-rust verifier |
| SEC-042 | Restore source never overwritten | f12_offline_restore.py |
| SEC-043 | Independent backup/restore recovery audit | f12_offline_restore.py |
| SEC-044 | Controlled missing dependency drill | test_security.py |
| SEC-045 | Backup corruption drill | test_security.py |
| SEC-046 | Corrupt audit drill | test_security.py |
| SEC-047 | Denied process kill not bypassed | explicit proof limitation |
| SEC-048 | GNATprove detection | verify_f12.ps1 |
| SEC-049 | Never claim unrun formal proof | formal status marker |
| SEC-050 | GNATprove actual run when available | verify_f12.ps1 |
| SEC-051 | Versioned JSON contracts | contracts |
| SEC-052 | Contract fixture verification | verify_f12_contracts.py |
| SEC-053 | Security purity scanner | verify_f12_purity.py |
| SEC-054 | Single phase command | verify_f12.ps1 |
| SEC-055 | F11 recovery retained | global baseline |
| SEC-056 | F10 economic ledger retained | global baseline |
| SEC-057 | Full regression baseline | verify_baseline.ps1 |
| SEC-058 | DR and key-management runbook | docs/F12_SECURITY_RUNBOOK.md |
| SEC-059 | F12→F13 continuity | state/EXECUTION_STATE.json |
| SEC-060 | Proven Git commit/tag/remote sync | release |
