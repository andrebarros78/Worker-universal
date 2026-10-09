# F12 Implementation Matrix

| Requirements | Implemented components | Proof |
|---|---|---|
| SEC-001..012 | security-runtime/security.py and cli.py | secret/identity tests |
| SEC-013..020 | AuditTrail, private ACL | audit + Windows tests |
| SEC-021..038 | backup/restore/inventory | corruption/path/recovery tests |
| SEC-039..046 | availability-rust fixture and security-rust verifier | offline restored F05 ledger + drills |
| SEC-047..050 | explicit process-block note and GNATprove discovery | honest limitation |
| SEC-051..054 | JSON schemas, contracts, purity, verify_f12.ps1 | dedicated gate |
| SEC-055..060 | global regression, runbook, continuity, Git tag | closure |
