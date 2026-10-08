# F08 Implementation Matrix

| Requirements | Implementation | Proof |
|---|---|---|
| QUA-001..007 | qualification-rust model/harness | Rust tests |
| QUA-008..023 | scorecard.rs | threshold/percentile/fingerprint tests |
| QUA-024..028 | lifecycle.rs | synthetic lifecycle tests |
| QUA-029..048 | promotion.rs + F02 registry hardening | promotion tests |
| QUA-049..055 | canonical encodings + dataset/seed/SLA | reproducibility/timed tests |
| QUA-056..058 | purity scanner | F08_PURITY_OK |
| QUA-059 | JSON contracts + fixtures | F08_CONTRACTS_OK |
| QUA-060 | scripts/verify_f08.ps1 | F08_VERIFY=PASS |
